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
                format!(" {} 示例 {}/{} ", symbol.name, idx + 1, symbol.examples.len()),
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
        SymbolPhase::TypingPractice { exercise_idx } => {
            render_typing_practice(frame, app, topic_index, *exercise_idx, &chunks);
        }
        SymbolPhase::Practice => {
            render_dictation_practice(frame, app, topic_index, &chunks);
        }
    }
}

fn render_typing_practice(
    frame: &mut Frame,
    app: &App,
    topic_index: usize,
    exercise_idx: usize,
    chunks: &[Rect],
) {
    let topic = match app.symbol_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };
    let sp = &app.symbol_practice;

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} — 对着打练习 ", topic.meta.topic),
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
    if sp.completed {
        render_symbol_summary_lines(app, &mut lines);
    } else if let Some(ex) = app.current_symbol_typing_exercise(topic_index, exercise_idx) {
        let total = sp.typing_indices.len();
        lines.push(Line::from(Span::styled(
            format!("题目 {}/{}（打字）", exercise_idx + 1, total),
            Style::default().fg(HEADER),
        )));
        lines.push(Line::from(""));

        if !ex.prompt.trim().is_empty() {
            lines.push(Line::from(Span::styled(
                format!("说明: {}", ex.prompt),
                Style::default().fg(DIM),
            )));
            lines.push(Line::from(""));
        }

        lines.push(render_typing_line("$ ", &app.typing_engine));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("当前准确率: {:.0}%", app.typing_engine.current_accuracy() * 100.0),
            Style::default().fg(DIM),
        )));
        lines.push(Line::from(Span::styled(
            format!("当前 WPM: {:.0}", app.typing_engine.current_wpm()),
            Style::default().fg(DIM),
        )));

        if app.typing_engine.is_complete() && sp.typing_showing_output {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("模拟输出:", Style::default().fg(ACCENT))));
            if let Some(output) = &ex.simulated_output {
                for line in output.lines() {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::White),
                    )));
                }
            }
        }
    } else {
        lines.push(Line::from(Span::styled("暂无打字练习", Style::default().fg(DIM))));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints = if sp.completed {
        hint_line(&[("Enter", "返回"), ("Esc", "返回")])
    } else if app.typing_engine.is_complete() {
        if sp.typing_showing_output {
            hint_line(&[("Enter", "下一题"), ("Esc", "返回")])
        } else {
            let has_output = app
                .current_symbol_typing_exercise(topic_index, exercise_idx)
                .and_then(|ex| ex.simulated_output.as_deref())
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false);
            if has_output {
                hint_line(&[("Enter", "查看输出"), ("Esc", "返回")])
            } else {
                hint_line(&[("Enter", "下一题"), ("Esc", "返回")])
            }
        }
    } else {
        hint_line(&[("输入字符", "继续"), ("Esc", "返回")])
    };
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn render_dictation_practice(frame: &mut Frame, app: &App, topic_index: usize, chunks: &[Rect]) {
    let topic = match app.symbol_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} — 默写练习 ", topic.meta.topic),
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
        render_symbol_summary_lines(app, &mut lines);
    } else if let Some(ex) = app.current_symbol_practice_exercise(topic_index) {
        lines.push(Line::from(Span::styled(
            format!(
                "题目 {}/{}（默写）",
                sp.current_index + 1,
                sp.dictation_indices.len()
            ),
            Style::default().fg(HEADER),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "提示（中文描述）:",
            Style::default().fg(ACCENT),
        )));
        lines.push(Line::from(format!("  {}", ex.prompt)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("你的答案:", Style::default().fg(ACCENT))));

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
        lines.push(Line::from(Span::styled("暂无默写练习", Style::default().fg(DIM))));
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

fn render_symbol_summary_lines(app: &App, lines: &mut Vec<Line>) {
    let sp = &app.symbol_practice;
    let typing_acc = if sp.typing_count == 0 {
        0.0
    } else {
        sp.typing_accuracy_sum / sp.typing_count as f64 * 100.0
    };
    let typing_wpm = if sp.typing_count == 0 {
        0.0
    } else {
        sp.typing_wpm_sum / sp.typing_count as f64
    };
    let dict_acc = if sp.dictation_count == 0 {
        0.0
    } else {
        sp.dictation_accuracy_sum / sp.dictation_count as f64 * 100.0
    };

    lines.push(Line::from(Span::styled(
        "✅ 练习完成",
        Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(format!("总题数: {}", sp.total_count)));
    lines.push(Line::from(format!(
        "对着打: {} 题（WPM {:.0} / 准确率 {:.0}%）",
        sp.typing_count, typing_wpm, typing_acc
    )));
    lines.push(Line::from(format!(
        "默写: {} 题（准确率 {:.0}%）",
        sp.dictation_count, dict_acc
    )));
}

fn render_typing_line<'a>(prompt: &str, engine: &crate::core::engine::TypingEngine) -> Line<'a> {
    let mut spans = Vec::new();
    spans.push(Span::styled(
        prompt.to_string(),
        Style::default().fg(PROMPT_COLOR),
    ));

    let is_flashing = engine.is_error_flashing();
    for (idx, ch) in engine.target.iter().enumerate() {
        let style = if idx < engine.cursor {
            Style::default().fg(TYPED_CORRECT)
        } else if idx == engine.cursor {
            if is_flashing {
                Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
            } else {
                Style::default().fg(CURSOR).bg(CURSOR_BG)
            }
        } else {
            Style::default().fg(PENDING).bg(PENDING_BG)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    Line::from(spans)
}
