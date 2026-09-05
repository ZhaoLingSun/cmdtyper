//! Guided case navigation. Commands are only typing targets and never executed.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppState};
use crate::data::models::RecordMode;
use crate::data::scenario_loader::{self, Scenario, ScenarioPhase};

pub fn handle_topics_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.state = AppState::LearnHub,
        KeyCode::Up | KeyCode::Char('k') => {
            app.scenario_state.selected_index = app.scenario_state.selected_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scenario_state.selected_index = (app.scenario_state.selected_index + 1)
                .min(app.scenario_catalog.len().saturating_sub(1));
        }
        KeyCode::Enter | KeyCode::Char('r') => {
            let Some(scenario) = app
                .scenario_catalog
                .get(app.scenario_state.selected_index)
                .cloned()
            else {
                return;
            };
            app.scenario_state
                .start(&scenario, key.code == KeyCode::Char('r'));
            reset_target(app, &scenario);
            app.state = AppState::ScenarioPractice;
            persist(app);
        }
        _ => {}
    }
}

pub fn active_scenario(app: &App) -> Option<&Scenario> {
    app.scenario_state.active_id.as_deref().and_then(|id| {
        app.scenario_catalog
            .iter()
            .find(|scenario| scenario.id == id)
    })
}

pub fn handle_practice_key(app: &mut App, key: KeyEvent) {
    let Some(scenario) = active_scenario(app).cloned() else {
        app.state = AppState::Scenarios;
        return;
    };
    if key.code == KeyCode::Esc {
        save_partial(app, &scenario);
        persist(app);
        app.state = AppState::Scenarios;
        return;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
        save_partial(app, &scenario);
        app.scenario_state.start(&scenario, true);
        reset_target(app, &scenario);
        persist(app);
        return;
    }
    // Page keys never become target input and work in every phase.
    match key.code {
        KeyCode::PageUp => {
            app.scenario_state.scroll = app.scenario_state.scroll.saturating_sub(5);
            return;
        }
        KeyCode::PageDown => {
            app.scenario_state.scroll = app.scenario_state.scroll.saturating_add(5);
            return;
        }
        _ => {}
    }
    match app.scenario_state.phase {
        ScenarioPhase::Intro => {
            if key.code == KeyCode::Enter {
                app.scenario_state.phase = ScenarioPhase::Typing;
                reset_target(app, &scenario);
                persist(app);
            }
        }
        ScenarioPhase::Typing => match key.code {
            KeyCode::Backspace => app.typing_engine.backspace(),
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                app.typing_engine.input(c);
            }
            KeyCode::Enter if app.typing_engine.is_complete() => {
                record_typing(app, &scenario);
                app.scenario_state.phase = ScenarioPhase::Evidence;
                app.scenario_state.scroll = 0;
                app.scenario_state.feedback = None;
                persist(app);
            }
            _ => {}
        },
        ScenarioPhase::Evidence => {
            if key.code == KeyCode::Enter {
                app.scenario_state.feedback = None;
                app.scenario_state.scroll = 0;
                if scenario.steps[app.scenario_state.step_index]
                    .decision
                    .is_some()
                {
                    app.scenario_state.phase = ScenarioPhase::Decision;
                    app.scenario_state.choice_index = 0;
                } else {
                    app.scenario_state.advance(&scenario);
                    reset_target(app, &scenario);
                }
                persist(app);
            }
        }
        ScenarioPhase::Decision => {
            let count = scenario.steps[app.scenario_state.step_index]
                .decision
                .as_ref()
                .map(|decision| decision.choices.len())
                .unwrap_or(0);
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    app.scenario_state.choice_index =
                        app.scenario_state.choice_index.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.scenario_state.choice_index =
                        (app.scenario_state.choice_index + 1).min(count.saturating_sub(1));
                }
                KeyCode::Enter => {
                    let selected = app.scenario_state.choice_index;
                    if app.scenario_state.answer(&scenario, selected) {
                        reset_target(app, &scenario);
                    }
                    persist(app);
                }
                _ => {}
            }
        }
        ScenarioPhase::Recap => {
            if key.code == KeyCode::Enter {
                app.state = AppState::Scenarios;
                persist(app);
            }
        }
    }
}

fn reset_target(app: &mut App, scenario: &Scenario) {
    if let Some(step) = scenario.steps.get(app.scenario_state.step_index) {
        app.typing_engine.reset(&step.command);
    }
}

fn save_partial(app: &mut App, scenario: &Scenario) {
    if app.scenario_state.phase == ScenarioPhase::Typing && app.typing_engine.has_activity() {
        record_typing(app, scenario);
        reset_target(app, scenario);
    }
}

fn record_typing(app: &mut App, scenario: &Scenario) {
    let Some(step) = scenario.steps.get(app.scenario_state.step_index) else {
        return;
    };
    let fallback = format!("scenario:{}:{}", scenario.id, step.id);
    let command_id = step.command_id.as_deref().unwrap_or(&fallback);
    let record =
        app.typing_engine
            .finish(command_id, scenario.difficulty, RecordMode::ScenarioTyping);
    app.persist_record(record);
}

fn persist(app: &mut App) {
    app.scenario_state.checkpoint();
    if let Err(error) =
        scenario_loader::save_state(app.progress_store.base_dir(), &app.scenario_state)
    {
        app.scenario_state.feedback = Some(format!("进度保存失败：{error}"));
    }
}
