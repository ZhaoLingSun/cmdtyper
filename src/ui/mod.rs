pub mod calendar;
pub mod command_lesson;
pub mod command_topics;
pub mod deep_explanation;
pub mod dictation;
pub mod filter_select;
pub mod home;
pub mod learn_hub;
pub mod practice;
pub mod review;
pub mod scenarios;
pub mod settings;
pub mod stats;
pub mod symbol_lesson;
pub mod symbol_topics;
pub mod system_lesson;
pub mod system_topics;
pub mod typing;
pub mod widgets;
pub mod workflow;

use ratatui::Frame;

use crate::app::{App, AppState, SystemPhase};

/// Top-level render dispatch: routes to the appropriate UI module based on AppState.
pub fn render(frame: &mut Frame, app: &App) {
    if crate::flow::workflow_flow::active_sequence(app).is_some() {
        workflow::render(frame, app);
    } else {
        match &app.state {
            AppState::Home => home::render(frame, app),
            AppState::Typing => typing::render(frame, app),
            AppState::TypingFilter => filter_select::render(frame, app),
            AppState::LearnHub => learn_hub::render(frame, app),
            AppState::Calendar => calendar::render(
                frame,
                &app.user_stats,
                &app.history,
                &app.calendar_state,
                frame.area(),
            ),
            AppState::Scenarios => scenarios::render_topics(frame, app),
            AppState::ScenarioPractice => scenarios::render_practice(frame, app),
            AppState::PracticeTopics => practice::render_topics(frame, app),
            AppState::PracticeGroup => practice::render_group(frame, app),
            AppState::CommandTopics => command_topics::render(frame, app),
            AppState::CommandLessonOverview {
                category_index,
                command_index,
                scroll,
            } => command_lesson::render_overview(
                frame,
                app,
                *category_index,
                *command_index,
                *scroll,
            ),
            AppState::CommandLessonPractice {
                category_index,
                command_index,
                example_index,
            } => command_lesson::render_practice(
                frame,
                app,
                *category_index,
                *command_index,
                *example_index,
            ),
            AppState::SymbolTopics => symbol_topics::render(frame, app),
            AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase,
            } => symbol_lesson::render(frame, app, *topic_index, *symbol_index, phase),
            AppState::SystemTopics => system_topics::render(frame, app),
            AppState::ReviewTopics => review::render_topics(frame, app),
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase,
                scroll,
            } => match phase {
                SystemPhase::Overview => system_lesson::render_overview(frame, app, *topic_index),
                SystemPhase::Detail => {
                    system_lesson::render_detail(frame, app, *topic_index, *section_index)
                }
                SystemPhase::TypingPractice { command_idx } => {
                    system_lesson::render_typing_practice(
                        frame,
                        app,
                        *topic_index,
                        *section_index,
                        *command_idx,
                        *scroll,
                    )
                }
                SystemPhase::ConfigFile(cf_idx) => system_lesson::render_config_file(
                    frame,
                    app,
                    *topic_index,
                    *section_index,
                    *cf_idx,
                    *scroll,
                ),
            },
            AppState::DeepExplanation { source, scroll } => {
                deep_explanation::render(frame, app, source, *scroll)
            }
            AppState::Review { source, phase } => review::render(frame, app, source, phase),
            AppState::Dictation => dictation::render(frame, app),
            AppState::Stats => stats::render(frame, app),
            AppState::Settings => settings::render(frame, app),
            AppState::Quitting => {}
        }
    }
    if let Some(error) = &app.persistence_error {
        let area = frame.area();
        if area.height > 0 {
            frame.render_widget(
                ratatui::widgets::Paragraph::new(error.as_str())
                    .style(ratatui::prelude::Style::default().fg(ratatui::prelude::Color::Red)),
                ratatui::prelude::Rect::new(area.x, area.y + area.height - 1, area.width, 1),
            );
        }
    }
}
