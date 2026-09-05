use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, _app: &App) {
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
        " cmdtyper 帮助 ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(title, chunks[0]);

    let body: Vec<Line> = vec![
        Line::from(Span::styled(
            "使用方式",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw("对着打 — 直接练习命令打字，速度与准确性")),
        Line::from(Span::raw("学习中心 — 系统学习命令语法与概念")),
        Line::from(Span::raw("默写模式 — 根据中文描述默写命令")),
        Line::from(Span::raw("专题训练 — 按专题与级别巩固命令")),
        Line::from(""),
        Line::from(Span::styled(
            "快捷键总览",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(
            "对着打: F2=切换模式  F3=提示  Tab=跳过  Ctrl+R=重试  Esc=返回",
        )),
        Line::from(Span::raw("学习中心: 上下=移动  Enter=选择  Esc=返回上一级")),
        Line::from(Span::raw("通用: Esc=返回")),
        Line::from(""),
        Line::from(Span::styled(
            "版本 0.4 — 2026",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let content = Paragraph::new(body).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints: Vec<Line> = vec![Line::from(Span::styled(
        "Esc = 返回主页",
        Style::default().fg(Color::DarkGray),
    ))];
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
