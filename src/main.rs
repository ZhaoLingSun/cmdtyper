mod app;
mod core;
mod data;
mod event;
mod ui;

use std::io;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, AppState};
use event::{poll_event, AppEvent};

fn main() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new();

    loop {
        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll events
        match poll_event()? {
            AppEvent::Key(key) => {
                // Global Ctrl+C handler
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }

                let new_state = match &app.state {
                    AppState::Home => ui::home::handle_key(key, &mut app),
                    AppState::Typing => ui::typing::handle_key(key, &mut app),
                    AppState::Learn => ui::learn::handle_key(key, &mut app),
                    AppState::Quitting => Some(AppState::Quitting),
                };

                if let Some(state) = new_state {
                    app.state = state;
                }
            }
            AppEvent::Tick => {
                // Tick events: update animations, clear expired error flashes, etc.
            }
            AppEvent::Resize(_, _) => {
                // Terminal will re-render on next loop iteration
            }
        }

        if matches!(app.state, AppState::Quitting) {
            break;
        }
    }

    Ok(())
}
