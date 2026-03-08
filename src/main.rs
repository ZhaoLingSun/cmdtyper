use std::io;

use anyhow::Result;
use crossterm::{
    event::KeyEventKind,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

use cmdtyper::app::{App, AppState};
use cmdtyper::event::{AppEvent, poll_event};
use cmdtyper::ui;

fn main() -> Result<()> {
    // Terminal init
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run app
    let result = run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;

    loop {
        // Render
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        // Check quit state
        if app.state == AppState::Quitting {
            break;
        }

        // Handle events
        match poll_event()? {
            AppEvent::Key(key) => {
                // Only handle key press events (not release/repeat on some platforms)
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
            AppEvent::Tick => {
                // Tick events can be used for animations (error flash expiry, etc.)
            }
            AppEvent::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
            }
        }
    }

    Ok(())
}
