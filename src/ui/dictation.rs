use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, AppState};
use crate::core::matcher::{DiffKind, MatchResult};
use crate::ui::widgets::colors;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(7),
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_top_bar(f, chunks[0], app);

    let Some(command) = app.current_dictation_command() else {
        let empty = Paragraph::new("  当前筛选条件下没有可默写的命令。按 Esc 返回菜单。")
            .alignment(Alignment::Left);
        f.render_widget(empty, chunks[1]);
        return;
    };

    render_prompt_block(f, chunks[1], &command.dictation.prompt);
    render_input_block(f, chunks[2], app);
    render_result_block(f, chunks[3], app);
    render_bottom_bar(f, chunks[4], app.dictation_result.is_some());

    set_input_cursor(f, chunks[2], app);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let progress = if app.commands.is_empty() {
        "0/0".to_string()
    } else {
        format!("{}/{}", app.dictation_command_index + 1, app.commands.len())
    };
    let category = match app.current_dictation_command() {
        Some(command) => command.category.label().to_string(),
        None => match app.selected_category {
            Some(category) => category.label().to_string(),
            None => "全部分类".to_string(),
        },
    };

    let line = Line::from(vec![
        Span::styled(
            " 默写模式 ",
            Style::default()
                .fg(colors::CURSOR)
                .bg(colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("难度: {}", app.selected_difficulty.label()),
            Style::default().fg(colors::HEADER),
        ),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            format!("分类: {}", category),
            Style::default().fg(colors::HEADER),
        ),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            format!("进度: {}", progress),
            Style::default().fg(colors::HEADER),
        ),
    ]);

    let bar = Paragraph::new(vec![Line::from(""), line]).alignment(Alignment::Left);
    f.render_widget(bar, area);
}

fn render_prompt_block(f: &mut Frame, area: Rect, prompt: &str) {
    let prompt = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            prompt,
            Style::default()
                .fg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .wrap(Wrap { trim: false })
    .block(Block::default().title(" 默写提示 ").borders(Borders::ALL));

    f.render_widget(prompt, area);
}

fn render_input_block(f: &mut Frame, area: Rect, app: &App) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let (visible_input, _, _) =
        visible_input_window(&app.dictation_input, app.dictation_cursor, inner_width);
    let input_line = if visible_input.is_empty() {
        Line::from(Span::styled(
            "请输入命令...",
            Style::default().fg(colors::PENDING),
        ))
    } else {
        Line::from(Span::styled(
            visible_input,
            Style::default().fg(Color::White),
        ))
    };

    let input = Paragraph::new(input_line)
        .block(Block::default().title(" 输入 ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(input, area);
}

fn render_result_block(f: &mut Frame, area: Rect, app: &App) {
    let Some(command) = app.current_dictation_command() else {
        return;
    };

    let block = Block::default().title(" 结果 ").borders(Borders::ALL);

    let Some(result) = &app.dictation_result else {
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "按 Enter 提交答案，按 Tab 跳到下一题。",
                Style::default().fg(colors::PENDING),
            )),
        ])
        .block(block);
        f.render_widget(hint, area);
        return;
    };

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    match &result.evaluation {
        MatchResult::Exact(index) => {
            lines.push(Line::from(Span::styled(
                "✓ 完全正确",
                Style::default()
                    .fg(colors::ACCENT)
                    .add_modifier(Modifier::BOLD),
            )));
            if let Some(answer) = command.dictation.answers.get(*index) {
                lines.push(Line::from(Span::styled(
                    format!("匹配答案: {}", answer),
                    Style::default().fg(Color::White),
                )));
            }
        }
        MatchResult::Normalized(index) => {
            lines.push(Line::from(Span::styled(
                "✓ 规范化匹配成功",
                Style::default()
                    .fg(colors::ACCENT)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "已忽略大小写和多余空白。",
                Style::default().fg(colors::PENDING),
            )));
            if let Some(answer) = command.dictation.answers.get(*index) {
                lines.push(Line::from(Span::styled(
                    format!("匹配答案: {}", answer),
                    Style::default().fg(Color::White),
                )));
            }
        }
        MatchResult::NoMatch { closest, diff } => {
            lines.push(Line::from(Span::styled(
                "✗ 未匹配到正确答案",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "正确答案:",
                Style::default().fg(colors::HEADER),
            )));
            for answer in &command.dictation.answers {
                lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(colors::PENDING)),
                    Span::styled(answer, Style::default().fg(Color::White)),
                ]));
            }
            if !closest.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("最接近答案: {}", closest),
                    Style::default().fg(colors::HEADER),
                )));
            }
            if !diff.is_empty() {
                lines.push(Line::from(Span::styled(
                    "差异高亮:",
                    Style::default().fg(colors::PENDING),
                )));
                let mut diff_spans = Vec::with_capacity(diff.len() + 1);
                diff_spans.push(Span::raw("  "));
                for segment in diff {
                    diff_spans.push(Span::styled(
                        segment.text.clone(),
                        Style::default().fg(diff_color(segment.kind)),
                    ));
                }
                lines.push(Line::from(diff_spans));
            }
        }
    }

    let result = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(result, area);
}

fn render_bottom_bar(f: &mut Frame, area: Rect, has_result: bool) {
    let enter_label = if has_result {
        " 下一题 "
    } else {
        " 提交 "
    };
    let help = Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(enter_label, Style::default().fg(colors::PENDING)),
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 跳过 ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 菜单", Style::default().fg(colors::PENDING)),
    ]);

    let bar = Paragraph::new(vec![Line::from(""), help]).alignment(Alignment::Center);
    f.render_widget(bar, area);
}

fn set_input_cursor(f: &mut Frame, area: Rect, app: &App) {
    if area.width <= 2 || area.height <= 2 {
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize;
    if inner_width == 0 {
        return;
    }

    let (_, _, visible_cursor) =
        visible_input_window(&app.dictation_input, app.dictation_cursor, inner_width);
    let cursor_prefix = app
        .dictation_input
        .chars()
        .skip(app.dictation_cursor.saturating_sub(visible_cursor))
        .take(visible_cursor)
        .collect::<String>();
    let cursor_offset = UnicodeWidthStr::width(cursor_prefix.as_str()) as u16;
    let max_x = area.right().saturating_sub(2);
    let x = (area.x + 1 + cursor_offset).min(max_x);
    let y = area.y + 1;
    f.set_cursor_position(Position::new(x, y));
}

fn visible_input_window(input: &str, cursor: usize, max_chars: usize) -> (String, usize, usize) {
    let char_count = input.chars().count();
    if max_chars == 0 {
        return (String::new(), 0, 0);
    }
    if char_count <= max_chars {
        return (input.to_string(), 0, cursor.min(char_count));
    }

    let mut start = cursor.saturating_sub(max_chars.saturating_sub(1));
    if start + max_chars > char_count {
        start = char_count.saturating_sub(max_chars);
    }

    let visible = input
        .chars()
        .skip(start)
        .take(max_chars)
        .collect::<String>();
    let visible_cursor = cursor.saturating_sub(start).min(max_chars);
    (visible, start, visible_cursor)
}

fn diff_color(kind: DiffKind) -> Color {
    match kind {
        DiffKind::Same => colors::ACCENT,
        DiffKind::Added => Color::Red,
        DiffKind::Removed => Color::Yellow,
    }
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<AppState> {
    match key.code {
        KeyCode::Esc => Some(app.return_home()),
        KeyCode::Enter => {
            if app.dictation_result.is_none() {
                app.submit_dictation();
            } else {
                app.next_dictation_command();
            }
            None
        }
        KeyCode::Tab => {
            app.next_dictation_command();
            None
        }
        KeyCode::Left => {
            app.move_dictation_cursor_left();
            None
        }
        KeyCode::Right => {
            app.move_dictation_cursor_right();
            None
        }
        KeyCode::Home => {
            app.move_dictation_cursor_home();
            None
        }
        KeyCode::End => {
            app.move_dictation_cursor_end();
            None
        }
        KeyCode::Backspace if app.dictation_result.is_none() => {
            app.backspace_dictation();
            None
        }
        KeyCode::Delete if app.dictation_result.is_none() => {
            app.delete_dictation();
            None
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppState::Quitting)
        }
        KeyCode::Char(ch)
            if app.dictation_result.is_none()
                && !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.insert_dictation_char(ch);
            None
        }
        _ => None,
    }
}
