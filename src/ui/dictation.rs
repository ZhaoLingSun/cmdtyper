use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::core::matcher::{DiffKind, MatchResult};
use crate::ui::widgets::*;

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

    // Title
    let progress = format!(
        "{}/{}",
        app.dictation_index + 1,
        app.dictation_commands.len()
    );
    let title = Paragraph::new(Line::from(Span::styled(
        format!(" \u{9ed8}\u{5199}\u{6a21}\u{5f0f}  {} ", progress),
        Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM)),
    );
    frame.render_widget(title, chunks[0]);

    // Content
    let content_area = chunks[1];
    let mut lines: Vec<Line> = Vec::new();

    if let Some(cmd) = app.current_dictation_command() {
        // Chinese prompt
        lines.push(Line::from(Span::styled(
            "\u{9898}\u{76ee}:",
            Style::default().fg(HEADER).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", cmd.dictation.prompt),
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Input area
        lines.push(Line::from(Span::styled(
            "\u{4f60}\u{7684}\u{7b54}\u{6848}:",
            Style::default().fg(ACCENT),
        )));

        let input_display = if app.dictation_submitted {
            app.dictation_input.clone()
        } else {
            format!("{}\u{2588}", app.dictation_input) // cursor block
        };
        lines.push(Line::from(Span::styled(
            format!("  {}", input_display),
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Show result after submission
        if app.dictation_submitted
            && let Some(result) = &app.dictation_result
        {
            match result {
                MatchResult::Exact(_) => {
                    lines.push(Line::from(Span::styled(
                        "\u{2705} \u{5b8c}\u{5168}\u{6b63}\u{786e}\u{ff01}",
                        Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                    )));
                }
                MatchResult::Normalized(_) => {
                    lines.push(Line::from(Span::styled(
                            "\u{2705} \u{6b63}\u{786e}\u{ff08}\u{5ffd}\u{7565}\u{5927}\u{5c0f}\u{5199}/\u{7a7a}\u{683c}\u{5dee}\u{5f02}\u{ff09}",
                            Style::default().fg(SUCCESS),
                        )));
                }
                MatchResult::NoMatch { closest, diff } => {
                    lines.push(Line::from(Span::styled(
                        "\u{274c} \u{4e0d}\u{6b63}\u{786e}",
                        Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(""));

                    // Show diff
                    lines.push(Line::from(Span::styled(
                        "\u{5dee}\u{5f02}\u{5bf9}\u{6bd4}:",
                        Style::default().fg(HEADER),
                    )));

                    let mut diff_spans = Vec::new();
                    diff_spans.push(Span::raw("  "));
                    for segment in diff {
                        let style = match segment.kind {
                            DiffKind::Same => Style::default().fg(Color::White),
                            DiffKind::Added => Style::default()
                                .fg(SUCCESS)
                                .add_modifier(Modifier::UNDERLINED),
                            DiffKind::Removed => Style::default()
                                .fg(ERROR)
                                .add_modifier(Modifier::CROSSED_OUT),
                        };
                        diff_spans.push(Span::styled(segment.text.clone(), style));
                    }
                    lines.push(Line::from(diff_spans));
                    lines.push(Line::from(""));

                    lines.push(Line::from(vec![
                        Span::styled(
                            "\u{6b63}\u{786e}\u{7b54}\u{6848}: ",
                            Style::default().fg(DIM),
                        ),
                        Span::styled(closest.clone(), Style::default().fg(ACCENT)),
                    ]));
                }
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{9ed8}\u{5199}\u{9898}\u{76ee}",
            Style::default().fg(DIM),
        )));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, content_area);

    // Hints
    let hints = if app.dictation_submitted {
        hint_line(&[
            ("Enter", "\u{4e0b}\u{4e00}\u{9898}"),
            ("Esc", "\u{8fd4}\u{56de}"),
        ])
    } else {
        hint_line(&[("Enter", "\u{63d0}\u{4ea4}"), ("Esc", "\u{8fd4}\u{56de}")])
    };
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[2],
    );
}
