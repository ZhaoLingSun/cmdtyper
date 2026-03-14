pub mod command_lesson;
pub mod command_topics;
pub mod dictation;
pub mod home;
pub mod learn_hub;
pub mod review;
pub mod round_result;
pub mod settings;
pub mod stats;
pub mod symbol_lesson;
pub mod symbol_topics;
pub mod system_lesson;
pub mod system_topics;
pub mod typing;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, AppState, SystemPhase};

/// Top-level render dispatch: routes to the appropriate UI module based on AppState.
pub fn render(frame: &mut Frame, app: &App) {
    match &app.state {
        AppState::Home => home::render(frame, app),
        AppState::Typing => typing::render(frame, app),
        AppState::LearnHub => learn_hub::render(frame, app),
        AppState::CommandTopics => command_topics::render(frame, app),
        AppState::CommandLessonOverview {
            category_index,
            command_index,
        } => command_lesson::render_overview(frame, app, *category_index, *command_index),
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
        AppState::SystemLesson {
            topic_index,
            section_index,
            phase,
        } => match phase {
            SystemPhase::Overview => system_lesson::render_overview(frame, app, *topic_index),
            SystemPhase::Detail => {
                system_lesson::render_detail(frame, app, *topic_index, *section_index)
            }
            SystemPhase::Commands(cmd_idx) => {
                system_lesson::render_commands(frame, app, *topic_index, *section_index, *cmd_idx)
            }
            SystemPhase::ConfigFile(cf_idx) => {
                system_lesson::render_config_file(frame, app, *topic_index, *section_index, *cf_idx)
            }
        },
        AppState::Review { source, phase } => review::render(frame, app, source, phase),
        AppState::Dictation => dictation::render(frame, app),
        AppState::Stats => stats::render(frame, app),
        AppState::Settings => settings::render(frame, app),
        AppState::RoundResult => round_result::render(frame, app),
        AppState::Quitting => {}
    }
}
