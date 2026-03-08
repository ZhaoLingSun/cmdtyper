use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::ui::widgets::*;

/// Render overview phase: explanation + syntax + options
pub fn render_overview(
    frame: &mut Frame,
    app: &App,
    category_index: usize,
    command_index: usize,
) {
    let area = frame.area();
    let categories = app.get_lesson_categories();
    let cat = match categories.get(category_index) {
        Some(c) => *c,
        None => return,
    };
    let lessons = app.get_lessons_for_category(cat);
    let lesson = match lessons.get(command_index) {
        Some(l) => l,
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

    // Title
    let title_text = if let Some(full) = &lesson.meta.full_name {
        format!(" {} ({}) \u{2014} \u{547d}\u{4ee4}\u{6982}\u{89c8} ", lesson.meta.command, full)
    } else {
        format!(" {} \u{2014} \u{547d}\u{4ee4}\u{6982}\u{89c8} ", lesson.meta.command)
    };
    let title = Paragraph::new(Line::from(Span::styled(
        title_text,
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    frame.render_widget(title, chunks[0]);

    // Content
    let content_area = chunks[1];
    let mut lines: Vec<Line> = Vec::new();

    // Summary
    lines.push(Line::from(Span::styled(
        format!("\u{7b80}\u{4ecb}: {}", lesson.overview.summary),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Explanation
    for line in lesson.overview.explanation.lines() {
        lines.push(Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        )));
    }
    lines.push(Line::from(""));

    // Syntax
    lines.push(Line::from(Span::styled(
        "\u{8bed}\u{6cd5}:",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", lesson.syntax.basic),
        Style::default().fg(ACCENT),
    )));

    for part in &lesson.syntax.parts {
        lines.push(Line::from(vec![
            Span::styled(format!("    {} ", part.name), Style::default().fg(TOKEN_DESC)),
            Span::styled(format!("\u{2014} {}", part.desc), Style::default().fg(DIM)),
        ]));
    }
    lines.push(Line::from(""));

    // Options
    if !lesson.options.is_empty() {
        lines.push(Line::from(Span::styled(
            "\u{5e38}\u{7528}\u{9009}\u{9879}:",
            Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
        )));
        for opt in &lesson.options {
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", opt.flag), Style::default().fg(ACCENT)),
                Span::styled(opt.name.clone(), Style::default().fg(Color::White)),
            ]));
            if let Some(example) = &opt.example {
                lines.push(Line::from(Span::styled(
                    format!("    \u{4f8b}: {}", example),
                    Style::default().fg(DIM),
                )));
            }
        }
        lines.push(Line::from(""));
    }

    // Gotchas
    if !lesson.gotchas.is_empty() {
        lines.push(Line::from(Span::styled(
            "\u{26a0}\u{fe0f}  \u{6ce8}\u{610f}\u{4e8b}\u{9879}:",
            Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
        )));
        for gotcha in &lesson.gotchas {
            lines.push(Line::from(Span::styled(
                format!("  \u{2022} {}", gotcha.title),
                Style::default().fg(WARNING),
            )));
            lines.push(Line::from(Span::styled(
                format!("    {}", gotcha.content),
                Style::default().fg(DIM),
            )));
        }
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, content_area);

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{4e0a}\u{4e0b}\u{547d}\u{4ee4}"),
        ("Enter/\u{2192}", "\u{67e5}\u{770b}\u{793a}\u{4f8b}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[2]);
}

/// Render example phase: command + tokens + simulated output
pub fn render_example(
    frame: &mut Frame,
    app: &App,
    category_index: usize,
    command_index: usize,
    example_index: usize,
) {
    let area = frame.area();
    let categories = app.get_lesson_categories();
    let cat = match categories.get(category_index) {
        Some(c) => *c,
        None => return,
    };
    let lessons = app.get_lessons_for_category(cat);
    let lesson = match lessons.get(command_index) {
        Some(l) => l,
        None => return,
    };
    let example = match lesson.examples.get(example_index) {
        Some(e) => e,
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

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        format!(
            " {} \u{2014} \u{793a}\u{4f8b} {}/{} ",
            lesson.meta.command,
            example_index + 1,
            lesson.examples.len()
        ),
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    frame.render_widget(title, chunks[0]);

    // Content
    let mut lines: Vec<Line> = Vec::new();

    // Summary
    lines.push(Line::from(Span::styled(
        example.summary.clone(),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Command
    lines.push(Line::from(vec![
        Span::styled("$ ", Style::default().fg(SIMULATED_PROMPT)),
        Span::styled(
            example.display.as_deref().unwrap_or(&example.command).to_string(),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // Token descriptions from command
    // Find matching command tokens
    let matching_cmd = app.commands.iter().find(|c| c.command == example.command);
    if let Some(cmd) = matching_cmd
        && !cmd.tokens.is_empty() {
            lines.push(Line::from(Span::styled(
                "\u{8bcd}\u{5143}\u{89e3}\u{6790}:",
                Style::default().fg(HEADER),
            )));
            for token in &cmd.tokens {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", token.text), Style::default().fg(ACCENT)),
                    Span::styled(format!("\u{2192} {}", token.desc), Style::default().fg(TOKEN_DESC)),
                ]));
            }
            lines.push(Line::from(""));
        }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    // Simulated output box (if any)
    if example.simulated_output.is_some() {
        let sim_area = Rect {
            x: chunks[1].x,
            y: chunks[1].y + chunks[1].height.saturating_sub(8),
            width: chunks[1].width,
            height: 8.min(chunks[1].height),
        };
        let sim = render_simulated_output(&example.command, example.simulated_output.as_deref());
        frame.render_widget(sim, sim_area);
    }

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{4e0a}\u{4e0b}\u{793a}\u{4f8b}"),
        ("Enter/\u{2192}", "\u{8ddf}\u{6253}\u{7ec3}\u{4e60}"),
        ("Esc/\u{2190}", "\u{8fd4}\u{56de}\u{6982}\u{89c8}"),
    ]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[2]);
}

/// Render practice phase: typing engine + simulated output after completion
pub fn render_practice(
    frame: &mut Frame,
    app: &App,
    category_index: usize,
    command_index: usize,
    example_index: usize,
) {
    let area = frame.area();
    let categories = app.get_lesson_categories();
    let cat = match categories.get(category_index) {
        Some(c) => *c,
        None => return,
    };
    let lessons = app.get_lessons_for_category(cat);
    let lesson = match lessons.get(command_index) {
        Some(l) => l,
        None => return,
    };
    let example = match lesson.examples.get(example_index) {
        Some(e) => e,
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

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        format!(
            " {} \u{2014} \u{8ddf}\u{6253}\u{7ec3}\u{4e60} ",
            lesson.meta.command
        ),
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    frame.render_widget(title, chunks[0]);

    // Typing area
    let content_area = chunks[1];
    let engine = &app.typing_engine;
    let is_flashing = engine.is_error_flashing();

    let mut spans = Vec::new();
    spans.push(Span::styled(
        "$ ",
        Style::default().fg(PROMPT_COLOR),
    ));

    for (i, ch) in engine.target.iter().enumerate() {
        let style = if i < engine.cursor {
            Style::default().fg(TYPED_CORRECT)
        } else if i == engine.cursor {
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

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        example.summary.clone(),
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(spans));

    // Show stats if completed
    if engine.is_complete() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(
                "\u{2705} \u{5b8c}\u{6210}\u{ff01}WPM: {:.0}  \u{51c6}\u{786e}\u{7387}: {:.0}%",
                engine.current_wpm(),
                engine.current_accuracy() * 100.0
            ),
            Style::default().fg(SUCCESS),
        )));

        // Show simulated output
        if let Some(output) = &example.simulated_output {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "\u{6a21}\u{62df}\u{8f93}\u{51fa}:",
                Style::default().fg(HEADER),
            )));
            for ol in output.lines() {
                lines.push(Line::from(Span::styled(
                    ol.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, content_area);

    // Hints
    let hints = if engine.is_complete() {
        hint_line(&[
            ("Enter", "\u{4e0b}\u{4e00}\u{4e2a}"),
            ("Ctrl+R", "\u{91cd}\u{7ec3}"),
            ("Esc", "\u{8fd4}\u{56de}"),
        ])
    } else {
        hint_line(&[
            ("Ctrl+R", "\u{91cd}\u{7ec3}"),
            ("Esc", "\u{8fd4}\u{56de}"),
        ])
    };
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[2]);
}
