use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::widgets::*;

const ITEMS: [(&str, &str); 4] = [
    (
        "\u{1f4bb} \u{547d}\u{4ee4}\u{4e13}\u{9898}",
        "\u{6309}\u{7c7b}\u{522b}\u{5b66}\u{4e60}\u{547d}\u{4ee4}\u{8be6}\u{89e3}",
    ),
    (
        "\u{2328}\u{fe0f}  \u{7b26}\u{53f7}\u{4e13}\u{9898}",
        "\u{7ba1}\u{9053}\u{3001}\u{91cd}\u{5b9a}\u{5411}\u{3001}\u{901a}\u{914d}\u{7b26}\u{7b49}",
    ),
    (
        "\u{1f3d7}\u{fe0f}  \u{7cfb}\u{7edf}\u{67b6}\u{6784}",
        "\u{76ee}\u{5f55}\u{7ed3}\u{6784}\u{3001}\u{6743}\u{9650}\u{3001}\u{8fdb}\u{7a0b}\u{7b49}",
    ),
    (
        "\u{1f504} \u{4e13}\u{9898}\u{590d}\u{4e60}",
        "\u{77e5}\u{8bc6}\u{68b3}\u{7406}\u{4e0e}\u{96c6}\u{4e2d}\u{7ec3}\u{4e60}",
    ),
];

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

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        " \u{1f4d6} \u{5b66}\u{4e60}\u{4e2d}\u{5fc3} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    // Menu
    let menu_area = chunks[1];
    let menu_height = ITEMS.len() as u16 * 2 + 2;
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
    for (i, (label, desc)) in ITEMS.iter().enumerate() {
        let is_selected = i == app.learn_hub_index;
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

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner[1]);

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{79fb}\u{52a8}"),
        ("Enter", "\u{9009}\u{62e9}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
