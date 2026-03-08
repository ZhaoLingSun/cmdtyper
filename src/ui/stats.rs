use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::{Datelike, Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table, Wrap},
    Frame,
};

use crate::app::{App, AppState, StatsTab};
use crate::core::scorer;
use crate::data::models::{Category, UserStats};
use crate::ui::widgets::{colors, format_duration_ms};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_top_bar(f, chunks[0], app);
    render_active_tab(f, chunks[1], app);
    render_bottom_bar(f, chunks[2]);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![
        Span::styled(
            " 统计面板 ",
            Style::default()
                .fg(colors::CURSOR)
                .bg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];

    for (index, tab) in StatsTab::ALL.iter().enumerate() {
        let style = if *tab == app.stats_tab {
            Style::default()
                .fg(Color::Black)
                .bg(colors::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(format!(" {} ", tab.label()), style));
        if index + 1 < StatsTab::ALL.len() {
            spans.push(Span::styled(" ", Style::default().fg(colors::PENDING)));
        }
    }

    let bar = Paragraph::new(vec![Line::from(""), Line::from(spans)]).alignment(Alignment::Left);
    f.render_widget(bar, area);
}

fn render_active_tab(f: &mut Frame, area: Rect, app: &App) {
    match app.stats_tab {
        StatsTab::SpeedOverview => render_speed_overview(f, area, &app.user_stats),
        StatsTab::CharacterAnalysis => render_character_analysis(f, area, &app.user_stats),
        StatsTab::CategoryMastery => render_category_mastery(f, area, app),
        StatsTab::PracticeCalendar => render_practice_calendar(f, area, &app.user_stats),
    }
}

fn render_speed_overview(f: &mut Frame, area: Rect, stats: &UserStats) {
    let chunks = Layout::vertical([Constraint::Length(7), Constraint::Fill(1)]).split(area);
    let cards = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .split(chunks[0]);

    render_stat_card(
        f,
        cards[0],
        "平均 WPM",
        format!("{:.1}", stats.overall_avg_wpm),
        colors::ACCENT,
    );
    render_stat_card(
        f,
        cards[1],
        "最佳 WPM",
        format!("{:.1}", stats.best_wpm),
        colors::HEADER,
    );
    render_stat_card(
        f,
        cards[2],
        "准确率",
        format!("{:.1}%", stats.overall_avg_accuracy * 100.0),
        Color::Cyan,
    );
    render_stat_card(
        f,
        cards[3],
        "总场次",
        stats.total_sessions.to_string(),
        Color::White,
    );

    let trend = stats
        .daily_stats
        .iter()
        .rev()
        .take(50)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|day| day.avg_wpm.max(0.0).round() as u64)
        .collect::<Vec<_>>();

    if trend.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "暂无每日练习数据，完成几轮练习后这里会显示最近 50 天的 WPM 趋势。",
                Style::default().fg(colors::PENDING),
            )),
        ])
        .wrap(Wrap { trim: false })
        .block(Block::default().title(" WPM 趋势 ").borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let trend_widget = Sparkline::default()
        .block(
            Block::default()
                .title(" 最近 50 天 WPM 趋势 ")
                .borders(Borders::ALL),
        )
        .data(&trend)
        .style(Style::default().fg(colors::ACCENT));
    f.render_widget(trend_widget, chunks[1]);
}

fn render_character_analysis(f: &mut Frame, area: Rect, stats: &UserStats) {
    let mut char_stats = stats
        .char_stats
        .iter()
        .filter(|stat| stat.total_samples > 0)
        .collect::<Vec<_>>();

    char_stats.sort_by(|left, right| {
        float_cmp(left.accuracy, right.accuracy)
            .then_with(|| right.total_samples.cmp(&left.total_samples))
            .then_with(|| left.char_key.cmp(&right.char_key))
    });

    if char_stats.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "暂无字符统计数据，先完成一些跟打或学习练习。",
                Style::default().fg(colors::PENDING),
            )),
        ])
        .block(Block::default().title(" 字符分析 ").borders(Borders::ALL));
        f.render_widget(empty, area);
        return;
    }

    let rows = char_stats.into_iter().map(|stat| {
        let accuracy = stat.accuracy * 100.0;
        let accuracy_style = accuracy_style(stat.accuracy);
        Row::new(vec![
            Cell::from(display_char(stat.char_key)),
            Cell::from(format!("{:.0}", stat.avg_cpm)),
            Cell::from(Span::styled(format!("{:.1}%", accuracy), accuracy_style)),
            Cell::from(stat.total_samples.to_string()),
            Cell::from(Span::styled(status_emoji(stat.accuracy), accuracy_style)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["字符", "CPM", "准确率", "样本数", "状态"]).style(
            Style::default()
                .fg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::default().title(" 字符分析 ").borders(Borders::ALL))
    .column_spacing(1);

    f.render_widget(table, area);
}

fn render_category_mastery(f: &mut Frame, area: Rect, app: &App) {
    let main = Block::default().title(" 分类掌握 ").borders(Borders::ALL);
    let inner = main.inner(area);
    f.render_widget(main, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(inner);
    let row_constraints = vec![Constraint::Length(3); 5];

    let left_rows = Layout::vertical(row_constraints.clone()).split(columns[0]);
    let right_rows = Layout::vertical(row_constraints).split(columns[1]);

    for (index, category) in Category::ALL.iter().enumerate() {
        let mastery = scorer::category_mastery(&app.user_stats, &app.all_commands, *category);
        let cell = if index < 5 {
            left_rows[index]
        } else {
            right_rows[index - 5]
        };
        render_category_gauge(f, cell, *category, mastery);
    }
}

fn render_practice_calendar(f: &mut Frame, area: Rect, stats: &UserStats) {
    let chunks = Layout::vertical([Constraint::Length(7), Constraint::Fill(1)]).split(area);
    let cards = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(chunks[0]);

    render_stat_card(
        f,
        cards[0],
        "当前连击",
        stats.current_streak.to_string(),
        colors::ACCENT,
    );
    render_stat_card(
        f,
        cards[1],
        "最长连击",
        stats.longest_streak.to_string(),
        colors::HEADER,
    );
    render_stat_card(
        f,
        cards[2],
        "总练习时长",
        format_duration_ms(stats.total_duration_ms),
        Color::White,
    );

    let today = Local::now().date_naive();
    let year = today.year();
    let month = today.month();
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("valid first day");
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid next month")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid next month")
    };
    let days_in_month = next_month.pred_opt().expect("month has previous day").day();

    let practiced_days = stats
        .daily_stats
        .iter()
        .filter(|day| day.sessions_count > 0)
        .filter_map(|day| NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok())
        .filter(|date| date.year() == year && date.month() == month)
        .map(|date| date.day())
        .collect::<HashSet<_>>();

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  {} 年 {:02} 月", year, month),
            Style::default()
                .fg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" 一 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 二 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 三 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 四 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 五 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 六 ", Style::default().fg(colors::PENDING)),
            Span::styled(" 日 ", Style::default().fg(colors::PENDING)),
        ]),
    ];

    let mut day = 1_u32;
    let offset = first_day.weekday().num_days_from_monday() as usize;
    for week_index in 0..6 {
        let mut spans = Vec::with_capacity(7);
        for weekday_index in 0..7 {
            let before_start = week_index == 0 && weekday_index < offset;
            if before_start || day > days_in_month {
                spans.push(Span::raw("    "));
                continue;
            }

            let practiced = practiced_days.contains(&day);
            let mut style = if practiced {
                Style::default().fg(Color::Black).bg(colors::ACCENT)
            } else {
                Style::default().fg(Color::White).bg(colors::PENDING_BG)
            };
            if day == today.day() {
                style = style.add_modifier(Modifier::BOLD);
            }

            spans.push(Span::styled(format!("{:^4}", day), style));
            day += 1;
        }
        lines.push(Line::from(spans));
        if day > days_in_month {
            break;
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "绿色方块表示该日完成过练习。",
        Style::default().fg(colors::PENDING),
    )));

    let calendar = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().title(" 练习日历 ").borders(Borders::ALL));
    f.render_widget(calendar, chunks[1]);
}

fn render_bottom_bar(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 下一页  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "Shift+Tab",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 上一页  ", Style::default().fg(colors::PENDING)),
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

fn render_category_gauge(f: &mut Frame, area: Rect, category: Category, mastery: f64) {
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" {} ", category.label()))
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(accuracy_style(mastery).fg.unwrap_or(colors::ACCENT)))
        .use_unicode(true)
        .label(format!("{:.0}%", mastery * 100.0))
        .ratio(mastery.clamp(0.0, 1.0));

    f.render_widget(gauge, area);
}

fn accuracy_style(accuracy: f64) -> Style {
    if accuracy >= 0.9 {
        Style::default().fg(colors::ACCENT)
    } else if accuracy >= 0.7 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Red)
    }
}

fn status_emoji(accuracy: f64) -> &'static str {
    if accuracy >= 0.9 {
        "🟢"
    } else if accuracy >= 0.7 {
        "🟡"
    } else {
        "🔴"
    }
}

fn display_char(ch: char) -> String {
    match ch {
        ' ' => "␠".to_string(),
        '\t' => "⇥".to_string(),
        '\n' => "↵".to_string(),
        other => other.to_string(),
    }
}

fn float_cmp(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<AppState> {
    match key.code {
        KeyCode::Esc => Some(app.return_home()),
        KeyCode::BackTab => {
            app.stats_tab = app.stats_tab.prev();
            None
        }
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.stats_tab = app.stats_tab.prev();
            None
        }
        KeyCode::Tab => {
            app.stats_tab = app.stats_tab.next();
            None
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppState::Quitting)
        }
        _ => None,
    }
}
