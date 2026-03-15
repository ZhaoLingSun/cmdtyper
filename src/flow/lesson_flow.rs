use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppState};
use crate::core::scorer;
use crate::data::models::{DeepSource, RecordMode};

pub fn handle_command_lesson_overview_key(
    app: &mut App,
    key: KeyEvent,
    category_index: usize,
    command_index: usize,
) {
    match key.code {
        KeyCode::Esc => app.state = AppState::CommandTopics,
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            let cmd_str = get_lesson_example_command(app, category_index, command_index, 0);
            if let Some(cmd) = cmd_str {
                app.typing_engine.reset(&cmd);
                app.state = AppState::CommandLessonPractice {
                    category_index,
                    command_index,
                    example_index: 0,
                };
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if command_index > 0 {
                app.state = AppState::CommandLessonOverview {
                    category_index,
                    command_index: command_index - 1,
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let cats = app.get_lesson_categories();
            if category_index < cats.len() {
                let lessons = app.get_lessons_for_category(cats[category_index]);
                if command_index + 1 < lessons.len() {
                    app.state = AppState::CommandLessonOverview {
                        category_index,
                        command_index: command_index + 1,
                    };
                }
            }
        }
        _ => {}
    }
}

pub fn handle_command_lesson_practice_key(
    app: &mut App,
    key: KeyEvent,
    category_index: usize,
    command_index: usize,
    example_index: usize,
) {
    match key.code {
        KeyCode::Esc => {
            app.state = AppState::CommandLessonOverview {
                category_index,
                command_index,
            };
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            let has_deep_explanation = {
                let cats = app.get_lesson_categories();
                if category_index < cats.len() {
                    let lessons = app.get_lessons_for_category(cats[category_index]);
                    lessons
                        .get(command_index)
                        .and_then(|lesson| lesson.examples.get(example_index))
                        .and_then(|example| example.deep_explanation.as_ref())
                        .is_some()
                } else {
                    false
                }
            };

            if has_deep_explanation {
                app.state = AppState::DeepExplanation {
                    source: DeepSource::LessonExample {
                        category_idx: category_index,
                        command_idx: command_index,
                        example_idx: example_index,
                    },
                    scroll: 0,
                };
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let cmd_str =
                get_lesson_example_command(app, category_index, command_index, example_index);
            if let Some(cmd) = cmd_str {
                app.typing_engine.reset(&cmd);
            }
        }
        KeyCode::Enter if app.typing_engine.is_complete() => {
            // Save lesson practice stats using lesson difficulty.
            let lesson_meta = {
                let cats = app.get_lesson_categories();
                if category_index < cats.len() {
                    let lessons = app.get_lessons_for_category(cats[category_index]);
                    lessons.get(command_index).map(|lesson| {
                        (
                            format!("lesson:{}:{}", lesson.meta.command, example_index),
                            lesson.meta.difficulty,
                            lesson.examples.len(),
                        )
                    })
                } else {
                    None
                }
            };

            if let Some((command_id, difficulty, example_len)) = lesson_meta {
                let record =
                    app.typing_engine
                        .finish(&command_id, difficulty, RecordMode::LessonPractice);
                scorer::update_stats(&mut app.user_stats, &record);
                let _ = app.progress_store.save_stats(&app.user_stats);
                let _ = app.progress_store.append_record(&record);
                app.history.push(record);

                // Move to next example or back to overview
                let next_example = example_index + 1;
                if next_example < example_len {
                    if let Some(cmd) =
                        get_lesson_example_command(app, category_index, command_index, next_example)
                    {
                        app.typing_engine.reset(&cmd);
                        app.state = AppState::CommandLessonPractice {
                            category_index,
                            command_index,
                            example_index: next_example,
                        };
                    }
                } else {
                    app.state = AppState::CommandLessonOverview {
                        category_index,
                        command_index,
                    };
                }
            }
        }
        KeyCode::Char(c) if !app.typing_engine.is_complete() => {
            app.typing_engine.input(c);
        }
        _ => {}
    }
}

fn get_lesson_example_command(
    app: &App,
    category_index: usize,
    command_index: usize,
    example_index: usize,
) -> Option<String> {
    let cats = app.get_lesson_categories();
    let cat = cats.get(category_index)?;
    let lessons = app.get_lessons_for_category(*cat);
    let lesson = lessons.get(command_index)?;
    let example = lesson.examples.get(example_index)?;
    Some(example.command.clone())
}
