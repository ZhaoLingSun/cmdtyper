use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::data::models::DeepSource;
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App, source: &DeepSource, scroll: usize) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let (command_name, deep_text) = get_deep_content(app, source);

    let title = Paragraph::new(Line::from(Span::styled(
        format!(" {} 详细解析 ", command_name),
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
    for raw_line in deep_text.lines() {
        if let Some(header) = raw_line.strip_prefix("### ") {
            lines.push(Line::from(Span::styled(
                header.to_string(),
                Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        if let Some(code_line) = raw_line.strip_prefix("    ") {
            lines.push(Line::from(Span::styled(
                format!("  {}", code_line),
                Style::default().fg(DIM).bg(Color::Rgb(35, 35, 35)),
            )));
            continue;
        }

        if let Some(bullet) = raw_line
            .strip_prefix("- ")
            .or_else(|| raw_line.strip_prefix("* "))
        {
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled(bullet.to_string(), Style::default().fg(Color::White)),
            ]));
            continue;
        }

        if let Some(tip) = raw_line.strip_prefix("> ") {
            lines.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(WARNING).add_modifier(Modifier::BOLD)),
                Span::styled(tip.to_string(), Style::default().fg(WARNING)),
            ]));
            continue;
        }

        lines.push(Line::from(Span::styled(
            raw_line.to_string(),
            Style::default().fg(Color::White),
        )));
    }

    let content = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[
        ("Esc", "返回"),
        ("→", "下一条"),
        ("↑↓/j k", "滚动"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn get_deep_content(app: &App, source: &DeepSource) -> (String, String) {
    match source {
        DeepSource::LessonExample {
            category_idx,
            command_idx,
            example_idx,
        } => {
            let cat = match app.get_lesson_categories().get(*category_idx) {
                Some(c) => *c,
                None => return ("未知命令".to_string(), "暂无详细解析".to_string()),
            };
            let lessons = app.get_lessons_for_category(cat);
            let lesson = match lessons.get(*command_idx) {
                Some(l) => l,
                None => return ("未知命令".to_string(), "暂无详细解析".to_string()),
            };
            let example = match lesson.examples.get(*example_idx) {
                Some(e) => e,
                None => return (lesson.meta.command.clone(), "暂无详细解析".to_string()),
            };

            (
                lesson.meta.command.clone(),
                example
                    .deep_explanation
                    .clone()
                    .unwrap_or_else(|| "暂无详细解析".to_string()),
            )
        }
        DeepSource::SymbolExample {
            topic_idx,
            symbol_idx,
            example_idx,
        } => {
            let topic = match app.symbol_topics.get(*topic_idx) {
                Some(t) => t,
                None => return ("未知示例".to_string(), "暂无详细解析".to_string()),
            };
            let symbol = match topic.symbols.get(*symbol_idx) {
                Some(s) => s,
                None => return ("未知示例".to_string(), "暂无详细解析".to_string()),
            };
            let example = match symbol.examples.get(*example_idx) {
                Some(e) => e,
                None => return (symbol.name.clone(), "暂无详细解析".to_string()),
            };

            (
                example.command.clone(),
                example
                    .deep_explanation
                    .clone()
                    .unwrap_or_else(|| "暂无详细解析".to_string()),
            )
        }
        DeepSource::SystemCommand {
            topic_idx,
            section_idx,
            command_idx,
        } => {
            let topic = match app.system_topics.get(*topic_idx) {
                Some(t) => t,
                None => return ("未知命令".to_string(), "暂无详细解析".to_string()),
            };
            let section = match topic.sections.get(*section_idx) {
                Some(s) => s,
                None => return ("未知命令".to_string(), "暂无详细解析".to_string()),
            };
            let command = match section.commands.get(*command_idx) {
                Some(c) => c,
                None => return (section.title.clone(), "暂无详细解析".to_string()),
            };

            (
                command.command.clone(),
                command
                    .deep_explanation
                    .clone()
                    .unwrap_or_else(|| "暂无详细解析".to_string()),
            )
        }
    }
}
