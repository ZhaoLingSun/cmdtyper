pub mod dictation;
pub mod home;
pub mod learn;
pub mod round_result;
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
        AppState::Dictation => dictation::render(f, app),
        AppState::Stats => stats::render(f, app),
        AppState::RoundResult => round_result::render(f, app),
        AppState::Quitting => {}
    }
}
