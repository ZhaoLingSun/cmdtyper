use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cmdtyper::app::{App, AppState};
use cmdtyper::core::engine::TypingEngine;
use cmdtyper::core::matcher::{MatchResult, check, normalize};
use cmdtyper::core::scorer;
use cmdtyper::data::models::{Difficulty, RecordMode, SessionRecord, UserStats};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn approx_eq(left: f64, right: f64, eps: f64) -> bool {
    (left - right).abs() <= eps
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn project_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn sample_record(
    id: &str,
    command_id: &str,
    mode: RecordMode,
    difficulty: Difficulty,
    wpm: f64,
    accuracy: f64,
) -> SessionRecord {
    SessionRecord {
        id: id.to_string(),
        command_id: command_id.to_string(),
        mode,
        started_at: 1_000,
        finished_at: 2_000,
        wpm,
        cpm: wpm * 5.0,
        accuracy,
        difficulty,
        ..SessionRecord::default()
    }
}

#[test]
fn typing_enter_is_ignored_when_incomplete() {
    let _guard = test_lock().lock().expect("lock poisoned");

    let user_dir = unique_temp_dir("cmdtyper-wave5-user-enter");
    fs::create_dir_all(&user_dir).expect("create user dir");

    // SAFETY: tests serialize env var mutation with a global mutex.
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", project_data_dir());
        env::set_var("CMDTYPER_USER_DIR", &user_dir);
    }

    let mut app = App::new().expect("app should initialize");

    let mut cmd = app.commands.first().cloned().expect("at least one command");
    cmd.command = "ab".to_string();
    cmd.simulated_output = None;

    app.typing_commands = vec![cmd.clone()];
    app.typing_index = 0;
    app.typing_showing_output = false;
    app.typing_round_records.clear();
    app.state = AppState::Typing;
    app.typing_engine.reset(&cmd.command);

    app.handle_key(key(KeyCode::Char('a')));
    assert!(!app.typing_engine.is_complete());

    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.state, AppState::Typing);
    assert_eq!(app.typing_index, 0);
    assert!(!app.typing_showing_output);
    assert!(app.typing_round_records.is_empty());
    assert!(app.history.is_empty());

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn typing_engine_wpm_freezes_after_completion() {
    let mut engine = TypingEngine::new("a");
    let _ = engine.input('a');
    assert!(engine.is_complete());

    let wpm_after_finish = engine.current_wpm();
    thread::sleep(Duration::from_millis(120));
    let wpm_later = engine.current_wpm();

    assert!(approx_eq(wpm_after_finish, wpm_later, 1e-9));
}

#[test]
fn typing_engine_backspace_works_and_resets_completion() {
    let mut engine = TypingEngine::new("ab");
    let _ = engine.input('a');
    let _ = engine.input('b');
    assert!(engine.is_complete());
    assert!(engine.completed_at.is_some());

    engine.backspace();

    assert_eq!(engine.cursor, 1);
    assert!(!engine.is_complete());
    assert_eq!(engine.keystrokes.len(), 1);
    assert!(engine.completed_at.is_none());
}

#[test]
fn typing_engine_finish_records_difficulty_and_mode() {
    let mut engine = TypingEngine::new("ab");
    let _ = engine.input('a');
    let _ = engine.input('b');

    let record = engine.finish("finish-check", Difficulty::Practical, RecordMode::Dictation);

    assert_eq!(record.command_id, "finish-check");
    assert_eq!(record.difficulty, Difficulty::Practical);
    assert_eq!(record.mode, RecordMode::Dictation);
}

#[test]
fn matcher_normalize_and_check_behave_correctly() {
    assert_eq!(normalize("  LS   -LA\t/VAR/LOG  "), "ls -la /var/log");

    let answers = vec!["ls -la /var/log".to_string(), "pwd".to_string()];
    assert_eq!(check("pwd", &answers), MatchResult::Exact(1));
    assert_eq!(
        check(" ls   -la /VAR/LOG ", &answers),
        MatchResult::Normalized(0)
    );

    match check("ls /tmp", &answers) {
        MatchResult::NoMatch { closest, diff } => {
            assert_eq!(closest, "ls -la /var/log");
            assert!(!diff.is_empty());
        }
        other => panic!("expected NoMatch, got {other:?}"),
    }
}

#[test]
fn data_path_reads_cmdtyper_data_dir_env_var() {
    let _guard = test_lock().lock().expect("lock poisoned");

    let empty_data_dir = unique_temp_dir("cmdtyper-wave5-empty-data");
    let user_dir = unique_temp_dir("cmdtyper-wave5-user-data-path");
    fs::create_dir_all(&empty_data_dir).expect("create empty data dir");
    fs::create_dir_all(&user_dir).expect("create user dir");

    // SAFETY: tests serialize env var mutation with a global mutex.
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", &empty_data_dir);
        env::set_var("CMDTYPER_USER_DIR", &user_dir);
    }

    let app = App::new().expect("app should initialize with empty data dir from env");
    assert!(app.commands.is_empty());
    assert!(app.lessons.is_empty());
    assert!(app.symbol_topics.is_empty());
    assert!(app.system_topics.is_empty());

    let _ = fs::remove_dir_all(&empty_data_dir);
    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn stats_policy_typing_mode_includes_wpm_in_aggregation() {
    let mut stats = UserStats::default();
    let record = sample_record(
        "typing-1",
        "cmd-1",
        RecordMode::Typing,
        Difficulty::Basic,
        72.0,
        0.95,
    );

    scorer::update_stats(&mut stats, &record);

    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_wpm_sessions, 1);
    assert!(approx_eq(stats.overall_avg_wpm, 72.0, 1e-12));
}

#[test]
fn stats_policy_dictation_mode_excludes_wpm_from_aggregation() {
    let mut stats = UserStats::default();

    let typing = sample_record(
        "typing-1",
        "cmd-1",
        RecordMode::Typing,
        Difficulty::Basic,
        60.0,
        0.95,
    );
    scorer::update_stats(&mut stats, &typing);

    let wpm_before = stats.overall_avg_wpm;
    let wpm_sessions_before = stats.total_wpm_sessions;

    let dictation = sample_record(
        "dictation-1",
        "cmd-2",
        RecordMode::Dictation,
        Difficulty::Advanced,
        5.0,
        0.80,
    );
    scorer::update_stats(&mut stats, &dictation);

    assert_eq!(stats.total_sessions, 2);
    assert_eq!(stats.total_wpm_sessions, wpm_sessions_before);
    assert!(approx_eq(stats.overall_avg_wpm, wpm_before, 1e-12));
}

#[test]
fn stats_policy_difficulty_is_used_not_default_beginner() {
    let mut stats = UserStats::default();

    // Practical target_attempts = 10, so first perfect practice should be mastery 0.1.
    let practical_record = sample_record(
        "practical-1",
        "same-command",
        RecordMode::Typing,
        Difficulty::Practical,
        40.0,
        1.0,
    );
    scorer::update_stats(&mut stats, &practical_record);

    let progress = stats
        .command_progress
        .iter()
        .find(|p| p.command_id == "same-command")
        .expect("command progress should exist");

    assert!(approx_eq(progress.mastery, 0.1, 1e-12));
}
