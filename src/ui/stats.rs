use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " \u{901f}\u{5ea6}\u{603b}\u{89c8}",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            "  \u{603b}\u{7ec3}\u{4e60}\u{6b21}\u{6570}: ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{}", stats.total_sessions),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  \u{603b}\u{51fb}\u{952e}\u{6b21}\u{6570}: ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{}", stats.total_keystrokes),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  \u{603b}\u{7ec3}\u{4e60}\u{65f6}\u{957f}: ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format_time(stats.total_duration_ms as f64 / 1000.0),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  \u{5e73}\u{5747} WPM:      ", Style::default().fg(DIM)),
        Span::styled(
            format!("{:.1}", stats.overall_avg_wpm),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  \u{6700}\u{9ad8} WPM:      ", Style::default().fg(DIM)),
        Span::styled(
            format!("{:.1}", stats.best_wpm),
            Style::default().fg(SUCCESS),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  \u{5e73}\u{5747}\u{51c6}\u{786e}\u{7387}:  ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{:.1}%", stats.overall_avg_accuracy * 100.0),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  \u{5f53}\u{524d}\u{8fde}\u{7eed}:    ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{} \u{5929}", stats.current_streak),
            Style::default().fg(SUCCESS),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  \u{6700}\u{957f}\u{8fde}\u{7eed}:    ",
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{} \u{5929}", stats.longest_streak),
            Style::default().fg(Color::White),
        ),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_char_analysis(frame: &mut Frame, app: &App, area: Rect) {
    let stats = &app.user_stats;
    let weak = scorer::weak_chars(stats, 10);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " \u{5b57}\u{7b26}\u{5206}\u{6790}",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if weak.is_empty() {
        lines.push(Line::from(Span::styled(
            "  \u{6682}\u{65e0}\u{5b57}\u{7b26}\u{7edf}\u{8ba1}\u{6570}\u{636e}\u{ff0c}\u{591a}\u{7ec3}\u{4e60}\u{540e}\u{67e5}\u{770b}",
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<6}", "\u{5b57}\u{7b26}"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}", "\u{51c6}\u{786e}\u{7387}"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}", "CPM"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{6837}\u{672c}\u{6570}",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]));

        for cs in &weak {
            let display_char = if cs.char_key == ' ' {
                "\u{2423}".to_string()
            } else {
                cs.char_key.to_string()
            };
            let acc_color = if cs.accuracy >= 0.95 {
                SUCCESS
            } else if cs.accuracy >= 0.80 {
                WARNING
            } else {
                ERROR
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<6}", format!("'{}'", display_char)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<10}", format!("{:.1}%", cs.accuracy * 100.0)),
                    Style::default().fg(acc_color),
                ),
                Span::styled(
                    format!("{:<10}", format!("{:.0}", cs.avg_cpm)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(format!("{}", cs.total_samples), Style::default().fg(DIM)),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
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
    let stats = &app.user_stats;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " \u{7ec3}\u{4e60}\u{65e5}\u{5386}",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if stats.daily_stats.is_empty() {
        lines.push(Line::from(Span::styled(
            "  \u{6682}\u{65e0}\u{7ec3}\u{4e60}\u{8bb0}\u{5f55}",
            Style::default().fg(DIM),
        )));
    } else {
        // Show last 14 days
        let recent: Vec<_> = stats
            .daily_stats
            .iter()
            .rev()
            .take(14)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12}", "\u{65e5}\u{671f}"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<8}", "\u{6b21}\u{6570}"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}", "WPM"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{51c6}\u{786e}\u{7387}",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]));

        for day in &recent {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<12}", day.date),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<8}", day.sessions_count),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<10}", format!("{:.1}", day.avg_wpm)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:.1}%", day.avg_accuracy * 100.0),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
