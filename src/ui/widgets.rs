use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

// ─────────────────────────────────────────────────────────────
// Color constants per PLAN.md §6.4
// ─────────────────────────────────────────────────────────────

pub const TYPED_CORRECT: Color = Color::White;
pub const PENDING: Color = Color::DarkGray;
pub const PENDING_BG: Color = Color::Rgb(40, 40, 40);
pub const CURSOR: Color = Color::Black;
pub const CURSOR_BG: Color = Color::White;
pub const ERROR_FLASH: Color = Color::White;
pub const ERROR_FLASH_BG: Color = Color::Red;
pub const COMPLETED: Color = Color::Green;
pub const PROMPT_COLOR: Color = Color::Cyan;
pub const SIMULATED_BORDER: Color = Color::DarkGray;
pub const SIMULATED_PROMPT: Color = Color::Green;
pub const HEADER: Color = Color::Yellow;
pub const TOKEN_DESC: Color = Color::Cyan;

// Additional UI colors
pub const MENU_SELECTED_BG: Color = Color::Rgb(50, 50, 80);
pub const MENU_NORMAL: Color = Color::White;
pub const ACCENT: Color = Color::Cyan;
pub const DIM: Color = Color::DarkGray;
pub const SUCCESS: Color = Color::Green;
pub const ERROR: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;

// ─────────────────────────────────────────────────────────────
// Utility functions
// ─────────────────────────────────────────────────────────────

/// Format seconds into MM:SS string.
pub fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let mins = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", mins, s)
}

/// Render a simulated terminal output box.
/// Shows a command with prompt and its output in a bordered box.
pub fn render_simulated_output<'a>(
    command: &str,
    output: Option<&str>,
) -> Paragraph<'a> {
    let mut lines_vec = Vec::new();

    // Command line with simulated prompt
    lines_vec.push(Line::from(vec![
        Span::styled("$ ", Style::default().fg(SIMULATED_PROMPT)),
        Span::styled(command.to_string(), Style::default().fg(Color::White)),
    ]));

    // Output lines
    if let Some(output_text) = output {
        for line in output_text.lines() {
            lines_vec.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            )));
        }
    }

    Paragraph::new(lines_vec)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SIMULATED_BORDER)),
        )
        .wrap(Wrap { trim: false })
}

/// Centered title block.
pub fn title_block(title: &str) -> Block<'_> {
    Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
}

/// Build a hint line at the bottom of a view.
pub fn hint_line(hints: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(DIM),
        ));
    }
    Line::from(spans)
}
