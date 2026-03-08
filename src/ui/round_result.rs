use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, AppState};
use crate::data::models::Mode;
use crate::ui::widgets::{colors, format_duration_ms};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_top_bar(f, chunks[0], app);
    render_command_block(f, chunks[1], app);
    render_stats_cards(f, chunks[2], app);
    render_error_block(f, chunks[3], app);
    render_bottom_bar(f, chunks[4]);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let Some(result) = &app.round_result else {
        let bar = Paragraph::new(" 未找到结果数据 ").alignment(Alignment::Center);
        f.render_widget(bar, area);
        return;
    };

    let mode_label = match result.mode {
        Mode::Learn => "学习模式",
        Mode::Type => "对着打",
        Mode::Dictation => "默写模式",
    };

    let line = Line::from(vec![
        Span::styled(
            " 结果 ",
            Style::default()
                .fg(Color::Black)
                .bg(colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(mode_label, Style::default().fg(colors::HEADER)),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled("已完成当前练习", Style::default().fg(Color::White)),
    ]);

    let bar = Paragraph::new(vec![Line::from(""), line]);
    f.render_widget(bar, area);
}

fn render_command_block(f: &mut Frame, area: Rect, app: &App) {
    let Some(result) = &app.round_result else {
        return;
    };

    let lines = vec![
        Line::from(Span::styled(
            result.summary.as_str(),
            Style::default()
                .fg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("$ {}", result.command_text),
            Style::default().fg(Color::White),
        )),
    ];

    let block = Paragraph::new(lines).block(Block::default().title(" 题目 ").borders(Borders::ALL));
    f.render_widget(block, area);
}

fn render_stats_cards(f: &mut Frame, area: Rect, app: &App) {
    let Some(result) = &app.round_result else {
        return;
    };

    let chunks = Layout::horizontal([
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(1, 5),
    ])
    .split(area);

    render_stat_card(
        f,
        chunks[0],
        "WPM",
        format!("{:.1}", result.wpm),
        colors::ACCENT,
    );
    render_stat_card(
        f,
        chunks[1],
        "CPM",
        format!("{:.1}", result.cpm),
        colors::HEADER,
    );
    render_stat_card(
        f,
        chunks[2],
        "准确率",
        format!("{:.1}%", result.accuracy * 100.0),
        Color::Cyan,
    );
    render_stat_card(
        f,
        chunks[3],
        "耗时",
        format_duration_ms(result.elapsed_ms),
        Color::White,
    );
    render_stat_card(
        f,
        chunks[4],
        "错误数",
        result.error_count.to_string(),
        if result.error_count == 0 {
            colors::ACCENT
        } else {
            Color::Red
        },
    );
}

fn render_error_block(f: &mut Frame, area: Rect, app: &App) {
    let Some(result) = &app.round_result else {
        return;
    };

    let mut lines = vec![Line::from(Span::styled(
        "Top 5 错误字符",
        Style::default()
            .fg(colors::HEADER)
            .add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(""));

    if result.error_chars.is_empty() {
        lines.push(Line::from(Span::styled(
            "没有记录到错误字符，本轮输入全部一次命中。",
            Style::default().fg(colors::ACCENT),
        )));
    } else {
        for (ch, count) in &result.error_chars {
            let display = match ch {
                ' ' => "␠".to_string(),
                '\t' => "⇥".to_string(),
                '\n' => "↵".to_string(),
                other => other.to_string(),
            };
            lines.push(Line::from(vec![
                Span::styled(display, Style::default().fg(Color::White)),
                Span::styled("  ×  ", Style::default().fg(colors::PENDING)),
                Span::styled(count.to_string(), Style::default().fg(Color::Red)),
            ]));
        }
    }

    let block =
        Paragraph::new(lines).block(Block::default().title(" 错误分析 ").borders(Borders::ALL));
    f.render_widget(block, area);
}

fn render_bottom_bar(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 下一题  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "R",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 重试  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 菜单", Style::default().fg(colors::PENDING)),
    ]);
    let bar = Paragraph::new(vec![Line::from(""), help]).alignment(Alignment::Center);
    f.render_widget(bar, area);
}

fn render_stat_card(f: &mut Frame, area: Rect, title: &str, value: String, color: Color) {
    let content = Paragraph::new(vec![
        Line::from(Span::styled(title, Style::default().fg(colors::PENDING))),
        Line::from(""),
        Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(content, area);
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<AppState> {
    match key.code {
        KeyCode::Enter => app.advance_round_result(),
        KeyCode::Esc => Some(app.return_home()),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppState::Quitting)
        }
        KeyCode::Char('r') | KeyCode::Char('R') => app.retry_round_result(),
        _ => None,
    }
}
