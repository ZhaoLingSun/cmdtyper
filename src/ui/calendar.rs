use chrono::{Datelike, Duration, Local, Months, NaiveDate};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::core::scorer;
use crate::data::models::{CharStat, SessionRecord, UserStats};
use crate::ui::widgets::{ACCENT, DIM, HEADER, SUCCESS, WARNING, format_time};

#[derive(Debug, Clone)]
pub struct CalendarState {
    pub selected_date: NaiveDate,
    pub char_scroll: usize,
    pub detail_scroll: usize,
}

impl Default for CalendarState {
    fn default() -> Self {
        Self {
            selected_date: Local::now().date_naive(),
            char_scroll: 0,
            detail_scroll: 0,
        }
    }
}

impl CalendarState {
    pub fn handle_key(&mut self, key: KeyCode) {
        let date = match key {
            KeyCode::Left => self.selected_date.checked_sub_signed(Duration::days(1)),
            KeyCode::Right => self.selected_date.checked_add_signed(Duration::days(1)),
            KeyCode::Up => self.selected_date.checked_sub_signed(Duration::days(7)),
            KeyCode::Down => self.selected_date.checked_add_signed(Duration::days(7)),
            KeyCode::PageUp => self.selected_date.checked_sub_months(Months::new(1)),
            KeyCode::PageDown => self.selected_date.checked_add_months(Months::new(1)),
            KeyCode::Home => Some(Local::now().date_naive()),
            KeyCode::Char('[') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
                None
            }
            KeyCode::Char(']') => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
                None
            }
            KeyCode::Char('j') => {
                self.char_scroll = self.char_scroll.saturating_add(1);
                None
            }
            KeyCode::Char('k') => {
                self.char_scroll = self.char_scroll.saturating_sub(1);
                None
            }
            _ => None,
        };
        if let Some(date) = date {
            self.selected_date = date;
            self.char_scroll = 0;
            self.detail_scroll = 0;
        }
    }
}

pub fn render(
    frame: &mut Frame,
    stats: &UserStats,
    history: &[SessionRecord],
    state: &CalendarState,
    area: Rect,
) {
    let layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new(format!(
            " 练习日历 · {}",
            state.selected_date.format("%Y 年 %m 月")
        ))
        .style(Style::default().fg(HEADER).bold()),
        layout[0],
    );
    let body = layout[1];
    let selected = state.selected_date.format("%Y-%m-%d").to_string();
    let chars = scorer::character_stats_as_of(history, &selected);
    if area.height < 20 {
        render_day(frame, stats, &selected, state.detail_scroll, body);
    } else if body.width >= 76 {
        let columns = Layout::horizontal([Constraint::Length(37), Constraint::Min(0)]).split(body);
        let left = Layout::vertical([Constraint::Length(10), Constraint::Min(0)]).split(columns[0]);
        render_month(frame, stats, state, left[0]);
        render_day(frame, stats, &selected, state.detail_scroll, left[1]);
        render_characters(
            frame,
            &chars,
            state.char_scroll,
            columns[1],
            &format!("截至 {selected} 的逐字符统计"),
        );
    } else {
        let rows = Layout::vertical([
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(body);
        render_month(frame, stats, state, rows[0]);
        render_day(frame, stats, &selected, state.detail_scroll, rows[1]);
        render_characters(
            frame,
            &chars,
            state.char_scroll,
            rows[2],
            "截至所选日期的逐字符统计",
        );
    }
    let hint = if area.width < 76 {
        "←→ 选日  PgUp/Dn 月  [/] 明细  Esc"
    } else {
        "←→ 选日  ↑↓ 选周  PgUp/PgDn 换月  Home 今天  j/k 字符  [/] 明细  Esc 返回"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(DIM)),
        layout[2],
    );
}

fn render_month(frame: &mut Frame, stats: &UserStats, state: &CalendarState, area: Rect) {
    let Some(first) = state.selected_date.with_day(1) else {
        return;
    };
    let offset = first.weekday().num_days_from_monday() as i64;
    let mut lines =
        vec![Line::from("  一   二   三   四   五   六   日").style(Style::default().fg(DIM))];
    for week in 0..6 {
        let mut cells = Vec::new();
        for weekday in 0..7 {
            let date = first.checked_add_signed(Duration::days(week * 7 + weekday - offset));
            let Some(date) = date.filter(|date| date.month() == first.month()) else {
                cells.push(Span::raw("     "));
                continue;
            };
            let practiced = stats.daily_stats.iter().any(|day| {
                day.date == date.format("%Y-%m-%d").to_string() && day.sessions_count > 0
            });
            let mut style = Style::default().fg(if practiced { SUCCESS } else { DIM });
            if date == state.selected_date {
                style = style.bg(ACCENT).fg(Color::Black).bold();
            }
            cells.push(Span::styled(
                format!("{:>2}{}  ", date.day(), if practiced { "·" } else { " " }),
                style,
            ));
        }
        lines.push(Line::from(cells));
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" · 有跟打练习 ")
                .border_style(Style::default().fg(DIM)),
        ),
        area,
    );
}

fn render_day(frame: &mut Frame, stats: &UserStats, date: &str, scroll: usize, area: Rect) {
    let mut lines = Vec::new();
    if let Some(day) = stats.daily_stats.iter().find(|day| day.date == date) {
        lines.push(Line::from(format!(
            " 时长 {}  练习片段 {}",
            format_time(day.total_duration_ms as f64 / 1000.0),
            day.sessions_count
        )));
        lines.push(
            Line::from(format!(
                " WPM {:.1}  CPM {:.1}  准确率 {:.1}%",
                day.avg_wpm,
                day.avg_cpm,
                day.avg_accuracy * 100.0
            ))
            .style(Style::default().fg(ACCENT)),
        );
        lines.push(Line::from(format!(
            " 有效字符 {}  错误位置 {}/{}",
            day.completed_chars, day.error_positions, day.attempted_positions
        )));
        if day.legacy_sessions_count > 0 {
            lines.push(
                Line::from(format!(
                    " 含 {} 条旧口径记录；间隔无法恢复",
                    day.legacy_sessions_count
                ))
                .style(Style::default().fg(WARNING)),
            );
        }
        for entry in &day.entries {
            lines.push(Line::from(format!(
                " {}：{} 完成 / {} 片段 · {}",
                scorer::mode_label(entry.mode),
                entry.completed_count,
                entry.sessions_count,
                format_time(entry.duration_ms as f64 / 1000.0)
            )));
            lines.push(
                Line::from(format!(
                    "   WPM {:.1}  CPM {:.1}  准确率 {:.1}%",
                    entry.avg_wpm,
                    entry.avg_cpm,
                    entry.avg_accuracy * 100.0
                ))
                .style(Style::default().fg(DIM)),
            );
        }
    } else {
        lines.push(Line::from(" 当天没有跟打记录").style(Style::default().fg(DIM)));
    }
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let line_count = lines
        .iter()
        .map(|line| line.width().div_ceil(inner_width).max(1))
        .sum::<usize>();
    let offset = scroll
        .min(line_count.saturating_sub(area.height.saturating_sub(2).max(1) as usize))
        .min(u16::MAX as usize) as u16;
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((offset, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {date} · 本地日期 "))
                    .border_style(Style::default().fg(DIM)),
            ),
        area,
    );
}

pub fn render_characters(
    frame: &mut Frame,
    chars: &[CharStat],
    scroll: usize,
    area: Rect,
    title: &str,
) {
    let mut lines =
        vec![Line::from("窗口50 · 至少10个样本 · 两端10%缩尾").style(Style::default().fg(DIM))];
    if chars.is_empty() {
        lines.push(
            Line::from("尚无可用字符记录；阅读、填空、默写不计入").style(Style::default().fg(DIM)),
        );
    } else {
        lines.push(
            Line::from("字符   样本    平滑CPM    耗时ms   错误率")
                .style(Style::default().fg(ACCENT)),
        );
        let visible = area.height.saturating_sub(4) as usize;
        let start = scroll.min(chars.len().saturating_sub(visible.max(1)));
        for stat in chars.iter().skip(start).take(visible) {
            let character = if stat.char_key == ' ' {
                "␣".to_owned()
            } else {
                stat.char_key.escape_default().to_string()
            };
            let speed = if stat.recent_latencies.len() < scorer::MIN_CHARACTER_SAMPLES {
                "样本不足".to_owned()
            } else {
                format!("{:.1}", stat.avg_cpm)
            };
            let latency = if stat.recent_latencies.len() < scorer::MIN_CHARACTER_SAMPLES {
                "—".to_owned()
            } else {
                format!("{:.0}", stat.avg_latency_ms)
            };
            lines.push(Line::from(format!(
                "{:<4} {:>2}/50  {} {} {:.1}%",
                character,
                stat.recent_latencies.len(),
                pad_column(&speed, 9),
                pad_column(&latency, 8),
                (1.0 - stat.accuracy) * 100.0
            )));
        }
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} "))
                .border_style(Style::default().fg(DIM)),
        ),
        area,
    );
}

fn pad_column(value: &str, width: usize) -> String {
    format!("{value}{}", " ".repeat(width.saturating_sub(value.width())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn calendar_navigation_clamps_month_end_and_preserves_leap_day() {
        let mut state = CalendarState {
            selected_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            char_scroll: 0,
            detail_scroll: 0,
        };
        state.handle_key(KeyCode::PageDown);
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        state.handle_key(KeyCode::Right);
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
        state.handle_key(KeyCode::Up);
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2024, 2, 23).unwrap()
        );
    }

    #[test]
    fn calendar_renders_empty_data_at_narrow_and_wide_terminal_sizes() {
        for (width, height) in [(30, 8), (72, 24), (120, 35)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        &UserStats::default(),
                        &[],
                        &CalendarState::default(),
                        frame.area(),
                    )
                })
                .unwrap();
            let text = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.replace(' ', "").contains("练习日历"));
        }
    }
}
