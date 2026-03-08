pub mod dictation;
pub mod home;
pub mod learn;
pub mod stats;
pub mod typing;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, AppState};

/// Top-level render dispatcher
pub fn render(f: &mut Frame, app: &App) {
    match &app.state {
        AppState::Home => home::render(f, app),
        AppState::Typing => typing::render(f, app),
        AppState::Learn => learn::render(f, app),
        AppState::Quitting => {}
    }
}
