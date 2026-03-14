use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::core::engine::TypingEngine;
use crate::data::models::{Command, LineStatus, TypingDisplayMode};
use crate::ui::widgets::*;

const DETAILED_MIN_WIDTH: u16 = 100;
const DETAILED_WIDTH_HINT: &str = "终端太窄，详解模式需要 100+ 列";

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (mode, detailed_width_fallback) = effective_typing_mode(app, area.width);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(0),    // terminal area
            Constraint::Length(1), // bottom bar
        ])
        .split(area);

    render_top_bar(frame, mode.clone(), chunks[0]);
    render_terminal_area(frame, app, chunks[1], mode.clone());
    render_bottom_bar(frame, app, chunks[2], mode, detailed_width_fallback);
}

fn effective_typing_mode(app: &App, terminal_width: u16) -> (TypingDisplayMode, bool) {
    if app.typing_mode == TypingDisplayMode::Detailed && terminal_width < DETAILED_MIN_WIDTH {
        return (TypingDisplayMode::Standard, true);
    }
    (app.typing_mode.clone(), false)
}

fn render_top_bar(frame: &mut Frame, mode: TypingDisplayMode, area: Rect) {
    let indicator = match mode {
        TypingDisplayMode::Terminal => "[终端]",
        TypingDisplayMode::Standard => "[标准]",
        TypingDisplayMode::Detailed => "[详解]",
    };

    let bar = Paragraph::new(Line::from(Span::styled(
        indicator,
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(bar, area);
}

fn render_terminal_area(frame: &mut Frame, app: &App, area: Rect, mode: TypingDisplayMode) {
    if app.typing_is_finished() {
        render_round_summary(frame, app, area, mode);
        return;
    }

    if app.typing_showing_output
        && app.typing_engine.is_complete()
        && let Some(cmd) = app.current_typing_command()
        && cmd
            .simulated_output
            .as_deref()
            .map(|text| !text.trim().is_empty())
            .unwrap_or(false)
    {
        render_output_display(frame, app, area, cmd, mode);
        return;
    }

    if mode == TypingDisplayMode::Detailed {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);
        render_active_typing(frame, app, chunks[0]);
        render_token_panel(frame, app, chunks[1]);
        return;
    }

    render_active_typing(frame, app, area);
}

fn render_active_typing(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = history_lines(app, area.height);

    if let Some(cmd) = app.current_typing_command() {
        let prompt = app.format_prompt();
        lines.extend(render_current_command_lines(
            &prompt,
            cmd.display_text(),
            &app.typing_engine,
        ));
    }

    fit_lines_to_height(&mut lines, area.height as usize);
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_token_panel(frame: &mut Frame, app: &App, area: Rect) {
    let Some(cmd) = app.current_typing_command() else {
        return;
    };

    let token_details = current_token_details(app, cmd);
    let active_idx = active_token_index(cmd.command.as_str(), &token_details, app.typing_engine.cursor);

    let mut lines: Vec<Line> = Vec::new();
    if token_details.is_empty() {
        lines.push(Line::from(Span::styled(
            "暂无词元说明",
            Style::default().fg(DIM),
        )));
    } else {
        for (idx, (token, desc)) in token_details.iter().enumerate() {
            let selected = active_idx == Some(idx);
            let token_style = if selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(ACCENT)
            };
            let desc_style = if selected {
                Style::default().fg(Color::White).bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(TOKEN_DESC)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{} ", token), token_style),
                Span::styled("→ ", Style::default().fg(DIM)),
                Span::styled(desc.clone(), desc_style),
            ]));
        }
    }

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" 词元详解 ")
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(DIM)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(panel, area);
}

fn current_token_details(app: &App, cmd: &Command) -> Vec<(String, String)> {
    for lesson in &app.lessons {
        for example in &lesson.examples {
            if example.command == cmd.command && !example.token_details.is_empty() {
                return example
                    .token_details
                    .iter()
                    .map(|detail| (detail.token.clone(), detail.explanation.clone()))
                    .collect();
            }
        }
    }

    cmd.tokens
        .iter()
        .map(|token| (token.text.clone(), token.desc.clone()))
        .collect()
}

fn active_token_index(command: &str, token_details: &[(String, String)], cursor: usize) -> Option<usize> {
    if token_details.is_empty() {
        return None;
    }

    let ranges = token_ranges(command, token_details);
    if ranges.is_empty() {
        return Some(0);
    }

    for (idx, (start, end)) in ranges.iter().enumerate() {
        if cursor <= *end {
            if cursor < *start {
                return Some(idx.saturating_sub(1));
            }
            return Some(idx);
        }
    }

    Some(ranges.len().saturating_sub(1))
}

fn token_ranges(command: &str, token_details: &[(String, String)]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut search_from_byte = 0usize;

    for (token, _) in token_details {
        if token.is_empty() {
            continue;
        }

        if let Some(relative_start) = command[search_from_byte..].find(token) {
            let start_byte = search_from_byte + relative_start;
            let end_byte = start_byte + token.len();
            let start_char = command[..start_byte].chars().count();
            let end_char = command[..end_byte].chars().count();
            ranges.push((start_char, end_char));
            search_from_byte = end_byte;
        }
    }

    ranges
}

fn render_output_display(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    cmd: &Command,
    mode: TypingDisplayMode,
) {
    let output_text = cmd.simulated_output.as_deref();
    let output_body_lines = output_text
        .map(|text| text.lines().count())
        .unwrap_or(0)
        .max(1) as u16;
    let output_height = (output_body_lines + 2).max(3);

    if mode == TypingDisplayMode::Terminal {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(output_height)])
            .split(area);

        let mut top_lines: Vec<Line> = history_lines(app, chunks[0].height);
        let prompt = app.format_prompt();
        top_lines.extend(render_completed_command_lines(&prompt, cmd.display_text()));
        fit_lines_to_height(&mut top_lines, chunks[0].height as usize);
        frame.render_widget(
            Paragraph::new(top_lines).wrap(Wrap { trim: false }),
            chunks[0],
        );

        frame.render_widget(
            render_simulated_output(&cmd.command, output_text),
            chunks[1],
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(output_height),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let mut top_lines: Vec<Line> = history_lines(app, chunks[0].height);
    let prompt = app.format_prompt();
    top_lines.extend(render_completed_command_lines(&prompt, cmd.display_text()));
    fit_lines_to_height(&mut top_lines, chunks[0].height as usize);
    frame.render_widget(
        Paragraph::new(top_lines).wrap(Wrap { trim: false }),
        chunks[0],
    );

    frame.render_widget(
        render_simulated_output(&cmd.command, output_text),
        chunks[1],
    );

    let stats = Line::from(vec![
        Span::styled(
            format!("WPM: {:.0}", app.typing_engine.current_wpm()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("准确: {:.0}%", app.typing_engine.current_accuracy() * 100.0),
            Style::default().fg(SUCCESS),
        ),
    ]);
    frame.render_widget(Paragraph::new(stats), chunks[2]);

    let hint = Line::from(Span::styled("Enter → 下一条", Style::default().fg(ACCENT)));
    frame.render_widget(Paragraph::new(hint), chunks[3]);
}

fn render_round_summary(frame: &mut Frame, app: &App, area: Rect, mode: TypingDisplayMode) {
    if mode == TypingDisplayMode::Terminal {
        let para = Paragraph::new(Line::from(Span::styled(
            "🎉 完成！",
            Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
        frame.render_widget(para, area);
        return;
    }

    let completed = app.typing_round_records.len() as f64;
    let avg_wpm = if completed > 0.0 {
        app.typing_round_records.iter().map(|r| r.wpm).sum::<f64>() / completed
    } else {
        0.0
    };
    let avg_acc = if completed > 0.0 {
        app.typing_round_records
            .iter()
            .map(|r| r.accuracy)
            .sum::<f64>()
            / completed
    } else {
        0.0
    };

    let summary = format!(
        "🎉 本轮完成！{} 条 | 平均 WPM {:.0} | 平均准确率 {:.1}% | Enter/Esc 返回主页",
        app.typing_round_records.len(),
        avg_wpm,
        avg_acc * 100.0
    );

    let para = Paragraph::new(Line::from(Span::styled(summary, Style::default().fg(SUCCESS))))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn history_lines(app: &App, visible_height: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for tl in app.terminal_history.visible_lines(visible_height) {
        if tl.status == LineStatus::Completed {
            lines.push(render_completed_line(&tl.prompt, &tl.command_display));
        }
    }
    lines
}

fn render_completed_line(prompt: &str, command: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(prompt.to_string(), Style::default().fg(PROMPT_COLOR)),
        Span::styled(command.to_string(), Style::default().fg(COMPLETED)),
    ])
}

fn render_completed_command_lines(prompt: &str, display: &str) -> Vec<Line<'static>> {
    let mapped = map_display_lines(display, 0);
    mapped
        .iter()
        .enumerate()
        .map(|(idx, chars)| {
            let line_prompt = if idx == 0 { prompt } else { "> " };
            let mut spans = vec![Span::styled(
                line_prompt.to_string(),
                Style::default().fg(PROMPT_COLOR),
            )];
            for (ch, _) in chars {
                spans.push(Span::styled(ch.to_string(), Style::default().fg(COMPLETED)));
            }
            Line::from(spans)
        })
        .collect()
}

fn render_current_command_lines(
    prompt: &str,
    display: &str,
    engine: &TypingEngine,
) -> Vec<Line<'static>> {
    let mapped_lines = map_display_lines(display, engine.target.len());
    let is_flashing = engine.is_error_flashing();

    mapped_lines
        .iter()
        .enumerate()
        .map(|(line_idx, chars)| {
            let line_prompt = if line_idx == 0 { prompt } else { "> " };
            let mut spans = Vec::new();
            spans.push(Span::styled(
                line_prompt.to_string(),
                Style::default().fg(PROMPT_COLOR),
            ));

            for (ch, target_idx) in chars {
                let style = match target_idx {
                    Some(i) if *i < engine.cursor => Style::default().fg(TYPED_CORRECT),
                    Some(i) if *i == engine.cursor && !engine.is_complete() => {
                        if is_flashing {
                            Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
                        } else {
                            Style::default().fg(CURSOR).bg(CURSOR_BG)
                        }
                    }
                    Some(_) => Style::default().fg(PENDING).bg(PENDING_BG),
                    None => Style::default().fg(PENDING),
                };
                spans.push(Span::styled(ch.to_string(), style));
            }

            Line::from(spans)
        })
        .collect()
}

fn map_display_lines(display: &str, target_len: usize) -> Vec<Vec<(char, Option<usize>)>> {
    let mut lines: Vec<Vec<(char, Option<usize>)>> = Vec::new();
    let mut current_line: Vec<(char, Option<usize>)> = Vec::new();
    let mut target_idx = 0usize;

    for ch in display.chars() {
        if ch == '\n' {
            lines.push(current_line);
            current_line = Vec::new();
            continue;
        }

        let mapped = if target_idx < target_len {
            let idx = target_idx;
            target_idx += 1;
            Some(idx)
        } else {
            None
        };
        current_line.push((ch, mapped));
    }

    lines.push(current_line);
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
}

fn fit_lines_to_height(lines: &mut Vec<Line<'static>>, height: usize) {
    while lines.len() < height {
        lines.push(Line::from(""));
    }

    if lines.len() > height {
        let start = lines.len() - height;
        *lines = lines[start..].to_vec();
    }
}

fn render_bottom_bar(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    mode: TypingDisplayMode,
    detailed_width_fallback: bool,
) {
    if app.typing_is_finished() {
        let text = if mode == TypingDisplayMode::Terminal {
            "🎉 完成！Enter/Esc 返回主页"
        } else {
            "Enter/Esc 返回主页"
        };
        let bar = Paragraph::new(Line::from(Span::styled(text, Style::default().fg(ACCENT))))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(DIM)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(bar, area);
        return;
    }

    if app.typing_showing_output {
        let text = if mode == TypingDisplayMode::Terminal {
            "自动继续..."
        } else {
            "Enter → 下一条"
        };
        let bar = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(DIM)),
        )
        .alignment(Alignment::Center);
        frame.render_widget(bar, area);
        return;
    }

    if detailed_width_fallback {
        let bar = Paragraph::new(Line::from(vec![
            Span::styled(DETAILED_WIDTH_HINT, Style::default().fg(WARNING)),
            Span::styled("  ", Style::default()),
            Span::styled("M 切换模式", Style::default().fg(ACCENT)),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(DIM)),
        );
        frame.render_widget(bar, area);
        return;
    }

    if mode == TypingDisplayMode::Terminal {
        let bar = Paragraph::new(Line::from(vec![
            Span::styled("M 切换模式", Style::default().fg(ACCENT)),
            Span::styled("  ", Style::default()),
            Span::styled("Esc 返回", Style::default().fg(DIM)),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(DIM)),
        );
        frame.render_widget(bar, area);
        return;
    }

    let mut spans = Vec::new();

    // Summary hint
    if let Some(cmd) = app.current_typing_command() {
        if app.show_hint {
            spans.push(Span::styled(
                cmd.short_summary().to_string(),
                Style::default().fg(DIM),
            ));
        }
        spans.push(Span::styled("  ", Style::default()));
    }

    spans.push(Span::styled("[H]", Style::default().fg(ACCENT)));
    spans.push(Span::styled("  ", Style::default()));
    spans.push(Span::styled("M 切换模式", Style::default().fg(ACCENT)));
    spans.push(Span::styled("  ", Style::default()));

    let wpm = app.typing_engine.current_wpm();
    spans.push(Span::styled(
        format!("WPM: {:.0}", wpm),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("  ", Style::default()));

    let acc = app.typing_engine.current_accuracy() * 100.0;
    let acc_color = if acc >= 95.0 {
        SUCCESS
    } else if acc >= 80.0 {
        WARNING
    } else {
        ERROR
    };
    spans.push(Span::styled(
        format!("准确: {:.0}%", acc),
        Style::default().fg(acc_color),
    ));

    let bar = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(bar, area);
}
