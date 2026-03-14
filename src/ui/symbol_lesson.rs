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

            let title = Paragraph::new(Line::from(Span::styled(
                format!(
                    " {} 「{}」 — {} ",
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

            let hints = hint_line(&[("→/Enter", "查看示例"), ("Esc", "返回")]);
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
                    " {} 示例 {}/{} ",
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

            let mut hint_items = vec![("←→", "翻页"), ("Enter", "下一步")];
            if example.deep_explanation.is_some() {
                hint_items.push(("D", "查看详解"));
            }
            hint_items.push(("Esc", "返回"));
            let hints = hint_line(&hint_items);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
        SymbolPhase::Practice => {
            let title = Paragraph::new(Line::from(Span::styled(
                format!(" {} — 练习 ", topic.meta.topic),
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
            let sp = &app.symbol_practice;

            if sp.completed {
                let accuracy = if sp.total_count == 0 {
                    0.0
                } else {
                    (sp.correct_count as f64 / sp.total_count as f64) * 100.0
                };
                lines.push(Line::from(Span::styled(
                    "✅ 练习完成",
                    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(format!("总题数: {}", sp.total_count)));
                lines.push(Line::from(format!("答对: {}", sp.correct_count)));
                lines.push(Line::from(format!("准确率: {:.0}%", accuracy)));
            } else if let Some(ex) = app.current_symbol_practice_exercise(topic_index) {
                lines.push(Line::from(Span::styled(
                    format!("题目 {}/{}", sp.current_index + 1, sp.total_count),
                    Style::default().fg(HEADER),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "提示（中文描述）:",
                    Style::default().fg(ACCENT),
                )));
                lines.push(Line::from(format!("  {}", ex.prompt)));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "你的答案:",
                    Style::default().fg(ACCENT),
                )));

                let input_display = if sp.submitted {
                    sp.current_input.clone()
                } else {
                    format!("{}█", sp.current_input)
                };
                lines.push(Line::from(format!("  {}", input_display)));

                if sp.submitted {
                    lines.push(Line::from(""));
                    if sp.last_correct == Some(true) {
                        lines.push(Line::from(Span::styled(
                            "✅ 正确",
                            Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!("❌ 错误（已错 {}/3）", sp.error_count),
                            Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
                        )));
                        if sp.show_answer {
                            lines.push(Line::from(vec![
                                Span::styled("正确答案: ", Style::default().fg(DIM)),
                                Span::styled(ex.answers.join(" / "), Style::default().fg(ACCENT)),
                            ]));
                        }
                    }
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "暂无练习题",
                    Style::default().fg(DIM),
                )));
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = if sp.completed {
                hint_line(&[("Enter", "返回"), ("Esc", "返回")])
            } else if sp.submitted && sp.last_correct == Some(true) {
                hint_line(&[("Enter", "下一题"), ("Esc", "返回")])
            } else if sp.submitted {
                hint_line(&[("Enter", "重试"), ("Esc", "返回")])
            } else {
                hint_line(&[("Enter", "提交"), ("Esc", "返回")])
            };
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
    }
}
