use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use cmdtyper::app::{App, AppState};
use cmdtyper::data::command_loader::load_command_catalog;
use cmdtyper::data::models::RecordMode;
use cmdtyper::data::scenario_loader::{
    Scenario, ScenarioPhase, ScenarioState, hydrate_scenarios, load_scenarios, load_state,
    save_state, validate_scenario,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn cases() -> Vec<Scenario> {
    let mut cases = load_scenarios(&data_dir().join("scenarios")).unwrap();
    let catalog = load_command_catalog(&data_dir()).unwrap();
    hydrate_scenarios(&mut cases, &catalog.commands).unwrap();
    cases
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn twenty_cases_have_substantive_steps_decisions_and_sources() {
    let cases = cases();
    assert!(cases.len() >= 20);
    let mut ids = BTreeSet::new();
    for case in &cases {
        assert!(ids.insert(&case.id));
        assert!(case.steps.len() >= 8, "{}", case.id);
        assert!(case.steps.iter().filter(|s| s.decision.is_some()).count() >= 2);
        assert!(!case.sources.is_empty());
        assert!(
            !case.background.is_empty() && !case.objective.is_empty() && !case.recap.is_empty()
        );
        for step in &case.steps {
            assert!(!step.command.is_empty());
            assert!(!step.command.contains(['\n', '\r']));
            assert!(!step.output.is_empty());
        }
    }
    let website = cases
        .iter()
        .find(|case| case.id == "website_bootstrap")
        .unwrap();
    let commands = website
        .steps
        .iter()
        .map(|s| s.command.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(commands.contains("docker compose up -d --build"));
    assert!(commands.contains("certbot certonly --webroot"));
    assert!(commands.contains("ops-cert-renew.timer"));
    assert!(commands.contains("git tag -a v1.0.1"));
    assert!(
        website
            .steps
            .iter()
            .any(|s| s.output.contains("service_healthy") && s.output.contains("pgdata"))
    );
}

#[test]
fn every_diagnostic_wrong_branch_returns_to_evidence_and_correct_path_finishes() {
    for case in cases() {
        let mut state = ScenarioState::default();
        state.start(&case, true);
        for index in 0..case.steps.len() {
            assert_eq!(state.step_index, index);
            state.phase = ScenarioPhase::Evidence;
            if let Some(decision) = &case.steps[index].decision {
                state.phase = ScenarioPhase::Decision;
                let wrong = decision.choices.iter().position(|c| !c.correct).unwrap();
                assert!(!state.answer(&case, wrong));
                assert_eq!(state.step_index, index);
                assert_eq!(state.phase, ScenarioPhase::Evidence);
                assert!(!state.feedback.as_ref().unwrap().is_empty());
                state.phase = ScenarioPhase::Decision;
                let right = decision.choices.iter().position(|c| c.correct).unwrap();
                assert!(state.answer(&case, right));
            } else {
                state.advance(&case);
            }
        }
        assert_eq!(state.phase, ScenarioPhase::Recap);
        assert!(state.saved[&case.id].completed);
    }
}

#[test]
fn all_twenty_cases_complete_through_app_input_evidence_and_diagnostic_choices() {
    with_app(|app, user_dir| {
        let cases = app.scenario_catalog.clone();
        assert!(cases.len() >= 20);
        let mut record_ids = BTreeSet::new();
        let mut completed_steps = 0;
        let mut answered_decisions = 0;
        app.state = AppState::Scenarios;

        for (case_index, case) in cases.iter().enumerate() {
            assert_eq!(app.state, AppState::Scenarios);
            assert_eq!(app.scenario_state.selected_index, case_index);
            let history_start = app.history.len();
            app.handle_key(key(KeyCode::Enter));
            assert_eq!(app.state, AppState::ScenarioPractice);
            assert_eq!(
                app.scenario_state.active_id.as_deref(),
                Some(case.id.as_str())
            );
            assert_eq!(app.scenario_state.phase, ScenarioPhase::Intro);
            app.handle_key(key(KeyCode::Enter));

            for (step_index, step) in case.steps.iter().enumerate() {
                let context = format!("{}:{}", case.id, step.id);
                assert_eq!(app.scenario_state.phase, ScenarioPhase::Typing, "{context}");
                assert_eq!(app.scenario_state.step_index, step_index, "{context}");
                assert_eq!(
                    app.typing_engine.target.iter().collect::<String>(),
                    step.command,
                    "{context}"
                );
                assert_eq!(app.typing_engine.cursor, 0, "{context}");

                // Use the public key route for both ignored early submission and
                // every target character; no phase or engine state is fabricated.
                app.handle_key(key(KeyCode::Enter));
                assert_eq!(app.scenario_state.phase, ScenarioPhase::Typing, "{context}");
                assert_eq!(app.history.len(), completed_steps, "{context}");
                for character in step.command.chars() {
                    app.handle_key(key(KeyCode::Char(character)));
                }
                assert!(app.typing_engine.is_complete(), "{context}");
                app.handle_key(key(KeyCode::Enter));
                completed_steps += 1;
                assert_eq!(
                    app.scenario_state.phase,
                    ScenarioPhase::Evidence,
                    "{context}"
                );
                assert_eq!(app.scenario_state.step_index, step_index, "{context}");
                assert_eq!(app.history.len(), completed_steps, "{context}");
                assert!(app.persistence_error.is_none(), "{context}");

                let record = app.history.last().unwrap();
                assert_eq!(record.mode, RecordMode::ScenarioTyping, "{context}");
                assert_eq!(
                    record.command_id,
                    *step
                        .command_id
                        .as_ref()
                        .expect("canonical scenario command"),
                    "{context}"
                );
                assert!(record.is_completed(), "{context}");
                assert_eq!(record.accuracy, 1.0, "{context}");
                assert!(
                    record_ids.insert(record.id.clone()),
                    "duplicate record: {context}"
                );

                // Check the actual evidence screen displays this step's output.
                let output_prefix = step
                    .output
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .unwrap()
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .take(30)
                    .collect::<String>();
                assert!(
                    scenario_screen_text(app).contains(&output_prefix),
                    "missing output: {context}"
                );
                app.handle_key(key(KeyCode::Enter));

                if let Some(decision) = &step.decision {
                    assert_eq!(
                        app.scenario_state.phase,
                        ScenarioPhase::Decision,
                        "{context}"
                    );
                    assert_eq!(app.scenario_state.choice_index, 0, "{context}");
                    let wrong = decision
                        .choices
                        .iter()
                        .position(|choice| !choice.correct)
                        .unwrap();
                    for _ in 0..wrong {
                        app.handle_key(key(KeyCode::Down));
                    }
                    assert_eq!(app.scenario_state.choice_index, wrong, "{context}");
                    app.handle_key(key(KeyCode::Enter));
                    assert_eq!(
                        app.scenario_state.phase,
                        ScenarioPhase::Evidence,
                        "{context}"
                    );
                    assert_eq!(app.scenario_state.step_index, step_index, "{context}");
                    assert_eq!(
                        app.scenario_state.feedback.as_deref(),
                        Some(decision.choices[wrong].feedback.as_str()),
                        "{context}"
                    );
                    assert_eq!(
                        app.history.len(),
                        completed_steps,
                        "wrong answer duplicated typing: {context}"
                    );

                    app.handle_key(key(KeyCode::Enter));
                    assert_eq!(
                        app.scenario_state.phase,
                        ScenarioPhase::Decision,
                        "{context}"
                    );
                    assert_eq!(app.scenario_state.choice_index, 0, "{context}");
                    let correct = decision
                        .choices
                        .iter()
                        .position(|choice| choice.correct)
                        .unwrap();
                    for _ in 0..correct {
                        app.handle_key(key(KeyCode::Down));
                    }
                    assert_eq!(app.scenario_state.choice_index, correct, "{context}");
                    app.handle_key(key(KeyCode::Enter));
                    answered_decisions += 1;
                }

                assert_eq!(
                    app.history.len(),
                    completed_steps,
                    "navigation duplicated typing: {context}"
                );
                if step_index + 1 < case.steps.len() {
                    assert_eq!(app.scenario_state.phase, ScenarioPhase::Typing, "{context}");
                    assert_eq!(app.scenario_state.step_index, step_index + 1, "{context}");
                } else {
                    assert_eq!(app.scenario_state.phase, ScenarioPhase::Recap, "{context}");
                }
            }

            assert!(app.scenario_state.saved[&case.id].completed, "{}", case.id);
            assert!(
                load_state(user_dir).saved[&case.id].completed,
                "{}",
                case.id
            );
            assert!(
                scenario_screen_text(app).contains("案例完成"),
                "{}",
                case.id
            );
            let records = &app.history[history_start..];
            assert_eq!(records.len(), case.steps.len(), "{}", case.id);
            assert_eq!(
                records
                    .iter()
                    .map(|record| record.command_id.as_str())
                    .collect::<Vec<_>>(),
                case.steps
                    .iter()
                    .map(|step| step.command_id.as_deref().unwrap())
                    .collect::<Vec<_>>(),
                "{}",
                case.id
            );
            app.handle_key(key(KeyCode::Enter));
            assert_eq!(app.state, AppState::Scenarios, "{}", case.id);
            assert_eq!(app.history.len(), completed_steps, "{}", case.id);
            if case_index + 1 < cases.len() {
                app.handle_key(key(KeyCode::Down));
            }
        }

        assert_eq!(
            completed_steps,
            cases.iter().map(|case| case.steps.len()).sum::<usize>()
        );
        assert_eq!(
            answered_decisions,
            cases
                .iter()
                .flat_map(|case| &case.steps)
                .filter(|step| step.decision.is_some())
                .count()
        );
        assert_eq!(record_ids.len(), completed_steps);
        assert_eq!(app.progress_store.load_history().unwrap(), app.history);
        let saved = load_state(user_dir);
        assert!(cases.iter().all(|case| saved.saved[&case.id].completed));
    });
}

fn scenario_screen_text(app: &App) -> String {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal
        .draw(|frame| cmdtyper::ui::render(frame, app))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .flat_map(|cell| cell.symbol().chars())
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn independent_case_progress_survives_restart_and_explicit_retry() {
    let cases = cases();
    let mut state = ScenarioState::default();
    state.start(&cases[0], false);
    state.step_index = 3;
    state.phase = ScenarioPhase::Evidence;
    state.checkpoint();
    state.start(&cases[1], false);
    state.step_index = 2;
    state.phase = ScenarioPhase::Decision;
    state.checkpoint();
    let dir = unique_dir();
    save_state(&dir, &state).unwrap();
    assert!(!dir.join("scenario_progress.json.tmp").exists());
    let mut loaded = load_state(&dir);
    loaded.start(&cases[0], false);
    assert_eq!(loaded.step_index, 3);
    assert_eq!(loaded.phase, ScenarioPhase::Evidence);
    loaded.start(&cases[0], true);
    assert_eq!(loaded.step_index, 0);
    assert_eq!(loaded.phase, ScenarioPhase::Intro);
    assert_eq!(loaded.saved[&cases[1].id].step_index, 2);
    fs::write(dir.join("scenario_progress.json"), "bad json").unwrap();
    assert!(load_state(&dir).saved.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn referenced_commands_are_required_and_embedded_disagreements_rejected() {
    let commands = load_command_catalog(&data_dir()).unwrap().commands;
    let mut case = cases().remove(0);
    case.steps[0].command_id = Some(commands[0].id.clone());
    case.steps[0].command.clear();
    hydrate_scenarios(std::slice::from_mut(&mut case), &commands).unwrap();
    assert_eq!(case.steps[0].command, commands[0].command);
    case.steps[0].command = "definitely different".into();
    assert!(hydrate_scenarios(std::slice::from_mut(&mut case), &commands).is_err());
    case.steps[0].command.clear();
    case.steps[0].command_id = Some("missing-command-id".into());
    assert!(hydrate_scenarios(std::slice::from_mut(&mut case), &commands).is_err());
    case.steps[0].command_id = None;
    assert!(validate_scenario(&case).is_err());
}

#[test]
fn enter_requires_complete_target_and_evidence_resume_does_not_record_twice() {
    with_app(|app, user_dir| {
        app.state = AppState::Scenarios;
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Intro);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Typing);
        app.handle_key(key(KeyCode::Enter));
        assert!(app.history.is_empty());
        for c in app.typing_engine.target.clone() {
            app.handle_key(key(KeyCode::Char(c)));
        }
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Evidence);
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history[0].mode, RecordMode::ScenarioTyping);
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.state, AppState::Scenarios);
        assert_eq!(app.history.len(), 1);
        let saved = load_state(user_dir);
        assert_eq!(saved.phase, ScenarioPhase::Evidence);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Evidence);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.step_index, 1);
        assert_eq!(app.history.len(), 1);
        let first = app.typing_engine.target[0];
        app.handle_key(key(KeyCode::Char(first)));
        assert_eq!(app.typing_engine.cursor, 1);
    });
}

#[test]
fn wrong_first_character_is_saved_on_exit_and_retry_starts_a_new_attempt() {
    with_app(|app, _| {
        app.state = AppState::Scenarios;
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('!')));
        app.handle_key(key(KeyCode::Char('!')));
        assert_eq!(app.typing_engine.cursor, 0);
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history[0].error_count, 1);
        let first_id = app.history[0].id.clone();
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Typing);
        assert_eq!(app.typing_engine.cursor, 0);
        let first = app.typing_engine.target[0];
        app.handle_key(key(KeyCode::Char(first)));
        app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(app.scenario_state.phase, ScenarioPhase::Intro);
        assert_eq!(app.history.len(), 2);
        assert_ne!(first_id, app.history[1].id);
    });
}

#[test]
fn scenario_ui_renders_small_terminals_long_output_and_ghost_characters() {
    with_app(|app, _| {
        app.state = AppState::Scenarios;
        for (width, height) in [(1, 1), (30, 8), (80, 24)] {
            render(app, width, height);
        }
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Enter));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| cmdtyper::ui::render(frame, app))
            .unwrap();
        assert!(
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .any(|cell| cell.fg == ratatui::style::Color::DarkGray && cell.symbol() == "o")
        );
        app.scenario_state.active_id = Some("website_bootstrap".into());
        let case = app
            .scenario_catalog
            .iter()
            .find(|c| c.id == "website_bootstrap")
            .unwrap();
        app.scenario_state.step_index = case
            .steps
            .iter()
            .position(|s| s.command == "nano compose.yaml")
            .unwrap();
        app.scenario_state.phase = ScenarioPhase::Evidence;
        app.scenario_state.scroll = usize::MAX;
        for (width, height) in [(1, 1), (30, 8), (80, 24)] {
            render(app, width, height);
        }
    });
}

fn render(app: &App, width: u16, height: u16) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| cmdtyper::ui::render(frame, app))
        .unwrap();
}

fn unique_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("cmdtyper-scenarios-{}-{nanos}", std::process::id()))
}

fn with_app(action: impl FnOnce(&mut App, &Path)) {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _lock = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let dir = unique_dir();
    let previous_data = env::var_os("CMDTYPER_DATA_DIR");
    let previous_user = env::var_os("CMDTYPER_USER_DIR");
    // SAFETY: every test in this executable that reads or mutates these variables
    // holds the same mutex; pure loader tests use explicit paths.
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", data_dir());
        env::set_var("CMDTYPER_USER_DIR", &dir);
    }
    let mut app = App::new().unwrap();
    action(&mut app, &dir);
    unsafe {
        match previous_data {
            Some(value) => env::set_var("CMDTYPER_DATA_DIR", value),
            None => env::remove_var("CMDTYPER_DATA_DIR"),
        }
        match previous_user {
            Some(value) => env::set_var("CMDTYPER_USER_DIR", value),
            None => env::remove_var("CMDTYPER_USER_DIR"),
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
