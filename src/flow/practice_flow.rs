use crate::{
    app::{App, AppState},
    data::models::RecordMode,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Default)]
pub struct PracticeState {
    pub selected: usize,
    pub filtered: Vec<usize>,
    pub step: usize,
    pub output: bool,
    pub done: bool,
    pub return_to: Option<Box<AppState>>,
}

pub fn enter(app: &mut App, kind: Option<&str>, source: Option<&str>) {
    app.practice_state = PracticeState {
        filtered: app
            .practice_groups
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                kind.is_none_or(|k| g.source_kind == k) && source.is_none_or(|s| g.source_id == s)
            })
            .map(|(i, _)| i)
            .collect(),
        return_to: Some(Box::new(app.state.clone())),
        ..PracticeState::default()
    };
    app.state = AppState::PracticeTopics;
}

fn current_group(app: &App) -> Option<&crate::data::practice_loader::PracticeGroup> {
    let index = *app
        .practice_state
        .filtered
        .get(app.practice_state.selected)?;
    app.practice_groups.get(index)
}

/// Foundation practice retains the source topic's calendar entry for both
/// completed commands and partial snapshots saved by the shared App gate.
pub fn current_record_mode(app: &App) -> Option<RecordMode> {
    match current_group(app)?.source_kind.as_str() {
        "lesson" => Some(RecordMode::LessonPractice),
        "symbol" => Some(RecordMode::SymbolTyping),
        "system" => Some(RecordMode::SystemTyping),
        _ => None,
    }
}

pub fn current_command(app: &App) -> Option<&crate::data::models::Command> {
    let group = current_group(app)?;
    let id = if app.practice_state.step == 0 {
        &group.teaching_command_id
    } else {
        group
            .exercise_command_ids
            .get(app.practice_state.step - 1)?
    };
    app.commands.iter().find(|c| &c.id == id)
}
fn reset(app: &mut App) {
    if let Some(command) = current_command(app).map(|c| c.command.clone()) {
        app.typing_engine.reset(&command);
    }
    app.practice_state.output = false;
}
pub fn handle_topics_key(app: &mut App, key: KeyEvent) {
    let len = app.practice_state.filtered.len();
    match key.code {
        KeyCode::Esc => {
            app.state = app
                .practice_state
                .return_to
                .take()
                .map(|s| *s)
                .unwrap_or(AppState::LearnHub)
        }
        KeyCode::Up => app.practice_state.selected = app.practice_state.selected.saturating_sub(1),
        KeyCode::Down => {
            app.practice_state.selected =
                (app.practice_state.selected + 1).min(len.saturating_sub(1))
        }
        KeyCode::PageUp => {
            app.practice_state.selected = app.practice_state.selected.saturating_sub(10)
        }
        KeyCode::PageDown => {
            app.practice_state.selected =
                (app.practice_state.selected + 10).min(len.saturating_sub(1))
        }
        KeyCode::Enter if len > 0 => {
            app.practice_state.step = 0;
            app.practice_state.done = false;
            reset(app);
            app.state = AppState::PracticeGroup;
        }
        _ => {}
    }
}
pub fn handle_group_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && key.code == KeyCode::Char('r')
    {
        if !app.practice_state.done {
            reset(app);
        }
        return;
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return;
    }
    if app.practice_state.output && !app.practice_state.done && matches!(key.code, KeyCode::Char(_))
    {
        advance(app);
    }
    match key.code {
        KeyCode::Esc => app.state = AppState::PracticeTopics,
        KeyCode::Enter if app.practice_state.done => app.state = AppState::PracticeTopics,
        KeyCode::Enter if app.practice_state.output => advance(app),
        KeyCode::Enter if app.typing_engine.is_complete() => {
            if let (Some(command), Some(mode)) = (current_command(app), current_record_mode(app)) {
                let record = app
                    .typing_engine
                    .finish(&command.id, command.difficulty, mode);
                app.persist_record(record);
            }
            app.practice_state.output = true;
        }
        KeyCode::Backspace if !app.practice_state.output && !app.practice_state.done => {
            app.typing_engine.backspace()
        }
        KeyCode::Char(ch) if !app.practice_state.output && !app.practice_state.done => {
            app.typing_engine.input(ch);
        }
        _ => {}
    }
}

fn advance(app: &mut App) {
    app.practice_state.step += 1;
    if current_command(app).is_none() {
        app.practice_state.done = true;
        app.practice_state.output = false;
    } else {
        reset(app);
    }
}
