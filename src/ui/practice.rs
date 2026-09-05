use crate::{app::App, flow::practice_flow::current_command, ui::widgets::*};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_topics(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new(format!(
            "基础用法练习 · {} 组 · {} 道题",
            app.practice_state.filtered.len(),
            app.practice_state
                .filtered
                .iter()
                .flat_map(|i| app.practice_groups[*i].exercise_command_ids.iter())
                .collect::<std::collections::HashSet<_>>()
                .len()
        ))
        .block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );
    let lines: Vec<_> = app
        .practice_state
        .filtered
        .iter()
        .enumerate()
        .map(|(i, index)| {
            let g = &app.practice_groups[*index];
            let completed = g
                .exercise_command_ids
                .iter()
                .filter(|id| {
                    app.user_stats
                        .command_progress
                        .iter()
                        .any(|p| &p.command_id == *id && p.times_practiced > 0)
                })
                .count();
            Line::styled(
                format!(
                    "{} {} · {}  [{}/3]",
                    if i == app.practice_state.selected {
                        "▶"
                    } else {
                        " "
                    },
                    g.source_id,
                    g.title,
                    completed
                ),
                Style::default().fg(if i == app.practice_state.selected {
                    ACCENT
                } else {
                    MENU_NORMAL
                }),
            )
        })
        .collect();
    let window = visible_menu_window(
        app.practice_state.selected,
        lines.len(),
        1,
        chunks[1].height,
    );
    frame.render_widget(
        Paragraph::new(lines).scroll((window.start as u16, 0)),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new("↑↓ / PgUp PgDn 选择 · Enter 教学 + 三题 · Esc 返回"),
        chunks[2],
    );
}
pub fn render_group(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(frame.area());
    let title = if app.practice_state.step == 0 {
        "教学示例".to_owned()
    } else {
        format!("简单练习 {}/3", app.practice_state.step.min(3))
    };
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );
    let mut lines = Vec::new();
    if app.practice_state.done {
        lines.push(Line::styled(
            "本组练习完成！Enter 返回用法列表",
            Style::default().fg(SUCCESS),
        ));
    } else if let Some(command) = current_command(app) {
        lines.push(Line::from(command.short_summary().to_owned()));
        lines.push(Line::from(""));
        lines.extend(crate::ui::widgets::typing_lines("$ ", &app.typing_engine));
        if app.practice_state.output {
            lines.push(Line::from(""));
            lines.extend(
                command
                    .simulated_output
                    .as_deref()
                    .unwrap_or("命令完成（模拟）")
                    .lines()
                    .map(|s| Line::styled(s.to_owned(), Style::default().fg(DIM))),
            );
            lines.push(Line::from(""));
            lines.extend(
                command
                    .tokens
                    .iter()
                    .map(|t| Line::from(format!("{}：{}", t.text, t.desc))),
            );
        }
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[1]);
    frame.render_widget(
        Paragraph::new("逐行 Enter 提交 · Ctrl+R 重练 · Esc 返回"),
        chunks[2],
    );
}
