use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use cmdtyper::app::{
    App, AppState, ReviewExerciseKind, ReviewPhase, ReviewSource, SymbolPhase, SymbolPracticeState,
    SystemPhase,
};
use cmdtyper::core::matcher::MatchResult;
use cmdtyper::core::scorer;
use cmdtyper::data::models::{
    Category, CommandLesson, CommandProgress, ConfigFile, ConfigLesson, DeepSource, Difficulty,
    Exercise, ExerciseKind, RecordMode, ResumeScreen, ResumeState, SessionRecord, SymbolTopic,
    SystemCommand, TopicTrainingLevel, TypingDisplayMode, UserStats,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

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

fn write_resume(user_dir: &Path, resume: &ResumeState) {
    fs::write(
        user_dir.join("resume_state.json"),
        serde_json::to_vec_pretty(resume).expect("serialize resume state"),
    )
    .expect("write resume state");
}

fn load_app_from_user_dir(user_dir: &Path) -> App {
    // SAFETY: callers hold the integration-test environment lock.
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", project_data_dir());
        env::set_var("CMDTYPER_USER_DIR", user_dir);
    }
    App::new().expect("app should initialize")
}

fn render_app(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| cmdtyper::ui::render(frame, app))
        .expect("render app");
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .concat()
}

fn system_lesson_scroll(app: &App) -> usize {
    match &app.state {
        AppState::SystemLesson { scroll, .. } => *scroll,
        state => panic!("expected system lesson state, got {state:?}"),
    }
}

fn type_current_target(app: &mut App) {
    let target: Vec<char> = app.typing_engine.target.clone();
    for character in target {
        app.handle_key(key(KeyCode::Char(character)));
    }
    assert!(app.typing_engine.is_complete());
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
    assert!(
        filtered
            .iter()
            .all(|command| command.difficulty == Difficulty::Beginner)
    );
}

#[test]
fn filtering_by_specific_category_returns_only_that_category() {
    let app = fresh_app("filter-category");
    let filtered = app.filtered_commands(None, Some(Category::FileOps));

    assert!(!filtered.is_empty());
    assert!(
        filtered
            .iter()
            .all(|command| command.category == Category::FileOps)
    );
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
    assert!(
        filtered
            .iter()
            .all(|c| c.difficulty == Difficulty::Beginner && c.category == Category::FileOps)
    );
}

#[test]
fn filtering_none_none_returns_all_commands() {
    let app = fresh_app("filter-all");
    let filtered = app.filtered_commands(None, None);

    assert!(!app.commands.is_empty());
    assert_eq!(filtered.len(), app.commands.len());
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

    let example_command = lessons[command_index].examples[example_index]
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
                    found = Some((
                        topic_index,
                        section_index,
                        command_idx,
                        command.command.clone(),
                    ));
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
        scroll: 0,
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
            scroll: 0,
        }
    );
}

#[test]
fn detect_data_dir_falls_back_to_repo_path_when_env_missing() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-detect-data");
    fs::create_dir_all(&user_dir).expect("create user dir");
    unsafe {
        env::remove_var("CMDTYPER_DATA_DIR");
        env::set_var("CMDTYPER_USER_DIR", &user_dir);
    }
    let app = App::new().expect("app should initialize without CMDTYPER_DATA_DIR");
    assert!(!app.commands.is_empty());
    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn missing_and_explicit_home_resume_restore_home() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-home");
    fs::create_dir_all(&user_dir).expect("create user dir");

    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::Home);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::Home,
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::Home);

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn ctrl_c_saves_pre_quitting_resume_state_for_restart() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-ctrl-c");
    fs::create_dir_all(&user_dir).expect("create user dir");

    let mut app = load_app_from_user_dir(&user_dir);
    let topic_index = app.command_training_topics.len() - 1;
    let topic_id = app.command_training_topics[topic_index].id.clone();
    app.review_topics_index = topic_index;
    app.topic_training_level = TopicTrainingLevel::L3;
    app.state = AppState::ReviewTopics;

    app.handle_key_and_save_resume(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert_eq!(app.state, AppState::Quitting);
    drop(app);

    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::ReviewTopics);
    assert_eq!(app.review_topics_index, topic_index);
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L3);
    let saved: ResumeState = serde_json::from_slice(
        &fs::read(user_dir.join("resume_state.json")).expect("read resume state"),
    )
    .expect("parse resume state");
    assert_eq!(saved.review_topic_id.as_deref(), Some(topic_id.as_str()));

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn review_topics_summary_and_practice_resume_at_stable_topic() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-review");
    fs::create_dir_all(&user_dir).expect("create user dir");

    let mut app = load_app_from_user_dir(&user_dir);
    let topic_index = app.command_training_topics.len() - 1;
    let topic_id = app.command_training_topics[topic_index].id.clone();
    app.review_topics_index = topic_index;
    app.topic_training_level = TopicTrainingLevel::L3;
    app.state = AppState::ReviewTopics;
    app.save_resume_state();
    drop(app);

    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::ReviewTopics);
    assert_eq!(app.review_topics_index, topic_index);
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L3);
    let saved: ResumeState = serde_json::from_slice(
        &fs::read(user_dir.join("resume_state.json")).expect("read resume state"),
    )
    .expect("parse resume state");
    assert_eq!(saved.review_topic_id.as_deref(), Some(topic_id.as_str()));
    assert_eq!(saved.topic_training_level, TopicTrainingLevel::L3);
    drop(app);

    let mut app = load_app_from_user_dir(&user_dir);
    app.topic_training_level = TopicTrainingLevel::L5;
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id.clone()),
        phase: ReviewPhase::Summary,
    };
    app.save_resume_state();
    drop(app);

    let mut app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::Review {
            source: ReviewSource::CommandTopic(topic_id.clone()),
            phase: ReviewPhase::Summary,
        }
    );
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L5);

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(
        app.state,
        AppState::Review {
            phase: ReviewPhase::Practice,
            ..
        }
    ));
    app.handle_key(key(KeyCode::Char('x')));
    app.save_resume_state();
    drop(app);

    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::Review {
            source: ReviewSource::CommandTopic(topic_id),
            phase: ReviewPhase::Summary,
        }
    );
    assert!(app.review_practice.exercises.is_empty());
    assert!(app.review_practice.dictation_input.is_empty());
    assert!(app.review_practice.cloze_input.is_empty());

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn dictation_resume_rebuilds_commands_and_accepts_submission() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-dictation");
    fs::create_dir_all(&user_dir).expect("create user dir");
    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::Dictation,
            ..ResumeState::default()
        },
    );

    let mut app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::Dictation);
    assert!(!app.dictation_commands.is_empty());
    assert!(app.current_dictation_command().is_some());

    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.state, AppState::Dictation);
    assert!(app.dictation_submitted);

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn stable_resume_ids_override_stale_indices() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-stable-ids");
    fs::create_dir_all(&user_dir).expect("create user dir");

    let seed = load_app_from_user_dir(&user_dir);
    let categories = seed.get_lesson_categories();
    let (
        lesson_category_index,
        lesson_command_index,
        lesson_command,
        example_index,
        example_command,
    ) = categories
        .iter()
        .enumerate()
        .rev()
        .find_map(|(category_index, category)| {
            let lessons = seed.get_lessons_for_category(*category);
            lessons
                .iter()
                .enumerate()
                .rev()
                .find(|(_, lesson)| !lesson.examples.is_empty())
                .map(|(command_index, lesson)| {
                    let example_index = lesson.examples.len() - 1;
                    (
                        category_index,
                        command_index,
                        lesson.meta.command.clone(),
                        example_index,
                        lesson.examples[example_index].command.clone(),
                    )
                })
        })
        .expect("need a lesson with an example");
    let symbol_topic = seed
        .symbol_topics
        .iter()
        .enumerate()
        .rev()
        .find(|(_, topic)| !topic.symbols.is_empty())
        .map(|(topic_index, topic)| {
            let symbol_index = topic.symbols.len() - 1;
            (
                topic_index,
                topic.meta.id.clone(),
                symbol_index,
                topic.symbols[symbol_index].id.clone(),
            )
        })
        .expect("need a symbol topic");
    let system_topic = seed
        .system_topics
        .iter()
        .enumerate()
        .rev()
        .find(|(_, topic)| !topic.sections.is_empty())
        .map(|(topic_index, topic)| {
            let section_index = topic.sections.len() - 1;
            (
                topic_index,
                topic.meta.id.clone(),
                section_index,
                topic.sections[section_index].id.clone(),
            )
        })
        .expect("need a system topic");
    drop(seed);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::CommandLessonPractice,
            category_index: usize::MAX,
            command_index: usize::MAX,
            example_index,
            lesson_command: Some(lesson_command),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::CommandLessonPractice {
            category_index: lesson_category_index,
            command_index: lesson_command_index,
            example_index,
        }
    );
    assert_eq!(
        app.typing_engine.target.iter().collect::<String>(),
        example_command
    );
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SymbolExplain,
            topic_index: usize::MAX,
            symbol_index: usize::MAX,
            symbol_topic_id: Some(symbol_topic.1),
            symbol_id: Some(symbol_topic.3),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::SymbolLesson {
            topic_index: symbol_topic.0,
            symbol_index: symbol_topic.2,
            phase: SymbolPhase::Explain,
        }
    );
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SystemDetail,
            topic_index: usize::MAX,
            section_index: usize::MAX,
            system_topic_id: Some(system_topic.1),
            system_section_id: Some(system_topic.3),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::SystemLesson {
            topic_index: system_topic.0,
            section_index: system_topic.2,
            phase: SystemPhase::Detail,
            scroll: 0,
        }
    );

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn legacy_indices_fallback_and_invalid_resume_locations_are_safe() {
    let _guard = test_lock().lock().expect("lock poisoned");
    let user_dir = unique_temp_dir("cmdtyper-resume-invalid");
    fs::create_dir_all(&user_dir).expect("create user dir");

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SymbolExplain,
            topic_index: 0,
            symbol_index: 0,
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert!(matches!(
        app.state,
        AppState::SymbolLesson {
            topic_index: 0,
            symbol_index: 0,
            phase: SymbolPhase::Explain
        }
    ));
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::CommandLessonPractice,
            category_index: 0,
            command_index: 0,
            example_index: 0,
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    let legacy_lesson_target = app.get_lessons_for_category(app.get_lesson_categories()[0])[0]
        .examples[0]
        .command
        .clone();
    assert_eq!(
        app.state,
        AppState::CommandLessonPractice {
            category_index: 0,
            command_index: 0,
            example_index: 0,
        }
    );
    assert_eq!(
        app.typing_engine.target.iter().collect::<String>(),
        legacy_lesson_target
    );
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SystemDetail,
            topic_index: 0,
            section_index: 0,
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::SystemLesson {
            topic_index: 0,
            section_index: 0,
            phase: SystemPhase::Detail,
            scroll: 0,
        }
    );
    drop(app);

    let seed = load_app_from_user_dir(&user_dir);
    let legacy_review_index = seed.command_training_topics.len() - 1;
    let legacy_review_id = seed.command_training_topics[legacy_review_index].id.clone();
    drop(seed);
    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::ReviewSummary,
            topic_index: legacy_review_index,
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::Review {
            source: ReviewSource::CommandTopic(legacy_review_id),
            phase: ReviewPhase::Summary,
        }
    );
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::CommandLessonOverview,
            category_index: 0,
            command_index: 0,
            lesson_command: Some("missing-lesson".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::CommandTopics);
    assert_eq!(app.command_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::CommandLessonPractice,
            category_index: 0,
            command_index: 0,
            example_index: 0,
            lesson_command: Some("missing-lesson".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::CommandTopics);
    assert!(app.typing_engine.target.is_empty());
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SymbolExplain,
            topic_index: 0,
            symbol_index: 0,
            symbol_topic_id: Some("missing-topic".to_string()),
            symbol_id: Some("missing-symbol".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::SymbolTopics);
    assert_eq!(app.symbol_topics_index, 0);
    drop(app);

    let seed = load_app_from_user_dir(&user_dir);
    let valid_symbol_topic_id = seed.symbol_topics[0].meta.id.clone();
    drop(seed);
    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SymbolExplain,
            topic_index: usize::MAX,
            symbol_index: 0,
            symbol_topic_id: Some(valid_symbol_topic_id),
            symbol_id: Some("missing-symbol".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::SymbolTopics);
    assert_eq!(app.symbol_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SystemDetail,
            topic_index: 0,
            section_index: 0,
            system_topic_id: Some("missing-topic".to_string()),
            system_section_id: Some("missing-section".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::SystemTopics);
    assert_eq!(app.system_topics_index, 0);
    drop(app);

    let seed = load_app_from_user_dir(&user_dir);
    let valid_system_topic_id = seed.system_topics[0].meta.id.clone();
    drop(seed);
    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SystemDetail,
            topic_index: usize::MAX,
            section_index: 0,
            system_topic_id: Some(valid_system_topic_id),
            system_section_id: Some("missing-section".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(
        app.state,
        AppState::SystemLesson {
            topic_index: 0,
            section_index: 0,
            phase: SystemPhase::Overview,
            scroll: 0,
        }
    );
    drop(app);

    let seed = load_app_from_user_dir(&user_dir);
    let review_index = seed.command_training_topics.len() - 1;
    drop(seed);
    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::ReviewTopics,
            topic_index: review_index,
            review_topic_id: Some("missing-review-topic".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::ReviewTopics);
    assert_eq!(app.review_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::ReviewSummary,
            topic_index: review_index,
            review_topic_id: Some("missing-review-topic".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::ReviewTopics);
    assert_eq!(app.review_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::CommandLessonOverview,
            category_index: usize::MAX,
            command_index: usize::MAX,
            lesson_command: Some("missing-lesson".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::CommandTopics);
    assert!(app.command_topics_index < app.get_lesson_categories().len());
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SymbolExplain,
            topic_index: usize::MAX,
            symbol_index: usize::MAX,
            symbol_topic_id: Some("missing-topic".to_string()),
            symbol_id: Some("missing-symbol".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::SymbolTopics);
    assert_eq!(app.symbol_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::SystemDetail,
            topic_index: usize::MAX,
            section_index: usize::MAX,
            system_topic_id: Some("missing-topic".to_string()),
            system_section_id: Some("missing-section".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::SystemTopics);
    assert_eq!(app.system_topics_index, 0);
    drop(app);

    write_resume(
        &user_dir,
        &ResumeState {
            screen: ResumeScreen::ReviewTopics,
            topic_index: usize::MAX,
            review_topic_id: Some("missing-review-topic".to_string()),
            ..ResumeState::default()
        },
    );
    let app = load_app_from_user_dir(&user_dir);
    assert_eq!(app.state, AppState::ReviewTopics);
    assert_eq!(app.review_topics_index, 0);

    let _ = fs::remove_dir_all(&user_dir);
}

#[test]
fn learn_hub_selection_stops_at_visible_index_seven() {
    let mut app = fresh_app("learn-hub-index-max");
    app.state = AppState::LearnHub;
    app.learn_hub_index = 7;

    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.learn_hub_index, 7);

    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.state, AppState::ReviewTopics);
}

#[test]
fn small_topic_menus_keep_last_selection_visible_with_continuation_hint() {
    let mut app = fresh_app("small-topic-menu-render");

    app.state = AppState::SymbolTopics;
    app.symbol_topics_index = app.symbol_topics.len() - 1;
    let selected_symbol_topic = app.symbol_topics[app.symbol_topics_index]
        .meta
        .topic
        .clone();
    let symbol_render = render_app(&app, 48, 9);
    let compact_symbol_render: String = symbol_render
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let compact_symbol_topic: String = selected_symbol_topic
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_symbol_render.contains(&compact_symbol_topic));
    if app.symbol_topics.len() > 2 {
        assert!(compact_symbol_render.contains("上方还有专题"));
    }

    app.state = AppState::SystemTopics;
    app.system_topics_index = app.system_topics.len() - 1;
    let selected_system_topic = app.system_topics[app.system_topics_index]
        .meta
        .topic
        .clone();
    let system_render = render_app(&app, 48, 9);
    let compact_system_render: String = system_render
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let compact_system_topic: String = selected_system_topic
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_system_render.contains(&compact_system_topic));
    if app.system_topics.len() > 2 {
        assert!(compact_system_render.contains("上方还有专题"));
    }
}

#[test]
fn narrow_learn_hub_and_command_topics_keep_last_selection_visible() {
    let mut app = fresh_app("narrow-learn-command-menus");

    app.state = AppState::LearnHub;
    app.learn_hub_index = 7;
    let learn_hub_render = render_app(&app, 40, 6);
    let compact_learn_hub: String = learn_hub_render
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_learn_hub.contains("专题训练"));

    let categories = app.get_lesson_categories();
    app.state = AppState::CommandTopics;
    app.command_topics_index = categories.len() - 1;
    let selected_category = categories[app.command_topics_index].label();
    let command_topics_render = render_app(&app, 40, 5);
    let compact_command_topics: String = command_topics_render
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let compact_category: String = selected_category
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_command_topics.contains(&compact_category));
}

#[test]
fn wrapped_lesson_views_reach_bottom_on_narrow_terminal() {
    let mut app = fresh_app("narrow-wrapped-lesson-scroll");
    let category = app.get_lesson_categories()[0];
    let lesson_index = app
        .lessons
        .iter()
        .position(|lesson| lesson.meta.category == category)
        .expect("need command lesson");

    app.lessons[lesson_index].overview.summary = "wrapped summary ".repeat(30);
    app.lessons[lesson_index].overview.explanation = "overview explanation".to_string();
    app.lessons[lesson_index].syntax.basic = "OVERVIEW_BOTTOM_MARKER".to_string();
    app.lessons[lesson_index].syntax.parts.clear();
    app.lessons[lesson_index].options.clear();
    app.lessons[lesson_index].gotchas.clear();
    app.state = AppState::CommandLessonOverview {
        category_index: 0,
        command_index: 0,
        scroll: usize::MAX,
    };

    let overview_render = render_app(&app, 26, 6);
    assert!(overview_render.contains("OVERVIEW_BOTTOM_MARKER"));

    app.lessons[lesson_index].examples[0].deep_explanation = Some(format!(
        "{}\nDEEP_BOTTOM_MARKER",
        "wrapped deep explanation ".repeat(30)
    ));
    app.state = AppState::DeepExplanation {
        source: DeepSource::LessonExample {
            category_idx: 0,
            command_idx: 0,
            example_idx: 0,
        },
        scroll: usize::MAX,
    };

    let deep_render = render_app(&app, 26, 6);
    assert!(deep_render.contains("DEEP_BOTTOM_MARKER"));
}

#[test]
fn config_file_scroll_keys_and_wrapped_clamp_work_on_narrow_terminal() {
    let mut app = fresh_app("config-file-scroll");
    app.system_topics.truncate(1);
    app.system_topics[0].sections.truncate(1);
    let section = &mut app.system_topics[0].sections[0];
    section.commands.clear();
    section.config_files = vec![ConfigFile {
        id: "scroll-config".to_string(),
        path: "/etc/scroll.conf".to_string(),
        name: "scroll.conf".to_string(),
        description: "wrapped config description ".repeat(30),
        sample_content: "setting=value".to_string(),
        lessons: vec![ConfigLesson {
            title: "change setting".to_string(),
            before: "before".to_string(),
            after: "after".to_string(),
            explanation: format!(
                "{}\nCONFIG_BOTTOM_MARKER",
                "wrapped config lesson ".repeat(30)
            ),
            practice_command: None,
        }],
    }];
    app.state = AppState::SystemLesson {
        topic_index: 0,
        section_index: 0,
        phase: SystemPhase::ConfigFile(0),
        scroll: 0,
    };

    app.handle_key(key(KeyCode::Down));
    assert_eq!(system_lesson_scroll(&app), 1);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(system_lesson_scroll(&app), 2);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(system_lesson_scroll(&app), 1);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(system_lesson_scroll(&app), 0);
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(system_lesson_scroll(&app), 5);
    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(system_lesson_scroll(&app), 0);

    app.state = AppState::SystemLesson {
        topic_index: 0,
        section_index: 0,
        phase: SystemPhase::ConfigFile(0),
        scroll: usize::MAX,
    };
    let rendered = render_app(&app, 26, 6);
    assert!(rendered.contains("CONFIG_BOTTOM_MARKER"));

    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.state, AppState::SystemTopics);
}

#[test]
fn long_system_simulated_output_scrolls_to_wrapped_bottom() {
    let mut app = fresh_app("system-output-scroll");
    app.system_topics.truncate(1);
    app.system_topics[0].sections.truncate(1);
    let section = &mut app.system_topics[0].sections[0];
    section.commands = vec![SystemCommand {
        id: Some("scroll-output".to_string()),
        command_id: None,
        command: "x".to_string(),
        summary: "show long output".to_string(),
        simulated_output: Some(format!(
            "{}\nSYSTEM_OUTPUT_BOTTOM",
            "wrapped simulated output ".repeat(30)
        )),
        deep_explanation: None,
    }];
    section.config_files.clear();
    app.typing_engine.reset("x");
    app.state = AppState::SystemLesson {
        topic_index: 0,
        section_index: 0,
        phase: SystemPhase::TypingPractice { command_idx: 0 },
        scroll: 0,
    };

    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Enter));
    assert!(app.system_typing_showing_output);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(system_lesson_scroll(&app), 1);
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(system_lesson_scroll(&app), 6);

    app.state = AppState::SystemLesson {
        topic_index: 0,
        section_index: 0,
        phase: SystemPhase::TypingPractice { command_idx: 0 },
        scroll: usize::MAX,
    };
    let rendered = render_app(&app, 26, 6);
    assert!(rendered.contains("SYSTEM_OUTPUT_BOTTOM"));
}

#[test]
fn all_sixteen_command_training_topics_are_selectable_in_small_terminal() {
    let mut app = fresh_app("review-all-topics");
    assert_eq!(app.command_training_topics.len(), 16);
    app.state = AppState::ReviewTopics;

    for index in 0..app.command_training_topics.len() {
        assert_eq!(app.review_topics_index, index);
        let topic = &app.command_training_topics[index];
        let rendered = render_app(&app, 120, 9);
        let compact_rendered: String = rendered
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        let compact_title: String = topic
            .title
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        assert!(
            compact_rendered.contains(&compact_title),
            "topic row {index} should remain visible in a constrained terminal"
        );

        if index + 1 < app.command_training_topics.len() {
            app.handle_key(key(KeyCode::Char('j')));
        }
    }

    let last_index = app.command_training_topics.len() - 1;
    let last_topic = app.command_training_topics[last_index].clone();
    assert_eq!(app.review_topics_index, last_index);
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(
        app.state,
        AppState::Review {
            source: ReviewSource::CommandTopic(last_topic.id),
            phase: ReviewPhase::Summary,
        }
    );
}

#[test]
fn review_selection_is_unseen_first_and_limited_to_ten() {
    let mut app = fresh_app("review-unseen-first");
    let topic = app
        .command_training_topics
        .iter()
        .find(|topic| topic.command_ids.len() >= 12)
        .expect("need topic with at least twelve commands")
        .clone();
    app.user_stats.command_progress = topic
        .command_ids
        .iter()
        .enumerate()
        .map(|(index, command_id)| CommandProgress {
            command_id: command_id.clone(),
            times_practiced: if index == 10 || index == 11 { 0 } else { 5 },
            ..CommandProgress::default()
        })
        .collect();

    let exercises = cmdtyper::flow::review_flow::build_review_exercises(
        &app,
        &ReviewSource::CommandTopic(topic.id.clone()),
    );
    let expected: Vec<&str> = [10, 11]
        .into_iter()
        .chain(0..8)
        .map(|index| topic.command_ids[index].as_str())
        .collect();
    let actual: Vec<&str> = exercises
        .iter()
        .map(|exercise| exercise.command_id.as_str())
        .collect();
    assert_eq!(actual, expected);

    let repeated = cmdtyper::flow::review_flow::build_review_exercises(
        &app,
        &ReviewSource::CommandTopic(topic.id),
    );
    assert_eq!(
        repeated
            .iter()
            .map(|exercise| exercise.command_id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn review_selection_preserves_author_order_for_ties() {
    let mut app = fresh_app("review-tie-order");
    let topic = app
        .command_training_topics
        .iter()
        .find(|topic| topic.command_ids.len() >= 10)
        .expect("need topic with ten commands")
        .clone();
    app.user_stats.command_progress = topic
        .command_ids
        .iter()
        .map(|command_id| CommandProgress {
            command_id: command_id.clone(),
            times_practiced: 3,
            ..CommandProgress::default()
        })
        .collect();

    let exercises = cmdtyper::flow::review_flow::build_review_exercises(
        &app,
        &ReviewSource::CommandTopic(topic.id),
    );
    let actual: Vec<&str> = exercises
        .iter()
        .map(|exercise| exercise.command_id.as_str())
        .collect();
    let expected: Vec<&str> = topic
        .command_ids
        .iter()
        .take(10)
        .map(String::as_str)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn l1_typing_supports_backspace_and_records_canonical_command_id() {
    let mut app = fresh_app("review-l1-record");
    let topic_id = app.command_training_topics[0].id.clone();
    app.topic_training_level = TopicTrainingLevel::L1;
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id.clone()),
        phase: ReviewPhase::Summary,
    };
    app.handle_key(key(KeyCode::Enter));

    let exercise = app.current_review_exercise().expect("L1 exercise").clone();
    assert_eq!(exercise.kind, ReviewExerciseKind::Typing);
    let target: Vec<char> = exercise.command.chars().collect();
    app.handle_key(key(KeyCode::Char(target[0])));
    if target.len() > 1 {
        app.handle_key(key(KeyCode::Char(target[1])));
    }
    let cursor_before_backspace = app.typing_engine.cursor;
    app.handle_key(key(KeyCode::Backspace));
    assert_eq!(
        app.typing_engine.cursor,
        cursor_before_backspace.saturating_sub(1)
    );
    for ch in target.iter().skip(app.typing_engine.cursor).copied() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    assert!(app.typing_engine.is_complete());

    app.handle_key(key(KeyCode::Enter));
    assert!(app.review_practice.typing_showing_output);
    let record = app.history.last().expect("review typing record");
    assert_eq!(record.command_id, exercise.command_id);
    assert_eq!(record.mode, RecordMode::ReviewTyping);
    assert_eq!(record.accuracy, app.review_practice.typing_accuracy_sum);
    let rendered = render_app(&app, 120, 24);
    let compact_rendered: String = rendered
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_rendered.contains("预设模拟输出"));
}

#[test]
fn l3_cloze_masks_one_token_is_case_sensitive_and_never_records_wpm() {
    let mut app = fresh_app("review-l3-case");
    app.topic_training_level = TopicTrainingLevel::L3;
    let (topic_id, command_id, expected_answer) = app
        .command_training_topics
        .iter()
        .find_map(|topic| {
            let exercises = cmdtyper::flow::review_flow::build_review_exercises(
                &app,
                &ReviewSource::CommandTopic(topic.id.clone()),
            );
            exercises.into_iter().find_map(|exercise| {
                let answer = exercise.cloze_answer?;
                let changed_case = answer.to_uppercase();
                (changed_case != answer).then(|| (topic.id.clone(), exercise.command_id, answer))
            })
        })
        .expect("need cloze answer with case-sensitive letters");
    let topic_ids = app
        .command_training_topics
        .iter()
        .find(|topic| topic.id == topic_id)
        .expect("selected topic")
        .command_ids
        .clone();
    app.user_stats.command_progress = topic_ids
        .iter()
        .map(|id| CommandProgress {
            command_id: id.clone(),
            times_practiced: u32::from(*id != command_id),
            ..CommandProgress::default()
        })
        .collect();
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id),
        phase: ReviewPhase::Summary,
    };
    app.handle_key(key(KeyCode::Enter));

    let exercise = app.current_review_exercise().expect("L3 exercise");
    assert_eq!(exercise.command_id, command_id);
    assert_eq!(exercise.kind, ReviewExerciseKind::Cloze);
    let skeleton = exercise.cloze_skeleton.as_deref().expect("cloze skeleton");
    assert_eq!(skeleton.matches("____").count(), 1);
    assert_eq!(
        skeleton.replacen("____", &expected_answer, 1),
        exercise.command
    );
    assert_ne!(skeleton, exercise.command);
    let wrong_case = expected_answer.to_uppercase();
    for ch in wrong_case.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Backspace));
    if let Some(last) = wrong_case.chars().last() {
        app.handle_key(key(KeyCode::Char(last)));
    }
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.review_practice.cloze_correct, Some(false));
    let record = app.history.last().expect("review cloze record");
    assert_eq!(record.command_id, command_id);
    assert_eq!(record.mode, RecordMode::ReviewCloze);
    assert_eq!(record.wpm, 0.0);
    assert_eq!(app.user_stats.total_wpm_sessions, 0);
    assert_eq!(app.user_stats.overall_avg_wpm, 0.0);
}

#[test]
fn l3_cloze_accepts_outer_trimmed_exact_case_answer() {
    let mut app = fresh_app("review-l3-trim");
    let topic_id = app.command_training_topics[0].id.clone();
    app.topic_training_level = TopicTrainingLevel::L3;
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id),
        phase: ReviewPhase::Summary,
    };
    app.handle_key(key(KeyCode::Enter));
    let answer = app
        .current_review_exercise()
        .and_then(|exercise| exercise.cloze_answer.clone())
        .expect("cloze answer");
    for ch in format!("  {answer}  ").chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.review_practice.cloze_correct, Some(true));
}

#[test]
fn l3_cloze_rejects_internal_whitespace_normalization() {
    let mut app = fresh_app("review-l3-internal-space");
    app.topic_training_level = TopicTrainingLevel::L3;
    let topic_id = app
        .command_training_topics
        .iter()
        .find_map(|topic| {
            cmdtyper::flow::review_flow::build_review_exercises(
                &app,
                &ReviewSource::CommandTopic(topic.id.clone()),
            )
            .into_iter()
            .next()
            .and_then(|exercise| {
                exercise
                    .cloze_answer
                    .filter(|answer| answer.chars().count() > 1)
                    .map(|_| topic.id.clone())
            })
        })
        .expect("need a multi-character cloze answer");
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id),
        phase: ReviewPhase::Summary,
    };
    app.handle_key(key(KeyCode::Enter));

    let answer = app
        .current_review_exercise()
        .and_then(|exercise| exercise.cloze_answer.clone())
        .expect("cloze answer");
    let mut characters = answer.chars();
    let first = characters.next().expect("multi-character answer");
    let internally_spaced = format!("{first} {}", characters.collect::<String>());
    for character in internally_spaced.chars() {
        app.handle_key(key(KeyCode::Char(character)));
    }
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.review_practice.cloze_correct, Some(false));
}

#[test]
fn l5_dictation_accepts_alternate_canonical_answer() {
    let mut app = fresh_app("review-l5-alternate");
    let (topic_id, command_id, alternate) =
        app.command_training_topics
            .iter()
            .find_map(|topic| {
                topic.command_ids.iter().find_map(|command_id| {
                    app.commands
                        .iter()
                        .find(|command| command.id == *command_id)
                        .and_then(|command| {
                            command.dictation.answers.get(1).map(|answer| {
                                (topic.id.clone(), command.id.clone(), answer.clone())
                            })
                        })
                })
            })
            .expect("need command with alternate answer");
    let topic_ids = app
        .command_training_topics
        .iter()
        .find(|topic| topic.id == topic_id)
        .expect("selected topic")
        .command_ids
        .clone();
    app.user_stats.command_progress = topic_ids
        .iter()
        .map(|id| CommandProgress {
            command_id: id.clone(),
            times_practiced: u32::from(*id != command_id),
            ..CommandProgress::default()
        })
        .collect();
    app.topic_training_level = TopicTrainingLevel::L5;
    app.state = AppState::Review {
        source: ReviewSource::CommandTopic(topic_id),
        phase: ReviewPhase::Summary,
    };
    app.handle_key(key(KeyCode::Enter));
    let exercise = app.current_review_exercise().expect("L5 exercise");
    assert_eq!(exercise.command_id, command_id);
    assert!(exercise.accepted_answers.contains(&alternate));

    for ch in alternate.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(
        app.review_practice.dictation_result,
        Some(MatchResult::Exact(_) | MatchResult::Normalized(_))
    ));
    assert_eq!(
        app.history.last().expect("dictation record").command_id,
        command_id
    );
}

#[test]
fn empty_training_topics_render_and_handle_keys_without_panicking() {
    let mut app = fresh_app("review-empty-topics");
    app.command_training_topics.clear();
    app.review_topics_index = usize::MAX;
    app.state = AppState::ReviewTopics;

    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.state, AppState::ReviewTopics);
    let empty_render = render_app(&app, 48, 8);
    let compact_empty_render: String = empty_render
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_empty_render.contains("暂无专题训练数据"));
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.state, AppState::LearnHub);
}

#[test]
fn review_escape_hierarchy_and_level_keys_follow_training_flow() {
    let mut app = fresh_app("review-escape-hierarchy");
    let topic_id = app.command_training_topics[0].id.clone();
    app.state = AppState::ReviewTopics;
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L1);
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L3);
    app.handle_key(key(KeyCode::Char('l')));
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L5);
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Char('h')));
    assert_eq!(app.topic_training_level, TopicTrainingLevel::L1);

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(
        app.state,
        AppState::Review {
            phase: ReviewPhase::Practice,
            ..
        }
    ));
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(
        app.state,
        AppState::Review {
            source: ReviewSource::CommandTopic(topic_id),
            phase: ReviewPhase::Summary,
        }
    );
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.state, AppState::ReviewTopics);
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.state, AppState::LearnHub);
}

#[test]
fn lesson_progress_prefers_canonical_then_stable_then_legacy_identity() {
    let mut app = fresh_app("lesson-progress-identity");
    let mut lesson = app
        .lessons
        .iter()
        .find(|lesson| !lesson.examples.is_empty())
        .expect("need lesson example")
        .clone();
    lesson.meta.command = "fixture-lesson".to_string();
    lesson.examples.truncate(1);
    lesson.examples[0].command = "x".to_string();
    app.lessons = vec![lesson];
    app.state = AppState::CommandTopics;
    let initial_render = render_app(&app, 80, 12);
    assert!(initial_render.contains("0/1"));

    let cases = [
        (
            Some("canonical-command"),
            Some("stable-example"),
            "canonical-command",
        ),
        (
            None,
            Some("stable-example"),
            "lesson:fixture-lesson:stable-example",
        ),
        (None, None, "fixture-lesson"),
    ];

    for (command_id, example_id, expected) in cases {
        let example = &mut app.lessons[0].examples[0];
        example.command_id = command_id.map(str::to_string);
        example.id = example_id.map(str::to_string);
        app.typing_engine.reset("x");
        app.state = AppState::CommandLessonPractice {
            category_index: 0,
            command_index: 0,
            example_index: 0,
        };

        type_current_target(&mut app);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(
            app.history
                .last()
                .expect("lesson practice record")
                .command_id,
            expected
        );
        app.state = AppState::CommandTopics;
        assert!(render_app(&app, 80, 12).contains("1/1"));
    }

    let template = app.lessons[0].examples[0].clone();
    let mut canonical_example = template.clone();
    canonical_example.command_id = Some("canonical-command".to_string());
    canonical_example.id = Some("stable-example".to_string());
    let mut stable_example = template.clone();
    stable_example.command_id = None;
    stable_example.id = Some("stable-example".to_string());
    let mut legacy_example = template;
    legacy_example.command_id = None;
    legacy_example.id = None;
    app.lessons[0].examples = vec![canonical_example, stable_example, legacy_example];

    app.state = AppState::CommandTopics;
    let completed_render = render_app(&app, 80, 12);
    assert!(completed_render.contains("1/1"));
    assert!(!completed_render.contains("3/1"));
}

#[test]
fn canonical_lesson_difficulty_drives_mastery_instead_of_container_difficulty() {
    let mut app = fresh_app("lesson-canonical-difficulty");
    let canonical = app
        .commands
        .iter()
        .find(|command| command.difficulty == Difficulty::Beginner)
        .expect("need beginner canonical command")
        .clone();
    let mut lesson = app
        .lessons
        .iter()
        .find(|lesson| !lesson.examples.is_empty())
        .expect("need lesson example")
        .clone();
    lesson.meta.command = "basic-container".to_string();
    lesson.meta.difficulty = Difficulty::Basic;
    lesson.examples.truncate(1);
    lesson.examples[0].id = Some("canonical-example".to_string());
    lesson.examples[0].command_id = Some(canonical.id.clone());
    lesson.examples[0].command = canonical.command.clone();
    app.lessons = vec![lesson];
    app.user_stats = UserStats::default();
    app.typing_engine.reset(&canonical.command);
    app.state = AppState::CommandLessonPractice {
        category_index: 0,
        command_index: 0,
        example_index: 0,
    };

    type_current_target(&mut app);
    app.handle_key(key(KeyCode::Enter));

    let record = app.history.last().expect("lesson practice record");
    assert_eq!(record.command_id, canonical.id);
    assert_eq!(record.difficulty, Difficulty::Beginner);
    let progress = app
        .user_stats
        .command_progress
        .iter()
        .find(|progress| progress.command_id == record.command_id)
        .expect("canonical command progress");
    assert!(approx_eq(progress.mastery, 1.0 / 3.0, 1e-12));
    assert!(progress.mastery > 1.0 / 5.0);
}

#[test]
fn symbol_progress_prefers_canonical_then_stable_then_legacy_identity() {
    let mut app = fresh_app("symbol-progress-identity");
    let mut topic = app
        .symbol_topics
        .first()
        .expect("need symbol topic")
        .clone();
    topic.meta.id = "fixture-symbol".to_string();
    topic.exercises = vec![Exercise {
        id: None,
        command_id: None,
        prompt: "type x".to_string(),
        answers: vec!["x".to_string()],
        kind: Some(ExerciseKind::Typing),
        command: Some("x".to_string()),
        simulated_output: None,
    }];
    app.symbol_topics = vec![topic];

    let cases = [
        (
            Some("canonical-command"),
            Some("stable-exercise"),
            "canonical-command",
        ),
        (
            None,
            Some("stable-exercise"),
            "symbol:fixture-symbol:stable-exercise",
        ),
        (None, None, "symbol:fixture-symbol:typing:0"),
    ];

    for (command_id, exercise_id, expected) in cases {
        let exercise = &mut app.symbol_topics[0].exercises[0];
        exercise.command_id = command_id.map(str::to_string);
        exercise.id = exercise_id.map(str::to_string);
        app.symbol_practice = SymbolPracticeState {
            typing_indices: vec![0],
            total_count: 1,
            ..SymbolPracticeState::default()
        };
        app.typing_engine.reset("x");
        app.state = AppState::SymbolLesson {
            topic_index: 0,
            symbol_index: 0,
            phase: SymbolPhase::TypingPractice { exercise_idx: 0 },
        };
        let history_start = app.history.len();

        type_current_target(&mut app);
        app.handle_key(key(KeyCode::Enter));
        let record = app.history[history_start..]
            .iter()
            .find(|record| record.mode == RecordMode::SymbolTyping)
            .expect("symbol typing record");
        assert_eq!(record.command_id, expected);
    }
}

#[test]
fn system_progress_prefers_canonical_then_stable_then_legacy_identity() {
    let mut app = fresh_app("system-progress-identity");
    let mut topic = app
        .system_topics
        .first()
        .expect("need system topic")
        .clone();
    topic.meta.id = "fixture-system".to_string();
    topic.sections.truncate(1);
    topic.sections[0].id = "fixture-section".to_string();
    topic.sections[0].commands = vec![SystemCommand {
        id: None,
        command_id: None,
        command: "x".to_string(),
        summary: "type x".to_string(),
        simulated_output: None,
        deep_explanation: None,
    }];
    topic.sections[0].config_files.clear();
    app.system_topics = vec![topic];

    let cases = [
        (
            Some("canonical-command"),
            Some("stable-command"),
            "canonical-command",
        ),
        (
            None,
            Some("stable-command"),
            "system:fixture-system:fixture-section:stable-command",
        ),
        (None, None, "system:fixture-system:0:0"),
    ];

    for (command_id, stable_id, expected) in cases {
        let command = &mut app.system_topics[0].sections[0].commands[0];
        command.command_id = command_id.map(str::to_string);
        command.id = stable_id.map(str::to_string);
        app.typing_engine.reset("x");
        app.state = AppState::SystemLesson {
            topic_index: 0,
            section_index: 0,
            phase: SystemPhase::TypingPractice { command_idx: 0 },
            scroll: 0,
        };

        type_current_target(&mut app);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(
            app.history.last().expect("system typing record").command_id,
            expected
        );
    }
}
