use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::core::scorer;
use crate::data::models::Category;
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

    // Tab bar in title
    let tabs = [
        "\u{901f}\u{5ea6}\u{603b}\u{89c8}",
        "\u{5b57}\u{7b26}\u{5206}\u{6790}",
        "\u{7c7b}\u{522b}\u{638c}\u{63e1}",
        "\u{65e5}\u{5386}",
    ];
    let mut tab_spans: Vec<Span> = Vec::new();
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(" | ", Style::default().fg(DIM)));
        }
        if i == app.stats_tab {
            tab_spans.push(Span::styled(
                tab.to_string(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ));
        } else {
            tab_spans.push(Span::styled(tab.to_string(), Style::default().fg(DIM)));
        }
    }

    let title = Paragraph::new(Line::from(tab_spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)),
        );
    frame.render_widget(title, chunks[0]);

    // Content per tab
    let content_area = chunks[1];
    match app.stats_tab {
        0 => render_speed_overview(frame, app, content_area),
        1 => render_char_analysis(frame, app, content_area),
        2 => render_category_mastery(frame, app, content_area),
        3 => render_calendar(frame, app, content_area),
        _ => {}
    }

    let hints = hint_line(&[
        ("Tab/\u{2190}\u{2192}", "\u{5207}\u{6362}\u{9762}\u{677f}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn render_speed_overview(frame: &mut Frame, app: &App, area: Rect) {
    let stats = &app.user_stats;
    let mut lines = vec![
        Line::from(" 跟打速度总览（含未完成练习）").style(Style::default().fg(HEADER).bold()),
        Line::from(""),
        Line::from(format!("  跟打练习片段：{}", stats.total_wpm_sessions)),
        Line::from(format!("  总击键次数：  {}", stats.total_keystrokes)),
        Line::from(format!(
            "  有效练习时长：{}",
            format_time(stats.total_duration_ms as f64 / 1000.0)
        )),
        Line::from(""),
        Line::from(format!(
            "  WPM {:.1}   CPM {:.1}   准确率 {:.1}%",
            stats.overall_avg_wpm,
            stats.overall_avg_wpm * 5.0,
            stats.overall_avg_accuracy * 100.0
        ))
        .style(Style::default().fg(ACCENT).bold()),
        Line::from(format!("  最高 WPM：{:.1}", stats.best_wpm))
            .style(Style::default().fg(SUCCESS)),
        Line::from(format!(
            "  当前连续 {} 天 · 最长连续 {} 天",
            stats.current_streak, stats.longest_streak
        )),
        Line::from(""),
        Line::from("  同一目标位置的错误只计一次；速度按字符数与有效时长汇总。"),
        Line::from("  阅读、填空、默写与诊断选择不计入跟打指标。").style(Style::default().fg(DIM)),
    ];
    if stats.legacy_sessions_count > 0
        || (stats.stats_version < scorer::STATS_VERSION && stats.total_sessions > 0)
    {
        lines.push(
            Line::from("  包含旧口径记录；旧记录的空闲与字符间隔无法恢复。")
                .style(Style::default().fg(WARNING)),
        );
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_char_analysis(frame: &mut Frame, app: &App, area: Rect) {
    let mut chars = app.user_stats.char_stats.clone();
    chars.sort_by_key(|stat| stat.char_key);
    crate::ui::calendar::render_characters(
        frame,
        &chars,
        app.calendar_state.char_scroll,
        area,
        "逐字符统计 · j/k 滚动",
    );
}

fn render_category_mastery(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " \u{7c7b}\u{522b}\u{638c}\u{63e1}\u{5ea6}",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for cat in Category::ALL {
        let mastery = scorer::category_mastery(&app.user_stats, &app.commands, cat);
        let bar_width = 20;
        let filled = (mastery * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let bar = format!(
            "\u{2588}{}{}",
            "\u{2588}".repeat(filled.saturating_sub(1).max(0)),
            "\u{2591}".repeat(empty)
        );

        let mastery_color = if mastery >= 0.8 {
            SUCCESS
        } else if mastery >= 0.5 {
            WARNING
        } else {
            DIM
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} {:<10}", cat.icon(), cat.label()),
                Style::default().fg(Color::White),
            ),
            Span::styled(bar, Style::default().fg(mastery_color)),
            Span::styled(
                format!(" {:.0}%", mastery * 100.0),
                Style::default().fg(mastery_color),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_calendar(frame: &mut Frame, app: &App, area: Rect) {
    crate::ui::calendar::render(
        frame,
        &app.user_stats,
        &app.history,
        &app.calendar_state,
        area,
    );
}
