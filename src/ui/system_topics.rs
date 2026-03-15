use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        " \u{7cfb}\u{7edf}\u{67b6}\u{6784}\u{4e13}\u{9898} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    if app.system_topics.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{7cfb}\u{7edf}\u{67b6}\u{6784}\u{6570}\u{636e}",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
    } else {
        let mut lines: Vec<Line> = Vec::new();
        for (i, topic) in app.system_topics.iter().enumerate() {
            let is_selected = i == app.system_topics_index;
            let prefix = if is_selected { " \u{25b6} " } else { "   " };
            let icon = topic.meta.icon.as_deref().unwrap_or("\u{1f4bb}");
            let style = if is_selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(MENU_NORMAL)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("{} ", icon), style),
                Span::styled(topic.meta.topic.clone(), style),
                Span::styled(
                    format!("  {} ", topic.meta.difficulty.stars()),
                    Style::default().fg(WARNING),
                ),
                Span::styled(
                    format!("  {}\u{4e2a}\u{7ae0}\u{8282}", topic.sections.len()),
                    Style::default().fg(DIM),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("      {}", topic.meta.description),
                Style::default().fg(DIM),
            )));
        }

        frame.render_widget(Paragraph::new(lines), chunks[1]);
    }

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
