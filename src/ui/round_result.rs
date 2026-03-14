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
        " \u{7ec3}\u{4e60}\u{7ed3}\u{679c} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(record) = &app.last_record {
        lines.push(Line::from(""));

        // WPM
        let wpm_arrow = comparison_arrow(record.wpm, app.last_prev_record.as_ref().map(|r| r.wpm));
        lines.push(Line::from(vec![
            Span::styled("  WPM:       ", Style::default().fg(DIM)),
            Span::styled(
                format!("{:.0}", record.wpm),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(wpm_arrow, Style::default().fg(ACCENT)),
        ]));

        // CPM
        lines.push(Line::from(vec![
            Span::styled("  CPM:       ", Style::default().fg(DIM)),
            Span::styled(
                format!("{:.0}", record.cpm),
                Style::default().fg(Color::White),
            ),
        ]));

        // Accuracy
        let acc_arrow = comparison_arrow(
            record.accuracy * 100.0,
            app.last_prev_record.as_ref().map(|r| r.accuracy * 100.0),
        );
        let acc_color = if record.accuracy >= 0.95 {
            SUCCESS
        } else if record.accuracy >= 0.80 {
            WARNING
        } else {
            ERROR
        };
        lines.push(Line::from(vec![
            Span::styled("  \u{51c6}\u{786e}\u{7387}:   ", Style::default().fg(DIM)),
            Span::styled(
                format!("{:.1}%", record.accuracy * 100.0),
                Style::default().fg(acc_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(acc_arrow, Style::default().fg(ACCENT)),
        ]));

        // Time
        let duration_secs = (record.finished_at - record.started_at) as f64 / 1000.0;
        lines.push(Line::from(vec![
            Span::styled("  \u{7528}\u{65f6}:     ", Style::default().fg(DIM)),
            Span::styled(
                format_time(duration_secs),
                Style::default().fg(Color::White),
            ),
        ]));

        // Error count
        lines.push(Line::from(vec![
            Span::styled("  \u{9519}\u{8bef}:     ", Style::default().fg(DIM)),
            Span::styled(
                format!("{}", record.error_count),
                Style::default().fg(if record.error_count == 0 {
                    SUCCESS
                } else {
                    WARNING
                }),
            ),
        ]));

        lines.push(Line::from(""));

        // Error top 5
        let mut error_chars: std::collections::HashMap<char, u32> =
            std::collections::HashMap::new();
        for ks in &record.keystrokes {
            if !ks.correct {
                *error_chars.entry(ks.expected).or_default() +=
                    ks.attempts.saturating_sub(1) as u32;
            }
        }
        let mut errors: Vec<_> = error_chars.into_iter().collect();
        errors.sort_by(|a, b| b.1.cmp(&a.1));

        if !errors.is_empty() {
            lines.push(Line::from(Span::styled(
                "  \u{9519}\u{8bef}\u{6700}\u{591a}\u{7684}\u{5b57}\u{7b26}:",
                Style::default().fg(HEADER),
            )));
            for (ch, count) in errors.iter().take(5) {
                let display = if *ch == ' ' {
                    "\u{2423}".to_string()
                } else {
                    ch.to_string()
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("    '{}'", display), Style::default().fg(ERROR)),
                    Span::styled(format!("  \u{00d7}{}", count), Style::default().fg(DIM)),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{7ec3}\u{4e60}\u{7ed3}\u{679c}",
            Style::default().fg(DIM),
        )));
    }

    let content = Paragraph::new(lines);
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[("Enter/Esc", "\u{8fd4}\u{56de}\u{4e3b}\u{83dc}\u{5355}")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn comparison_arrow(current: f64, previous: Option<f64>) -> String {
    match previous {
        Some(prev) => {
            let diff = current - prev;
            if diff > 0.5 {
                format!(" \u{2191}{:.0}", diff)
            } else if diff < -0.5 {
                format!(" \u{2193}{:.0}", diff.abs())
            } else {
                " \u{2192}".to_string()
            }
        }
        None => String::new(),
    }
}
