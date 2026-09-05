use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, ReviewExerciseKind, ReviewPhase, ReviewSource};
use crate::core::matcher::{DiffKind, MatchResult};
use crate::data::models::TopicTrainingLevel;
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

    let source_name = source_name(app, source);
    let title = Paragraph::new(Line::from(Span::styled(
        format!(" 专题训练 — {source_name} "),
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    match phase {
        ReviewPhase::Summary => render_summary(frame, app, source, chunks[1], chunks[2]),
        ReviewPhase::Practice => render_practice(frame, app, chunks[1], chunks[2]),
    }
}

fn render_summary(frame: &mut Frame, app: &App, source: &ReviewSource, area: Rect, hints: Rect) {
    let (practiced, total) = source_progress(app, source);
    let session_size = total.min(10);
    let level = app.topic_training_level;
    let level_description = match level {
        TopicTrainingLevel::L1 => "完整输入命令，记录准确率与 WPM",
        TopicTrainingLevel::L3 => "补全一个关键 token，严格区分大小写",
        TopicTrainingLevel::L5 => "根据中文提示默写完整命令",
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("训练级别: ", Style::default().fg(ACCENT)),
            Span::styled(
                format!("◀  {}  ▶", level.label()),
                Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(level_description, Style::default().fg(DIM))),
        Line::from(""),
        Line::from(format!("专题命令: {total}")),
        Line::from(format!("已练覆盖: {practiced}/{total}")),
        Line::from(format!("本次题数: {session_size}（最多 10 题）")),
        Line::from(""),
        Line::from(Span::styled(
            "命令按练习次数从少到多选择；次数相同时保持题库顺序。",
            Style::default().fg(DIM),
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);

    frame.render_widget(
        Paragraph::new(hint_line(&[
            ("←→/h/l", "级别"),
            ("Enter", "开始"),
            ("Esc", "专题列表"),
        ]))
        .alignment(Alignment::Center),
        hints,
    );
}

fn render_practice(frame: &mut Frame, app: &App, area: Rect, hints: Rect) {
    let rp = &app.review_practice;
    let mut lines = Vec::new();

    if rp.completed {
        lines.push(Line::from(Span::styled(
            "训练完成",
            Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(format!("完成题数: {}", rp.total_count)));
        match app.topic_training_level {
            TopicTrainingLevel::L1 => {
                lines.push(Line::from(format!(
                    "准确率: {:.0}%",
                    average(rp.typing_accuracy_sum, rp.typing_count) * 100.0
                )));
                lines.push(Line::from(format!(
                    "平均 WPM: {:.0}",
                    average(rp.typing_wpm_sum, rp.typing_count)
                )));
            }
            TopicTrainingLevel::L3 => lines.push(Line::from(format!(
                "填空准确率: {:.0}%",
                average(rp.cloze_accuracy_sum, rp.cloze_count) * 100.0
            ))),
            TopicTrainingLevel::L5 => lines.push(Line::from(format!(
                "默写准确率: {:.0}%",
                average(rp.dictation_accuracy_sum, rp.dictation_count) * 100.0
            ))),
        }
    } else if let Some(exercise) = app.current_review_exercise() {
        lines.push(Line::from(Span::styled(
            format!("题目 {}/{}", rp.current_index + 1, rp.total_count),
            Style::default().fg(HEADER),
        )));
        lines.push(Line::from(""));

        match exercise.kind {
            ReviewExerciseKind::Typing => render_typing_exercise(&mut lines, app),
            ReviewExerciseKind::Cloze => render_cloze_exercise(&mut lines, app),
            ReviewExerciseKind::Dictation => render_dictation_exercise(&mut lines, app),
        }
    } else {
        lines.push(Line::from(Span::styled(
            "当前专题暂无可用命令",
            Style::default().fg(DIM),
        )));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
    frame.render_widget(
        Paragraph::new(practice_hints(app)).alignment(Alignment::Center),
        hints,
    );
}

fn render_typing_exercise(lines: &mut Vec<Line<'static>>, app: &App) {
    let Some(exercise) = app.current_review_exercise() else {
        return;
    };
    if app.review_practice.typing_showing_output {
        lines.push(Line::from(Span::styled(
            "预设模拟输出",
            Style::default().fg(ACCENT),
        )));
        lines.push(Line::from(vec![
            Span::styled("$ ", Style::default().fg(SIMULATED_PROMPT)),
            Span::raw(
                exercise
                    .display
                    .clone()
                    .unwrap_or_else(|| exercise.command.clone()),
            ),
        ]));
        if let Some(output) = &exercise.simulated_output {
            lines.extend(output.lines().map(|line| Line::from(line.to_string())));
        } else {
            lines.push(Line::from(Span::styled(
                "（此命令无预设输出）",
                Style::default().fg(DIM),
            )));
        }
        return;
    }

    lines.push(Line::from(Span::styled(
        "L1 完整输入",
        Style::default().fg(ACCENT),
    )));
    lines.push(Line::from(""));
    lines.extend(crate::ui::widgets::typing_lines("$ ", &app.typing_engine));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "当前准确率: {:.0}%  WPM: {:.0}",
            app.typing_engine.current_accuracy() * 100.0,
            app.typing_engine.current_wpm()
        ),
        Style::default().fg(DIM),
    )));
}

fn render_cloze_exercise(lines: &mut Vec<Line<'static>>, app: &App) {
    let Some(exercise) = app.current_review_exercise() else {
        return;
    };
    lines.push(Line::from(Span::styled(
        "L3 单词填空",
        Style::default().fg(ACCENT),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(
        exercise.cloze_skeleton.clone().unwrap_or_default(),
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "补全内容（区分大小写）:",
        Style::default().fg(ACCENT),
    )));
    let input = if app.review_practice.cloze_submitted {
        app.review_practice.cloze_input.clone()
    } else {
        format!("{}█", app.review_practice.cloze_input)
    };
    lines.push(Line::from(format!("  {input}")));

    if app.review_practice.cloze_submitted {
        lines.push(Line::from(""));
        if app.review_practice.cloze_correct == Some(true) {
            lines.push(Line::from(Span::styled(
                "正确",
                Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "错误",
                Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(vec![
                Span::styled("正确答案: ", Style::default().fg(DIM)),
                Span::styled(
                    exercise.cloze_answer.clone().unwrap_or_default(),
                    Style::default().fg(ACCENT),
                ),
            ]));
        }
    }
}

fn render_dictation_exercise(lines: &mut Vec<Line<'static>>, app: &App) {
    let Some(exercise) = app.current_review_exercise() else {
        return;
    };
    lines.push(Line::from(Span::styled(
        "L5 命令默写",
        Style::default().fg(ACCENT),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "提示:",
        Style::default().fg(ACCENT),
    )));
    lines.push(Line::from(format!("  {}", exercise.description)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "你的答案:",
        Style::default().fg(ACCENT),
    )));
    let input = if app.review_practice.dictation_submitted {
        app.review_practice.dictation_input.clone()
    } else {
        format!("{}█", app.review_practice.dictation_input)
    };
    lines.push(Line::from(format!("  {input}")));

    if app.review_practice.dictation_submitted
        && let Some(result) = &app.review_practice.dictation_result
    {
        lines.push(Line::from(""));
        match result {
            MatchResult::Exact(_) | MatchResult::Normalized(_) => {
                lines.push(Line::from(Span::styled(
                    "正确",
                    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                )))
            }
            MatchResult::NoMatch { closest, diff } => {
                lines.push(Line::from(Span::styled(
                    "错误",
                    Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
                )));
                let mut spans = vec![Span::raw("  ")];
                for segment in diff {
                    let style = match segment.kind {
                        DiffKind::Same => Style::default().fg(Color::White),
                        DiffKind::Added => Style::default()
                            .fg(SUCCESS)
                            .add_modifier(Modifier::UNDERLINED),
                        DiffKind::Removed => Style::default()
                            .fg(ERROR)
                            .add_modifier(Modifier::CROSSED_OUT),
                    };
                    spans.push(Span::styled(segment.text.clone(), style));
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

fn practice_hints(app: &App) -> Line<'static> {
    let rp = &app.review_practice;
    if rp.completed {
        return hint_line(&[("Enter/Esc", "训练摘要")]);
    }
    let Some(exercise) = app.current_review_exercise() else {
        return hint_line(&[("Esc", "训练摘要")]);
    };
    match exercise.kind {
        ReviewExerciseKind::Typing if rp.typing_showing_output => {
            hint_line(&[("Enter", "下一题"), ("Esc", "训练摘要")])
        }
        ReviewExerciseKind::Typing if app.typing_engine.is_complete() => hint_line(&[
            ("Enter", "查看输出"),
            ("Backspace", "退格"),
            ("Esc", "训练摘要"),
        ]),
        ReviewExerciseKind::Typing => hint_line(&[
            ("输入字符", "继续"),
            ("Backspace", "退格"),
            ("Esc", "训练摘要"),
        ]),
        ReviewExerciseKind::Cloze if rp.cloze_submitted => {
            hint_line(&[("Enter", "下一题"), ("Esc", "训练摘要")])
        }
        ReviewExerciseKind::Cloze => hint_line(&[
            ("Enter", "提交"),
            ("Backspace", "退格"),
            ("Esc", "训练摘要"),
        ]),
        ReviewExerciseKind::Dictation if rp.dictation_submitted => {
            hint_line(&[("Enter", "下一题"), ("Esc", "训练摘要")])
        }
        ReviewExerciseKind::Dictation => hint_line(&[
            ("Enter", "提交"),
            ("Backspace", "退格"),
            ("Esc", "训练摘要"),
        ]),
    }
}

fn source_name(app: &App, source: &ReviewSource) -> String {
    match source {
        ReviewSource::CommandTopic(id) => app
            .command_training_topics
            .iter()
            .find(|topic| topic.id == *id)
            .map(|topic| format!("{} {}", topic.icon.as_deref().unwrap_or("⌨"), topic.title))
            .unwrap_or_else(|| id.clone()),
        ReviewSource::CommandCategory(category) => {
            format!("{} {}", category.icon(), category.label())
        }
        ReviewSource::SymbolTopic(name) | ReviewSource::SystemTopic(name) => name.clone(),
    }
}

fn source_progress(app: &App, source: &ReviewSource) -> (usize, usize) {
    if let Some(topic) = app.command_training_topic_for_source(source) {
        return (
            app.command_topic_practiced_count(topic),
            topic.command_ids.len(),
        );
    }

    let command_ids: Vec<&str> = match source {
        ReviewSource::SymbolTopic(name) => app
            .symbol_topics
            .iter()
            .find(|topic| topic.meta.id == *name || topic.meta.topic == *name)
            .map(|topic| {
                topic
                    .exercises
                    .iter()
                    .filter_map(|exercise| exercise.command_id.as_deref())
                    .collect()
            })
            .unwrap_or_default(),
        ReviewSource::SystemTopic(name) => app
            .system_topics
            .iter()
            .find(|topic| topic.meta.id == *name || topic.meta.topic == *name)
            .map(|topic| {
                topic
                    .sections
                    .iter()
                    .flat_map(|section| section.commands.iter())
                    .filter_map(|command| command.command_id.as_deref())
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let practiced = command_ids
        .iter()
        .filter(|command_id| {
            app.user_stats
                .command_progress
                .iter()
                .any(|progress| progress.command_id == **command_id && progress.times_practiced > 0)
        })
        .count();
    (practiced, command_ids.len())
}

fn average(sum: f64, count: usize) -> f64 {
    if count == 0 { 0.0 } else { sum / count as f64 }
}

pub fn render_topics(frame: &mut Frame, app: &App) {
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
        " 专题训练 ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    if app.command_training_topics.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("暂无专题训练数据", Style::default().fg(DIM)))
                .alignment(Alignment::Center),
            chunks[1],
        );
    } else {
        render_topic_menu(frame, app, chunks[1]);
    }

    frame.render_widget(
        Paragraph::new(hint_line(&[
            ("↑↓/j/k", "移动"),
            ("Enter", "训练摘要"),
            ("Esc", "学习中心"),
        ]))
        .alignment(Alignment::Center),
        chunks[2],
    );
}

fn render_topic_menu(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app
        .review_topics_index
        .min(app.command_training_topics.len() - 1);
    let show_description = area.height >= 3;
    let description_height = u16::from(show_description);
    let menu_height = area.height.saturating_sub(description_height);
    let menu_area = Rect::new(area.x, area.y, area.width, menu_height);
    let description_area = Rect::new(area.x, area.y + menu_height, area.width, description_height);
    let window = visible_menu_window(
        selected,
        app.command_training_topics.len(),
        1,
        menu_area.height,
    );

    let lines = window
        .map(|topic_index| {
            let topic = &app.command_training_topics[topic_index];
            let is_selected = topic_index == selected;
            let style = if is_selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(MENU_NORMAL)
            };
            let practiced = app.command_topic_practiced_count(topic);
            Line::from(vec![
                Span::styled(if is_selected { " ▶ " } else { "   " }, style),
                Span::styled(
                    format!("{} {}", topic.icon.as_deref().unwrap_or("⌨"), topic.title),
                    style,
                ),
                Span::styled(
                    format!("  {}", topic.difficulty.stars()),
                    Style::default().fg(WARNING),
                ),
                Span::styled(
                    format!("  已练 {practiced}/{}", topic.command_ids.len()),
                    Style::default().fg(DIM),
                ),
                Span::styled(
                    format!("  {}条命令", topic.command_ids.len()),
                    Style::default().fg(DIM),
                ),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines), menu_area);

    if show_description {
        let topic = &app.command_training_topics[selected];
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("简介: ", Style::default().fg(ACCENT)),
                Span::styled(topic.description.clone(), Style::default().fg(DIM)),
            ])),
            description_area,
        );
    }
}
