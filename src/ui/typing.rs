use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::data::models::LineStatus;
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // terminal area
            Constraint::Length(1), // bottom bar
        ])
        .split(area);

    render_terminal_area(frame, app, chunks[0]);
    render_bottom_bar(frame, app, chunks[1]);
}

fn render_terminal_area(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let terminal_height = area.height as usize;

    // Completed lines from history
    let visible = app.terminal_history.visible_lines(area.height);
    for tl in visible {
        match tl.status {
            LineStatus::Completed => {
                lines.push(render_completed_line(&tl.prompt, &tl.command_display));
            }
            _ => {}
        }
    }

    // Current line (if we still have commands to type)
    if let Some(cmd) = app.current_typing_command() {
        let prompt = app.format_prompt();
        let display_text = cmd.display_text();

        // Handle multi-line commands (lines ending with \)
        let display_lines: Vec<&str> = if display_text.contains("\\\n") {
            display_text.split('\n').collect()
        } else {
            vec![display_text]
        };

        for (line_idx, display_line) in display_lines.iter().enumerate() {
            let line_prompt = if line_idx == 0 {
                prompt.clone()
            } else {
                "> ".to_string()
            };

            if line_idx == 0 {
                // Only the first line is the active typing line
                lines.push(render_current_line(
                    &line_prompt,
                    display_line,
                    &app.typing_engine,
                    line_idx == 0,
                ));
            } else {
                // Continuation lines shown as pending
                lines.push(Line::from(vec![
                    Span::styled(line_prompt, Style::default().fg(PROMPT_COLOR)),
                    Span::styled(display_line.to_string(), Style::default().fg(PENDING)),
                ]));
            }
        }
    } else if app.typing_is_finished() {
        lines.push(Line::from(Span::styled(
            "  \u{5168}\u{90e8}\u{5b8c}\u{6210}\u{ff01}\u{6309} Esc \u{8fd4}\u{56de}\u{4e3b}\u{83dc}\u{5355}",
            Style::default().fg(SUCCESS),
        )));
    }

    // Pad remaining lines
    while lines.len() < terminal_height {
        lines.push(Line::from(""));
    }

    // Only keep what fits
    if lines.len() > terminal_height {
        let start = lines.len() - terminal_height;
        lines = lines[start..].to_vec();
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_completed_line<'a>(prompt: &str, command: &str) -> Line<'a> {
    Line::from(vec![
        Span::styled(prompt.to_string(), Style::default().fg(PROMPT_COLOR)),
        Span::styled(command.to_string(), Style::default().fg(COMPLETED)),
    ])
}

fn render_current_line<'a>(
    prompt: &str,
    _display: &str,
    engine: &crate::core::engine::TypingEngine,
    _is_active: bool,
) -> Line<'a> {
    let mut spans = Vec::new();

    // Prompt
    spans.push(Span::styled(
        prompt.to_string(),
        Style::default().fg(PROMPT_COLOR),
    ));

    let is_flashing = engine.is_error_flashing();

    // Three-state coloring for the command characters
    for (i, ch) in engine.target.iter().enumerate() {
        let style = if i < engine.cursor {
            // Already typed correctly
            Style::default().fg(TYPED_CORRECT)
        } else if i == engine.cursor {
            // Current cursor position
            if is_flashing {
                Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
            } else {
                Style::default().fg(CURSOR).bg(CURSOR_BG)
            }
        } else {
            // Pending
            Style::default().fg(PENDING).bg(PENDING_BG)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    Line::from(spans)
}

fn render_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    // Summary hint
    if let Some(cmd) = app.current_typing_command() {
        if app.show_hint {
            spans.push(Span::styled(
                cmd.short_summary().to_string(),
                Style::default().fg(DIM),
            ));
        }
        spans.push(Span::styled("  ", Style::default()));
    }

    // Toggle hint indicator
    spans.push(Span::styled(
        "[H]",
        Style::default().fg(ACCENT),
    ));
    spans.push(Span::styled("  ", Style::default()));

    // WPM
    let wpm = app.typing_engine.current_wpm();
    spans.push(Span::styled(
        format!("WPM: {:.0}", wpm),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("  ", Style::default()));

    // Accuracy
    let acc = app.typing_engine.current_accuracy() * 100.0;
    let acc_color = if acc >= 95.0 {
        SUCCESS
    } else if acc >= 80.0 {
        WARNING
    } else {
        ERROR
    };
    spans.push(Span::styled(
        format!("\u{51c6}\u{786e}: {:.0}%", acc),
        Style::default().fg(acc_color),
    ));

    let bar = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(DIM)),
        );
    frame.render_widget(bar, area);
}
