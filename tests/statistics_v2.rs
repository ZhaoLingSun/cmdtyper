use std::time::{Duration, Instant};

use chrono::{Local, TimeZone};
use cmdtyper::core::{engine::TypingEngine, scorer};
use cmdtyper::data::models::{CharLatencySample, Difficulty, RecordMode, SessionRecord, UserStats};

fn start_timestamp() -> i64 {
    Local
        .with_ymd_and_hms(2026, 8, 10, 12, 0, 0)
        .single()
        .unwrap()
        .timestamp_millis()
}

fn input(engine: &mut TypingEngine, ch: char, origin: Instant, wall: i64, millis: u64) {
    engine.input_at(
        ch,
        origin + Duration::from_millis(millis),
        wall + millis as i64,
    );
}

fn finish(engine: &TypingEngine, origin: Instant, wall: i64, millis: u64) -> SessionRecord {
    engine.finish_at(
        "example",
        Difficulty::Beginner,
        RecordMode::Typing,
        origin + Duration::from_millis(millis),
        wall + millis as i64,
    )
}

#[test]
fn repeated_errors_and_backspace_count_a_target_position_once() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("abc");
    for (ch, ms) in [('x', 0), ('x', 100), ('a', 200), ('b', 300)] {
        input(&mut engine, ch, origin, wall, ms);
    }
    engine.backspace_at(origin + Duration::from_millis(400), wall + 400);
    input(&mut engine, 'x', origin, wall, 500);
    input(&mut engine, 'b', origin, wall, 600);
    input(&mut engine, 'c', origin, wall, 700);
    let record = finish(&engine, origin, wall, 9_000);
    assert_eq!(record.error_count, 2);
    assert!((record.accuracy - 1.0 / 3.0).abs() < 1e-10);
    let metrics = record.typing.unwrap();
    assert_eq!(metrics.positions.len(), 3);
    assert_eq!(metrics.positions[1].latency_ms, 400);
    assert!(metrics.positions[1].speed_sample_valid);
    assert_eq!(metrics.active_duration_ms, 700);
}

#[test]
fn uncompleted_error_is_recorded_and_does_not_advance_mastery() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("ab");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'x', origin, wall, 100);
    input(&mut engine, 'x', origin, wall, 200);
    let record = finish(&engine, origin, wall, 300);
    assert!(!record.is_completed());
    assert_eq!(record.error_count, 1);
    assert_eq!(record.accuracy, 0.5);
    let mut stats = UserStats::default();
    scorer::update_stats(&mut stats, &record);
    assert_eq!(stats.attempted_positions, 2);
    assert_eq!(stats.completed_chars, 1);
    assert_eq!(stats.total_duration_ms, 300);
    assert!(stats.command_progress.is_empty());
    assert_eq!(stats.daily_stats[0].error_positions, 1);
}

#[test]
fn idle_intervals_are_capped_and_invalidated_for_character_speed() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("abcd");
    for (ch, ms) in [('a', 0), ('b', 100), ('c', 60_100), ('d', 60_200)] {
        input(&mut engine, ch, origin, wall, ms);
    }
    let metrics = finish(&engine, origin, wall, 90_000).typing.unwrap();
    assert_eq!(metrics.active_duration_ms, 30_200);
    assert!(!metrics.positions[0].speed_sample_valid);
    assert!(metrics.positions[1].speed_sample_valid);
    assert!(!metrics.positions[2].speed_sample_valid);
    assert!(metrics.positions[3].speed_sample_valid);
}

#[test]
fn pauses_and_multiline_enter_never_count_as_effective_characters() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("ab\ncd");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'b', origin, wall, 100);
    input(&mut engine, '\n', origin, wall, 40_000);
    input(&mut engine, 'c', origin, wall, 50_000);
    engine.pause_at(origin + Duration::from_millis(50_100), wall + 50_100);
    engine.resume();
    input(&mut engine, 'd', origin, wall, 90_000);
    let record = finish(&engine, origin, wall, 99_000);
    assert_eq!(record.keystrokes.len(), 4);
    let metrics = record.typing.unwrap();
    assert_eq!(metrics.active_duration_ms, 200);
    assert_eq!(metrics.positions.len(), 4);
    assert!(!metrics.positions[2].speed_sample_valid);
    assert!(!metrics.positions[3].speed_sample_valid);
}

#[test]
fn midnight_splits_active_duration_and_preserves_input_local_dates() {
    let origin = Instant::now();
    let wall = Local
        .with_ymd_and_hms(2026, 8, 10, 23, 59, 59)
        .single()
        .unwrap()
        .timestamp_millis()
        + 500;
    let mut engine = TypingEngine::new("ab");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'b', origin, wall, 1_000);
    let stats = scorer::rebuild_from_history(&[finish(&engine, origin, wall, 1_000)]);
    assert_eq!(stats.daily_stats.len(), 2);
    for day in &stats.daily_stats {
        assert_eq!(day.total_duration_ms, 500);
        assert_eq!(day.completed_chars, 1);
        assert_eq!(day.avg_cpm, 120.0);
    }
    assert_eq!(stats.daily_stats[0].date, "2026-08-10");
    assert_eq!(stats.daily_stats[1].date, "2026-08-11");
}

#[test]
fn daily_speeds_and_accuracy_use_weighted_characters_and_positions() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut short = TypingEngine::new("ab");
    input(&mut short, 'a', origin, wall, 0);
    input(&mut short, 'b', origin, wall, 1_000);
    let mut long = TypingEngine::new("abcd");
    input(&mut long, 'x', origin, wall, 2_000);
    for (ch, ms) in [('a', 3_000), ('b', 4_000), ('c', 5_000), ('d', 6_000)] {
        input(&mut long, ch, origin, wall, ms);
    }
    let stats = scorer::rebuild_from_history(&[
        finish(&short, origin, wall, 1_000),
        finish(&long, origin, wall, 6_000),
    ]);
    assert_eq!(stats.total_duration_ms, 5_000);
    assert_eq!(stats.completed_chars, 6);
    assert!((stats.overall_avg_wpm - 14.4).abs() < 1e-10);
    assert!((stats.overall_avg_accuracy - 5.0 / 6.0).abs() < 1e-10);
    assert_eq!(stats.daily_stats[0].avg_cpm, 72.0);
}

#[test]
fn every_followtyping_mode_counts_and_other_modes_do_not_change_calendar() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("a ");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, ' ', origin, wall, 100);
    let base = finish(&engine, origin, wall, 100);
    let typing_modes = [
        RecordMode::Typing,
        RecordMode::LessonPractice,
        RecordMode::SymbolTyping,
        RecordMode::SystemTyping,
        RecordMode::ReviewTyping,
        RecordMode::ScenarioTyping,
    ];
    let excluded = [
        RecordMode::Dictation,
        RecordMode::SymbolPractice,
        RecordMode::ReviewDictation,
        RecordMode::ReviewCloze,
    ];
    let mut stats = UserStats::default();
    for (index, mode) in typing_modes.into_iter().chain(excluded).enumerate() {
        let mut record = base.clone();
        record.id = format!("record-{index}");
        record.mode = mode;
        scorer::update_stats(&mut stats, &record);
    }
    assert_eq!(stats.total_wpm_sessions, 6);
    assert_eq!(stats.total_duration_ms, 600);
    assert_eq!(stats.daily_stats[0].sessions_count, 6);
    assert_eq!(
        stats
            .char_stats
            .iter()
            .find(|stat| stat.char_key == ' ')
            .unwrap()
            .total_samples,
        6
    );
}

fn sample(latency_ms: u64) -> CharLatencySample {
    CharLatencySample {
        latency_ms,
        ..CharLatencySample::default()
    }
}

#[test]
fn character_window_hides_small_samples_and_winsorizes_latency() {
    assert!(scorer::smoothed_latency(&vec![sample(100); 9]).is_none());
    let mut samples = vec![sample(100); 8];
    samples.insert(0, sample(1));
    samples.push(sample(20_000));
    assert_eq!(scorer::smoothed_latency(&samples), Some(100.0));
    let samples = [vec![sample(100); 5], vec![sample(300); 5]].concat();
    assert_eq!(scorer::smoothed_latency(&samples), Some(200.0));
    assert!(scorer::smoothed_latency(&vec![sample(0); 50]).is_none());
}

#[test]
fn character_window_is_bounded_persistent_and_excludes_zero_intervals() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut records = Vec::new();
    for session in 0..65 {
        let mut engine = TypingEngine::new("aa");
        input(&mut engine, 'a', origin, wall, session * 1_000);
        input(
            &mut engine,
            'a',
            origin,
            wall,
            session * 1_000 + if session == 64 { 0 } else { 100 },
        );
        records.push(finish(&engine, origin, wall, session * 1_000 + 100));
    }
    let stats = scorer::rebuild_from_history(&records);
    let character = &stats.char_stats[0];
    assert_eq!(character.total_samples, 130);
    assert_eq!(character.recent_latencies.len(), 50);
    assert_eq!(character.avg_cpm, 600.0);
    let restored: UserStats =
        serde_json::from_str(&serde_json::to_string(&stats).unwrap()).unwrap();
    assert_eq!(restored, stats);
}

#[test]
fn record_ids_are_stable_and_replayed_snapshots_replace_partials() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("ab");
    input(&mut engine, 'a', origin, wall, 0);
    let partial = finish(&engine, origin, wall, 100);
    input(&mut engine, 'b', origin, wall, 200);
    let complete = finish(&engine, origin, wall, 300);
    assert_eq!(partial.id, complete.id);
    assert_eq!(engine.session_id(), complete.id);
    let stats = scorer::rebuild_from_history(&[partial, complete.clone(), complete]);
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.completed_chars, 2);
    assert_eq!(stats.command_progress[0].times_practiced, 1);
}

#[test]
fn historical_character_windows_do_not_include_future_samples() {
    let origin = Instant::now();
    let mut records = Vec::new();
    for (day, latency) in [(10, 100), (11, 200)] {
        let wall = Local
            .with_ymd_and_hms(2026, 8, day, 12, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        let mut engine = TypingEngine::new(&"a".repeat(11));
        for index in 0..11 {
            input(&mut engine, 'a', origin, wall, index * latency);
        }
        records.push(finish(&engine, origin, wall, 10 * latency));
    }
    let before = scorer::character_stats_as_of(&records, "2026-08-10");
    let after = scorer::character_stats_as_of(&records, "2026-08-11");
    assert_eq!(before[0].recent_latencies.len(), 10);
    assert_eq!(before[0].avg_cpm, 600.0);
    assert_eq!(after[0].recent_latencies.len(), 20);
    assert_eq!(after[0].avg_cpm, 400.0);
}

#[test]
fn future_day_retyping_preserves_historical_latency_and_error_windows() {
    let origin = Instant::now();
    let wall = Local
        .with_ymd_and_hms(2026, 8, 10, 23, 59, 58)
        .single()
        .unwrap()
        .timestamp_millis()
        + 500;
    let mut engine = TypingEngine::new(&"a".repeat(12));
    for index in 0..11 {
        input(&mut engine, 'a', origin, wall, index * 100);
    }
    let before = finish(&engine, origin, wall, 1_000);
    let day_ten = scorer::character_stats_as_of(&[before], "2026-08-10");
    assert_eq!(day_ten[0].recent_latencies.len(), 10);
    assert_eq!(day_ten[0].avg_cpm, 600.0);
    engine.backspace_at(origin + Duration::from_millis(1_600), wall + 1_600);
    input(&mut engine, 'x', origin, wall, 1_700);
    input(&mut engine, 'a', origin, wall, 1_800);
    let revisited = finish(&engine, origin, wall, 1_800);
    assert_eq!(
        scorer::character_stats_as_of(std::slice::from_ref(&revisited), "2026-08-10"),
        day_ten,
    );
    let day_eleven = scorer::character_stats_as_of(std::slice::from_ref(&revisited), "2026-08-11");
    assert_eq!(day_eleven[0].recent_latencies.len(), 10);
    assert_eq!(
        day_eleven[0].recent_latencies.last().unwrap().latency_ms,
        900
    );
    assert_eq!(day_eleven[0].total_errors, 1);
    // An idle revisit on the third day invalidates today's sample without
    // changing either earlier day's timestamp, latency or validity.
    let third_day = 86_401_600;
    engine.backspace_at(
        origin + Duration::from_millis(third_day),
        wall + third_day as i64,
    );
    let invalidated = finish(&engine, origin, wall, third_day);
    assert_eq!(
        scorer::character_stats_as_of(std::slice::from_ref(&invalidated), "2026-08-10"),
        day_ten,
    );
    assert_eq!(
        scorer::character_stats_as_of(std::slice::from_ref(&invalidated), "2026-08-11"),
        day_eleven,
    );
    assert_eq!(
        scorer::character_stats_as_of(std::slice::from_ref(&invalidated), "2026-08-12")[0]
            .recent_latencies
            .len(),
        9,
    );
    assert_eq!(
        invalidated
            .typing
            .as_ref()
            .unwrap()
            .position_day_snapshots
            .len(),
        2
    );
    let restored: SessionRecord =
        serde_json::from_str(&serde_json::to_string(&invalidated).unwrap()).unwrap();
    assert_eq!(restored.id, invalidated.id);
    assert_eq!(restored.typing, invalidated.typing);
    assert!((restored.accuracy - invalidated.accuracy).abs() < 1e-12);
    assert_eq!(
        scorer::character_stats_as_of(&[restored], "2026-08-10"),
        day_ten
    );
}

#[test]
fn same_day_revisits_do_not_grow_day_snapshots_or_duplicate_speed_samples() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("aaa");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'a', origin, wall, 100);
    for revisit in 1..=100 {
        let elapsed = revisit * 200;
        engine.backspace_at(
            origin + Duration::from_millis(elapsed),
            wall + elapsed as i64,
        );
        input(&mut engine, 'a', origin, wall, elapsed + 100);
    }
    let record = finish(&engine, origin, wall, 20_100);
    assert!(
        record
            .typing
            .as_ref()
            .unwrap()
            .position_day_snapshots
            .is_empty()
    );
    let stats = scorer::rebuild_from_history(std::slice::from_ref(&record));
    assert_eq!(stats.char_stats[0].total_samples, 2);
    assert_eq!(stats.char_stats[0].recent_latencies.len(), 1);
    assert_eq!(
        scorer::character_stats_as_of(&[record], "2026-08-10")[0]
            .recent_latencies
            .len(),
        1
    );
}

#[test]
fn later_day_pause_preserves_prior_incomplete_position_state() {
    let origin = Instant::now();
    let wall = Local
        .with_ymd_and_hms(2026, 8, 10, 23, 59, 59)
        .single()
        .unwrap()
        .timestamp_millis()
        + 500;
    let mut engine = TypingEngine::new("abc");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'x', origin, wall, 100);
    let prior = scorer::character_stats_as_of(&[finish(&engine, origin, wall, 100)], "2026-08-10");
    engine.pause_at(origin + Duration::from_millis(600), wall + 600);
    engine.resume();
    input(&mut engine, 'b', origin, wall, 700);
    input(&mut engine, 'c', origin, wall, 800);
    let record = finish(&engine, origin, wall, 800);
    assert_eq!(
        scorer::character_stats_as_of(std::slice::from_ref(&record), "2026-08-10"),
        prior
    );
    let snapshots = &record.typing.as_ref().unwrap().position_day_snapshots;
    assert_eq!(snapshots.len(), 1);
    assert!(!snapshots[0].completed);
    assert!(snapshots[0].speed_sample_valid);
}

#[test]
fn completion_day_uses_saved_local_dates_even_when_timestamps_render_on_another_day() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("ab");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'b', origin, wall, 100);
    let mut record = finish(&engine, origin, wall, 100);
    assert_eq!(record.typing.as_ref().unwrap().completed_date, "2026-08-10");
    // Simulate replay in another timezone: epoch timestamps now format as
    // August 11, while the original input-time local dates remain August 10.
    record.finished_at = Local
        .with_ymd_and_hms(2026, 8, 11, 0, 0, 0)
        .single()
        .unwrap()
        .timestamp_millis();
    let stats = scorer::rebuild_from_history(std::slice::from_ref(&record));
    assert_eq!(stats.daily_stats.len(), 1);
    assert_eq!(stats.daily_stats[0].date, "2026-08-10");
    assert_eq!(stats.daily_stats[0].entries[0].completed_count, 1);
    // Earlier version-2 records recover the fixed date from their latest
    // completed position instead of changing it to the current timezone.
    record.typing.as_mut().unwrap().completed_date.clear();
    assert_eq!(
        scorer::rebuild_from_history(&[record]).daily_stats[0].entries[0].completed_count,
        1
    );
}

#[test]
fn old_version_two_records_read_without_new_checkpoint_fields() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("aa");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'a', origin, wall, 100);
    let record = finish(&engine, origin, wall, 100);
    let expected = scorer::rebuild_from_history(std::slice::from_ref(&record));
    let mut old = serde_json::to_value(&record).unwrap();
    let metrics = old["typing"].as_object_mut().unwrap();
    metrics.remove("completed_date");
    metrics.remove("position_day_snapshots");
    for position in metrics["positions"].as_array_mut().unwrap() {
        position.as_object_mut().unwrap().remove("updated_date");
    }
    let restored: SessionRecord = serde_json::from_value(old).unwrap();
    assert!(
        restored
            .typing
            .as_ref()
            .unwrap()
            .position_day_snapshots
            .is_empty()
    );
    assert_eq!(
        scorer::rebuild_from_history(std::slice::from_ref(&restored)),
        expected
    );
    assert_eq!(
        scorer::character_stats_as_of(&[restored], "2026-08-10")[0]
            .recent_latencies
            .len(),
        1
    );
}

#[test]
fn exactly_thirty_seconds_remains_valid_but_longer_idle_does_not() {
    let (origin, wall) = (Instant::now(), start_timestamp());
    let mut engine = TypingEngine::new("abc");
    input(&mut engine, 'a', origin, wall, 0);
    input(&mut engine, 'b', origin, wall, 30_000);
    input(&mut engine, 'c', origin, wall, 60_001);
    let record = finish(&engine, origin, wall, 60_001);
    let metrics = record.typing.unwrap();
    assert_eq!(metrics.active_duration_ms, 60_000);
    assert!(metrics.positions[1].speed_sample_valid);
    assert!(!metrics.positions[2].speed_sample_valid);
}
