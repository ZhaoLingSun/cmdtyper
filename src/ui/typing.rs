use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, AppState};
use crate::ui::widgets::{colors, format_time};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // top bar
        Constraint::Fill(1),   // main area
        Constraint::Length(2), // bottom bar
    ])
    .split(area);

    render_top_bar(f, chunks[0], app);
    render_main_area(f, chunks[1], app);
    render_bottom_bar(f, chunks[2], app);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let progress = if app.commands.is_empty() {
        "0/0".to_string()
    } else {
        format!("{}/{}", app.current_command_index + 1, app.commands.len())
    };

    let line = Line::from(vec![
        Span::styled(
            " 对着打 ",
            Style::default()
                .fg(colors::CURSOR)
                .bg(colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("难度: {}", app.selected_difficulty.label()),
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

fn render_main_area(f: &mut Frame, area: Rect, app: &App) {
    if app.commands.is_empty() {
        let empty = Paragraph::new("  当前难度下没有可练习的命令。按 Esc 返回菜单。");
        f.render_widget(empty, area);
        return;
    }

    let cmd = &app.commands[app.current_command_index];
    let engine = match &app.typing_engine {
        Some(e) => e,
        None => return,
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Command text display
    lines.push(Line::from(Span::styled(
        format!("  {}", cmd.command),
        Style::default()
            .fg(colors::HEADER)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Token tree annotations
    let token_count = cmd.tokens.len();
    for (i, token) in cmd.tokens.iter().enumerate() {
        let connector = if i < token_count - 1 {
            "├─"
        } else {
            "└─"
        };
        let tree_line = Line::from(vec![
            Span::styled(
                format!("  {} ", connector),
                Style::default().fg(colors::TREE_LINE),
            ),
            Span::styled(
                format!("{:<12}", token.text),
                Style::default()
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&token.desc, Style::default().fg(colors::TOKEN_DESC)),
        ]);
        lines.push(tree_line);
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Typing input line with 3-state coloring
    let mut input_spans: Vec<Span> = Vec::new();
    input_spans.push(Span::styled("  ", Style::default()));

    let is_flashing = engine.is_error_flashing();

    for (i, &ch) in engine.target.iter().enumerate() {
        let ch_str = ch.to_string();
        if i < engine.cursor {
            // Already typed correctly: white text
            input_spans.push(Span::styled(
                ch_str,
                Style::default().fg(colors::TYPED_CORRECT),
            ));
        } else if i == engine.cursor {
            // Current cursor position
            if is_flashing {
                // Error flash: red background
                input_spans.push(Span::styled(
                    ch_str,
                    Style::default()
                        .fg(colors::ERROR_FLASH)
                        .bg(colors::ERROR_FLASH_BG),
                ));
            } else {
                // Normal cursor: white bg, black text
                input_spans.push(Span::styled(
                    ch_str,
                    Style::default().fg(colors::CURSOR).bg(colors::CURSOR_BG),
                ));
            }
        } else {
            // Pending: gray text
            input_spans.push(Span::styled(
                ch_str,
                Style::default().fg(colors::PENDING).bg(colors::PENDING_BG),
            ));
        }
    }

    // If complete, show all white
    if engine.is_complete() {
        input_spans.clear();
        input_spans.push(Span::styled("  ", Style::default()));
        for &ch in &engine.target {
            input_spans.push(Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(colors::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        input_spans.push(Span::styled(
            "  ✓",
            Style::default()
                .fg(colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        ));
    }

    lines.push(Line::from(input_spans));

    // Completion message
    if engine.is_complete() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  完成！按 Tab 下一题 / Ctrl+R 重练 / Esc 返回",
            Style::default().fg(colors::ACCENT),
        )));
    }

    let content = Paragraph::new(lines);
    f.render_widget(content, area);
}

fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let engine = match &app.typing_engine {
        Some(e) => e,
        None => return,
    };

    let wpm = engine.current_wpm();
    let accuracy = engine.current_accuracy() * 100.0;
    let elapsed = format_time(engine.elapsed_secs());

    let stats_line = Line::from(vec![
        Span::styled(
            format!(" WPM: {:.0}", wpm),
            Style::default()
                .fg(colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            format!("准确率: {:.1}%", accuracy),
            Style::default().fg(colors::HEADER),
        ),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            format!("时间: {}", elapsed),
            Style::default().fg(ratatui::style::Color::White),
        ),
        Span::styled("  │  ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 菜单 ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "Tab",
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 跳过 ", Style::default().fg(colors::PENDING)),
        Span::styled(
            "^R",
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 重练", Style::default().fg(colors::PENDING)),
    ]);

    let bar = Paragraph::new(vec![Line::from(""), stats_line]).alignment(Alignment::Left);
    f.render_widget(bar, area);
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<AppState> {
    match key.code {
        KeyCode::Esc => Some(app.return_home()),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppState::Quitting)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.retry_typing_command();
            None
        }
        KeyCode::Tab => {
            app.next_typing_command();
            None
        }
        KeyCode::Char(ch) => {
            let mut completed = false;
            if let Some(engine) = &mut app.typing_engine {
                engine.input(ch);
                completed = engine.is_complete();
            }
            if completed {
                return app.complete_typing_round();
            }
            None
        }
        _ => None,
    }
}
