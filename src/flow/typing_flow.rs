use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppState};
use crate::core::scorer;
use crate::data::models::RecordMode;

pub fn enter_typing(app: &mut App) {
    app.terminal_history.clear();
    app.typing_commands = if app.user_config.adaptive_recommend {
        scorer::recommend_commands(&app.user_stats, &app.commands, app.commands.len())
            .into_iter()
            .cloned()
            .collect()
    } else {
        app.commands.clone()
    };

    if app.typing_commands.is_empty() {
        return;
    }
    app.typing_index = 0;
    app.typing_round_records.clear();
    app.typing_showing_output = false;
    let cmd = &app.typing_commands[0];
    app.typing_engine.reset(&cmd.command);
    app.show_hint = app.user_config.show_token_hints;
    app.state = AppState::Typing;
}

pub fn handle_typing_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.typing_showing_output = false;
            app.state = AppState::Home;
        }
        KeyCode::Enter if app.typing_is_finished() => {
            app.typing_showing_output = false;
            app.state = AppState::Home;
        }
        KeyCode::Enter => typing_submit_or_advance(app),
        KeyCode::Char('h') | KeyCode::Char('H')
            if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
        {
            // If engine hasn't started or is complete, toggle hint
            if app.typing_engine.start_time.is_none() || app.typing_engine.is_complete() {
                app.show_hint = !app.show_hint;
            } else if !app.typing_showing_output {
                // Otherwise it's a regular char input
                handle_typing_char_input(app, key.code);
            }
        }
        KeyCode::Tab if !app.typing_is_finished() => typing_skip(app),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            typing_retry(app);
        }
        KeyCode::Backspace if !app.typing_showing_output && !app.typing_engine.is_complete() => {
            app.typing_engine.backspace();
        }
        KeyCode::Char(c) if !app.typing_showing_output => {
            handle_typing_char_input(app, KeyCode::Char(c));
        }
        _ => {}
    }
}

pub fn handle_typing_char_input(app: &mut App, key: KeyCode) {
    if let KeyCode::Char(c) = key {
        let _ = app.typing_engine.input(c);
    }
}

fn typing_submit_or_advance(app: &mut App) {
    if !app.typing_engine.is_complete() || app.typing_is_finished() {
        return;
    }

    if app.typing_showing_output {
        typing_finalize_current_command(app);
        return;
    }

    let has_output = app
        .typing_commands
        .get(app.typing_index)
        .and_then(|cmd| cmd.simulated_output.as_deref())
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);

    if has_output {
        app.typing_showing_output = true;
    } else {
        typing_finalize_current_command(app);
    }
}

fn typing_finalize_current_command(app: &mut App) {
    let Some(cmd) = app.typing_commands.get(app.typing_index) else {
        return;
    };

    let prompt = app.format_prompt();
    let display = cmd.display_text().to_string();
    let command_id = cmd.id.clone();
    let difficulty = cmd.difficulty;

    app.terminal_history.push_completed(&prompt, &display);

    // Record session
    let record = app
        .typing_engine
        .finish(&command_id, difficulty, RecordMode::Typing);
    scorer::update_stats(&mut app.user_stats, &record);
    let _ = app.progress_store.save_stats(&app.user_stats);
    let _ = app.progress_store.append_record(&record);
    app.history.push(record.clone());
    app.typing_round_records.push(record);

    // Advance to next command
    app.typing_showing_output = false;
    app.typing_index += 1;
    if app.typing_index < app.typing_commands.len() {
        let next_cmd = &app.typing_commands[app.typing_index];
        app.typing_engine.reset(&next_cmd.command);
    }
}

fn typing_skip(app: &mut App) {
    let cmd = &app.typing_commands[app.typing_index];
    let prompt = app.format_prompt();
    let display = cmd.display_text().to_string();
    app.terminal_history.push_completed(&prompt, &display);

    app.typing_showing_output = false;
    app.typing_index += 1;
    if app.typing_index < app.typing_commands.len() {
        let next_cmd = &app.typing_commands[app.typing_index];
        app.typing_engine.reset(&next_cmd.command);
    }
}

fn typing_retry(app: &mut App) {
    if !app.typing_commands.is_empty() && app.typing_index < app.typing_commands.len() {
        let cmd = &app.typing_commands[app.typing_index];
        app.typing_showing_output = false;
        app.typing_engine.reset(&cmd.command);
    }
}
