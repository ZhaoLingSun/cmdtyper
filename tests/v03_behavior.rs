use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use cmdtyper::app::{App, AppState, SystemPhase};
use cmdtyper::core::scorer;
use cmdtyper::data::models::{
    Category, CommandLesson, Difficulty, Exercise, ExerciseKind, RecordMode, SessionRecord,
    SymbolTopic, SystemCommand, TypingDisplayMode, UserStats,
};
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

fn fresh_app(test_name: &str) -> App {
    let _guard = test_lock().lock().expect("lock poisoned");

    let user_dir = unique_temp_dir(&format!("cmdtyper-v03-{test_name}-user"));
    fs::create_dir_all(&user_dir).expect("create user dir");

    // SAFETY: tests serialize env var mutation with a global mutex.
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", project_data_dir());
        env::set_var("CMDTYPER_USER_DIR", &user_dir);
    }

    let app = App::new().expect("app should initialize");

    let _ = fs::remove_dir_all(&user_dir);
    app
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
fn typing_display_mode_default_is_standard() {
    assert_eq!(TypingDisplayMode::default(), TypingDisplayMode::Standard);

    let app = fresh_app("typing-mode-default");
    assert_eq!(app.typing_mode, TypingDisplayMode::Standard);
}

#[test]
fn typing_mode_cycles_standard_detailed_terminal_standard() {
    let mut app = fresh_app("typing-mode-cycle");
    app.state = AppState::Typing;

    app.typing_mode = TypingDisplayMode::Standard;
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.typing_mode, TypingDisplayMode::Detailed);

    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.typing_mode, TypingDisplayMode::Terminal);

    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.typing_mode, TypingDisplayMode::Standard);
}

#[test]
fn filtering_beginner_returns_only_beginner_commands() {
    let app = fresh_app("filter-beginner");
    let filtered = app.filtered_commands(Some(Difficulty::Beginner), None);

    assert!(!filtered.is_empty());
    assert!(filtered
        .iter()
        .all(|command| command.difficulty == Difficulty::Beginner));
}

#[test]
fn filtering_by_specific_category_returns_only_that_category() {
    let app = fresh_app("filter-category");
    let filtered = app.filtered_commands(None, Some(Category::FileOps));

    assert!(!filtered.is_empty());
    assert!(filtered
        .iter()
        .all(|command| command.category == Category::FileOps));
}

#[test]
fn filtering_by_difficulty_and_category_narrows_correctly() {
    let app = fresh_app("filter-combined");

    let expected = app
        .commands
        .iter()
        .filter(|c| c.difficulty == Difficulty::Beginner && c.category == Category::FileOps)
        .count();

    let filtered = app.filtered_commands(Some(Difficulty::Beginner), Some(Category::FileOps));

    assert_eq!(filtered.len(), expected);
    assert!(filtered
        .iter()
        .all(|c| c.difficulty == Difficulty::Beginner && c.category == Category::FileOps));
}

#[test]
fn filtering_none_none_returns_all_273_commands() {
    let app = fresh_app("filter-all");
    let filtered = app.filtered_commands(None, None);

    assert_eq!(app.commands.len(), 273);
    assert_eq!(filtered.len(), 273);
}

#[test]
fn exercise_kind_typing_deserializes_from_typing() {
    let ex: Exercise = toml::from_str(
        r#"
prompt = "type ls"
answers = ["ls"]
kind = "typing"
"#,
    )
    .expect("exercise parse");

    assert_eq!(ex.kind, Some(ExerciseKind::Typing));
}

#[test]
fn exercise_kind_dictation_deserializes_from_dictation() {
    let ex: Exercise = toml::from_str(
        r#"
prompt = "type ls"
answers = ["ls"]
kind = "dictation"
"#,
    )
    .expect("exercise parse");

    assert_eq!(ex.kind, Some(ExerciseKind::Dictation));
}

#[test]
fn exercise_kind_missing_defaults_to_none_dictation_compat() {
    let ex: Exercise = toml::from_str(
        r#"
prompt = "type ls"
answers = ["ls"]
"#,
    )
    .expect("exercise parse");

    assert_eq!(ex.kind, None);
}

#[test]
fn deep_explanation_loads_when_present_and_none_when_absent() {
    let lesson: CommandLesson = toml::from_str(
        r#"
[meta]
command = "ls"
category = "file_ops"
difficulty = "beginner"

[overview]
summary = "list"
explanation = "list files"

[syntax]
basic = "ls"

[[examples]]
level = 1
command = "ls"
summary = "basic"
deep_explanation = "line-by-line deep explanation"
"#,
    )
    .expect("lesson parse");
    assert_eq!(
        lesson.examples[0].deep_explanation.as_deref(),
        Some("line-by-line deep explanation")
    );

    let symbol: SymbolTopic = toml::from_str(
        r#"
[meta]
id = "pipe"
topic = "pipe"
description = "desc"
difficulty = "basic"

[[symbols]]
id = "s1"
char_repr = "|"
name = "pipe"
summary = "sum"
explanation = "exp"

[[symbols.examples]]
command = "ls | wc -l"
explanation = "pipe"
"#,
    )
    .expect("symbol parse");
    assert_eq!(symbol.symbols[0].examples[0].deep_explanation, None);

    let sys_cmd: SystemCommand = toml::from_str(
        r#"
command = "systemctl status"
summary = "status"
"#,
    )
    .expect("system command parse");
    assert_eq!(sys_cmd.deep_explanation, None);
}

#[test]
fn symbol_typing_mode_is_wpm_bearing_in_scorer() {
    let mut stats = UserStats::default();

    let record = sample_record(
        "symbol-typing-1",
        "symbol-cmd-1",
        RecordMode::SymbolTyping,
        Difficulty::Basic,
        88.0,
        0.97,
    );

    scorer::update_stats(&mut stats, &record);

    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_wpm_sessions, 1);
    assert!(approx_eq(stats.overall_avg_wpm, 88.0, 1e-12));
}

#[test]
fn system_typing_mode_is_wpm_bearing_in_scorer() {
    let mut stats = UserStats::default();

    let record = sample_record(
        "system-typing-1",
        "system-cmd-1",
        RecordMode::SystemTyping,
        Difficulty::Advanced,
        76.0,
        0.92,
    );

    scorer::update_stats(&mut stats, &record);

    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_wpm_sessions, 1);
    assert!(approx_eq(stats.overall_avg_wpm, 76.0, 1e-12));
}


#[test]
fn lesson_practice_allows_typing_d_before_completion() {
    let mut app = fresh_app("lesson-practice-d-input");
    let cat_idx = app
        .get_lesson_categories()
        .iter()
        .position(|c| *c == Category::FileOps)
        .expect("file ops category exists");

    let lessons = app.get_lessons_for_category(Category::FileOps);
    let command_index = lessons
        .iter()
        .position(|lesson| {
            lesson
                .examples
                .iter()
                .any(|example| example.command.contains('d'))
        })
        .expect("need a lesson example containing d");

    let example_index = lessons[command_index]
        .examples
        .iter()
        .position(|example| example.command.contains('d'))
        .expect("example containing d");

    let example_command = lessons[command_index]
        .examples[example_index]
        .command
        .clone();
    let d_pos = example_command
        .chars()
        .position(|c| c == 'd')
        .expect("d position should exist");

    app.state = AppState::CommandLessonPractice {
        category_index: cat_idx,
        command_index,
        example_index,
    };
    app.typing_engine.reset(&example_command);

    for ch in example_command.chars().take(d_pos + 1) {
        app.handle_key(key(KeyCode::Char(ch)));
    }

    assert_eq!(app.typing_engine.cursor, d_pos + 1);
    assert_eq!(
        app.state,
        AppState::CommandLessonPractice {
            category_index: cat_idx,
            command_index,
            example_index,
        }
    );
}


#[test]
fn system_typing_allows_typing_d_before_completion() {
    let mut app = fresh_app("system-typing-d-input");

    let mut found = None;
    'outer: for (topic_index, topic) in app.system_topics.iter().enumerate() {
        for (section_index, section) in topic.sections.iter().enumerate() {
            for (command_idx, command) in section.commands.iter().enumerate() {
                if command.command.contains('d') {
                    found = Some((topic_index, section_index, command_idx, command.command.clone()));
                    break 'outer;
                }
            }
        }
    }

    let (topic_index, section_index, command_idx, command_str) =
        found.expect("need a system command containing d");
    let d_pos = command_str
        .chars()
        .position(|c| c == 'd')
        .expect("d position should exist");

    app.state = AppState::SystemLesson {
        topic_index,
        section_index,
        phase: SystemPhase::TypingPractice { command_idx },
    };
    app.typing_engine.reset(&command_str);

    for ch in command_str.chars().take(d_pos + 1) {
        app.handle_key(key(KeyCode::Char(ch)));
    }

    assert_eq!(app.typing_engine.cursor, d_pos + 1);
    assert_eq!(
        app.state,
        AppState::SystemLesson {
            topic_index,
            section_index,
            phase: SystemPhase::TypingPractice { command_idx },
        }
    );
}
