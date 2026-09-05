//! Shared step submission for full-command typing in every learning entry.
//! A workflow keeps one measured typing session; output reading never becomes input.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{App, AppState, SymbolPhase, SystemPhase},
    data::sequence_loader::CommandSequence,
};

#[derive(Debug, Clone, Copy)]
pub struct WorkflowOutput {
    pub step_index: usize,
    pub final_step: bool,
}

#[derive(Debug, Default)]
pub struct WorkflowState {
    session_id: String,
    pub output: Option<WorkflowOutput>,
    pub scroll: usize,
    pub manual_scroll: bool,
    handed_back: bool,
}

impl WorkflowState {
    pub fn output_for(&self, session_id: &str) -> Option<WorkflowOutput> {
        (self.session_id == session_id)
            .then_some(self.output)
            .flatten()
    }
}

pub fn active_sequence(app: &App) -> Option<&CommandSequence> {
    if !app.is_follow_typing_screen()
        || (app.workflow_state.session_id == app.typing_engine.session_id()
            && app.workflow_state.handed_back)
    {
        return None;
    }
    let (id, _, _) = app.active_typing_identity()?;
    app.sequences.iter().find(|sequence| {
        sequence.id == id
            && !sequence.steps.is_empty()
            && sequence.steps.len() == sequence.commands.len()
            && sequence
                .commands
                .join("\n")
                .chars()
                .eq(app.typing_engine.target.iter().copied())
    })
}

/// Called before normal screen dispatch. Returning false delivers this same key
/// to the screen, including the first character typed after intermediate output.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    let session_id = app.typing_engine.session_id().to_owned();
    if app.workflow_state.session_id != session_id {
        app.workflow_state = WorkflowState {
            session_id: session_id.clone(),
            ..WorkflowState::default()
        };
    }
    let Some(sequence) = active_sequence(app) else {
        app.workflow_state.output = None;
        return false;
    };
    let step_count = sequence.steps.len();
    let printable = matches!(key.code, KeyCode::Char(character) if !character.is_control())
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    let system_submit = matches!(
        app.state,
        AppState::SystemLesson {
            phase: SystemPhase::TypingPractice { .. },
            ..
        }
    ) && (key.code == KeyCode::Right
        || (key.code == KeyCode::Char('l')
            && app.typing_engine.is_complete()
            && app.workflow_state.output.is_none()));
    let enter = (key.code == KeyCode::Enter || system_submit)
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);

    match key.code {
        KeyCode::PageUp | KeyCode::Up => {
            app.workflow_state.manual_scroll = true;
            app.workflow_state.scroll = app.workflow_state.scroll.saturating_sub(5);
            return true;
        }
        KeyCode::PageDown | KeyCode::Down => {
            app.workflow_state.manual_scroll = true;
            app.workflow_state.scroll = app.workflow_state.scroll.saturating_add(5);
            return true;
        }
        _ => {}
    }
    if let Some(output) = app.workflow_state.output {
        if key.code == KeyCode::Backspace {
            return true;
        }
        let finish_with_tab =
            output.final_step && app.state == AppState::Typing && key.code == KeyCode::Tab;
        if !enter && !printable && !finish_with_tab {
            // Esc/Tab/Ctrl+R/Ctrl+C use the normal durable-save gate.
            return false;
        }
        app.workflow_state.output = None;
        app.workflow_state.scroll = 0;
        app.workflow_state.manual_scroll = false;
        if !output.final_step {
            app.typing_engine.resume();
            return enter;
        }
        // Each screen already owns its completion/progress transition. At most
        // two Enters submit then leave its old whole-command output state.
        for _ in 0..2 {
            app.handle_key_without_workflow(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
            if app.typing_engine.session_id() != session_id
                || !app.is_follow_typing_screen()
                || app.active_typing_identity().is_none()
                || (matches!(app.state, AppState::SymbolLesson { .. })
                    && app.symbol_practice.completed)
            {
                break;
            }
        }
        app.workflow_state.handed_back = true;
        let next_typing =
            app.typing_engine.session_id() != session_id && app.is_follow_typing_screen();
        let next_symbol_dictation = matches!(
            app.state,
            AppState::SymbolLesson {
                phase: SymbolPhase::Practice,
                ..
            }
        ) && !app.symbol_practice.completed;
        if printable && (next_typing || next_symbol_dictation) {
            app.handle_key_without_workflow(key);
        }
        return true;
    }

    if printable {
        app.workflow_state.manual_scroll = false;
    }
    if !enter {
        return false;
    }
    let step_index = app
        .typing_engine
        .target
        .iter()
        .take(app.typing_engine.cursor)
        .filter(|character| **character == '\n')
        .count();
    if app.typing_engine.target.get(app.typing_engine.cursor) == Some(&'\n') {
        app.typing_engine.input('\n');
        app.workflow_state.output = Some(WorkflowOutput {
            step_index,
            final_step: false,
        });
        app.workflow_state.scroll = 0;
        return true;
    }
    if app.typing_engine.is_complete() && step_index + 1 == step_count {
        // Scenario evidence and decisions remain explicit teaching steps.
        if app.state == AppState::ScenarioPractice {
            return false;
        }
        if app.save_active_typing() {
            app.workflow_state.output = Some(WorkflowOutput {
                step_index,
                final_step: true,
            });
            app.workflow_state.scroll = 0;
        }
        return true;
    }
    false
}
