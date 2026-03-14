use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, ReviewPhase, ReviewSource};
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App, source: &ReviewSource, phase: &ReviewPhase) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let source_name = match source {
        ReviewSource::CommandCategory(cat) => format!("{} {}", cat.icon(), cat.label()),
        ReviewSource::SymbolTopic(name) => name.clone(),
        ReviewSource::SystemTopic(name) => name.clone(),
    };

    match phase {
        ReviewPhase::Summary => {
            let title = Paragraph::new(Line::from(Span::styled(
                format!(" \u{4e13}\u{9898}\u{590d}\u{4e60} \u{2014} {} \u{2014} \u{77e5}\u{8bc6}\u{68b3}\u{7406} ", source_name),
                Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
            frame.render_widget(title, chunks[0]);

            let mut lines: Vec<Line> = Vec::new();

            // Build summary table from available data
            match source {
                ReviewSource::CommandCategory(cat) => {
                    let cmds: Vec<_> = app.commands.iter().filter(|c| c.category == *cat).collect();
                    if cmds.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "\u{8be5}\u{7c7b}\u{522b}\u{6682}\u{65e0}\u{547d}\u{4ee4}",
                            Style::default().fg(DIM),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!(
                                "\u{547d}\u{4ee4}\u{6982}\u{89c8} ({} \u{4e2a}):",
                                cmds.len()
                            ),
                            Style::default().fg(HEADER),
                        )));
                        lines.push(Line::from(""));

                        // Table header
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {:<24}", "\u{547d}\u{4ee4}"),
                                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "\u{8bf4}\u{660e}",
                                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(Span::styled(
                            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                            Style::default().fg(DIM),
                        )));

                        for cmd in &cmds {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format!("  {:<24}", cmd.command),
                                    Style::default().fg(Color::White),
                                ),
                                Span::styled(cmd.summary.clone(), Style::default().fg(DIM)),
                            ]));
                        }
                    }
                }
                _ => {
                    lines.push(Line::from(Span::styled(
                        "\u{590d}\u{4e60}\u{5185}\u{5bb9}\u{52a0}\u{8f7d}\u{4e2d}...",
                        Style::default().fg(DIM),
                    )));
                }
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[
                ("Enter", "\u{5f00}\u{59cb}\u{7ec3}\u{4e60}"),
                ("Esc", "\u{8fd4}\u{56de}"),
            ]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
        ReviewPhase::Practice(idx) => {
            let title = Paragraph::new(Line::from(Span::styled(
                format!(" \u{4e13}\u{9898}\u{590d}\u{4e60} \u{2014} {} \u{2014} \u{96c6}\u{4e2d}\u{7ec3}\u{4e60} ", source_name),
                Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
            frame.render_widget(title, chunks[0]);

            let content = Paragraph::new(Line::from(Span::styled(
                format!(
                    "\u{7ec3}\u{4e60}\u{9898} #{} \u{2014} \u{6309} Enter \u{5b8c}\u{6210}",
                    idx + 1
                ),
                Style::default().fg(Color::White),
            )));
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[("Enter", "\u{5b8c}\u{6210}"), ("Esc", "\u{8fd4}\u{56de}")]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
    }
}
