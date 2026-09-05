use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::data::scenario_loader::ScenarioPhase;
use crate::flow::scenario_flow::active_scenario;
use crate::ui::widgets::*;

pub fn render_topics(frame: &mut Frame, app: &App) {
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new("场景实训 · 真实需求与故障演练")
            .alignment(Alignment::Center)
            .style(Style::default().fg(HEADER))
            .block(title_block(" 学习中心 ")),
        areas[0],
    );
    let mut lines = Vec::new();
    let available = areas[1].height.saturating_sub(2);
    let range = visible_menu_window(
        app.scenario_state.selected_index,
        app.scenario_catalog.len(),
        2,
        available,
    );
    for i in range {
        let case = &app.scenario_catalog[i];
        let progress = app.scenario_state.saved.get(&case.id);
        let completed = progress.is_some_and(|saved| saved.completed);
        let practiced = progress.map(|saved| saved.step_index).unwrap_or(0);
        let status = if completed {
            "已完成".to_string()
        } else if progress.is_some() {
            format!("继续 {}/{}", practiced + 1, case.steps.len())
        } else {
            format!("{} 步", case.steps.len())
        };
        let selected = i == app.scenario_state.selected_index;
        lines.push(Line::from(Span::styled(
            format!(
                "{} {}  [{}]",
                if selected { "▶" } else { " " },
                case.title,
                status
            ),
            Style::default()
                .fg(if completed { SUCCESS } else { MENU_NORMAL })
                .bg(if selected {
                    MENU_SELECTED_BG
                } else {
                    Color::Reset
                }),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "   {} · {} 个诊断判断",
                case.category,
                case.steps
                    .iter()
                    .filter(|step| step.decision.is_some())
                    .count()
            ),
            Style::default().fg(DIM),
        )));
    }
    if app.scenario_catalog.is_empty() {
        lines.push(Line::from("暂无场景实训数据"));
    }
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL)),
        areas[1],
    );
    frame.render_widget(
        Paragraph::new(vec![
            hint_line(&[
                ("↑↓", "选择"),
                ("Enter", "开始/继续"),
                ("r", "从头练习"),
                ("Esc", "返回"),
            ]),
            Line::from(Span::styled(
                "命令与输出均为教学模拟；进度自动保存。",
                Style::default().fg(DIM),
            )),
        ]),
        areas[2],
    );
}

pub fn render_practice(frame: &mut Frame, app: &App) {
    let Some(case) = active_scenario(app) else {
        return;
    };
    let state = &app.scenario_state;
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new(format!(
            "{} · {}/{}",
            case.title,
            state.step_index + 1,
            case.steps.len()
        ))
        .style(Style::default().fg(HEADER))
        .block(title_block(" 场景实训 · 模拟终端 ")),
        areas[0],
    );
    let mut lines: Vec<Line<'static>> = Vec::new();
    if let Some(feedback) = &state.feedback {
        lines.push(Line::from(Span::styled(
            feedback.clone(),
            Style::default().fg(WARNING),
        )));
        lines.push(Line::default());
    }
    match state.phase {
        ScenarioPhase::Intro => {
            add_text(&mut lines, "背景", &case.background);
            add_text(&mut lines, "目标", &case.objective);
            add_text(&mut lines, "练习环境", &case.environment);
            lines.push(Line::from(
                "逐条跟打 → 查看证据 → 判断下一步 → 修复并验证。",
            ));
            lines.push(Line::from(
                "Enter 开始；随时 Esc 保存并退出，未完成命令下次从头输入。",
            ));
        }
        ScenarioPhase::Recap => {
            lines.push(Line::from(Span::styled(
                "案例完成",
                Style::default().fg(SUCCESS),
            )));
            add_text(&mut lines, "复盘", &case.recap);
            for source in &case.sources {
                lines.push(Line::from(Span::styled(
                    source.clone(),
                    Style::default().fg(DIM),
                )));
            }
            lines.push(Line::from("Enter 返回案例目录；Ctrl+R 重做本案例。"));
        }
        _ => {
            let Some(step) = case.steps.get(state.step_index) else {
                return;
            };
            lines.push(Line::from(Span::styled(
                step.title.clone(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )));
            add_text(&mut lines, "指导", &step.instruction);
            if state.phase == ScenarioPhase::Typing {
                let engine = &app.typing_engine;
                let mut spans = vec![Span::styled(
                    step.prompt.clone(),
                    Style::default().fg(PROMPT_COLOR),
                )];
                for (i, c) in engine.target.iter().enumerate() {
                    let style = if i < engine.cursor {
                        Style::default().fg(TYPED_CORRECT)
                    } else if i == engine.cursor {
                        if engine.is_error_flashing() {
                            Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
                        } else {
                            Style::default().fg(CURSOR).bg(CURSOR_BG)
                        }
                    } else {
                        Style::default().fg(PENDING)
                    };
                    spans.push(Span::styled(c.to_string(), style));
                }
                lines.push(Line::from(spans));
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    if engine.is_complete() {
                        "已输入完整命令，按 Enter 查看模拟输出。"
                    } else {
                        "沿灰字输入；回车只在整条命令完成后提交。"
                    },
                    Style::default().fg(DIM),
                )));
                lines.push(Line::from(format!(
                    "WPM {:.1}  CPM {:.1}  准确率 {:.1}%",
                    engine.current_wpm(),
                    engine.current_cpm(),
                    engine.current_accuracy() * 100.0
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("{}{}", step.prompt, step.command),
                    Style::default().fg(PROMPT_COLOR),
                )));
                for output in step.output.lines() {
                    lines.push(Line::from(output.to_string()));
                }
                add_text(&mut lines, "证据解读", &step.explanation);
                if state.phase == ScenarioPhase::Decision {
                    if let Some(decision) = &step.decision {
                        lines.push(Line::from(Span::styled(
                            decision.question.clone(),
                            Style::default().fg(HEADER),
                        )));
                        for (i, choice) in decision.choices.iter().enumerate() {
                            let selected = i == state.choice_index;
                            lines.push(Line::from(Span::styled(
                                format!("{} {}", if selected { "▶" } else { " " }, choice.text),
                                Style::default().fg(Color::White).bg(if selected {
                                    MENU_SELECTED_BG
                                } else {
                                    Color::Reset
                                }),
                            )));
                        }
                    }
                } else {
                    lines.push(Line::from(Span::styled(
                        "Enter 继续；PgUp/PgDn 阅读长输出。",
                        Style::default().fg(DIM),
                    )));
                }
            }
        }
    }
    let body = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let inner = body.inner(areas[1]);
    let height = rendered_wrapped_line_count(&lines, inner.width, false);
    let maximum_scroll = height.saturating_sub(usize::from(inner.height));
    let scroll = if state.phase == ScenarioPhase::Typing && app.typing_engine.cursor > 0 {
        // Keep a long command's cursor visible while retaining manual scrolling before typing.
        let prefix = app
            .typing_engine
            .target
            .iter()
            .take(app.typing_engine.cursor)
            .collect::<String>();
        let preceding = lines
            .iter()
            .take_while(|line| {
                !line
                    .spans
                    .first()
                    .is_some_and(|span| span.content == case.steps[state.step_index].prompt)
            })
            .cloned()
            .collect::<Vec<_>>();
        let before = rendered_wrapped_line_count(&preceding, inner.width, false);
        let cursor_row = before
            + (unicode_width::UnicodeWidthStr::width(prefix.as_str())
                + unicode_width::UnicodeWidthStr::width(
                    case.steps[state.step_index].prompt.as_str(),
                ))
                / usize::from(inner.width.max(1));
        state
            .scroll
            .max(cursor_row.saturating_sub(usize::from(inner.height.saturating_sub(2))))
            .min(maximum_scroll)
    } else {
        state.scroll.min(maximum_scroll)
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(body)
            .wrap(Wrap { trim: false })
            .scroll((scroll.min(u16::MAX as usize) as u16, 0)),
        areas[1],
    );
    frame.render_widget(
        Paragraph::new(vec![
            hint_line(&[
                ("Enter", "提交/继续"),
                ("↑↓", "选择判断"),
                ("PgUp/PgDn", "滚动"),
            ]),
            hint_line(&[("Esc", "保存并返回"), ("Ctrl+R", "重做案例")]),
        ]),
        areas[2],
    );
}

fn add_text(lines: &mut Vec<Line<'static>>, label: &str, text: &str) {
    lines.push(Line::from(Span::styled(
        format!("{label}："),
        Style::default().fg(ACCENT),
    )));
    lines.extend(text.lines().map(|line| Line::from(line.to_string())));
    lines.push(Line::default());
}
