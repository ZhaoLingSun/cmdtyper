use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, ReviewExerciseKind, ReviewPhase, ReviewSource};
use crate::core::matcher::{DiffKind, MatchResult};
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
                format!(" 专题复习 — {} ", source_name),
                Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DIM)),
            );
            frame.render_widget(title, chunks[0]);

            let mut lines: Vec<Line> = vec![
                Line::from("按 Enter 开始复习练习。"),
                Line::from(""),
                Line::from(Span::styled(
                    "题型比例：打字题 70% + 默写题 30%",
                    Style::default().fg(DIM),
                )),
                Line::from(""),
            ];

            if let ReviewSource::CommandCategory(cat) = source {
                let count = app.commands.iter().filter(|c| c.category == *cat).count();
                lines.push(Line::from(format!("可用命令: {}", count)));
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = hint_line(&[("Enter", "开始练习"), ("Esc", "返回")]);
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
        ReviewPhase::Practice(_) => {
            let title = Paragraph::new(Line::from(Span::styled(
                format!(" 专题复习 — {} ", source_name),
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
            let rp = &app.review_practice;

            if rp.completed {
                let accuracy = app.review_accuracy() * 100.0;
                let typing_acc = if rp.typing_count == 0 {
                    0.0
                } else {
                    rp.typing_accuracy_sum / rp.typing_count as f64 * 100.0
                };
                let typing_wpm = if rp.typing_count == 0 {
                    0.0
                } else {
                    rp.typing_wpm_sum / rp.typing_count as f64
                };
                let dict_acc = if rp.dictation_count == 0 {
                    0.0
                } else {
                    rp.dictation_accuracy_sum / rp.dictation_count as f64 * 100.0
                };

                lines.push(Line::from(Span::styled(
                    "✅ 复习完成",
                    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(format!("总题数: {}", rp.total_count)));
                lines.push(Line::from(format!("总体准确率: {:.0}%", accuracy)));
                lines.push(Line::from(format!(
                    "打字题: {}（准确率 {:.0}% / WPM {:.0}）",
                    rp.typing_count, typing_acc, typing_wpm
                )));
                lines.push(Line::from(format!(
                    "默写题: {}（准确率 {:.0}%）",
                    rp.dictation_count, dict_acc
                )));
            } else if let Some(ex) = app.current_review_exercise() {
                lines.push(Line::from(Span::styled(
                    format!("题目 {}/{}", rp.current_index + 1, rp.total_count),
                    Style::default().fg(HEADER),
                )));
                lines.push(Line::from(""));

                match ex.kind {
                    ReviewExerciseKind::Typing => {
                        lines.push(Line::from(Span::styled(
                            "题型: 打字",
                            Style::default().fg(ACCENT),
                        )));
                        lines.push(Line::from(""));
                        lines.push(render_typing_line("$ ", &app.typing_engine));
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            format!(
                                "当前准确率: {:.0}%",
                                app.typing_engine.current_accuracy() * 100.0
                            ),
                            Style::default().fg(DIM),
                        )));
                        lines.push(Line::from(Span::styled(
                            format!("当前 WPM: {:.0}", app.typing_engine.current_wpm()),
                            Style::default().fg(DIM),
                        )));
                    }
                    ReviewExerciseKind::Dictation => {
                        lines.push(Line::from(Span::styled(
                            "题型: 默写",
                            Style::default().fg(ACCENT),
                        )));
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "提示（中文描述）:",
                            Style::default().fg(ACCENT),
                        )));
                        lines.push(Line::from(format!("  {}", ex.description)));
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "你的答案:",
                            Style::default().fg(ACCENT),
                        )));
                        let input_display = if rp.dictation_submitted {
                            rp.dictation_input.clone()
                        } else {
                            format!("{}█", rp.dictation_input)
                        };
                        lines.push(Line::from(format!("  {}", input_display)));

                        if rp.dictation_submitted
                            && let Some(result) = &rp.dictation_result
                        {
                            lines.push(Line::from(""));
                            match result {
                                MatchResult::Exact(_) | MatchResult::Normalized(_) => {
                                    lines.push(Line::from(Span::styled(
                                        "✅ 正确",
                                        Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                                    )));
                                }
                                MatchResult::NoMatch { closest, diff } => {
                                    lines.push(Line::from(Span::styled(
                                        "❌ 错误",
                                        Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
                                    )));
                                    let mut spans = vec![Span::raw("  ")];
                                    for seg in diff {
                                        let style = match seg.kind {
                                            DiffKind::Same => Style::default().fg(Color::White),
                                            DiffKind::Added => Style::default()
                                                .fg(SUCCESS)
                                                .add_modifier(Modifier::UNDERLINED),
                                            DiffKind::Removed => Style::default()
                                                .fg(ERROR)
                                                .add_modifier(Modifier::CROSSED_OUT),
                                        };
                                        spans.push(Span::styled(seg.text.clone(), style));
                                    }
                                    lines.push(Line::from(spans));
                                    lines.push(Line::from(vec![
                                        Span::styled("正确答案: ", Style::default().fg(DIM)),
                                        Span::styled(closest.clone(), Style::default().fg(ACCENT)),
                                    ]));
                                }
                            }
                        }
                    }
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "暂无复习题",
                    Style::default().fg(DIM),
                )));
            }

            let content = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(content, chunks[1]);

            let hints = if rp.completed {
                hint_line(&[("Enter", "返回"), ("Esc", "返回")])
            } else if let Some(ex) = app.current_review_exercise() {
                match ex.kind {
                    ReviewExerciseKind::Typing => {
                        if app.typing_engine.is_complete() {
                            hint_line(&[("Enter", "下一题"), ("Esc", "返回")])
                        } else {
                            hint_line(&[("输入字符", "继续"), ("Esc", "返回")])
                        }
                    }
                    ReviewExerciseKind::Dictation => {
                        if rp.dictation_submitted {
                            hint_line(&[("Enter", "下一题"), ("Esc", "返回")])
                        } else {
                            hint_line(&[("Enter", "提交"), ("Esc", "返回")])
                        }
                    }
                }
            } else {
                hint_line(&[("Esc", "返回")])
            };
            frame.render_widget(
                Paragraph::new(hints).alignment(Alignment::Center),
                chunks[2],
            );
        }
    }
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
