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
        format!(" {} \u{2014} \u{603b}\u{89c8} ", topic.meta.topic),
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
        format!("\u{5171} {} \u{4e2a}\u{7ae0}\u{8282}", topic.sections.len()),
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

    let hints = hint_line(&[
        ("Enter/\u{2192}", "\u{8fdb}\u{5165}\u{7ae0}\u{8282}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
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
            " {} \u{2014} {}/{} {} ",
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
            format!(
                "\u{5e38}\u{7528}\u{547d}\u{4ee4}: {} \u{4e2a}",
                section.commands.len()
            ),
            Style::default().fg(DIM),
        )));
    }
    if !section.config_files.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(
                "\u{914d}\u{7f6e}\u{6587}\u{4ef6}: {} \u{4e2a}",
                section.config_files.len()
            ),
            Style::default().fg(DIM),
        )));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{4e0a}\u{4e0b}\u{7ae0}\u{8282}"),
        ("Enter/\u{2192}", "\u{67e5}\u{770b}\u{547d}\u{4ee4}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

/// Commands phase: command + simulated output
pub fn render_commands(
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
        format!(
            " {} \u{2014} \u{547d}\u{4ee4} {}/{} ",
            section.title,
            cmd_idx + 1,
            section.commands.len()
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

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            cmd.summary.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let sim = render_simulated_output(&cmd.command, cmd.simulated_output.as_deref());
    let sim_height = 2 + cmd
        .simulated_output
        .as_deref()
        .map_or(1, |o| o.lines().count() + 1);
    let sim_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y + 3,
        width: chunks[1].width,
        height: (sim_height as u16).min(chunks[1].height.saturating_sub(3)),
    };

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);
    frame.render_widget(sim, sim_area);

    let mut hint_items = vec![("Enter/\u{2192}", "\u{4e0b}\u{4e00}\u{4e2a}")];
    if cmd.deep_explanation.is_some() {
        hint_items.push(("D", "\u{67e5}\u{770b}\u{8be6}\u{89e3}"));
    }
    hint_items.push(("Esc", "\u{8fd4}\u{56de}"));
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
        format!(" {} \u{2014} {} ", cf.name, cf.path),
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

    // Sample content
    lines.push(Line::from(Span::styled(
        "\u{793a}\u{4f8b}\u{5185}\u{5bb9}:",
        Style::default().fg(HEADER),
    )));
    for line in cf.sample_content.lines() {
        lines.push(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(ACCENT),
        )));
    }
    lines.push(Line::from(""));

    // Config lessons
    for lesson in &cf.lessons {
        lines.push(Line::from(Span::styled(
            format!("\u{2022} {}", lesson.title),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled("  \u{524d}: ", Style::default().fg(ERROR)),
            Span::styled(lesson.before.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  \u{540e}: ", Style::default().fg(SUCCESS)),
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

    let hints = hint_line(&[
        ("Enter/\u{2192}", "\u{4e0b}\u{4e00}\u{4e2a}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
