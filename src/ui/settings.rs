use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::data::models::PromptStyle;
use crate::ui::widgets::*;

struct SettingItem {
    label: &'static str,
    value: String,
    selectable: bool,
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
        " 设置 ",
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    let config = &app.user_config;
    let items = [
        SettingItem {
            label: "提示符风格",
            value: match config.prompt_style {
                PromptStyle::Full => "完整 (user@host:~$)".to_string(),
                PromptStyle::Simple => "简单 ($)".to_string(),
                PromptStyle::Minimal => "最简 (>)".to_string(),
            },
            selectable: true,
        },
        SettingItem {
            label: "目标 WPM",
            value: format!("{:.0}", config.target_wpm),
            selectable: true,
        },
        SettingItem {
            label: "错误闪烁时间 (ms)",
            value: format!("{}", config.error_flash_ms),
            selectable: true,
        },
        SettingItem {
            label: "显示词元提示",
            value: if config.show_token_hints {
                "开启".to_string()
            } else {
                "关闭".to_string()
            },
            selectable: true,
        },
        SettingItem {
            label: "自适应推荐",
            value: if config.adaptive_recommend {
                "开启".to_string()
            } else {
                "关闭".to_string()
            },
            selectable: true,
        },
        SettingItem {
            label: "显示路径",
            value: if config.show_path {
                "开启".to_string()
            } else {
                "关闭".to_string()
            },
            selectable: true,
        },
        SettingItem {
            label: "用户名",
            value: config.prompt_username.clone(),
            selectable: false,
        },
        SettingItem {
            label: "主机名",
            value: config.prompt_hostname.clone(),
            selectable: false,
        },
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, item) in items.iter().enumerate() {
        let is_selected = item.selectable && i == app.settings_index;
        let prefix = if is_selected { " ▶ " } else { "   " };
        let style = if is_selected {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD)
                .bg(MENU_SELECTED_BG)
        } else if item.selectable {
            Style::default().fg(MENU_NORMAL)
        } else {
            Style::default().fg(DIM)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::White).bg(MENU_SELECTED_BG)
        } else {
            Style::default().fg(DIM)
        };

        if item.selectable {
            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("{:<20}", item.label), style),
                Span::styled(format!("  {}", item.value), val_style),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default().fg(DIM)),
                Span::styled(
                    format!("{}: {} (只读)", item.label, item.value),
                    Style::default().fg(DIM),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  修改后自动保存",
        Style::default().fg(DIM),
    )));

    // Preview prompt
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  预览:",
        Style::default().fg(HEADER),
    )));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(app.format_prompt(), Style::default().fg(PROMPT_COLOR)),
        Span::styled("ls -la /var/log", Style::default().fg(Color::White)),
    ]));

    let content = Paragraph::new(lines);
    frame.render_widget(content, chunks[1]);

    let hints = hint_line(&[("↑↓", "移动"), ("←→/Enter", "调整"), ("Esc", "保存返回")]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
