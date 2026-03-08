use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::data::models::PromptStyle;
use crate::ui::widgets::*;

struct SettingItem {
    label: &'static str,
    value: String,
}

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
        " \u{8bbe}\u{7f6e} ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    frame.render_widget(title, chunks[0]);

    let config = &app.user_config;
    let items = [
        SettingItem {
            label: "\u{63d0}\u{793a}\u{7b26}\u{98ce}\u{683c}",
            value: match config.prompt_style {
                PromptStyle::Full => "\u{5b8c}\u{6574} (user@host:~$)".to_string(),
                PromptStyle::Simple => "\u{7b80}\u{5355} ($)".to_string(),
                PromptStyle::Minimal => "\u{6700}\u{7b80} (>)".to_string(),
            },
        },
        SettingItem {
            label: "\u{76ee}\u{6807} WPM",
            value: format!("{:.0}", config.target_wpm),
        },
        SettingItem {
            label: "\u{9519}\u{8bef}\u{95ea}\u{70c1}\u{65f6}\u{95f4} (ms)",
            value: format!("{}", config.error_flash_ms),
        },
        SettingItem {
            label: "\u{663e}\u{793a}\u{8bcd}\u{5143}\u{63d0}\u{793a}",
            value: if config.show_token_hints {
                "\u{5f00}\u{542f}".to_string()
            } else {
                "\u{5173}\u{95ed}".to_string()
            },
        },
        SettingItem {
            label: "\u{81ea}\u{9002}\u{5e94}\u{63a8}\u{8350}",
            value: if config.adaptive_recommend {
                "\u{5f00}\u{542f}".to_string()
            } else {
                "\u{5173}\u{95ed}".to_string()
            },
        },
        SettingItem {
            label: "\u{663e}\u{793a}\u{8def}\u{5f84}",
            value: if config.show_path {
                "\u{5f00}\u{542f}".to_string()
            } else {
                "\u{5173}\u{95ed}".to_string()
            },
        },
        SettingItem {
            label: "\u{7528}\u{6237}\u{540d}",
            value: config.prompt_username.clone(),
        },
        SettingItem {
            label: "\u{4e3b}\u{673a}\u{540d}",
            value: config.prompt_hostname.clone(),
        },
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, item) in items.iter().enumerate() {
        let is_selected = i == app.settings_index;
        let prefix = if is_selected { " \u{25b6} " } else { "   " };
        let style = if is_selected {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD).bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(MENU_NORMAL)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::White).bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(DIM)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix.to_string(), style),
            Span::styled(format!("{:<20}", item.label), style),
            Span::styled(format!("  {}", item.value), val_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  \u{4fee}\u{6539}\u{540e}\u{81ea}\u{52a8}\u{4fdd}\u{5b58}",
        Style::default().fg(DIM),
    )));

    // Preview prompt
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  \u{9884}\u{89c8}:",
        Style::default().fg(HEADER),
    )));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(app.format_prompt(), Style::default().fg(PROMPT_COLOR)),
        Span::styled(
            "ls -la /var/log",
            Style::default().fg(Color::White),
        ),
    ]));

    let content = Paragraph::new(lines);
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[
        ("\u{2191}\u{2193}", "\u{79fb}\u{52a8}"),
        ("\u{2190}\u{2192}/Enter", "\u{8c03}\u{6574}"),
        ("Esc", "\u{4fdd}\u{5b58}\u{8fd4}\u{56de}"),
    ]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[2]);
}
