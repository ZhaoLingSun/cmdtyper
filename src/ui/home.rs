use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::widgets::*;

const MENU_ITEMS: [(&str, &str); 5] = [
    (
        "\u{2328}\u{fe0f}  \u{5bf9}\u{7740}\u{6253}",
        "\u{7ec8}\u{7aef}\u{6a21}\u{62df}\u{6253}\u{5b57}\u{7ec3}\u{4e60}",
    ),
    (
        "\u{1f4d6} \u{5b66}\u{4e60}\u{4e2d}\u{5fc3}",
        "\u{547d}\u{4ee4}\u{00b7}\u{7b26}\u{53f7}\u{00b7}\u{7cfb}\u{7edf}\u{67b6}\u{6784}\u{4e13}\u{9898}",
    ),
    (
        "\u{1f4dd} \u{9ed8}\u{5199}\u{6a21}\u{5f0f}",
        "\u{770b}\u{4e2d}\u{6587}\u{5199}\u{547d}\u{4ee4}",
    ),
    (
        "\u{1f4ca} \u{7edf}\u{8ba1}\u{9762}\u{677f}",
        "\u{7ec3}\u{4e60}\u{6570}\u{636e}\u{5206}\u{6790}",
    ),
    (
        "\u{2699}\u{fe0f}  \u{8bbe}\u{7f6e}",
        "\u{81ea}\u{5b9a}\u{4e49}\u{914d}\u{7f6e}",
    ),
];

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(0),    // menu
            Constraint::Length(1), // hints
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " cmdtyper v0.2 ",
            Style::default()
                .fg(HEADER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " \u{2014} Linux \u{547d}\u{4ee4}\u{884c}\u{4ea4}\u{4e92}\u{5f0f}\u{6559}\u{5b66}\u{7cfb}\u{7edf}",
            Style::default().fg(DIM),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    // Menu items
    let menu_area = chunks[1];
    let menu_height = MENU_ITEMS.len() as u16 * 2 + 2;
    let v_pad = menu_area.height.saturating_sub(menu_height) / 2;

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Min(0),
            Constraint::Length(v_pad),
        ])
        .split(menu_area);

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, desc)) in MENU_ITEMS.iter().enumerate() {
        let is_selected = i == app.home_index;
        let prefix = if is_selected { " \u{25b6} " } else { "   " };

        let style = if is_selected {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD)
                .bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(MENU_NORMAL)
        };
        let desc_style = if is_selected {
            Style::default().fg(Color::White).bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(DIM)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix.to_string(), style),
            Span::styled(label.to_string(), style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled(desc.to_string(), desc_style),
        ]));
    }

    let menu = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(menu, inner[1]);

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{79fb}\u{52a8}"),
        ("Enter", "\u{9009}\u{62e9}"),
        ("q", "\u{9000}\u{51fa}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
