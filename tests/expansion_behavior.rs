use cmdtyper::{
    app::{App, AppState},
    core::engine::TypingEngine,
    data::models::{Command, Difficulty, RecordMode, TypingDisplayMode},
    ui,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::{
    env, fs,
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
static ENV_LOCK: Mutex<()> = Mutex::new(());
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn with_app(f: impl FnOnce(&mut App)) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = env::temp_dir().join(format!(
        "cmdtyper-expansion-{}",
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
    let mut app = App::new().expect("expanded data loads");
    f(&mut app);
    unsafe {
        if let Some(v) = old_data {
            env::set_var("CMDTYPER_DATA_DIR", v)
        } else {
            env::remove_var("CMDTYPER_DATA_DIR")
        };
        if let Some(v) = old_user {
            env::set_var("CMDTYPER_USER_DIR", v)
        } else {
            env::remove_var("CMDTYPER_USER_DIR")
        };
    }
    fs::remove_dir_all(path).ok();
}
fn command(id: &str, text: &str, output: Option<&str>) -> Command {
    Command {
        id: id.into(),
        command: text.into(),
        simulated_output: output.map(String::from),
        ..Default::default()
    }
}
fn start(app: &mut App, commands: Vec<Command>) {
    app.typing_engine.reset(&commands[0].command);
    app.typing_commands = commands;
    app.typing_index = 0;
    app.state = AppState::Typing;
    app.typing_showing_output = false;
    app.terminal_auto_advance = false;
}
fn render(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| ui::render(f, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>()
}
#[test]
fn workflow_enter_advances_line_without_becoming_a_character_or_completing_early() {
    with_app(|app| {
        start(app, vec![command("steps-test", "a\nb", None)]);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.typing_engine.cursor, 0);
        app.handle_key(key(KeyCode::Char('a')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.typing_engine.cursor, 2);
        assert!(app.history.is_empty());
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.typing_engine.cursor, 2, "submitted line is immutable");
        app.handle_key(key(KeyCode::Char('b')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.history.len(), 1);
        let record = &app.history[0];
        assert_eq!(record.typing.as_ref().unwrap().positions.len(), 2);
        assert_eq!(record.error_count, 0);
    });
}
#[test]
fn first_character_after_output_is_delivered_even_when_it_is_a_menu_shortcut() {
    with_app(|app| {
        for mode in [
            TypingDisplayMode::Terminal,
            TypingDisplayMode::Standard,
            TypingDisplayMode::Detailed,
        ] {
            start(
                app,
                vec![
                    command("output-a", "a", Some("ready")),
                    command("output-m", "mkdir", None),
                ],
            );
            app.typing_mode = mode.clone();
            app.handle_key(key(KeyCode::Char('a')));
            app.handle_key(key(KeyCode::Enter));
            assert!(app.typing_showing_output);
            app.handle_key(key(KeyCode::Char('m')));
            assert_eq!(app.typing_index, 1);
            assert_eq!(app.typing_engine.cursor, 1);
            assert_eq!(app.typing_mode, mode);
        }
    });
}
#[test]
fn abandoning_a_wrong_first_position_saves_once_without_mastery() {
    with_app(|app| {
        start(app, vec![command("partial-error", "ab", None)]);
        app.handle_key(key(KeyCode::Char('x')));
        app.handle_key(key(KeyCode::Char('y')));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history[0].error_count, 1);
        assert!(!app.history[0].is_completed());
        assert_eq!(app.user_stats.error_positions, 1);
        assert!(
            !app.user_stats
                .command_progress
                .iter()
                .any(|p| p.command_id == "partial-error")
        );
        let record = app.history[0].clone();
        app.persist_record(record);
        assert_eq!(app.history.len(), 1);
        let store = app.progress_store.load_history().unwrap();
        assert_eq!(store.len(), 1);
    });
}
#[test]
fn same_session_snapshot_can_be_completed_without_double_counting() {
    with_app(|app| {
        start(app, vec![command("snapshot-target", "ab", None)]);
        app.handle_key(key(KeyCode::Char('a')));
        app.save_active_typing();
        assert_eq!(app.history.len(), 1);
        app.handle_key(key(KeyCode::Char('b')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.history.len(), 1);
        assert!(app.history[0].is_completed());
        assert_eq!(app.user_stats.attempted_positions, 2);
        assert_eq!(
            app.user_stats
                .command_progress
                .iter()
                .find(|p| p.command_id == "snapshot-target")
                .unwrap()
                .times_practiced,
            1
        );
    });
}
#[test]
fn learning_hub_routes_to_topics_and_new_modules_without_duplicate_difficulty_shortcuts() {
    with_app(|app| {
        for (index, expected) in [
            (0, AppState::CommandTopics),
            (1, AppState::SymbolTopics),
            (2, AppState::SystemTopics),
            (3, AppState::ReviewTopics),
            (4, AppState::PracticeTopics),
            (5, AppState::Scenarios),
        ] {
            app.state = AppState::LearnHub;
            app.learn_hub_index = index;
            app.handle_key(key(KeyCode::Enter));
            assert_eq!(app.state, expected);
        }
        app.state = AppState::Home;
        app.home_index = 5;
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.state, AppState::Calendar);
        let date = app.calendar_state.selected_date;
        app.handle_key(key(KeyCode::Left));
        assert!(app.calendar_state.selected_date < date);
    });
}
#[test]
fn completed_practice_group_advances_through_one_teaching_and_three_exercises() {
    with_app(|app| {
        assert!(!app.practice_groups.is_empty());
        cmdtyper::flow::practice_flow::enter(app, None, None);
        app.handle_key(key(KeyCode::Enter));
        for step in 0..4 {
            assert_eq!(app.practice_state.step, step);
            let target = app.typing_engine.target.clone();
            for ch in target {
                app.handle_key(key(if ch == '\n' {
                    KeyCode::Enter
                } else {
                    KeyCode::Char(ch)
                }));
            }
            app.handle_key(key(KeyCode::Enter));
            assert!(app.practice_state.output);
            app.handle_key(key(KeyCode::Enter));
        }
        assert!(app.practice_state.done);
        assert_eq!(app.history.len(), 4);
    });
}
#[test]
fn screenshot_artifacts_cover_calendar_workflow_topics_and_scenario_at_multiple_sizes() {
    with_app(|app| {
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/ui-validation");
        fs::create_dir_all(&out).unwrap();
        let now = Instant::now();
        let wall = chrono::Utc::now().timestamp_millis();
        let mut engine = TypingEngine::new("aaaaaaaaaaaa");
        for i in 0..12 {
            engine.input_at(
                'a',
                now + Duration::from_millis(i * 200),
                wall + i as i64 * 200,
            );
        }
        app.persist_record(engine.finish_at(
            "render-sample",
            Difficulty::Basic,
            RecordMode::Typing,
            now + Duration::from_secs(3),
            wall + 3000,
        ));
        for (name, state) in [
            ("home", AppState::Home),
            ("calendar", AppState::Calendar),
            ("learn", AppState::LearnHub),
            ("scenarios", AppState::Scenarios),
        ] {
            app.state = state;
            for (w, h) in [(40, 10), (80, 24), (120, 36)] {
                let text = render(app, w, h);
                assert!(!text.is_empty());
                if name == "calendar" {
                    assert!(text.contains("WPM"));
                    assert!(text.contains("CPM"));
                    assert!(text.replace(' ', "").contains("准确率"));
                }
                let rows = text.chars().collect::<Vec<_>>();
                let _ = rows;
                let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
                terminal.draw(|f| ui::render(f, app)).unwrap();
                let buffer = terminal.backend().buffer();
                let lines = (0..h)
                    .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(out.join(format!("{name}-{w}x{h}.txt")), lines).unwrap();
            }
        }
        start(
            app,
            vec![command(
                "workflow-render",
                "mkdir project\ncd project\npwd",
                None,
            )],
        );
        for ch in "mkdir project".chars() {
            app.handle_key(key(KeyCode::Char(ch)));
        }
        app.handle_key(key(KeyCode::Enter));
        let text = render(app, 80, 24);
        assert!(text.contains("Enter"));
        assert!(text.contains("F2"));
        assert!(text.contains("F3"));
    });
}
