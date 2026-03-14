use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::widgets::*;

const ITEMS: [(&str, &str); 8] = [
    ("🌱 基础入门训练", "Beginner 难度快速开始"),
    ("📖 常见命令训练", "Basic 难度快速开始"),
    ("🚀 进阶组合训练", "Advanced 难度快速开始"),
    ("💪 实战场景训练", "Practical 难度快速开始"),
    ("💻 命令专题", "按类别学习命令详解"),
    ("⌨️  符号专题", "管道、重定向、通配符等"),
    ("🏗️  系统架构", "目录结构、权限、进程等"),
    ("🔄 专题复习", "知识梳理与集中练习"),
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

    let title = Paragraph::new(Line::from(Span::styled(
        " 📖 学习中心 ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    let menu_area = chunks[1];
    let mut lines: Vec<Line> = Vec::new();

    for (i, (label, desc)) in ITEMS.iter().enumerate() {
        let is_selected = i == app.learn_hub_index;
        let prefix = if is_selected { " ▶ " } else { "   " };
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

        if i == 3 {
            lines.push(Line::from(Span::styled(
                "      ─────────────────",
                Style::default().fg(DIM),
            )));
        }
    }

    let menu_height = lines.len() as u16 + 2;
    let v_pad = menu_area.height.saturating_sub(menu_height) / 2;
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Min(0),
            Constraint::Length(v_pad),
        ])
        .split(menu_area);

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner[1]);

    let hints = hint_line(&[("↑↓", "移动"), ("Enter", "选择"), ("Esc", "返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
