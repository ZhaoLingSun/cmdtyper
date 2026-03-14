use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, SystemPhase};
use crate::data::models::DeepSource;

pub fn handle_system_topics_key(app: &mut App, key: KeyEvent) {
    let count = app.system_topics.len();
    match key.code {
        KeyCode::Esc => app.state = AppState::LearnHub,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.system_topics_index > 0 {
                app.system_topics_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.system_topics_index < count.saturating_sub(1) {
                app.system_topics_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.system_topics_index < count {
                app.system_section_index = 0;
                app.state = AppState::SystemLesson {
                    topic_index: app.system_topics_index,
                    section_index: 0,
                    phase: SystemPhase::Overview,
                };
            }
        }
        _ => {}
    }
}

pub fn handle_system_lesson_key(
    app: &mut App,
    key: KeyEvent,
    topic_index: usize,
    section_index: usize,
    phase: SystemPhase,
) {
    let topic = match app.system_topics.get(topic_index) {
        Some(t) => t,
        None => {
            app.state = AppState::SystemTopics;
            return;
        }
    };

    match key.code {
        KeyCode::Esc => app.state = AppState::SystemTopics,
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let SystemPhase::Commands(cmd_idx) = &phase {
                let has_deep_explanation = topic
                    .sections
                    .get(section_index)
                    .and_then(|section| section.commands.get(*cmd_idx))
                    .and_then(|cmd| cmd.deep_explanation.as_ref())
                    .is_some();

                if has_deep_explanation {
                    app.state = AppState::DeepExplanation {
                        source: DeepSource::SystemCommand {
                            topic_idx: topic_index,
                            section_idx: section_index,
                            command_idx: *cmd_idx,
                        },
                        scroll: 0,
                    };
                }
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
            SystemPhase::Overview => {
                if !topic.sections.is_empty() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: 0,
                        phase: SystemPhase::Detail,
                    };
                }
            }
            SystemPhase::Detail => {
                if section_index < topic.sections.len()
                    && !topic.sections[section_index].commands.is_empty()
                {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::Commands(0),
                    };
                } else if section_index < topic.sections.len()
                    && !topic.sections[section_index].config_files.is_empty()
                {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::ConfigFile(0),
                    };
                } else if section_index + 1 < topic.sections.len() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index + 1,
                        phase: SystemPhase::Detail,
                    };
                } else {
                    app.state = AppState::SystemTopics;
                }
            }
            SystemPhase::Commands(idx) => {
                let section = &topic.sections[section_index];
                let next = idx + 1;
                if next < section.commands.len() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::Commands(next),
                    };
                } else if !section.config_files.is_empty() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::ConfigFile(0),
                    };
                } else if section_index + 1 < topic.sections.len() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index + 1,
                        phase: SystemPhase::Detail,
                    };
                } else {
                    app.state = AppState::SystemTopics;
                }
            }
            SystemPhase::ConfigFile(idx) => {
                let section = &topic.sections[section_index];
                let next = idx + 1;
                if next < section.config_files.len() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::ConfigFile(next),
                    };
                } else if section_index + 1 < topic.sections.len() {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index + 1,
                        phase: SystemPhase::Detail,
                    };
                } else {
                    app.state = AppState::SystemTopics;
                }
            }
        },
        KeyCode::Up | KeyCode::Char('k') => {
            // Navigate sections
            if matches!(phase, SystemPhase::Detail) && section_index > 0 {
                app.state = AppState::SystemLesson {
                    topic_index,
                    section_index: section_index - 1,
                    phase: SystemPhase::Detail,
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if matches!(phase, SystemPhase::Detail) && section_index + 1 < topic.sections.len() {
                app.state = AppState::SystemLesson {
                    topic_index,
                    section_index: section_index + 1,
                    phase: SystemPhase::Detail,
                };
            }
        }
        _ => {}
    }
}
