use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::data::models::{Category, Difficulty};
use crate::ui::widgets::*;

const DIFFICULTY_OPTIONS: [&str; 5] = ["全部", "Beginner", "Basic", "Advanced", "Practical"];
const CATEGORY_OPTIONS: [&str; 11] = [
    "全部",
    "FileOps",
    "Permission",
    "TextProcess",
    "Search",
    "Process",
    "Network",
    "Archive",
    "System",
    "Pipeline",
    "Scripting",
];

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        " 选择训练范围 ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    let mut lines = vec![Line::from("")];

    lines.extend(render_option_lines(
        "难度:",
        &DIFFICULTY_OPTIONS,
        app.filter_difficulty.map_or(0, difficulty_to_index),
        app.typing_filter_row == 0,
        &[3, 2],
    ));

    lines.push(Line::from(""));

    lines.extend(render_option_lines(
        "类别:",
        &CATEGORY_OPTIONS,
        app.filter_category.map_or(0, category_to_index),
        app.typing_filter_row == 1,
        &[3, 3, 3, 2],
    ));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" 匹配命令: ", Style::default().fg(DIM)),
        Span::styled(
            app.current_filter_match_count().to_string(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 条", Style::default().fg(DIM)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Enter 开始  |  Esc 返回 ",
        Style::default().fg(DIM),
    )));

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(DIM)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(body, chunks[1]);

    let hints = hint_line(&[
        ("↑↓", "切换行"),
        ("←→", "切换筛选"),
        ("Enter", "开始"),
        ("Esc", "返回"),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}

fn render_option_lines<'a>(
    label: &'a str,
    options: &'a [&'a str],
    selected: usize,
    focused: bool,
    groups: &[usize],
) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut cursor = 0usize;

    for (line_idx, group_size) in groups.iter().enumerate() {
        let mut spans = Vec::new();

        if line_idx == 0 {
            spans.push(Span::styled(
                format!(" {} ", label),
                if focused {
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(DIM)
                },
            ));
        } else {
            spans.push(Span::raw("      "));
        }

        let end = (cursor + *group_size).min(options.len());
        for idx in cursor..end {
            if idx > cursor {
                spans.push(Span::raw(" "));
            }

            let is_selected = idx == selected;
            let style = if is_selected {
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
                    .bg(MENU_SELECTED_BG)
            } else {
                Style::default().fg(MENU_NORMAL)
            };

            let text = if is_selected {
                format!("[{}]", options[idx])
            } else {
                options[idx].to_string()
            };
            spans.push(Span::styled(text, style));
        }

        lines.push(Line::from(spans));
        cursor = end;

        if cursor >= options.len() {
            break;
        }
    }

    lines
}

fn difficulty_to_index(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::Beginner => 1,
        Difficulty::Basic => 2,
        Difficulty::Advanced => 3,
        Difficulty::Practical => 4,
    }
}

fn category_to_index(category: Category) -> usize {
    match category {
        Category::FileOps => 1,
        Category::Permission => 2,
        Category::TextProcess => 3,
        Category::Search => 4,
        Category::Process => 5,
        Category::Network => 6,
        Category::Archive => 7,
        Category::System => 8,
        Category::Pipeline => 9,
        Category::Scripting => 10,
    }
}
