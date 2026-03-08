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
use core::scorer;
use data::progress::ProgressStore;
use event::{poll_event, AppEvent};
use ui::{
    dictation::handle_key as handle_dictation_key, home::handle_key as handle_home_key,
    learn::handle_key as handle_learn_key, round_result::handle_key as handle_round_result_key,
    stats::handle_key as handle_stats_key, typing::handle_key as handle_typing_key,
};

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

    // Load persistent data
    let store = ProgressStore::new().ok();
    if let Some(ref store) = store {
        if let Ok(stats) = store.load_stats() {
            app.user_stats = stats;
        }
        if let Ok(config) = store.load_config() {
            app.apply_config(config);
        }
    }

    loop {
        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll events
        match poll_event()? {
            AppEvent::Key(key) => {
                // Global Ctrl+C handler
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                let new_state = match app.state {
                    AppState::Home => handle_home_key(key, &mut app),
                    AppState::Typing => handle_typing_key(key, &mut app),
                    AppState::Learn => handle_learn_key(key, &mut app),
                    AppState::Dictation => handle_dictation_key(key, &mut app),
                    AppState::Stats => handle_stats_key(key, &mut app),
                    AppState::RoundResult => handle_round_result_key(key, &mut app),
                    AppState::Quitting => Some(AppState::Quitting),
                };

                if let Some(state) = new_state {
                    app.state = state;
                }

                // Persist completed exercise records
                if let Some(record) = app.take_pending_record() {
                    scorer::update_stats(&mut app.user_stats, &record);
                    if let Some(ref store) = store {
                        let _ = store.append_record(&record);
                        let _ = store.save_stats(&app.user_stats);
                    }
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

    // Save config on exit
    if let Some(ref store) = store {
        app.sync_user_config();
        let _ = store.save_config(&app.user_config);
        let _ = store.save_stats(&app.user_stats);
    }

    Ok(())
}
