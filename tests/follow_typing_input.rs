use cmdtyper::{
    app::{
        App, AppState, ReviewExercise, ReviewExerciseKind, ReviewPhase, ReviewPracticeState,
        ReviewSource, SymbolPhase, SymbolPracticeState, SystemPhase,
    },
    data::{
        models::{Command, Exercise, ExerciseKind, RecordMode, SystemCommand},
        practice_loader::PracticeGroup,
    },
    flow::practice_flow::{self, PracticeState},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    env, fs,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug)]
enum Entry {
    Practice,
    Review,
    Symbol,
    System,
    Lesson,
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn with_app(f: impl FnOnce(&mut App, &Path)) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let path = env::temp_dir().join(format!(
        "cmdtyper-follow-input-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let old_data = env::var_os("CMDTYPER_DATA_DIR");
    let old_user = env::var_os("CMDTYPER_USER_DIR");
    unsafe {
        env::set_var(
            "CMDTYPER_DATA_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/data"),
        );
        env::set_var("CMDTYPER_USER_DIR", &path);
    }
    let mut app = App::new().expect("load fixture content");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut app, &path)));
    unsafe {
        match old_data {
            Some(value) => env::set_var("CMDTYPER_DATA_DIR", value),
            None => env::remove_var("CMDTYPER_DATA_DIR"),
        }
        match old_user {
            Some(value) => env::set_var("CMDTYPER_USER_DIR", value),
            None => env::remove_var("CMDTYPER_USER_DIR"),
        }
    }
    fs::remove_dir_all(path).ok();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

fn setup(app: &mut App, entry: Entry, next: &str) {
    let commands: Vec<Command> = ["ab", next, "third", "fourth"]
        .into_iter()
        .enumerate()
        .map(|(index, text)| Command {
            id: format!("input-fixture-{index}"),
            command: text.to_string(),
            simulated_output: Some("simulated result".to_string()),
            ..Command::default()
        })
        .collect();
    app.commands = commands.clone();
    app.typing_engine.reset("ab");
    match entry {
        Entry::Practice => {
            app.practice_groups = vec![PracticeGroup {
                id: "input-fixture".into(),
                source_kind: "lesson".into(),
                source_id: "fixture".into(),
                unit_id: "fixture".into(),
                title: "input fixture".into(),
                teaching_command_id: commands[0].id.clone(),
                exercise_command_ids: commands[1..]
                    .iter()
                    .map(|command| command.id.clone())
                    .collect(),
            }];
            app.practice_state = PracticeState {
                filtered: vec![0],
                ..PracticeState::default()
            };
            app.state = AppState::PracticeGroup;
        }
        Entry::Review => {
            let exercises = commands[..2]
                .iter()
                .map(|command| ReviewExercise {
                    kind: ReviewExerciseKind::Typing,
                    command_id: command.id.clone(),
                    command: command.command.clone(),
                    display: None,
                    description: String::new(),
                    accepted_answers: vec![command.command.clone()],
                    tokens: vec![],
                    simulated_output: command.simulated_output.clone(),
                    difficulty: command.difficulty,
                    cloze_skeleton: None,
                    cloze_answer: None,
                })
                .collect();
            app.review_practice = ReviewPracticeState {
                exercises,
                total_count: 2,
                ..ReviewPracticeState::default()
            };
            app.state = AppState::Review {
                source: ReviewSource::CommandTopic("fixture".into()),
                phase: ReviewPhase::Practice,
            };
        }
        Entry::Symbol => {
            let mut topic = app.symbol_topics[0].clone();
            topic.exercises = commands[..2]
                .iter()
                .map(|command| Exercise {
                    id: None,
                    command_id: Some(command.id.clone()),
                    prompt: "type command".into(),
                    answers: vec![command.command.clone()],
                    kind: Some(ExerciseKind::Typing),
                    command: Some(command.command.clone()),
                    simulated_output: command.simulated_output.clone(),
                })
                .collect();
            app.symbol_topics = vec![topic];
            app.symbol_practice = SymbolPracticeState {
                typing_indices: vec![0, 1],
                total_count: 2,
                ..SymbolPracticeState::default()
            };
            app.state = AppState::SymbolLesson {
                topic_index: 0,
                symbol_index: 0,
                phase: SymbolPhase::TypingPractice { exercise_idx: 0 },
            };
        }
        Entry::System => {
            let mut topic = app.system_topics[0].clone();
            topic.sections.truncate(1);
            topic.sections[0].commands = commands[..2]
                .iter()
                .map(|command| SystemCommand {
                    id: None,
                    command_id: Some(command.id.clone()),
                    command: command.command.clone(),
                    summary: "type command".into(),
                    simulated_output: command.simulated_output.clone(),
                    deep_explanation: Some("explanation".into()),
                })
                .collect();
            topic.sections[0].config_files.clear();
            app.system_topics = vec![topic];
            app.system_typing_showing_output = false;
            app.state = AppState::SystemLesson {
                topic_index: 0,
                section_index: 0,
                phase: SystemPhase::TypingPractice { command_idx: 0 },
                scroll: 0,
            };
        }
        Entry::Lesson => {
            let mut lesson = app.lessons[0].clone();
            let template = lesson.examples[0].clone();
            lesson.examples = commands[..2]
                .iter()
                .map(|command| {
                    let mut example = template.clone();
                    example.command_id = Some(command.id.clone());
                    example.command = command.command.clone();
                    example.simulated_output = command.simulated_output.clone();
                    example
                })
                .collect();
            app.lessons = vec![lesson];
            app.state = AppState::CommandLessonPractice {
                category_index: 0,
                command_index: 0,
                example_index: 0,
            };
        }
    }
}

fn showing_output(app: &App, entry: Entry) -> bool {
    match entry {
        Entry::Practice => app.practice_state.output,
        Entry::Review => app.review_practice.typing_showing_output,
        Entry::Symbol => app.symbol_practice.typing_showing_output,
        Entry::System => app.system_typing_showing_output,
        Entry::Lesson => false,
    }
}

fn current_index(app: &App, entry: Entry) -> usize {
    match entry {
        Entry::Practice => app.practice_state.step,
        Entry::Review => app.review_practice.current_index,
        Entry::Symbol => match app.state {
            AppState::SymbolLesson {
                phase: SymbolPhase::TypingPractice { exercise_idx },
                ..
            } => exercise_idx,
            ref other => panic!("unexpected {other:?}"),
        },
        Entry::System => match app.state {
            AppState::SystemLesson {
                phase: SystemPhase::TypingPractice { command_idx },
                ..
            } => command_idx,
            ref other => panic!("unexpected {other:?}"),
        },
        Entry::Lesson => match app.state {
            AppState::CommandLessonPractice { example_index, .. } => example_index,
            ref other => panic!("unexpected {other:?}"),
        },
    }
}

fn finish_target(app: &mut App) {
    for ch in app.typing_engine.target[app.typing_engine.cursor..].to_vec() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
}

fn assert_first_character_reaches_next_target(entry: Entry) {
    with_app(|app, _| {
        for first in ['j', 'k', 'd', 'D', 'l', 'p', 'r', '!'] {
            setup(app, entry, &format!("{first}z"));
            let history_count = app.history.len();
            finish_target(app);
            assert_eq!(showing_output(app, entry), !matches!(entry, Entry::Lesson));
            app.handle_key(KeyEvent::new(
                KeyCode::Char(first),
                if first == 'D' || first == '!' {
                    KeyModifiers::SHIFT
                } else {
                    KeyModifiers::NONE
                },
            ));
            assert_eq!(current_index(app, entry), 1, "{entry:?}, {first}");
            assert_eq!(app.typing_engine.cursor, 1, "{entry:?}, {first}");
            assert_eq!(app.typing_engine.current_accuracy(), 1.0);
            assert!(!showing_output(app, entry));
            assert_eq!(app.history.len(), history_count + 1, "one saved completion");
        }
    });
}

fn assert_retry_saves_partial_and_filters_shortcuts(entry: Entry) {
    with_app(|app, path| {
        setup(app, entry, "next");
        for modifier in [
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ] {
            app.handle_key(KeyEvent::new(KeyCode::Char('a'), modifier));
            assert_eq!(app.typing_engine.cursor, 0);
            assert!(
                !app.typing_engine.has_activity(),
                "shortcut must not start an attempt"
            );
        }
        app.handle_key(key(KeyCode::Char('x')));
        app.handle_key(key(KeyCode::Char('a')));
        let state = app.state.clone();

        fs::create_dir_all(path.join("history.json")).unwrap();
        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(app.typing_engine.cursor, 1, "failed save must retain input");
        assert_eq!(app.state, state);
        assert!(app.history.is_empty());
        assert!(app.persistence_error.is_some());
        fs::remove_dir(path.join("history.json")).unwrap();

        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(app.typing_engine.cursor, 0);
        assert_eq!(app.state, state);
        assert!(!app.typing_engine.has_activity());
        assert!(!showing_output(app, entry));
        assert_eq!(app.history.len(), 1);
        assert!(!app.history[0].is_completed());
        assert_eq!(app.history[0].error_count, 1);
        assert_eq!(app.user_stats.attempted_positions, 1);
        assert_eq!(app.progress_store.load_history().unwrap(), app.history);

        finish_target(app);
        assert_eq!(app.history.len(), 2);
        assert!(app.history[1].is_completed());
        assert_ne!(app.history[0].id, app.history[1].id);
        assert_eq!(app.history[1].error_count, 0);
        assert_eq!(app.history[1].accuracy, 1.0);
        assert_eq!(app.user_stats.attempted_positions, 3);
    });
}

fn assert_retry_from_output_stays_on_current_target(entry: Entry) {
    with_app(|app, _| {
        setup(app, entry, "next");
        finish_target(app);
        assert!(showing_output(app, entry));
        for modifier in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
            app.handle_key(KeyEvent::new(KeyCode::Char('j'), modifier));
            assert!(showing_output(app, entry));
            assert_eq!(current_index(app, entry), 0);
        }
        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(current_index(app, entry), 0);
        assert_eq!(app.typing_engine.cursor, 0);
        assert!(!showing_output(app, entry));
        assert_eq!(app.history.len(), 1);
        assert!(app.history[0].is_completed());
        app.handle_key(key(KeyCode::Char('a')));
        assert_eq!(app.typing_engine.cursor, 1);
    });
}

#[test]
fn practice_output_preserves_next_first_character() {
    assert_first_character_reaches_next_target(Entry::Practice);
}
#[test]
fn review_l1_output_preserves_next_first_character() {
    assert_first_character_reaches_next_target(Entry::Review);
}
#[test]
fn symbol_output_preserves_next_first_character() {
    assert_first_character_reaches_next_target(Entry::Symbol);
}
#[test]
fn system_output_preserves_next_first_character() {
    assert_first_character_reaches_next_target(Entry::System);
}
#[test]
fn lesson_submit_preserves_next_first_character() {
    assert_first_character_reaches_next_target(Entry::Lesson);
}

#[test]
fn practice_retry_saves_partial_and_filters_shortcuts() {
    assert_retry_saves_partial_and_filters_shortcuts(Entry::Practice);
}
#[test]
fn review_l1_retry_saves_partial_and_filters_shortcuts() {
    assert_retry_saves_partial_and_filters_shortcuts(Entry::Review);
}
#[test]
fn symbol_retry_saves_partial_and_filters_shortcuts() {
    assert_retry_saves_partial_and_filters_shortcuts(Entry::Symbol);
}
#[test]
fn system_retry_saves_partial_and_filters_shortcuts() {
    assert_retry_saves_partial_and_filters_shortcuts(Entry::System);
}
#[test]
fn lesson_retry_saves_partial_and_filters_shortcuts() {
    assert_retry_saves_partial_and_filters_shortcuts(Entry::Lesson);
}

#[test]
fn practice_retry_from_output_stays_on_current_target() {
    assert_retry_from_output_stays_on_current_target(Entry::Practice);
}
#[test]
fn review_l1_retry_from_output_stays_on_current_target() {
    assert_retry_from_output_stays_on_current_target(Entry::Review);
}
#[test]
fn symbol_retry_from_output_stays_on_current_target() {
    assert_retry_from_output_stays_on_current_target(Entry::Symbol);
}
#[test]
fn system_retry_from_output_stays_on_current_target() {
    assert_retry_from_output_stays_on_current_target(Entry::System);
}

#[test]
fn last_symbol_typing_output_preserves_first_dictation_character() {
    with_app(|app, _| {
        setup(app, Entry::Symbol, "next");
        app.symbol_topics[0].exercises[1].kind = Some(ExerciseKind::Dictation);
        app.symbol_practice.typing_indices = vec![0];
        app.symbol_practice.dictation_indices = vec![1];
        finish_target(app);
        app.handle_key(key(KeyCode::Char('n')));
        assert!(matches!(
            app.state,
            AppState::SymbolLesson {
                phase: SymbolPhase::Practice,
                ..
            }
        ));
        assert_eq!(app.symbol_practice.current_input, "n");
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history[0].mode, RecordMode::SymbolTyping);
    });
}

#[test]
fn practice_last_output_completes_group_without_starting_another_attempt() {
    with_app(|app, _| {
        setup(app, Entry::Practice, "next");
        app.practice_state.step = 3;
        app.typing_engine
            .reset(&practice_flow::current_command(app).unwrap().command.clone());
        finish_target(app);
        app.handle_key(key(KeyCode::Char('j')));
        assert!(app.practice_state.done);
        assert!(!app.practice_state.output);
        assert_eq!(app.history.len(), 1);
        assert!(app.history[0].is_completed());
    });
}

fn assert_foundation_source_mode_survives_partial_and_complete_saves(
    source: &str,
    mode: RecordMode,
) {
    with_app(|app, _| {
        setup(app, Entry::Practice, "next");
        app.practice_groups[0].source_kind = source.to_owned();
        app.state = AppState::LearnHub;
        practice_flow::enter(app, Some(source), Some("fixture"));
        assert_eq!(app.state, AppState::PracticeTopics);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.state, AppState::PracticeGroup);
        assert_eq!(app.active_typing_identity().unwrap().2, mode);
        finish_target(app);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.practice_state.step, 1);
        app.handle_key(key(KeyCode::Char('n')));
        app.handle_key(key(KeyCode::Char('x')));
        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(app.typing_engine.cursor, 0);
        assert_eq!(app.history.len(), 2);
        assert_eq!(app.history[1].mode, mode);
        assert!(!app.history[1].is_completed());
        assert_eq!(app.history[1].error_count, 1);
        finish_target(app);
        assert!(app.practice_state.output);
        assert_eq!(app.history.len(), 3);
        assert!(app.history[2].is_completed());
        assert_eq!(app.history[1].command_id, app.history[2].command_id);
        assert_ne!(app.history[1].id, app.history[2].id);
        let saved = app.progress_store.load_history().unwrap();
        assert_eq!(saved.len(), 3);
        assert!(saved.iter().all(|record| record.mode == mode));
        let stats = app.progress_store.load_stats().unwrap();
        assert_eq!(stats.total_sessions, 3);
        let entries = stats
            .daily_stats
            .iter()
            .flat_map(|day| &day.entries)
            .collect::<Vec<_>>();
        assert!(entries.iter().all(|entry| entry.mode == mode));
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.completed_count)
                .sum::<u32>(),
            2
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.sessions_count)
                .sum::<u32>(),
            3
        );
    });
}

#[test]
fn symbol_foundation_partial_and_complete_records_keep_symbol_calendar_entry() {
    assert_foundation_source_mode_survives_partial_and_complete_saves(
        "symbol",
        RecordMode::SymbolTyping,
    );
}

#[test]
fn system_foundation_partial_and_complete_records_keep_system_calendar_entry() {
    assert_foundation_source_mode_survives_partial_and_complete_saves(
        "system",
        RecordMode::SystemTyping,
    );
}
