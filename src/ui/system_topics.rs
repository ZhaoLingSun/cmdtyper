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
        " 系统架构专题 ",
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
            "暂无系统架构数据",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
    } else {
        render_topic_menu(frame, app, chunks[1]);
    }

    let hints = hint_line(&[("↑↓", "移动"), ("Enter", "进入"), ("Esc", "返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn render_topic_menu(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app.system_topics_index.min(app.system_topics.len() - 1);
    let show_description = area.height >= 4;
    let description_height = u16::from(show_description);
    let show_continuations = area.height.saturating_sub(description_height) >= 3;
    let continuation_height = u16::from(show_continuations);
    let menu_height = area
        .height
        .saturating_sub(description_height + continuation_height * 2);

    let above_area = Rect::new(area.x, area.y, area.width, continuation_height);
    let menu_area = Rect::new(
        area.x,
        area.y + continuation_height,
        area.width,
        menu_height,
    );
    let below_area = Rect::new(
        area.x,
        menu_area.y + menu_area.height,
        area.width,
        continuation_height,
    );
    let description_area = Rect::new(
        area.x,
        below_area.y + below_area.height,
        area.width,
        description_height,
    );

    let window = visible_menu_window(selected, app.system_topics.len(), 1, menu_area.height);
    let mut lines = Vec::with_capacity(window.len());
    for topic_index in window.clone() {
        let topic = &app.system_topics[topic_index];
        let is_selected = topic_index == selected;
        let prefix = if is_selected { " ▶ " } else { "   " };
        let icon = topic.meta.icon.as_deref().unwrap_or("💻");
        let style = if is_selected {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD)
                .bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(MENU_NORMAL)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{} {}", icon, topic.meta.topic), style),
            Span::styled(
                format!("  {}", topic.meta.difficulty.stars()),
                Style::default().fg(WARNING),
            ),
            Span::styled(
                format!("  {}个章节", topic.sections.len()),
                Style::default().fg(DIM),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), menu_area);

    if show_continuations {
        if window.start > 0 {
            frame.render_widget(
                Paragraph::new(Span::styled("↑ 上方还有专题", Style::default().fg(DIM)))
                    .alignment(Alignment::Center),
                above_area,
            );
        }
        if window.end < app.system_topics.len() {
            frame.render_widget(
                Paragraph::new(Span::styled("↓ 下方还有专题", Style::default().fg(DIM)))
                    .alignment(Alignment::Center),
                below_area,
            );
        }
    }

    if show_description {
        let topic = &app.system_topics[selected];
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("简介: ", Style::default().fg(ACCENT)),
                Span::styled(topic.meta.description.clone(), Style::default().fg(DIM)),
            ])),
            description_area,
        );
    }
}
