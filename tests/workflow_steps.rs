use cmdtyper::{
    app::{
        App, AppState, ReviewExercise, ReviewExerciseKind, ReviewPhase, ReviewPracticeState,
        ReviewSource, SymbolPhase, SymbolPracticeState, SystemPhase,
    },
    data::{
        models::{Command, Difficulty, Exercise, ExerciseKind, RecordMode, SystemCommand},
        practice_loader::PracticeGroup,
        sequence_loader::{CommandSequence, SequenceStep},
    },
    flow::practice_flow::PracticeState,
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
    Main,
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
        "cmdtyper-workflow-{}-{}",
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
    let commands: Vec<Command> = ["alpha\nbeta", next, "third", "fourth"]
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
    app.typing_engine.reset("alpha\nbeta");
    app.sequences = vec![CommandSequence {
        id: commands[0].id.clone(),
        command_ids: vec!["step-one".into(), "step-two".into()],
        commands: vec!["alpha".into(), "beta".into()],
        steps: vec![
            SequenceStep {
                command_id: "step-one".into(),
                command: "alpha".into(),
                output: Some("FIRST OUTPUT".into()),
                explanation: "FIRST CONTEXT".into(),
                prompt: Some("host:~/one$ ".into()),
            },
            SequenceStep {
                command_id: "step-two".into(),
                command: "beta".into(),
                output: Some("SECOND OUTPUT".into()),
                explanation: "SECOND CONTEXT".into(),
                prompt: Some("host:~/two$ ".into()),
            },
        ],
    }];
    match entry {
        Entry::Main => {
            app.typing_commands = commands[..2].to_vec();
            app.typing_index = 0;
            app.typing_showing_output = false;
            app.terminal_auto_advance = false;
            app.state = AppState::Typing;
        }
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

fn text(app: &App) -> String {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| cmdtyper::ui::render(frame, app))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}
fn input(app: &mut App, value: &str) {
    for ch in value.chars() {
        app.handle_key(key(if ch == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(ch)
        }));
    }
}
fn output(app: &App) -> bool {
    app.workflow_state
        .output_for(app.typing_engine.session_id())
        .is_some()
}

#[test]
fn every_follow_typing_entry_shows_step_context_and_output_then_keeps_the_next_character() {
    with_app(|app, _| {
        for entry in [
            Entry::Main,
            Entry::Practice,
            Entry::Review,
            Entry::Symbol,
            Entry::System,
            Entry::Lesson,
        ] {
            setup(app, entry, "mkdir");
            let before = app.history.len();
            input(app, "alpha");
            app.handle_key(key(KeyCode::Enter));
            assert!(output(app), "{entry:?}");
            let screen = text(app);
            assert!(
                screen.contains("FIRST OUTPUT") && screen.contains("FIRST CONTEXT"),
                "{entry:?}"
            );
            assert!(screen.contains("host:~/one$"));
            assert!(!screen.contains("SECOND OUTPUT"));
            assert_eq!(app.history.len(), before);
            app.handle_key(key(KeyCode::Char('b')));
            assert!(!output(app));
            assert_eq!(app.typing_engine.cursor, 7, "{entry:?}");
            input(app, "eta");
            app.handle_key(key(KeyCode::Enter));
            assert!(output(app), "final output {entry:?}");
            assert!(text(app).contains("SECOND OUTPUT"));
            app.handle_key(key(KeyCode::Char('m')));
            assert_eq!(
                app.typing_engine.target.iter().collect::<String>(),
                "mkdir",
                "{entry:?}"
            );
            assert_eq!(app.typing_engine.cursor, 1, "{entry:?}");
            assert!(!output(app));
            assert_eq!(
                app.history.len(),
                before + 1,
                "one measured workflow, {entry:?}"
            );
            assert!(app.history.last().unwrap().is_completed());
            assert_eq!(
                app.history
                    .last()
                    .unwrap()
                    .typing
                    .as_ref()
                    .unwrap()
                    .positions
                    .len(),
                9
            );
        }
    });
}

#[test]
fn modified_enter_and_alt_characters_cannot_skip_or_type_behind_step_output() {
    with_app(|app, _| {
        setup(app, Entry::Main, "next");
        input(app, "alpha");
        for modifiers in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
            app.handle_key(KeyEvent::new(KeyCode::Enter, modifiers));
            assert_eq!(app.typing_engine.cursor, 5);
            assert!(!output(app));
        }
        app.handle_key(key(KeyCode::Enter));
        for modifiers in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
            app.handle_key(KeyEvent::new(KeyCode::Char('b'), modifiers));
            assert_eq!(app.typing_engine.cursor, 6);
            assert!(output(app));
        }
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.typing_engine.cursor, 6);
        assert!(!output(app));
    });
}

#[test]
fn retry_and_exit_preserve_partial_measurements_without_reusing_stale_output() {
    with_app(|app, _| {
        setup(app, Entry::Practice, "next");
        input(app, "alpha\n");
        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(app.history.len(), 1);
        assert!(!app.history[0].is_completed());
        assert_eq!(app.typing_engine.cursor, 0);
        assert!(!output(app));
        input(app, "alpha\nb");
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.history.len(), 2);
        assert!(!app.history[1].is_completed());
        assert_eq!(app.state, AppState::PracticeTopics);
    });
}

#[test]
fn failing_history_write_holds_final_step_and_can_be_retried() {
    with_app(|app, path| {
        setup(app, Entry::Main, "next");
        input(app, "alpha\nbeta");
        fs::create_dir(path.join("history.json")).unwrap();
        app.handle_key(key(KeyCode::Enter));
        assert!(!output(app));
        assert_eq!(app.typing_index, 0);
        assert!(app.history.is_empty());
        assert!(app.persistence_error.is_some());
        fs::remove_dir(path.join("history.json")).unwrap();
        app.handle_key(key(KeyCode::Enter));
        assert!(output(app));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.typing_index, 1);
        assert_eq!(app.history.len(), 1);
    });
}

#[test]
fn final_symbol_workflow_returns_to_completion_page_without_reopening_output() {
    with_app(|app, _| {
        setup(app, Entry::Symbol, "next");
        app.symbol_practice.typing_indices.truncate(1);
        app.symbol_practice.total_count = 1;
        input(app, "alpha\nbeta");
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.symbol_practice.completed);
        assert!(cmdtyper::flow::workflow_flow::active_sequence(app).is_none());
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.state, AppState::SymbolTopics);
        assert_eq!(
            app.history
                .iter()
                .filter(|record| record.mode == RecordMode::SymbolTyping)
                .count(),
            1
        );
    });
}

#[test]
fn final_symbol_workflow_keeps_the_first_character_of_following_dictation() {
    with_app(|app, _| {
        setup(app, Entry::Symbol, "mkdir");
        app.symbol_practice.typing_indices = vec![0];
        app.symbol_practice.dictation_indices = vec![1];
        app.symbol_topics[0].exercises[1].kind = Some(ExerciseKind::Dictation);
        input(app, "alpha\nbeta");
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('m')));
        assert!(matches!(
            app.state,
            AppState::SymbolLesson {
                phase: SymbolPhase::Practice,
                ..
            }
        ));
        assert_eq!(app.symbol_practice.current_input, "m");
        assert_eq!(app.history.len(), 1);
    });
}

#[test]
fn step_output_reading_never_adds_typing_duration_or_a_cross_step_speed_sample() {
    use std::time::{Duration, Instant};
    with_app(|app, _| {
        setup(app, Entry::Main, "next");
        let now = Instant::now();
        let wall = chrono::Utc::now().timestamp_millis();
        for (index, ch) in "alpha".chars().enumerate() {
            app.typing_engine.input_at(
                ch,
                now + Duration::from_millis(index as u64 * 100),
                wall + index as i64 * 100,
            );
        }
        app.handle_key(key(KeyCode::Enter));
        let reading = app.typing_engine.finish_at(
            "input-fixture-0",
            Difficulty::Basic,
            RecordMode::Typing,
            now + Duration::from_secs(120),
            wall + 120000,
        );
        assert_eq!(reading.typing.unwrap().active_duration_ms, 400);
        app.handle_key(key(KeyCode::Enter));
        for (index, ch) in "beta".chars().enumerate() {
            app.typing_engine.input_at(
                ch,
                now + Duration::from_millis(121000 + index as u64 * 100),
                wall + 121000 + index as i64 * 100,
            );
        }
        let completed = app.typing_engine.finish_at(
            "input-fixture-0",
            Difficulty::Basic,
            RecordMode::Typing,
            now + Duration::from_secs(180),
            wall + 180000,
        );
        let typing = completed.typing.unwrap();
        assert_eq!(typing.active_duration_ms, 700);
        assert!(
            !typing
                .positions
                .iter()
                .find(|position| position.expected == 'b')
                .unwrap()
                .speed_sample_valid
        );
    });
}

#[test]
fn legacy_sequences_load_but_nested_multiline_steps_are_rejected_before_rendering() {
    with_app(|app, path| {
        let directory = path.join("data/sequences");
        fs::create_dir_all(&directory).unwrap();
        let file = directory.join("legacy.toml");
        fs::write(
            &file,
            "[[sequences]]\nid = 'legacy'\ncommands = ['alpha', 'beta']\n",
        )
        .unwrap();
        let mut command = Command {
            id: "legacy".into(),
            command: "alpha\nbeta".into(),
            ..Command::default()
        };
        let loaded =
            cmdtyper::data::sequence_loader::load_sequences(&path.join("data"), &[command.clone()])
                .unwrap();
        assert_eq!(loaded[0].steps.len(), 2);
        assert_eq!(loaded[0].steps[1].command, "beta");
        command.command = "alpha\ninner\nbeta".into();
        fs::write(
            &file,
            "[[sequences]]\nid = 'legacy'\ncommands = [\"alpha\\ninner\", 'beta']\n",
        )
        .unwrap();
        assert!(
            cmdtyper::data::sequence_loader::load_sequences(&path.join("data"), &[command])
                .is_err()
        );
        assert!(!app.commands.is_empty());
    });
}

#[test]
fn long_workflow_target_keeps_the_cursor_visible_in_a_small_terminal() {
    with_app(|app, _| {
        setup(app, Entry::Main, "next");
        let long = "a".repeat(200);
        let target = format!("{long}\nbeta");
        app.sequences[0].commands[0] = long.clone();
        app.sequences[0].steps[0].command = long.clone();
        app.sequences[0].steps[0].explanation = "教学前提和上下文。".repeat(12);
        app.commands[0].command = target.clone();
        app.typing_commands[0].command = target.clone();
        app.typing_engine.reset(&target);
        for count in [0, 50, 150, 200] {
            while app.typing_engine.cursor < count {
                app.handle_key(key(KeyCode::Char('a')));
            }
            let mut terminal =
                ratatui::Terminal::new(ratatui::backend::TestBackend::new(40, 10)).unwrap();
            terminal
                .draw(|frame| cmdtyper::ui::render(frame, app))
                .unwrap();
            assert!(
                terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .any(|cell| cell.bg == cmdtyper::ui::widgets::CURSOR_BG),
                "cursor {count} should remain visible while the long input wraps"
            );
        }
    });
}

#[test]
fn workflow_display_shortcuts_change_presentation_without_moving_or_restarting_input() {
    use cmdtyper::data::models::TypingDisplayMode;
    with_app(|app, _| {
        setup(app, Entry::Main, "next");
        app.typing_mode = TypingDisplayMode::Standard;
        app.show_hint = true;
        input(app, "al");
        assert!(text(app).contains("FIRST CONTEXT"));
        app.handle_key(key(KeyCode::F(3)));
        assert!(!text(app).contains("FIRST CONTEXT"));
        app.handle_key(key(KeyCode::F(2)));
        assert_eq!(app.typing_mode, TypingDisplayMode::Detailed);
        app.handle_key(key(KeyCode::F(2)));
        assert_eq!(app.typing_mode, TypingDisplayMode::Terminal);
        assert!(!text(app).contains("WPM"));
        assert_eq!(app.typing_engine.cursor, 2);
        assert!(app.history.is_empty());
        input(app, "pha");
        app.handle_key(key(KeyCode::Enter));
        assert!(text(app).contains("FIRST OUTPUT"));
    });
}

#[test]
fn system_right_submits_each_workflow_step_and_holds_failed_final_save() {
    with_app(|app, path| {
        setup(app, Entry::System, "next");
        input(app, "alpha");
        app.handle_key(key(KeyCode::Right));
        assert!(output(app));
        assert!(text(app).contains("FIRST OUTPUT"));
        input(app, "beta");

        fs::create_dir(path.join("history.json")).unwrap();
        for _ in 0..2 {
            app.handle_key(key(KeyCode::Right));
            assert_eq!(
                app.typing_engine.target.iter().collect::<String>(),
                "alpha\nbeta"
            );
            assert!(app.typing_engine.is_complete());
            assert!(app.history.is_empty());
            assert!(!output(app));
            assert!(app.persistence_error.is_some());
        }
        fs::remove_dir(path.join("history.json")).unwrap();
        app.handle_key(key(KeyCode::Right));
        assert!(output(app));
        assert!(text(app).contains("SECOND OUTPUT"));
        assert_eq!(app.history.len(), 1);
        app.handle_key(key(KeyCode::Right));
        assert_eq!(app.typing_engine.target.iter().collect::<String>(), "next");
        assert_eq!(app.typing_engine.cursor, 0);
        assert!(!output(app));
        assert_eq!(app.history.len(), 1);
        assert!(app.history[0].is_completed());
    });
}

#[test]
fn system_navigation_aliases_save_before_output_or_advance_without_a_workflow() {
    for (submit, advance, has_output) in [
        (KeyCode::Right, KeyCode::Right, true),
        (KeyCode::Char('l'), KeyCode::Char('n'), true),
        (KeyCode::Right, KeyCode::Right, false),
    ] {
        with_app(|app, path| {
            setup(app, Entry::System, "next");
            app.sequences.clear();
            if !has_output {
                app.system_topics[0].sections[0].commands[0].simulated_output = None;
            }
            input(app, "alpha\nbeta");
            fs::create_dir(path.join("history.json")).unwrap();
            app.handle_key(key(submit));
            assert_eq!(
                app.typing_engine.target.iter().collect::<String>(),
                "alpha\nbeta"
            );
            assert!(app.typing_engine.is_complete());
            assert!(!app.system_typing_showing_output);
            assert!(app.history.is_empty());
            assert!(app.persistence_error.is_some());

            fs::remove_dir(path.join("history.json")).unwrap();
            app.handle_key(key(submit));
            assert_eq!(app.history.len(), 1);
            assert!(app.history[0].is_completed());
            assert_eq!(app.history[0].command_id, "input-fixture-0");
            assert_eq!(app.history[0].mode, RecordMode::SystemTyping);
            assert_eq!(app.system_typing_showing_output, has_output);
            if has_output {
                app.handle_key(key(advance));
            }
            assert_eq!(app.typing_engine.target.iter().collect::<String>(), "next");
            assert_eq!(
                app.typing_engine.cursor,
                usize::from(advance == KeyCode::Char('n'))
            );
            assert_eq!(app.history.len(), 1);
            assert_eq!(app.progress_store.load_history().unwrap(), app.history);
        });
    }
}

#[test]
fn system_l_submission_shows_final_step_output_then_delivers_the_next_l() {
    with_app(|app, path| {
        setup(app, Entry::System, "ls");
        input(app, "alpha\nbeta");
        fs::create_dir(path.join("history.json")).unwrap();
        app.handle_key(key(KeyCode::Char('l')));
        assert!(app.typing_engine.is_complete());
        assert!(!output(app));
        assert!(app.history.is_empty());
        assert!(app.persistence_error.is_some());
        fs::remove_dir(path.join("history.json")).unwrap();

        app.handle_key(key(KeyCode::Char('l')));
        assert!(output(app));
        assert!(text(app).contains("SECOND OUTPUT"));
        app.handle_key(key(KeyCode::Char('l')));
        assert_eq!(app.typing_engine.target.iter().collect::<String>(), "ls");
        assert_eq!(app.typing_engine.cursor, 1);
        assert_eq!(app.history.len(), 1);
        assert!(app.history[0].is_completed());
    });
}

#[test]
fn tab_after_final_workflow_output_completes_once_for_last_and_nonlast_targets() {
    for last in [false, true] {
        with_app(|app, _| {
            setup(app, Entry::Main, "next");
            if last {
                app.typing_commands.truncate(1);
            }
            input(app, "alpha\nbeta");
            app.handle_key(key(KeyCode::Enter));
            assert!(output(app));
            app.handle_key(key(KeyCode::Tab));
            assert_eq!(app.typing_index, 1);
            assert_eq!(app.typing_round_records.len(), 1);
            assert_eq!(app.history.len(), 1);
            assert!(app.history[0].is_completed());
            assert_eq!(app.typing_round_records[0], app.history[0]);
            assert!(!output(app));
            assert_eq!(app.typing_is_finished(), last);
            if last {
                let summary: String = text(app)
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .collect();
                assert!(summary.contains("本轮完成！1条"));
                app.handle_key(key(KeyCode::Enter));
                assert_eq!(app.state, AppState::Home);
            } else {
                assert_eq!(app.typing_engine.cursor, 0);
                assert_eq!(app.typing_engine.target.iter().collect::<String>(), "next");
                app.handle_key(key(KeyCode::Char('n')));
                assert_eq!(app.typing_engine.cursor, 1);
            }
            assert_eq!(app.history.len(), 1);
            assert_eq!(app.typing_round_records.len(), 1);
        });
    }
}

#[test]
fn tab_during_intermediate_output_still_skips_and_saves_only_a_partial() {
    with_app(|app, _| {
        setup(app, Entry::Main, "next");
        input(app, "alpha\n");
        assert!(output(app));
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.typing_index, 1);
        assert_eq!(app.typing_engine.cursor, 0);
        assert_eq!(app.history.len(), 1);
        assert!(!app.history[0].is_completed());
        assert!(app.typing_round_records.is_empty());
        assert!(!output(app));
        app.handle_key(key(KeyCode::Char('n')));
        assert_eq!(app.typing_engine.cursor, 1);
    });
}
