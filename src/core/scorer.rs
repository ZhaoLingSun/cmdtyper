use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Local, NaiveDate, TimeZone};

use crate::data::models::{
    Category, CharLatencySample, CharSpeedPoint, CharStat, Command, CommandProgress, DailyEntry,
    DailyStat, Keystroke, RecordMode, SessionRecord, TypingPosition, UserStats,
};

pub const STATS_VERSION: u32 = 2;
pub const CHARACTER_WINDOW: usize = 50;
pub const MIN_CHARACTER_SAMPLES: usize = 10;

/// Records are applied once. Rebuild from history when replacing a partial
/// snapshot with a newer snapshot sharing the same stable record ID.
pub fn update_stats(stats: &mut UserStats, record: &SessionRecord) {
    if stats.applied_record_ids.contains(&record.id) {
        return;
    }
    stats.applied_record_ids.push(record.id.clone());
    stats.stats_version = STATS_VERSION;
    stats.total_sessions += 1;
    if record.is_completed() {
        update_command_progress(stats, record);
    }
    if !is_typing_wpm_mode(record.mode) {
        return;
    }
    if record
        .typing
        .as_ref()
        .is_some_and(|metrics| metrics.positions.is_empty())
    {
        return;
    }

    let duration_ms = record_duration_ms(record);
    let previous_duration = stats.total_duration_ms;
    stats.total_duration_ms = stats.total_duration_ms.saturating_add(duration_ms);
    stats.total_wpm_sessions += 1;
    stats.best_wpm = stats.best_wpm.max(record.wpm);
    stats.overall_avg_wpm = weighted_average(
        stats.overall_avg_wpm,
        previous_duration as f64,
        record.wpm,
        duration_ms as f64,
    );
    let mut days: BTreeMap<String, DailyStat> = BTreeMap::new();

    if let Some(metrics) = &record.typing {
        stats.total_keystrokes += metrics
            .positions
            .iter()
            .map(|position| position.attempts)
            .sum::<u64>();
        for span in &metrics.active_spans {
            day_mut(&mut days, &span.date).total_duration_ms += span.duration_ms;
        }
        for position in &metrics.positions {
            stats.attempted_positions += 1;
            stats.error_positions += u64::from(position.error);
            let day = day_mut(&mut days, &position.attempted_date);
            day.attempted_positions += 1;
            day.error_positions += u64::from(position.error);
            if position.completed {
                stats.completed_chars += 1;
                day_mut(&mut days, &position.completed_date).completed_chars += 1;
            }
            let stat = get_or_insert_char_stat(stats, position.expected);
            apply_position(stat, position, true);
        }
        for day in days.values_mut() {
            day.accuracy_weight = day.attempted_positions;
            day.avg_accuracy = accuracy(day.attempted_positions, day.error_positions);
            day.avg_cpm = speed(day.completed_chars, day.total_duration_ms);
            day.avg_wpm = day.avg_cpm / 5.0;
        }
        let count = metrics.positions.len() as u64;
        stats.overall_avg_accuracy = weighted_average(
            stats.overall_avg_accuracy,
            stats.accuracy_weight as f64,
            record.accuracy,
            count as f64,
        );
        stats.accuracy_weight += count;
    } else {
        // Legacy records retain reported aggregates. Missing idle/backspace/
        // position information is explicitly labelled and never invented.
        stats.legacy_sessions_count += 1;
        stats.total_keystrokes += record
            .keystrokes
            .iter()
            .map(|stroke| stroke.attempts as u64)
            .sum::<u64>();
        let count = record.keystrokes.len().max(1) as u64;
        stats.overall_avg_accuracy = weighted_average(
            stats.overall_avg_accuracy,
            stats.accuracy_weight as f64,
            record.accuracy,
            count as f64,
        );
        stats.accuracy_weight += count;
        let day = day_mut(&mut days, &format_session_date(record.finished_at));
        day.total_duration_ms = duration_ms;
        day.avg_wpm = record.wpm;
        day.avg_cpm = record.cpm;
        day.avg_accuracy = record.accuracy;
        day.accuracy_weight = count;
        day.legacy_sessions_count = 1;
        day.completed_chars = record.keystrokes.len() as u64;
        day.attempted_positions = record.keystrokes.len() as u64;
        day.error_positions = record
            .keystrokes
            .iter()
            .filter(|stroke| !stroke.correct)
            .count() as u64;
        for stroke in &record.keystrokes {
            let stat = get_or_insert_char_stat(stats, stroke.expected);
            stat.total_samples += 1;
            stat.total_errors += u64::from(!stroke.correct);
            stat.total_correct += u64::from(stroke.correct);
            stat.accuracy = accuracy(stat.total_samples, stat.total_errors);
        }
    }
    if stats.legacy_sessions_count == 0 {
        stats.overall_avg_wpm = speed(stats.completed_chars, stats.total_duration_ms) / 5.0;
    }
    let completed_date = completion_date(record);
    if record.is_completed() {
        day_mut(&mut days, &completed_date);
    }
    for (_, mut contribution) in days {
        contribution.sessions_count = 1;
        contribution.wpm_sessions_count = 1;
        contribution.entries.push(DailyEntry {
            mode: record.mode,
            sessions_count: 1,
            completed_count: u32::from(
                record.is_completed() && contribution.date == completed_date,
            ),
            duration_ms: contribution.total_duration_ms,
            completed_chars: contribution.completed_chars,
            attempted_positions: contribution.attempted_positions,
            error_positions: contribution.error_positions,
            avg_wpm: contribution.avg_wpm,
            avg_cpm: contribution.avg_cpm,
            avg_accuracy: contribution.avg_accuracy,
            accuracy_weight: contribution.accuracy_weight,
            legacy_sessions_count: contribution.legacy_sessions_count,
        });
        merge_day(stats, contribution);
    }
    recalculate_streaks(stats);
}

/// New records preserve the completion-time local day. Earlier version-2
/// records can recover it from the latest completed position; only records
/// without saved input dates fall back to the timestamp in the current zone.
fn completion_date(record: &SessionRecord) -> String {
    if let Some(metrics) = &record.typing {
        if !metrics.completed_date.is_empty() {
            return metrics.completed_date.clone();
        }
        if let Some(position) = metrics
            .positions
            .iter()
            .filter(|position| position.completed && !position.completed_date.is_empty())
            .max_by_key(|position| position.completed_at_ms)
        {
            return position.completed_date.clone();
        }
    }
    format_session_date(record.finished_at)
}

/// The last snapshot for each ID wins; replay order is chronological.
/// Mastery is rebuilt from all completed modes, while follow-typing alone
/// contributes to calendar, speed and character measurements.
pub fn rebuild_from_history(history: &[SessionRecord]) -> UserStats {
    let mut seen = HashSet::new();
    let mut records: Vec<&SessionRecord> = history
        .iter()
        .rev()
        .filter(|record| seen.insert(record.id.as_str()))
        .collect();
    records.sort_by_key(|record| record.finished_at);
    let mut stats = UserStats {
        stats_version: STATS_VERSION,
        ..UserStats::default()
    };
    for record in records {
        update_stats(&mut stats, record);
    }
    stats
}

/// Preserve the aggregate-only baseline of installations whose old event
/// history is absent. It remains explicitly legacy and creates no intervals.
pub fn merge_legacy_stats(stats: &mut UserStats, baseline: &UserStats) {
    if baseline.total_sessions == 0
        && baseline.daily_stats.is_empty()
        && baseline.command_progress.is_empty()
    {
        return;
    }
    let duration = baseline.total_duration_ms;
    stats.overall_avg_wpm = weighted_average(
        stats.overall_avg_wpm,
        stats.total_duration_ms as f64,
        baseline.overall_avg_wpm,
        duration as f64,
    );
    let accuracy_weight = baseline
        .accuracy_weight
        .max(baseline.total_wpm_sessions)
        .max(baseline.total_sessions);
    stats.overall_avg_accuracy = weighted_average(
        stats.overall_avg_accuracy,
        stats.accuracy_weight as f64,
        baseline.overall_avg_accuracy,
        accuracy_weight as f64,
    );
    stats.accuracy_weight += accuracy_weight;
    stats.total_sessions += baseline.total_sessions;
    stats.total_wpm_sessions += baseline.total_wpm_sessions;
    stats.total_keystrokes += baseline.total_keystrokes;
    stats.total_duration_ms += duration;
    stats.best_wpm = stats.best_wpm.max(baseline.best_wpm);
    stats.legacy_sessions_count += baseline.total_sessions;
    for old in &baseline.char_stats {
        let stat = get_or_insert_char_stat(stats, old.char_key);
        stat.total_correct += old.total_correct;
        stat.total_samples += old.total_samples;
        stat.total_errors += old.total_samples.saturating_sub(old.total_correct);
        stat.accuracy = accuracy(stat.total_samples, stat.total_errors);
        // Keep only measured version-2 interval windows already in `stats`.
    }
    for old in &baseline.daily_stats {
        let mut day = old.clone();
        day.legacy_sessions_count = old.sessions_count;
        day.avg_cpm = old.avg_wpm * 5.0;
        day.accuracy_weight = old.accuracy_weight.max(old.sessions_count as u64);
        merge_day(stats, day);
    }
    merge_command_progress(stats, baseline);
    recalculate_streaks(stats);
}

pub fn merge_command_progress(stats: &mut UserStats, previous: &UserStats) {
    for old in &previous.command_progress {
        if let Some(progress) = stats
            .command_progress
            .iter_mut()
            .find(|progress| progress.command_id == old.command_id)
        {
            progress.times_practiced = progress.times_practiced.max(old.times_practiced);
            progress.best_wpm = progress.best_wpm.max(old.best_wpm);
            progress.best_accuracy = progress.best_accuracy.max(old.best_accuracy);
            progress.last_practiced = progress.last_practiced.max(old.last_practiced);
            progress.mastery = progress.mastery.max(old.mastery);
        } else {
            stats.command_progress.push(old.clone());
        }
    }
}

pub fn is_typing_wpm_mode(mode: RecordMode) -> bool {
    matches!(
        mode,
        RecordMode::Typing
            | RecordMode::LessonPractice
            | RecordMode::ReviewTyping
            | RecordMode::SymbolTyping
            | RecordMode::SystemTyping
            | RecordMode::ScenarioTyping
    )
}

pub fn mode_label(mode: RecordMode) -> &'static str {
    match mode {
        RecordMode::Typing => "对着打",
        RecordMode::LessonPractice => "命令专题",
        RecordMode::SymbolTyping => "符号专题",
        RecordMode::SystemTyping => "系统专题",
        RecordMode::ReviewTyping => "专题训练",
        RecordMode::ScenarioTyping => "场景实训",
        RecordMode::Dictation | RecordMode::ReviewDictation => "默写",
        RecordMode::SymbolPractice | RecordMode::ReviewCloze => "填空",
    }
}

/// Direct callers provide reliable interval samples. Legacy migration does not
/// call this helper because its timing cannot be reconstructed reliably.
pub fn update_char_stat(stat: &mut CharStat, keystrokes: &[Keystroke]) {
    for stroke in keystrokes {
        apply_position(
            stat,
            &TypingPosition {
                expected: stroke.expected,
                completed: true,
                error: !stroke.correct,
                attempts: stroke.attempts as u64,
                completed_at_ms: stroke.timestamp_ms,
                completed_date: format_session_date(stroke.timestamp_ms),
                latency_ms: stroke.latency_ms,
                speed_sample_valid: stroke.latency_ms > 0 && stroke.latency_ms <= 30_000,
                ..TypingPosition::default()
            },
            true,
        );
    }
    if !keystrokes.is_empty() {
        stat.history.push(CharSpeedPoint {
            session_index: stat.history.len() as u32 + 1,
            cpm: stat.avg_cpm,
            accuracy: stat.accuracy,
        });
        if stat.history.len() > CHARACTER_WINDOW {
            stat.history.remove(0);
        }
    }
}

fn apply_position(stat: &mut CharStat, position: &TypingPosition, include_sample: bool) {
    stat.total_samples += 1;
    stat.total_errors += u64::from(position.error);
    stat.total_correct += u64::from(!position.error);
    stat.accuracy = accuracy(stat.total_samples, stat.total_errors);
    if include_sample
        && position.completed
        && position.speed_sample_valid
        && position.latency_ms > 0
    {
        stat.recent_latencies.push(CharLatencySample {
            timestamp_ms: position.completed_at_ms,
            date: position.completed_date.clone(),
            latency_ms: position.latency_ms,
        });
        stat.recent_latencies
            .sort_by_key(|sample| sample.timestamp_ms);
        if stat.recent_latencies.len() > CHARACTER_WINDOW {
            stat.recent_latencies
                .drain(..stat.recent_latencies.len() - CHARACTER_WINDOW);
        }
    }
    let average = smoothed_latency(&stat.recent_latencies);
    stat.avg_latency_ms = average.unwrap_or(0.0);
    stat.avg_cpm = average.map(|latency| 60_000.0 / latency).unwrap_or(0.0);
}

/// Winsorize the fastest/slowest 10% of a bounded latency window, then invert
/// mean latency. Never average instantaneous CPM values.
pub fn smoothed_latency(samples: &[CharLatencySample]) -> Option<f64> {
    let mut latencies = samples
        .iter()
        .rev()
        .filter(|sample| sample.latency_ms > 0)
        .take(CHARACTER_WINDOW)
        .map(|sample| sample.latency_ms)
        .collect::<Vec<_>>();
    if latencies.len() < MIN_CHARACTER_SAMPLES {
        return None;
    }
    latencies.sort_unstable();
    let trim = latencies.len() / 10;
    let low = latencies[trim];
    let high = latencies[latencies.len() - 1 - trim];
    Some(
        latencies
            .iter()
            .map(|latency| (*latency).clamp(low, high) as f64)
            .sum::<f64>()
            / latencies.len() as f64,
    )
}

/// Reconstruct the bounded character windows as they stood at the end of a
/// selected local date, rather than leaking future practice into older dates.
pub fn character_stats_as_of(history: &[SessionRecord], date: &str) -> Vec<CharStat> {
    let mut seen = HashSet::new();
    let mut records = history
        .iter()
        .rev()
        .filter(|record| seen.insert(record.id.as_str()))
        .collect::<Vec<_>>();
    records.sort_by_key(|record| record.finished_at);
    let mut stats = UserStats::default();
    for record in records {
        if !is_typing_wpm_mode(record.mode) {
            continue;
        }
        if let Some(metrics) = &record.typing {
            for position in &metrics.positions {
                if position.attempted_date.as_str() <= date {
                    let stat = get_or_insert_char_stat(&mut stats, position.expected);
                    let mut snapshot = position.clone();
                    if !position.updated_date.is_empty() && position.updated_date.as_str() > date {
                        if let Some(prior) = metrics
                            .position_day_snapshots
                            .iter()
                            .filter(|prior| {
                                prior.index == position.index && prior.date.as_str() <= date
                            })
                            .max_by(|left, right| left.date.cmp(&right.date))
                        {
                            snapshot.completed = prior.completed;
                            snapshot.completed_at_ms = prior.completed_at_ms;
                            snapshot.completed_date = prior.completed_date.clone();
                            snapshot.latency_ms = prior.latency_ms;
                            snapshot.speed_sample_valid = prior.speed_sample_valid;
                        } else {
                            snapshot.completed = false;
                        }
                    }
                    if snapshot
                        .error_date
                        .as_deref()
                        .is_some_and(|error_date| error_date > date)
                    {
                        snapshot.error = false;
                    }
                    apply_position(stat, &snapshot, snapshot.completed_date.as_str() <= date);
                }
            }
        }
    }
    stats.char_stats.sort_by_key(|stat| stat.char_key);
    stats.char_stats
}

fn day_mut<'a>(days: &'a mut BTreeMap<String, DailyStat>, date: &str) -> &'a mut DailyStat {
    days.entry(date.to_owned()).or_insert_with(|| DailyStat {
        date: date.to_owned(),
        ..DailyStat::default()
    })
}

fn merge_day(stats: &mut UserStats, incoming: DailyStat) {
    if let Some(day) = stats
        .daily_stats
        .iter_mut()
        .find(|day| day.date == incoming.date)
    {
        day.avg_wpm = weighted_average(
            day.avg_wpm,
            day.total_duration_ms as f64,
            incoming.avg_wpm,
            incoming.total_duration_ms as f64,
        );
        day.avg_cpm = day.avg_wpm * 5.0;
        day.avg_accuracy = weighted_average(
            day.avg_accuracy,
            day.accuracy_weight as f64,
            incoming.avg_accuracy,
            incoming.accuracy_weight as f64,
        );
        day.sessions_count += incoming.sessions_count;
        day.wpm_sessions_count += incoming.wpm_sessions_count;
        day.total_duration_ms += incoming.total_duration_ms;
        day.completed_chars += incoming.completed_chars;
        day.attempted_positions += incoming.attempted_positions;
        day.error_positions += incoming.error_positions;
        day.accuracy_weight += incoming.accuracy_weight;
        day.legacy_sessions_count += incoming.legacy_sessions_count;
        if day.legacy_sessions_count == 0 {
            day.avg_cpm = speed(day.completed_chars, day.total_duration_ms);
            day.avg_wpm = day.avg_cpm / 5.0;
        }
        for entry in incoming.entries {
            if let Some(existing) = day
                .entries
                .iter_mut()
                .find(|existing| existing.mode == entry.mode)
            {
                existing.avg_wpm = weighted_average(
                    existing.avg_wpm,
                    existing.duration_ms as f64,
                    entry.avg_wpm,
                    entry.duration_ms as f64,
                );
                existing.avg_accuracy = weighted_average(
                    existing.avg_accuracy,
                    existing.accuracy_weight as f64,
                    entry.avg_accuracy,
                    entry.accuracy_weight as f64,
                );
                existing.accuracy_weight += entry.accuracy_weight;
                existing.legacy_sessions_count += entry.legacy_sessions_count;
                existing.sessions_count += entry.sessions_count;
                existing.completed_count += entry.completed_count;
                existing.duration_ms += entry.duration_ms;
                existing.completed_chars += entry.completed_chars;
                existing.attempted_positions += entry.attempted_positions;
                existing.error_positions += entry.error_positions;
                if existing.legacy_sessions_count == 0 {
                    existing.avg_wpm = speed(existing.completed_chars, existing.duration_ms) / 5.0;
                }
                existing.avg_cpm = existing.avg_wpm * 5.0;
            } else {
                day.entries.push(entry);
            }
        }
    } else {
        stats.daily_stats.push(incoming);
    }
    stats
        .daily_stats
        .sort_by(|left, right| left.date.cmp(&right.date));
}

pub fn accuracy(attempted: u64, errors: u64) -> f64 {
    if attempted == 0 {
        1.0
    } else {
        1.0 - errors.min(attempted) as f64 / attempted as f64
    }
}

pub fn speed(completed_chars: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        0.0
    } else {
        completed_chars as f64 * 60_000.0 / duration_ms as f64
    }
}

fn recalculate_streaks(stats: &mut UserStats) {
    let mut dates = stats
        .daily_stats
        .iter()
        .filter(|day| day.sessions_count > 0)
        .filter_map(|day| NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok())
        .collect::<Vec<_>>();
    dates.sort_unstable();
    dates.dedup();
    let mut current = 0;
    let mut longest = 0;
    let mut previous: Option<NaiveDate> = None;
    for date in &dates {
        current = if previous.and_then(|day| day.succ_opt()) == Some(*date) {
            current + 1
        } else {
            1
        };
        longest = longest.max(current);
        previous = Some(*date);
    }
    let today = Local::now().date_naive();
    stats.current_streak = if dates
        .last()
        .is_some_and(|date| *date == today || date.succ_opt() == Some(today))
    {
        current
    } else {
        0
    };
    stats.longest_streak = longest;
}

fn format_session_date(timestamp_ms: i64) -> String {
    Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_owned())
}

pub fn record_duration_ms(record: &SessionRecord) -> u64 {
    record
        .typing
        .as_ref()
        .map(|metrics| metrics.active_duration_ms)
        .unwrap_or_else(|| record.finished_at.saturating_sub(record.started_at).max(0) as u64)
}

fn weighted_average(current: f64, current_weight: f64, next: f64, next_weight: f64) -> f64 {
    let total = current_weight + next_weight;
    if total == 0.0 {
        0.0
    } else {
        (current * current_weight + next * next_weight) / total
    }
}

fn float_cmp(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

/// Compute mastery score: accuracy * min(times/target, 1.0).
pub fn compute_mastery(accuracy: f64, times: u32, target: u32) -> f64 {
    if target == 0 {
        return accuracy.clamp(0.0, 1.0);
    }

    let practice_factor = (times as f64 / target as f64).min(1.0);
    accuracy.clamp(0.0, 1.0) * practice_factor
}

/// Return the `n` weakest characters sorted by lowest accuracy first.
pub fn weak_chars(stats: &UserStats, n: usize) -> Vec<&CharStat> {
    let mut chars = stats
        .char_stats
        .iter()
        .filter(|stat| stat.total_samples > 0)
        .collect::<Vec<_>>();

    chars.sort_by(|left, right| {
        float_cmp(left.accuracy, right.accuracy)
            .then_with(|| right.total_samples.cmp(&left.total_samples))
            .then_with(|| left.char_key.cmp(&right.char_key))
    });
    chars.truncate(n);
    chars
}

/// Compute average mastery for all commands in a given category.
pub fn category_mastery(stats: &UserStats, commands: &[Command], category: Category) -> f64 {
    let category_commands = commands
        .iter()
        .filter(|command| command.category == category)
        .collect::<Vec<_>>();

    if category_commands.is_empty() {
        return 0.0;
    }

    let progress_by_id = stats
        .command_progress
        .iter()
        .map(|progress| (progress.command_id.as_str(), progress.mastery))
        .collect::<HashMap<_, _>>();

    let total_mastery = category_commands
        .iter()
        .map(|command| {
            progress_by_id
                .get(command.id.as_str())
                .copied()
                .unwrap_or(0.0)
        })
        .sum::<f64>();

    total_mastery / category_commands.len() as f64
}

/// Recommend `n` commands that best target the user's weak characters
/// and lowest mastery commands.
pub fn recommend_commands<'a>(
    stats: &UserStats,
    commands: &'a [Command],
    n: usize,
) -> Vec<&'a Command> {
    if n == 0 || commands.is_empty() {
        return Vec::new();
    }

    let weak_pool = weak_chars(
        stats,
        stats.char_stats.len().min(n.saturating_mul(2).max(5)),
    );
    if weak_pool.is_empty() {
        return commands.iter().take(n).collect();
    }

    let weak_weights = weak_pool
        .into_iter()
        .map(|stat| (stat.char_key, 1.0 - stat.accuracy))
        .collect::<HashMap<_, _>>();
    let progress_by_id = stats
        .command_progress
        .iter()
        .map(|progress| (progress.command_id.as_str(), progress.mastery))
        .collect::<HashMap<_, _>>();

    let mut scored = commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let weak_score = command
                .command
                .chars()
                .map(|ch| weak_weights.get(&ch).copied().unwrap_or(0.0))
                .sum::<f64>();
            let mastery_score = 1.0
                - progress_by_id
                    .get(command.id.as_str())
                    .copied()
                    .unwrap_or(0.0);
            (index, weak_score * 2.0 + mastery_score)
        })
        .collect::<Vec<_>>();

    if scored.iter().all(|(_, score)| *score == 0.0) {
        return commands.iter().take(n).collect();
    }

    scored.sort_by(|left, right| float_cmp(right.1, left.1).then_with(|| left.0.cmp(&right.0)));

    scored
        .into_iter()
        .take(n)
        .map(|(index, _)| &commands[index])
        .collect()
}

fn update_command_progress(stats: &mut UserStats, record: &SessionRecord) {
    let index = if let Some(index) = stats
        .command_progress
        .iter()
        .position(|progress| progress.command_id == record.command_id)
    {
        index
    } else {
        stats.command_progress.push(CommandProgress {
            command_id: record.command_id.clone(),
            ..CommandProgress::default()
        });
        stats.command_progress.len() - 1
    };
    let progress = &mut stats.command_progress[index];

    progress.times_practiced += 1;
    progress.best_wpm = progress.best_wpm.max(record.wpm);
    progress.best_accuracy = progress.best_accuracy.max(record.accuracy);
    progress.last_practiced = Some(record.finished_at);
    progress.mastery = compute_mastery(
        progress.best_accuracy,
        progress.times_practiced,
        record.difficulty.target_attempts(),
    );
}

fn get_or_insert_char_stat(stats: &mut UserStats, char_key: char) -> &mut CharStat {
    if let Some(index) = stats
        .char_stats
        .iter()
        .position(|stat| stat.char_key == char_key)
    {
        return &mut stats.char_stats[index];
    }

    stats.char_stats.push(CharStat {
        char_key,
        ..CharStat::default()
    });
    stats
        .char_stats
        .last_mut()
        .expect("char stat was just inserted")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{
        Category, CharStat, Command, CommandProgress, Difficulty, Keystroke, RecordMode,
        SessionRecord, UserStats,
    };

    fn approx_eq(left: f64, right: f64) {
        let delta = (left - right).abs();
        assert!(delta < 1e-6, "left={left}, right={right}, delta={delta}");
    }

    fn ts(date: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(&format!("{date}T00:00:00Z"))
            .expect("valid RFC3339 timestamp")
            .timestamp_millis()
    }

    fn keystroke(expected: char, correct: bool, attempts: u8, latency_ms: u64) -> Keystroke {
        Keystroke {
            expected,
            actual: expected,
            correct,
            attempts,
            latency_ms,
            timestamp_ms: 0,
        }
    }

    fn record(id: &str, command_id: &str, date: &str, difficulty: Difficulty) -> SessionRecord {
        SessionRecord {
            id: id.to_string(),
            command_id: command_id.to_string(),
            mode: RecordMode::Typing,
            keystrokes: vec![
                keystroke('g', false, 2, 120),
                keystroke('r', true, 1, 100),
                keystroke('e', true, 1, 90),
            ],
            started_at: ts(date),
            finished_at: ts(date) + 30_000,
            wpm: 48.0,
            cpm: 240.0,
            accuracy: 0.8,
            error_count: 1,
            difficulty,
            ..SessionRecord::default()
        }
    }

    #[test]
    fn compute_mastery_scales_by_target_attempts() {
        approx_eq(compute_mastery(0.9, 2, 4), 0.45);
        approx_eq(compute_mastery(0.9, 10, 4), 0.9);
        approx_eq(compute_mastery(1.2, 3, 0), 1.0);
    }

    #[test]
    fn compute_mastery_zero_accuracy() {
        approx_eq(compute_mastery(0.0, 5, 5), 0.0);
    }

    #[test]
    fn compute_mastery_negative_accuracy_clamped() {
        approx_eq(compute_mastery(-0.5, 5, 5), 0.0);
    }

    #[test]
    fn update_char_stat_aggregates_accuracy_latency_and_history() {
        let mut stat = CharStat {
            char_key: 'g',
            ..CharStat::default()
        };
        let keystrokes = vec![keystroke('g', true, 1, 100), keystroke('g', false, 3, 200)];

        update_char_stat(&mut stat, &keystrokes);

        assert_eq!(stat.total_samples, 2);
        assert_eq!(stat.total_correct, 1);
        assert_eq!(stat.total_errors, 1);
        assert_eq!(stat.recent_latencies.len(), 2);
        approx_eq(stat.avg_latency_ms, 0.0);
        approx_eq(stat.avg_cpm, 0.0);
        approx_eq(stat.accuracy, 0.5);
        assert_eq!(stat.history.len(), 1);
        assert_eq!(stat.history[0].session_index, 1);
        approx_eq(stat.history[0].accuracy, 0.5);
    }

    #[test]
    fn update_char_stat_empty_keystrokes_is_noop() {
        let mut stat = CharStat {
            char_key: 'x',
            total_samples: 5,
            ..CharStat::default()
        };
        update_char_stat(&mut stat, &[]);
        assert_eq!(stat.total_samples, 5);
    }

    #[test]
    fn update_stats_updates_global_totals_daily_stats_and_progress() {
        let mut stats = UserStats::default();

        update_stats(
            &mut stats,
            &record("1", "grep-help", "2026-03-06", Difficulty::Basic),
        );
        update_stats(
            &mut stats,
            &record("2", "grep-help", "2026-03-07", Difficulty::Basic),
        );

        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.total_keystrokes, 8); // (2+1+1) * 2
        assert_eq!(stats.total_duration_ms, 60_000);
        approx_eq(stats.overall_avg_wpm, 48.0);
        approx_eq(stats.overall_avg_accuracy, 0.8);
        approx_eq(stats.best_wpm, 48.0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 2);
        assert_eq!(stats.daily_stats.len(), 2);
        assert_eq!(stats.command_progress.len(), 1);
        assert_eq!(stats.command_progress[0].times_practiced, 2);
        approx_eq(
            stats.command_progress[0].mastery,
            compute_mastery(0.8, 2, 5),
        );
        assert_eq!(stats.char_stats.len(), 3);
    }

    #[test]
    fn update_stats_resets_current_streak_after_gap() {
        let mut stats = UserStats::default();

        update_stats(
            &mut stats,
            &record("1", "cmd-1", "2026-03-01", Difficulty::Basic),
        );
        update_stats(
            &mut stats,
            &record("2", "cmd-2", "2026-03-02", Difficulty::Basic),
        );
        update_stats(
            &mut stats,
            &record("3", "cmd-3", "2026-03-04", Difficulty::Basic),
        );

        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 2);
    }

    #[test]
    fn weak_chars_returns_lowest_accuracy_first() {
        let stats = UserStats {
            char_stats: vec![
                CharStat {
                    char_key: 'a',
                    total_samples: 10,
                    accuracy: 0.9,
                    ..CharStat::default()
                },
                CharStat {
                    char_key: 'b',
                    total_samples: 20,
                    accuracy: 0.6,
                    ..CharStat::default()
                },
                CharStat {
                    char_key: 'c',
                    total_samples: 5,
                    accuracy: 0.7,
                    ..CharStat::default()
                },
            ],
            ..UserStats::default()
        };

        let weak = weak_chars(&stats, 2);

        assert_eq!(weak.len(), 2);
        assert_eq!(weak[0].char_key, 'b');
        assert_eq!(weak[1].char_key, 'c');
    }

    #[test]
    fn weak_chars_skips_zero_sample_chars() {
        let stats = UserStats {
            char_stats: vec![
                CharStat {
                    char_key: 'a',
                    total_samples: 0,
                    accuracy: 0.0,
                    ..CharStat::default()
                },
                CharStat {
                    char_key: 'b',
                    total_samples: 5,
                    accuracy: 0.8,
                    ..CharStat::default()
                },
            ],
            ..UserStats::default()
        };

        let weak = weak_chars(&stats, 5);
        assert_eq!(weak.len(), 1);
        assert_eq!(weak[0].char_key, 'b');
    }

    #[test]
    fn category_mastery_averages_matching_commands_and_missing_progress_as_zero() {
        let stats = UserStats {
            command_progress: vec![CommandProgress {
                command_id: "ls-basic".to_string(),
                mastery: 0.8,
                ..CommandProgress::default()
            }],
            ..UserStats::default()
        };
        let commands = vec![
            Command {
                id: "ls-basic".to_string(),
                command: "ls -la".to_string(),
                category: Category::FileOps,
                ..Command::default()
            },
            Command {
                id: "cp-basic".to_string(),
                command: "cp a b".to_string(),
                category: Category::FileOps,
                ..Command::default()
            },
            Command {
                id: "grep-basic".to_string(),
                command: "grep foo".to_string(),
                category: Category::Search,
                ..Command::default()
            },
        ];

        approx_eq(category_mastery(&stats, &commands, Category::FileOps), 0.4);
        approx_eq(category_mastery(&stats, &commands, Category::Search), 0.0);
    }

    #[test]
    fn category_mastery_empty_category_returns_zero() {
        let stats = UserStats::default();
        let commands = vec![Command {
            id: "ls".to_string(),
            category: Category::FileOps,
            ..Command::default()
        }];
        approx_eq(category_mastery(&stats, &commands, Category::Network), 0.0);
    }

    #[test]
    fn recommend_commands_prefers_weak_character_coverage() {
        let stats = UserStats {
            char_stats: vec![
                CharStat {
                    char_key: 'g',
                    total_samples: 20,
                    accuracy: 0.4,
                    ..CharStat::default()
                },
                CharStat {
                    char_key: 'l',
                    total_samples: 20,
                    accuracy: 0.95,
                    ..CharStat::default()
                },
            ],
            command_progress: vec![CommandProgress {
                command_id: "ls".to_string(),
                mastery: 0.9,
                ..CommandProgress::default()
            }],
            ..UserStats::default()
        };
        let commands = vec![
            Command {
                id: "ls".to_string(),
                command: "ls -la".to_string(),
                category: Category::FileOps,
                ..Command::default()
            },
            Command {
                id: "grep".to_string(),
                command: "grep foo file".to_string(),
                category: Category::Search,
                ..Command::default()
            },
            Command {
                id: "git-grep".to_string(),
                command: "git grep foo".to_string(),
                category: Category::Search,
                ..Command::default()
            },
        ];

        let recommended = recommend_commands(&stats, &commands, 2);

        assert_eq!(recommended.len(), 2);
        assert_eq!(recommended[0].id, "git-grep");
        assert_eq!(recommended[1].id, "grep");
    }

    #[test]
    fn recommend_commands_empty_stats_returns_first_n() {
        let stats = UserStats::default();
        let commands = vec![
            Command {
                id: "a".to_string(),
                ..Command::default()
            },
            Command {
                id: "b".to_string(),
                ..Command::default()
            },
            Command {
                id: "c".to_string(),
                ..Command::default()
            },
        ];

        let recommended = recommend_commands(&stats, &commands, 2);
        assert_eq!(recommended.len(), 2);
        assert_eq!(recommended[0].id, "a");
        assert_eq!(recommended[1].id, "b");
    }

    #[test]
    fn recommend_commands_zero_n_returns_empty() {
        let stats = UserStats::default();
        let commands = vec![Command {
            id: "a".to_string(),
            ..Command::default()
        }];
        assert!(recommend_commands(&stats, &commands, 0).is_empty());
    }

    #[test]
    fn recommend_commands_empty_commands_returns_empty() {
        let stats = UserStats::default();
        assert!(recommend_commands(&stats, &[], 5).is_empty());
    }
}
