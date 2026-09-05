use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::data::models::lesson_example_progress_key;
use crate::ui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let categories = app.get_lesson_categories();

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
        " \u{547d}\u{4ee4}\u{4e13}\u{9898} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    // Category list
    if categories.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{8bfe}\u{7a0b}\u{6570}\u{636e}\u{ff0c}\u{8bf7}\u{6dfb}\u{52a0} data/lessons/*.toml",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
    } else {
        let selected = app.command_topics_index.min(categories.len() - 1);
        let mut lines: Vec<Line> = Vec::new();

        for (i, cat) in categories.iter().enumerate() {
            let is_selected = i == selected;
            let prefix = if is_selected { " \u{25b6} " } else { "   " };

            let lesson_count = app
                .lessons
                .iter()
                .filter(|l| l.meta.category == *cat)
                .count();
            let practiced = app
                .lessons
                .iter()
                .filter(|lesson| lesson.meta.category == *cat)
                .filter(|lesson| {
                    lesson
                        .examples
                        .iter()
                        .enumerate()
                        .any(|(example_index, _)| {
                            let progress_key = lesson_example_progress_key(lesson, example_index);
                            app.user_stats.command_progress.iter().any(|progress| {
                                progress.command_id == progress_key && progress.times_practiced > 0
                            })
                        })
                })
                .count();

            let exercise_count: usize = app
                .lessons
                .iter()
                .filter(|l| l.meta.category == *cat)
                .map(|l| app.practice_counts("lesson", &l.meta.command).1)
                .sum();
            let style = if is_selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(MENU_NORMAL)
            };

            let difficulty = app
                .lessons
                .iter()
                .find(|l| l.meta.category == *cat)
                .map(|l| l.meta.difficulty)
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("{} ", cat.icon()), style),
                Span::styled(cat.label().to_string(), style),
                Span::styled(
                    format!("  {} ", difficulty.stars()),
                    Style::default().fg(WARNING),
                ),
                Span::styled(
                    format!("  {}/{} · {}题", practiced, lesson_count, exercise_count),
                    Style::default().fg(DIM),
                ),
            ]));
        }

        let window = visible_menu_window(selected, categories.len(), 1, chunks[1].height);
        let list = Paragraph::new(lines).scroll((window.start as u16, 0));
        frame.render_widget(list, chunks[1]);
    }

    // Hints
    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{79fb}\u{52a8}"),
        ("Enter", "\u{8fdb}\u{5165}"),
        ("Esc", "\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
