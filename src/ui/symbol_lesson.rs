use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, SymbolPhase};
use crate::ui::widgets::*;

pub fn render(
    frame: &mut Frame,
    app: &App,
    topic_index: usize,
    symbol_index: usize,
    phase: &SymbolPhase,
) {
    let area = frame.area();
    let topic = match app.symbol_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    match phase {
        SymbolPhase::Explain => {
            let symbol = match topic.symbols.get(symbol_index) {
                Some(s) => s,
                None => return,
            };

            // Title
            let title = Paragraph::new(Line::from(Span::styled(
                format!(
                    " {} \u{300c}{}\u{300d} \u{2014} {} ",
                    symbol.char_repr, symbol.name, topic.meta.topic
                ),
                Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DIM)),
            );
            frame.render_widget(title, chunks[0]);

            // Content
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(Span::styled(
                symbol.summary.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for line in symbol.explanation.lines() {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[
                ("\u{2192}/Enter", "\u{67e5}\u{770b}\u{793a}\u{4f8b}"),
                ("Esc", "\u{8fd4}\u{56de}"),
            ]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
        SymbolPhase::Example(idx) => {
            let symbol = match topic.symbols.get(symbol_index) {
                Some(s) => s,
                None => return,
            };
            let example = match symbol.examples.get(*idx) {
                Some(e) => e,
                None => return,
            };

            let title = Paragraph::new(Line::from(Span::styled(
                format!(
                    " {} \u{793a}\u{4f8b} {}/{} ",
                    symbol.name,
                    idx + 1,
                    symbol.examples.len()
                ),
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
            lines.push(Line::from(vec![
                Span::styled("$ ", Style::default().fg(SIMULATED_PROMPT)),
                Span::styled(
                    example
                        .display
                        .as_deref()
                        .unwrap_or(&example.command)
                        .to_string(),
                    Style::default().fg(Color::White),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                example.explanation.clone(),
                Style::default().fg(TOKEN_DESC),
            )));

            if let Some(output) = &example.simulated_output {
                lines.push(Line::from(""));
                for ol in output.lines() {
                    lines.push(Line::from(Span::styled(
                        ol.to_string(),
                        Style::default().fg(Color::White),
                    )));
                }
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[
                ("\u{2190}\u{2192}", "\u{7ffb}\u{9875}"),
                ("Enter", "\u{4e0b}\u{4e00}\u{6b65}"),
                ("Esc", "\u{8fd4}\u{56de}"),
            ]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
        SymbolPhase::Practice => {
            let title = Paragraph::new(Line::from(Span::styled(
                format!(" {} \u{2014} \u{7ec3}\u{4e60} ", topic.meta.topic),
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
            if topic.exercises.is_empty() {
                lines.push(Line::from(Span::styled(
                    "\u{672c}\u{4e13}\u{9898}\u{6682}\u{65e0}\u{7ec3}\u{4e60}\u{9898}",
                    Style::default().fg(DIM),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "\u{7ec3}\u{4e60}\u{9898}:",
                    Style::default().fg(HEADER),
                )));
                for (i, ex) in topic.exercises.iter().enumerate() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}. {}", i + 1, ex.prompt),
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("     \u{53c2}\u{8003}: {}", ex.answers.join(" / ")),
                        Style::default().fg(DIM),
                    )));
                }
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[("Enter", "\u{5b8c}\u{6210}"), ("Esc", "\u{8fd4}\u{56de}")]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
    }
}
