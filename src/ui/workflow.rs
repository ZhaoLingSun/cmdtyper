use crate::{
    app::{App, AppState},
    data::models::TypingDisplayMode,
    flow::workflow_flow::active_sequence,
    ui::widgets::*,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Render the current referenced step and its own simulated context/output.
pub fn render(frame: &mut Frame, app: &App) {
    let Some(sequence) = active_sequence(app) else {
        return;
    };
    let area = frame.area();
    let main_typing = app.state == AppState::Typing;
    let terminal_mode = main_typing && app.typing_mode == TypingDisplayMode::Terminal;
    let detailed =
        main_typing && app.typing_mode == TypingDisplayMode::Detailed && area.width >= 100;
    let show_teaching = !main_typing || (!terminal_mode && app.show_hint);
    let mode_label = if !main_typing {
        "分步练习"
    } else if terminal_mode {
        "终端"
    } else if detailed {
        "详解"
    } else {
        "标准"
    };

    let output = app
        .workflow_state
        .output_for(app.typing_engine.session_id());
    let current = app
        .typing_engine
        .target
        .iter()
        .take(app.typing_engine.cursor)
        .filter(|character| **character == '\n')
        .count()
        .min(sequence.steps.len() - 1);
    let index = output.map(|output| output.step_index).unwrap_or(current);
    let step = &sequence.steps[index];
    let title = app
        .commands
        .iter()
        .find(|command| command.id == sequence.id)
        .map(|command| command.short_summary())
        .unwrap_or("分步命令练习");
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new(format!(
            "[{mode_label}] 步骤 {}/{} · {title}",
            index + 1,
            sequence.steps.len()
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)),
        ),
        chunks[0],
    );
    let mut lines = Vec::new();
    let prompt = step.prompt.clone().unwrap_or_else(|| app.format_prompt());
    if output.is_some() {
        lines.push(Line::from(vec![
            Span::styled(prompt, Style::default().fg(PROMPT_COLOR)),
            Span::styled(step.command.clone(), Style::default().fg(COMPLETED)),
        ]));
        lines.push(Line::from(""));
        let text = step
            .output
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or("（本步无终端输出）");
        lines.extend(
            text.lines()
                .map(|line| Line::styled(line.to_owned(), Style::default().fg(Color::White))),
        );
        lines.push(Line::from(""));
        if show_teaching {
            lines.push(Line::styled(
                step.explanation.clone(),
                Style::default().fg(ACCENT),
            ));
        }
        if index + 1 < sequence.steps.len() {
            let next = &sequence.steps[index + 1];
            lines.push(Line::from(""));
            lines.push(Line::styled("下一条：", Style::default().fg(DIM)));
            lines.push(Line::styled(
                format!(
                    "{}{}",
                    next.prompt.clone().unwrap_or_else(|| app.format_prompt()),
                    next.command
                ),
                Style::default().fg(PENDING),
            ));
        }
    } else {
        if show_teaching {
            lines.push(Line::styled(
                step.explanation.clone(),
                Style::default().fg(DIM),
            ));
            lines.push(Line::from(""));
        }
        // Show recent submitted commands to preserve variable/directory context.
        for previous in sequence
            .steps
            .iter()
            .take(index)
            .skip(index.saturating_sub(3))
        {
            lines.push(Line::styled(
                format!(
                    "{}{}",
                    previous
                        .prompt
                        .clone()
                        .unwrap_or_else(|| app.format_prompt()),
                    previous.command
                ),
                Style::default().fg(COMPLETED),
            ));
        }
        let offset = sequence
            .commands
            .iter()
            .take(index)
            .map(|text| text.chars().count() + 1)
            .sum::<usize>();
        let mut spans = vec![Span::styled(prompt, Style::default().fg(PROMPT_COLOR))];
        for (local, character) in step.command.chars().enumerate() {
            let position = offset + local;
            let style = if position < app.typing_engine.cursor {
                Style::default().fg(TYPED_CORRECT)
            } else if position == app.typing_engine.cursor && app.typing_engine.is_error_flashing()
            {
                Style::default().fg(ERROR_FLASH).bg(ERROR_FLASH_BG)
            } else if position == app.typing_engine.cursor {
                Style::default().fg(CURSOR).bg(CURSOR_BG)
            } else {
                Style::default().fg(PENDING).bg(PENDING_BG)
            };
            spans.push(Span::styled(character.to_string(), style));
        }
        let enter_style = if app.typing_engine.cursor == offset + step.command.chars().count() {
            Style::default().fg(CURSOR).bg(CURSOR_BG)
        } else {
            Style::default().fg(ACCENT)
        };
        spans.push(Span::styled(" [Enter]", enter_style));
        lines.push(Line::from(spans));
    }
    if detailed && app.show_hint {
        if let Some(command) = app
            .commands
            .iter()
            .find(|command| command.id == step.command_id)
        {
            lines.push(Line::from(""));
            lines.push(Line::styled("词元详解", Style::default().fg(HEADER)));
            lines.extend(command.tokens.iter().map(|token| {
                Line::styled(
                    format!("{} → {}", token.text, token.desc),
                    Style::default().fg(ACCENT),
                )
            }));
        }
    }
    if main_typing && !terminal_mode {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!(
                "WPM {:.1}  CPM {:.1}  准确率 {:.1}%",
                app.typing_engine.current_wpm(),
                app.typing_engine.current_cpm(),
                app.typing_engine.current_accuracy() * 100.0
            ),
            Style::default().fg(SUCCESS),
        ));
    }
    let lines = wrap_terminal_lines(lines, chunks[1].width as usize);
    let cursor_row = lines.iter().position(|line| {
        line.spans
            .iter()
            .any(|span| span.style.bg == Some(CURSOR_BG) || span.style.bg == Some(ERROR_FLASH_BG))
    });
    let automatic = cursor_row
        .map(|row| row.saturating_sub(chunks[1].height.saturating_sub(1) as usize))
        .unwrap_or(0);
    let scroll = if app.workflow_state.manual_scroll {
        app.workflow_state.scroll
    } else {
        automatic
    }
    .min(lines.len().saturating_sub(chunks[1].height as usize));
    frame.render_widget(
        Paragraph::new(lines).scroll((scroll.min(u16::MAX as usize) as u16, 0)),
        chunks[1],
    );
    let hint = if output.is_some_and(|step| step.final_step) {
        "Enter 完成 / 输入下一题 · PgUp/PgDn 阅读 · Ctrl+R 重练 · Esc 返回"
    } else if output.is_some() {
        "输入下一条 / Enter 继续 · PgUp/PgDn 阅读 · Ctrl+R 重练 · Esc 返回"
    } else if main_typing && app.typing_mode == TypingDisplayMode::Detailed && area.width < 100 {
        "详解需100列 · F2 模式 · Enter 提交 · PgUp/PgDn 阅读 · Esc 返回"
    } else if main_typing {
        "Enter 提交 · F2 模式 · F3 提示 · PgUp/PgDn 阅读 · Esc 返回"
    } else {
        "逐条 Enter 提交 · PgUp/PgDn 阅读 · Ctrl+R 重练 · Esc 返回"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(ACCENT)),
        chunks[2],
    );
}

/// Physical terminal wrapping keeps cursor placement deterministic for both
/// ASCII command text and full-width Chinese explanations in short windows.
fn wrap_terminal_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    use unicode_width::UnicodeWidthChar;
    if width == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    for line in lines {
        let mut spans = Vec::new();
        let mut column = 0;
        for span in line.spans {
            for character in span.content.chars() {
                let character_width = character.width().unwrap_or(0);
                if character == '\n' || (column + character_width > width && column > 0) {
                    result.push(Line::from(std::mem::take(&mut spans)).style(line.style));
                    column = 0;
                }
                if character != '\n' {
                    spans.push(Span::styled(character.to_string(), span.style));
                    column += character_width;
                }
            }
        }
        result.push(Line::from(spans).style(line.style));
    }
    result
}
