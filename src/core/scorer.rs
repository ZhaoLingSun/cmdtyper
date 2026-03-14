use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use chrono::{NaiveDate, TimeZone, Utc};

use crate::data::models::{
    Category, CharSpeedPoint, CharStat, Command, CommandProgress, DailyStat, Keystroke,
    RecordMode, SessionRecord, UserStats,
};

/// Update global user statistics with a completed session record.
pub fn update_stats(stats: &mut UserStats, record: &SessionRecord) {
    let previous_sessions = stats.total_sessions as f64;
    let previous_wpm_sessions = stats.total_wpm_sessions as f64;

    stats.total_sessions += 1;
    stats.total_keystrokes += record
        .keystrokes
        .iter()
        .map(|keystroke| keystroke.attempts as u64)
        .sum::<u64>();

    let duration_ms = record_duration_ms(record);
    stats.total_duration_ms += duration_ms;
    if is_typing_wpm_mode(record.mode) {
        stats.overall_avg_wpm = weighted_average(
            stats.overall_avg_wpm,
            previous_wpm_sessions,
            record.wpm,
            1.0,
        );
        stats.best_wpm = stats.best_wpm.max(record.wpm);
        stats.total_wpm_sessions += 1;
    }
    stats.overall_avg_accuracy = weighted_average(
        stats.overall_avg_accuracy,
        previous_sessions,
        record.accuracy,
        1.0,
    );

    let date = format_session_date(record.finished_at);
    update_daily_stat(stats, &date, duration_ms, record.wpm, record.accuracy, is_typing_wpm_mode(record.mode));
    recalculate_streaks(stats);

    // Group keystrokes by expected character
    let mut grouped: BTreeMap<char, Vec<Keystroke>> = BTreeMap::new();
    for keystroke in &record.keystrokes {
        grouped
            .entry(keystroke.expected)
            .or_default()
            .push(keystroke.clone());
    }

    for (char_key, keystrokes) in grouped {
        let stat = get_or_insert_char_stat(stats, char_key);
        update_char_stat(stat, &keystrokes);
    }

    update_command_progress(stats, record);
}

/// Update a single character's statistics from a batch of keystrokes.
pub fn update_char_stat(stat: &mut CharStat, keystrokes: &[Keystroke]) {
    if keystrokes.is_empty() {
        return;
    }

    let session_samples = keystrokes.len() as u64;
    let session_correct = keystrokes
        .iter()
        .filter(|keystroke| keystroke.correct)
        .count() as u64;
    let session_errors = keystrokes
        .iter()
        .map(|keystroke| keystroke.attempts.saturating_sub(1) as u64)
        .sum::<u64>();
    let session_latency = keystrokes
        .iter()
        .map(|keystroke| keystroke.latency_ms as f64)
        .sum::<f64>()
        / session_samples as f64;
    let session_cpm = keystrokes
        .iter()
        .map(|keystroke| cpm_for_latency(keystroke.latency_ms))
        .sum::<f64>()
        / session_samples as f64;

    let previous_samples = stat.total_samples as f64;
    stat.total_correct += session_correct;
    stat.total_errors += session_errors;
    stat.total_samples += session_samples;
    stat.avg_latency_ms = weighted_average(
        stat.avg_latency_ms,
        previous_samples,
        session_latency,
        session_samples as f64,
    );
    stat.avg_cpm = weighted_average(
        stat.avg_cpm,
        previous_samples,
        session_cpm,
        session_samples as f64,
    );
    stat.accuracy = if stat.total_samples == 0 {
        0.0
    } else {
        stat.total_correct as f64 / stat.total_samples as f64
    };
    stat.history.push(CharSpeedPoint {
        session_index: stat.history.len() as u32 + 1,
        cpm: session_cpm,
        accuracy: session_correct as f64 / session_samples as f64,
    });
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

fn is_typing_wpm_mode(mode: RecordMode) -> bool {
    matches!(
        mode,
        RecordMode::Typing | RecordMode::LessonPractice | RecordMode::ReviewPractice
    )
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

fn update_daily_stat(
    stats: &mut UserStats,
    date: &str,
    duration_ms: u64,
    wpm: f64,
    accuracy: f64,
    include_wpm: bool,
) {
    if let Some(day) = stats.daily_stats.iter_mut().find(|day| day.date == date) {
        let previous_sessions = day.sessions_count as f64;
        day.sessions_count += 1;
        day.total_duration_ms += duration_ms;
        if include_wpm {
            let previous_wpm_sessions = day.wpm_sessions_count as f64;
            day.avg_wpm = weighted_average(day.avg_wpm, previous_wpm_sessions, wpm, 1.0);
            day.wpm_sessions_count += 1;
        }
        day.avg_accuracy = weighted_average(day.avg_accuracy, previous_sessions, accuracy, 1.0);
    } else {
        stats.daily_stats.push(DailyStat {
            date: date.to_string(),
            sessions_count: 1,
            total_duration_ms: duration_ms,
            avg_wpm: if include_wpm { wpm } else { 0.0 },
            avg_accuracy: accuracy,
            wpm_sessions_count: if include_wpm { 1 } else { 0 },
        });
        stats
            .daily_stats
            .sort_by(|left, right| left.date.cmp(&right.date));
    }
}

fn recalculate_streaks(stats: &mut UserStats) {
    let mut dates = stats
        .daily_stats
        .iter()
        .filter(|day| day.sessions_count > 0)
        .filter_map(|day| NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok())
        .collect::<Vec<_>>();

    if dates.is_empty() {
        stats.current_streak = 0;
        stats.longest_streak = 0;
        return;
    }

    dates.sort_unstable();

    let mut longest = 1_u32;
    let mut current_run = 1_u32;

    for window in dates.windows(2) {
        let previous = window[0];
        let current = window[1];
        let is_consecutive = previous.succ_opt() == Some(current);

        if is_consecutive {
            current_run += 1;
        } else {
            current_run = 1;
        }

        longest = longest.max(current_run);
    }

    stats.current_streak = current_run;
    stats.longest_streak = longest;
}

fn format_session_date(timestamp_ms: i64) -> String {
    Utc.timestamp_millis_opt(timestamp_ms)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

fn record_duration_ms(record: &SessionRecord) -> u64 {
    record.finished_at.saturating_sub(record.started_at) as u64
}

fn cpm_for_latency(latency_ms: u64) -> f64 {
    if latency_ms == 0 {
        0.0
    } else {
        60_000.0 / latency_ms as f64
    }
}

fn weighted_average(current: f64, current_weight: f64, next: f64, next_weight: f64) -> f64 {
    let total_weight = current_weight + next_weight;
    if total_weight == 0.0 {
        0.0
    } else {
        ((current * current_weight) + (next * next_weight)) / total_weight
    }
}

fn float_cmp(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
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
        assert_eq!(stat.total_errors, 2);
        approx_eq(stat.avg_latency_ms, 150.0);
        approx_eq(stat.avg_cpm, (600.0 + 300.0) / 2.0);
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
        assert_eq!(stats.current_streak, 2);
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

        assert_eq!(stats.current_streak, 1);
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
