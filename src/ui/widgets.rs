use std::ops::Range;

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

/// Return the contiguous item range that fits while keeping the selection visible.
pub fn visible_menu_window(
    selected_index: usize,
    item_count: usize,
    item_height: u16,
    available_height: u16,
) -> Range<usize> {
    if item_count == 0 || item_height == 0 {
        return 0..0;
    }

    let capacity = usize::from(available_height / item_height).min(item_count);
    if capacity == 0 {
        return 0..0;
    }

    let selected_index = selected_index.min(item_count - 1);
    let mut start = selected_index.saturating_sub(capacity / 2);
    start = start.min(item_count - capacity);
    start..start + capacity
}

/// Count wrapped rows by rendering a sentinel line with Ratatui's Paragraph.
///
/// Ratatui 0.29 keeps `Paragraph::line_count` behind an unstable feature, so this
/// uses the same renderer directly and measures where the sentinel lands.
pub fn rendered_wrapped_line_count(lines: &[Line<'_>], width: u16, trim: bool) -> usize {
    if width == 0 {
        return 0;
    }

    const MARKER_FG: Color = Color::Rgb(1, 2, 3);
    const MARKER_BG: Color = Color::Rgb(4, 5, 6);

    let mut measured_lines = lines.to_vec();
    measured_lines.push(Line::from(Span::styled(
        "X",
        Style::default().fg(MARKER_FG).bg(MARKER_BG),
    )));

    let height_upper_bound = lines
        .iter()
        .map(|line| line.width().max(1))
        .sum::<usize>()
        .saturating_add(lines.len())
        .saturating_add(1)
        .min(usize::from(u16::MAX)) as u16;
    let area = Rect::new(0, 0, width, height_upper_bound.max(1));
    let mut buffer = Buffer::empty(area);
    Paragraph::new(measured_lines)
        .wrap(Wrap { trim })
        .render(area, &mut buffer);

    buffer
        .content()
        .iter()
        .position(|cell| cell.fg == MARKER_FG && cell.bg == MARKER_BG)
        .map(|index| index / usize::from(width))
        .unwrap_or(usize::from(area.height))
}

/// Format seconds into MM:SS string.
pub fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let mins = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", mins, s)
}

/// Render a simulated terminal output box.
/// Shows a command with prompt and its output in a bordered box.
pub fn render_simulated_output<'a>(command: &str, output: Option<&str>) -> Paragraph<'a> {
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
        spans.push(Span::styled(format!(" {}", desc), Style::default().fg(DIM)));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::visible_menu_window;

    #[test]
    fn visible_menu_window_handles_empty_and_zero_height_inputs() {
        assert_eq!(visible_menu_window(0, 0, 1, 5), 0..0);
        assert_eq!(visible_menu_window(0, 5, 0, 5), 0..0);
        assert_eq!(visible_menu_window(0, 5, 1, 0), 0..0);
    }

    #[test]
    fn visible_menu_window_keeps_selection_visible_at_each_edge() {
        assert_eq!(visible_menu_window(0, 10, 1, 4), 0..4);
        assert_eq!(visible_menu_window(5, 10, 1, 4), 3..7);
        assert_eq!(visible_menu_window(9, 10, 1, 4), 6..10);
    }

    #[test]
    fn visible_menu_window_accounts_for_item_height_and_clamps_selection() {
        assert_eq!(visible_menu_window(4, 5, 2, 6), 2..5);
        assert_eq!(visible_menu_window(99, 5, 1, 2), 3..5);
    }
}

/// Render actual typing targets, preserving sequential Enter boundaries and cursor offsets.
pub fn typing_lines(
    prompt: &str,
    engine: &crate::core::engine::TypingEngine,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut spans = vec![Span::styled(
        prompt.to_owned(),
        Style::default().fg(PROMPT_COLOR),
    )];
    let current_line = engine
        .target
        .iter()
        .take(engine.cursor)
        .filter(|c| **c == '\n')
        .count();
    for (index, ch) in engine.target.iter().enumerate() {
        let style = if index < engine.cursor {
            Style::default().fg(TYPED_CORRECT)
        } else if index == engine.cursor && engine.is_error_flashing() {
            Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
        } else if index == engine.cursor {
            Style::default().fg(CURSOR).bg(CURSOR_BG)
        } else {
            Style::default().fg(PENDING).bg(PENDING_BG)
        };
        if *ch == '\n' {
            spans.push(Span::styled(" [Enter]", style));
            lines.push(Line::from(spans));
            spans = vec![Span::styled(
                prompt.to_owned(),
                Style::default().fg(PROMPT_COLOR),
            )];
        } else {
            spans.push(Span::styled(ch.to_string(), style));
        }
    }
    lines.push(Line::from(spans));
    // Keep the active command visible even for long preparation workflows.
    let start = current_line.saturating_sub(3);
    lines
        .into_iter()
        .skip(start)
        .take(current_line.saturating_sub(start) + 1)
        .collect()
}
