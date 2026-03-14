use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::ui::widgets::*;

/// Overview phase: topic summary + ASCII art overview
pub fn render_overview(frame: &mut Frame, app: &App, topic_index: usize) {
    let area = frame.area();
    let topic = match app.system_topics.get(topic_index) {
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

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} — 总览 ", topic.meta.topic),
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
        topic.meta.description.clone(),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    if let Some(overview) = &topic.overview {
        for line in overview.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(ACCENT),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("共 {} 个章节", topic.sections.len()),
        Style::default().fg(DIM),
    )));
    for (i, section) in topic.sections.iter().enumerate() {
        lines.push(Line::from(Span::styled(
            format!("  {}. {}", i + 1, section.title),
            Style::default().fg(Color::White),
        )));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[("Enter/→", "进入章节"), ("Esc", "返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

/// Detail phase: section description
pub fn render_detail(frame: &mut Frame, app: &App, topic_index: usize, section_index: usize) {
    let area = frame.area();
    let topic = match app.system_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };
    let section = match topic.sections.get(section_index) {
        Some(s) => s,
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

    let title = Paragraph::new(Line::from(Span::styled(
        format!(
            " {} — {}/{} {} ",
            topic.meta.topic,
            section_index + 1,
            topic.sections.len(),
            section.title
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
    for line in section.description.lines() {
        lines.push(Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        )));
    }

    if !section.commands.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("常用命令: {} 个", section.commands.len()),
            Style::default().fg(DIM),
        )));
    }
    if !section.config_files.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("配置文件: {} 个", section.config_files.len()),
            Style::default().fg(DIM),
        )));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[("↑↓", "上下章节"), ("Enter/→", "开始命令练习"), ("Esc", "返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

/// TypingPractice phase: command typing + simulated output
pub fn render_typing_practice(
    frame: &mut Frame,
    app: &App,
    topic_index: usize,
    section_index: usize,
    cmd_idx: usize,
) {
    let area = frame.area();
    let topic = match app.system_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };
    let section = match topic.sections.get(section_index) {
        Some(s) => s,
        None => return,
    };
    let cmd = match section.commands.get(cmd_idx) {
        Some(c) => c,
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

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} — 命令练习 {}/{} ", section.title, cmd_idx + 1, section.commands.len()),
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
        Line::from(Span::styled(
            cmd.summary.clone(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        render_typing_line("$ ", &app.typing_engine),
        Line::from(""),
        Line::from(Span::styled(
            format!("当前准确率: {:.0}%", app.typing_engine.current_accuracy() * 100.0),
            Style::default().fg(DIM),
        )),
        Line::from(Span::styled(
            format!("当前 WPM: {:.0}", app.typing_engine.current_wpm()),
            Style::default().fg(DIM),
        )),
    ];

    if app.typing_engine.is_complete() && app.system_typing_showing_output {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("模拟输出:", Style::default().fg(ACCENT))));
        if let Some(output) = &cmd.simulated_output {
            for line in output.lines() {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let has_output = cmd
        .simulated_output
        .as_deref()
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);
    let mut hint_items = if app.typing_engine.is_complete() {
        if app.system_typing_showing_output {
            vec![("Enter", "下一步")]
        } else if has_output {
            vec![("Enter", "查看输出")]
        } else {
            vec![("Enter", "下一步")]
        }
    } else {
        vec![("输入字符", "继续")]
    };

    if cmd.deep_explanation.is_some() {
        hint_items.push(("D", "查看详解"));
    }
    hint_items.push(("Esc", "返回"));
    let hints = hint_line(&hint_items);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

/// ConfigFile phase: config file sample + lessons
pub fn render_config_file(
    frame: &mut Frame,
    app: &App,
    topic_index: usize,
    section_index: usize,
    cf_idx: usize,
) {
    let area = frame.area();
    let topic = match app.system_topics.get(topic_index) {
        Some(t) => t,
        None => return,
    };
    let section = match topic.sections.get(section_index) {
        Some(s) => s,
        None => return,
    };
    let cf = match section.config_files.get(cf_idx) {
        Some(c) => c,
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

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} — {} ", cf.name, cf.path),
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
        cf.description.clone(),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled("示例内容:", Style::default().fg(HEADER))));
    for line in cf.sample_content.lines() {
        lines.push(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(ACCENT),
        )));
    }
    lines.push(Line::from(""));

    for lesson in &cf.lessons {
        lines.push(Line::from(Span::styled(
            format!("• {}", lesson.title),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled("  前: ", Style::default().fg(ERROR)),
            Span::styled(lesson.before.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  后: ", Style::default().fg(SUCCESS)),
            Span::styled(lesson.after.clone(), Style::default().fg(Color::White)),
        ]));
        for line in lesson.explanation.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(DIM),
            )));
        }
        lines.push(Line::from(""));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[("Enter/→", "下一个"), ("Esc", "返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
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
