use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, SystemPhase};
use crate::core::scorer;
use crate::data::models::{DeepSource, RecordMode};

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
    if app.system_topics.get(topic_index).is_none() {
        app.state = AppState::SystemTopics;
        return;
    }

    if let SystemPhase::TypingPractice { command_idx } = &phase {
        handle_system_typing_key(app, key, topic_index, section_index, *command_idx);
        return;
    }

    match key.code {
        KeyCode::Esc => app.state = AppState::SystemTopics,
        KeyCode::Char('d') | KeyCode::Char('D') => {}
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
            SystemPhase::Overview => {
                let has_sections = app
                    .system_topics
                    .get(topic_index)
                    .map(|topic| !topic.sections.is_empty())
                    .unwrap_or(false);

                if has_sections {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: 0,
                        phase: SystemPhase::Detail,
                    };
                }
            }
            SystemPhase::Detail => {
                let (has_commands, has_config, section_len) = app
                    .system_topics
                    .get(topic_index)
                    .and_then(|topic| topic.sections.get(section_index).map(|section| (section, topic.sections.len())))
                    .map(|(section, len)| {
                        (
                            !section.commands.is_empty(),
                            !section.config_files.is_empty(),
                            len,
                        )
                    })
                    .unwrap_or((false, false, 0));

                if has_commands {
                    enter_system_typing(app, topic_index, section_index, 0);
                } else if has_config {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::ConfigFile(0),
                    };
                } else if section_index + 1 < section_len {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index + 1,
                        phase: SystemPhase::Detail,
                    };
                } else {
                    app.state = AppState::SystemTopics;
                }
            }
            SystemPhase::TypingPractice { .. } => {}
            SystemPhase::ConfigFile(idx) => {
                let (config_len, section_len) = app
                    .system_topics
                    .get(topic_index)
                    .and_then(|topic| topic.sections.get(section_index).map(|section| (section.config_files.len(), topic.sections.len())))
                    .unwrap_or((0, 0));

                let next = idx + 1;
                if next < config_len {
                    app.state = AppState::SystemLesson {
                        topic_index,
                        section_index,
                        phase: SystemPhase::ConfigFile(next),
                    };
                } else if section_index + 1 < section_len {
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
            if matches!(phase, SystemPhase::Detail) && section_index > 0 {
                app.state = AppState::SystemLesson {
                    topic_index,
                    section_index: section_index - 1,
                    phase: SystemPhase::Detail,
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let section_len = app
                .system_topics
                .get(topic_index)
                .map(|topic| topic.sections.len())
                .unwrap_or(0);

            if matches!(phase, SystemPhase::Detail) && section_index + 1 < section_len {
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

fn enter_system_typing(app: &mut App, topic_index: usize, section_index: usize, command_idx: usize) {
    let command = app
        .system_topics
        .get(topic_index)
        .and_then(|topic| topic.sections.get(section_index))
        .and_then(|section| section.commands.get(command_idx))
        .map(|cmd| cmd.command.clone());

    if let Some(command) = command {
        app.typing_engine.reset(&command);
        app.system_typing_showing_output = false;
        app.state = AppState::SystemLesson {
            topic_index,
            section_index,
            phase: SystemPhase::TypingPractice { command_idx },
        };
    }
}

fn handle_system_typing_key(
    app: &mut App,
    key: KeyEvent,
    topic_index: usize,
    section_index: usize,
    command_idx: usize,
) {
    let command = match app
        .system_topics
        .get(topic_index)
        .and_then(|topic| topic.sections.get(section_index))
        .and_then(|section| section.commands.get(command_idx))
        .cloned()
    {
        Some(cmd) => cmd,
        None => {
            app.state = AppState::SystemTopics;
            return;
        }
    };

    match key.code {
        KeyCode::Esc => app.state = AppState::SystemTopics,
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if command.deep_explanation.is_some() {
                app.state = AppState::DeepExplanation {
                    source: DeepSource::SystemCommand {
                        topic_idx: topic_index,
                        section_idx: section_index,
                        command_idx,
                    },
                    scroll: 0,
                };
            }
        }
        KeyCode::Backspace if !app.typing_engine.is_complete() => {
            app.typing_engine.backspace();
        }
        KeyCode::Char(c) if !app.typing_engine.is_complete() => {
            app.typing_engine.input(c);
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if !app.typing_engine.is_complete() {
                return;
            }

            let has_output = command
                .simulated_output
                .as_deref()
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false);

            if !app.system_typing_showing_output && has_output {
                app.system_typing_showing_output = true;
                return;
            }

            finalize_system_typing_command(app, topic_index, section_index, command_idx);
            advance_system_typing(app, topic_index, section_index, command_idx + 1);
        }
        _ => {}
    }
}

fn finalize_system_typing_command(
    app: &mut App,
    topic_index: usize,
    section_index: usize,
    command_idx: usize,
) {
    let Some(topic) = app.system_topics.get(topic_index) else {
        return;
    };

    let command_id = format!("system:{}:{}:{}", topic.meta.id, section_index, command_idx);
    let record = app
        .typing_engine
        .finish(&command_id, topic.meta.difficulty, RecordMode::SystemTyping);

    scorer::update_stats(&mut app.user_stats, &record);
    let _ = app.progress_store.save_stats(&app.user_stats);
    let _ = app.progress_store.append_record(&record);
    app.history.push(record);
}

fn advance_system_typing(app: &mut App, topic_index: usize, section_index: usize, next_cmd_idx: usize) {
    app.system_typing_showing_output = false;

    let (command_len, has_config, section_len) = app
        .system_topics
        .get(topic_index)
        .and_then(|topic| {
            topic
                .sections
                .get(section_index)
                .map(|section| (section.commands.len(), !section.config_files.is_empty(), topic.sections.len()))
        })
        .unwrap_or((0, false, 0));

    if next_cmd_idx < command_len {
        enter_system_typing(app, topic_index, section_index, next_cmd_idx);
    } else if has_config {
        app.state = AppState::SystemLesson {
            topic_index,
            section_index,
            phase: SystemPhase::ConfigFile(0),
        };
    } else if section_index + 1 < section_len {
        app.state = AppState::SystemLesson {
            topic_index,
            section_index: section_index + 1,
            phase: SystemPhase::Detail,
        };
    } else {
        app.state = AppState::SystemTopics;
    }
}
