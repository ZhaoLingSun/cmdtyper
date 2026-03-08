use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, AppState, MenuItem};
use crate::ui::widgets::colors;

const TITLE_ART: &str = r#"
                     _ _
  ___ _ __ ___   __| | |_ _   _ _ __   ___ _ __
 / __| '_ ` _ \ / _` | __| | | | '_ \ / _ \ '__|
| (__| | | | | | (_| | |_| |_| | |_) |  __/ |
 \___|_| |_| |_|\__,_|\__|\__, | .__/ \___|_|
                           |___/|_|
"#;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Build content lines
    let mut lines: Vec<Line> = Vec::new();

    // ASCII art title
    for l in TITLE_ART.lines() {
        lines.push(Line::from(Span::styled(
            l.to_string(),
            Style::default().fg(colors::ACCENT),
        )));
    }

    lines.push(Line::from(Span::styled(
        "  Linux 命令行打字练习  v0.1.0",
        Style::default()
            .fg(colors::HEADER)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Menu items
    for (i, item) in MenuItem::ALL.iter().enumerate() {
        let is_selected = i == app.menu_index;

        let prefix = if is_selected { " ▸ " } else { "   " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(colors::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let label = format!("{}{}", prefix, item.label());
        let padded = format!("{:<36}", label);
        lines.push(Line::from(Span::styled(padded, style)));

        // Description under each item
        let desc_style = if is_selected {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(
            format!("     {}", item.desc()),
            desc_style,
        )));
        lines.push(Line::from(""));
    }

    // Difficulty selector
    lines.push(Line::from(""));
    let diff = app.selected_difficulty;
    let diff_line = Line::from(vec![
        Span::styled("  难度: ", Style::default().fg(Color::DarkGray)),
        Span::styled("◀ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} {}", diff.label(), diff.stars()),
            Style::default()
                .fg(colors::HEADER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ▶", Style::default().fg(Color::DarkGray)),
    ]);
    lines.push(diff_line);

    // Stats summary
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  累计练习: {} 次  |  最佳 WPM: {:.0}",
            app.user_stats.total_sessions, app.user_stats.best_wpm
        ),
        Style::default().fg(Color::DarkGray),
    )));

    let content = Paragraph::new(lines).alignment(Alignment::Left);

    // Center the content block
    let v_chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(30),
        Constraint::Fill(1),
    ])
    .split(area);

    let h_chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(50),
        Constraint::Fill(1),
    ])
    .split(v_chunks[1]);

    f.render_widget(content, h_chunks[1]);

    // Bottom help bar
    let help = Line::from(vec![
        Span::styled(
            " ↑↓",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 选择  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "◀▶",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 难度  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 确认  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "q/Esc",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 退出", Style::default().fg(Color::DarkGray)),
    ]);
    let help_bar = Paragraph::new(help).alignment(Alignment::Center);
    let bottom = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
    f.render_widget(help_bar, bottom[1]);
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<AppState> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(AppState::Quitting),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppState::Quitting)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.menu_index > 0 {
                app.menu_index -= 1;
            } else {
                app.menu_index = MenuItem::ALL.len() - 1;
            }
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.menu_index = (app.menu_index + 1) % MenuItem::ALL.len();
            None
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.selected_difficulty = app.selected_difficulty.prev();
            None
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.selected_difficulty = app.selected_difficulty.next();
            None
        }
        KeyCode::Enter => match app.current_menu_item() {
            MenuItem::Learn => app.enter_learn_mode(),
            MenuItem::Type => app.enter_typing_mode(),
            MenuItem::Dictation => app.enter_dictation_mode(),
            MenuItem::Stats => Some(app.enter_stats_mode()),
        },
        _ => None,
    }
}
