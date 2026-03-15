use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let categories = app.get_lesson_categories();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        " \u{547d}\u{4ee4}\u{4e13}\u{9898} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    // Category list
    if categories.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{8bfe}\u{7a0b}\u{6570}\u{636e}\u{ff0c}\u{8bf7}\u{6dfb}\u{52a0} data/lessons/*.toml",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
    } else {
        let mut lines: Vec<Line> = Vec::new();

        for (i, cat) in categories.iter().enumerate() {
            let is_selected = i == app.command_topics_index;
            let prefix = if is_selected { " \u{25b6} " } else { "   " };

            let lesson_count = app
                .lessons
                .iter()
                .filter(|l| l.meta.category == *cat)
                .count();
            let practiced = app
                .user_stats
                .command_progress
                .iter()
                .filter(|p| {
                    app.lessons
                        .iter()
                        .any(|l| l.meta.category == *cat && l.meta.command == p.command_id)
                })
                .count();

            let style = if is_selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(MENU_NORMAL)
            };

            let difficulty = app
                .lessons
                .iter()
                .find(|l| l.meta.category == *cat)
                .map(|l| l.meta.difficulty)
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("{} ", cat.icon()), style),
                Span::styled(cat.label().to_string(), style),
                Span::styled(
                    format!("  {} ", difficulty.stars()),
                    Style::default().fg(WARNING),
                ),
                Span::styled(
                    format!("  {}/{}", practiced, lesson_count),
                    Style::default().fg(DIM),
                ),
            ]));
        }

        let list = Paragraph::new(lines);
        frame.render_widget(list, chunks[1]);
    }

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{79fb}\u{52a8}"),
        ("Enter", "\u{8fdb}\u{5165}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
