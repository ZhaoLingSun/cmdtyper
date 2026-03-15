use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, SymbolPhase};
use crate::core::matcher::{self, MatchResult};
use crate::core::scorer;
use crate::data::models::{DeepSource, ExerciseKind, RecordMode, SessionRecord};

pub fn handle_symbol_topics_key(app: &mut App, key: KeyEvent) {
    let count = app.symbol_topics.len();

    match key.code {
        KeyCode::Esc => app.state = AppState::LearnHub,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.symbol_topics_index > 0 {
                app.symbol_topics_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.symbol_topics_index < count.saturating_sub(1) {
                app.symbol_topics_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.symbol_topics_index < count
                && !app.symbol_topics[app.symbol_topics_index].symbols.is_empty()
            {
                app.state = AppState::SymbolLesson {
                    topic_index: app.symbol_topics_index,
                    symbol_index: 0,
                    phase: SymbolPhase::Explain,
                };
            }
        }
        _ => {}
    }
}

pub fn handle_symbol_lesson_key(
    app: &mut App,
    key: KeyEvent,
    topic_index: usize,
    symbol_index: usize,
    phase: SymbolPhase,
) {
    if app.symbol_topics.get(topic_index).is_none() {
        app.state = AppState::SymbolTopics;
        return;
    }

    if let SymbolPhase::TypingPractice { exercise_idx } = &phase {
        handle_symbol_typing_key(app, key, topic_index, symbol_index, *exercise_idx);
        return;
    }

    if matches!(&phase, SymbolPhase::Practice) {
        handle_symbol_practice_key(app, key, topic_index, symbol_index);
        return;
    }

    match key.code {
        KeyCode::Esc => app.state = AppState::SymbolTopics,
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let SymbolPhase::Example(example_idx) = &phase {
                let has_deep_explanation = app
                    .symbol_topics
                    .get(topic_index)
                    .and_then(|topic| topic.symbols.get(symbol_index))
                    .and_then(|symbol| symbol.examples.get(*example_idx))
                    .and_then(|example| example.deep_explanation.as_ref())
                    .is_some();

                if has_deep_explanation {
                    app.state = AppState::DeepExplanation {
                        source: DeepSource::SymbolExample {
                            topic_idx: topic_index,
                            symbol_idx: symbol_index,
                            example_idx: *example_idx,
                        },
                        scroll: 0,
                    };
                }
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
            SymbolPhase::Explain => {
                let symbol_has_examples = app
                    .symbol_topics
                    .get(topic_index)
                    .and_then(|topic| topic.symbols.get(symbol_index))
                    .map(|sym| !sym.examples.is_empty())
                    .unwrap_or(false);
                let has_exercises = app
                    .symbol_topics
                    .get(topic_index)
                    .map(|topic| !topic.exercises.is_empty())
                    .unwrap_or(false);

                if symbol_has_examples {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(0),
                    };
                } else if has_exercises {
                    start_symbol_practice(app, topic_index);
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: next_symbol_practice_phase(app),
                    };
                }
            }
            SymbolPhase::Example(idx) => {
                let (example_count, symbol_count, has_exercises) = app
                    .symbol_topics
                    .get(topic_index)
                    .map(|topic| {
                        let ex_cnt = topic
                            .symbols
                            .get(symbol_index)
                            .map(|s| s.examples.len())
                            .unwrap_or(0);
                        (ex_cnt, topic.symbols.len(), !topic.exercises.is_empty())
                    })
                    .unwrap_or((0, 0, false));

                let next = idx + 1;
                if next < example_count {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(next),
                    };
                } else if symbol_index + 1 < symbol_count {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index: symbol_index + 1,
                        phase: SymbolPhase::Explain,
                    };
                } else if has_exercises {
                    start_symbol_practice(app, topic_index);
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: next_symbol_practice_phase(app),
                    };
                } else {
                    app.state = AppState::SymbolTopics;
                }
            }
            SymbolPhase::TypingPractice { .. } | SymbolPhase::Practice => {
                app.state = AppState::SymbolTopics;
            }
        },
        KeyCode::Left | KeyCode::Char('h') => match &phase {
            SymbolPhase::Explain => app.state = AppState::SymbolTopics,
            SymbolPhase::Example(0) => {
                app.state = AppState::SymbolLesson {
                    topic_index,
                    symbol_index,
                    phase: SymbolPhase::Explain,
                };
            }
            SymbolPhase::Example(idx) => {
                app.state = AppState::SymbolLesson {
                    topic_index,
                    symbol_index,
                    phase: SymbolPhase::Example(idx - 1),
                };
            }
            SymbolPhase::TypingPractice { .. } | SymbolPhase::Practice => {
                let last_example_idx = app
                    .symbol_topics
                    .get(topic_index)
                    .and_then(|topic| topic.symbols.get(symbol_index))
                    .map(|sym| sym.examples.len())
                    .unwrap_or(0)
                    .saturating_sub(1);

                if last_example_idx > 0
                    || app
                        .symbol_topics
                        .get(topic_index)
                        .and_then(|topic| topic.symbols.get(symbol_index))
                        .map(|sym| !sym.examples.is_empty())
                        .unwrap_or(false)
                {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(last_example_idx),
                    };
                } else {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Explain,
                    };
                }
            }
        },
        _ => {}
    }
}

fn start_symbol_practice(app: &mut App, topic_index: usize) {
    let mut typing_indices = Vec::new();
    let mut dictation_indices = Vec::new();

    if let Some(topic) = app.symbol_topics.get(topic_index) {
        for (idx, exercise) in topic.exercises.iter().enumerate() {
            match exercise.kind {
                Some(ExerciseKind::Typing) => typing_indices.push(idx),
                Some(ExerciseKind::Dictation) | None => dictation_indices.push(idx),
            }
        }
    }

    let mut state = crate::app::SymbolPracticeState {
        total_count: typing_indices.len() + dictation_indices.len(),
        typing_indices,
        dictation_indices,
        ..crate::app::SymbolPracticeState::default()
    };

    if let Some(first_idx) = state.typing_indices.first().copied()
        && let Some(command) = app
            .symbol_topics
            .get(topic_index)
            .and_then(|topic| topic.exercises.get(first_idx))
            .and_then(extract_typing_command)
    {
        app.typing_engine.reset(&command);
    }

    state.current_index = 0;
    app.symbol_practice = state;
}

fn next_symbol_practice_phase(app: &App) -> SymbolPhase {
    if !app.symbol_practice.typing_indices.is_empty() {
        SymbolPhase::TypingPractice { exercise_idx: 0 }
    } else {
        SymbolPhase::Practice
    }
}

fn handle_symbol_typing_key(
    app: &mut App,
    key: KeyEvent,
    topic_index: usize,
    symbol_index: usize,
    exercise_idx: usize,
) {
    if app.symbol_practice.completed {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => app.state = AppState::SymbolTopics,
            _ => {}
        }
        return;
    }

    let exercise = match app
        .current_symbol_typing_exercise(topic_index, exercise_idx)
        .cloned()
    {
        Some(ex) => ex,
        None => {
            app.state = AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase: SymbolPhase::Practice,
            };
            return;
        }
    };

    match key.code {
        KeyCode::Esc => app.state = AppState::SymbolTopics,
        KeyCode::Backspace if !app.typing_engine.is_complete() => {
            app.typing_engine.backspace();
        }
        KeyCode::Char(c) if !app.typing_engine.is_complete() => {
            app.typing_engine.input(c);
        }
        KeyCode::Enter => {
            if !app.typing_engine.is_complete() {
                return;
            }

            let has_output = exercise
                .simulated_output
                .as_deref()
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false);

            if !app.symbol_practice.typing_showing_output && has_output {
                app.symbol_practice.typing_showing_output = true;
                return;
            }

            finalize_symbol_typing_exercise(app, topic_index, exercise_idx);

            let next = exercise_idx + 1;
            if next < app.symbol_practice.typing_indices.len() {
                if let Some(next_command) = app
                    .current_symbol_typing_exercise(topic_index, next)
                    .and_then(extract_typing_command)
                {
                    app.typing_engine.reset(&next_command);
                }
                app.symbol_practice.typing_showing_output = false;
                app.state = AppState::SymbolLesson {
                    topic_index,
                    symbol_index,
                    phase: SymbolPhase::TypingPractice { exercise_idx: next },
                };
            } else {
                app.symbol_practice.typing_showing_output = false;
                app.symbol_practice.current_index = 0;
                app.symbol_practice.current_input.clear();
                app.symbol_practice.submitted = false;
                app.symbol_practice.last_correct = None;
                app.symbol_practice.show_answer = false;
                app.symbol_practice.error_count = 0;

                if app.symbol_practice.dictation_indices.is_empty() {
                    finish_symbol_practice(app, topic_index);
                } else {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Practice,
                    };
                }
            }
        }
        _ => {}
    }
}

fn finalize_symbol_typing_exercise(app: &mut App, topic_index: usize, exercise_idx: usize) {
    let Some(topic) = app.symbol_topics.get(topic_index) else {
        return;
    };
    let Some(raw_idx) = app.symbol_practice.typing_indices.get(exercise_idx).copied() else {
        return;
    };

    let command_id = format!("symbol:{}:typing:{}", topic.meta.id, raw_idx);
    let record = app
        .typing_engine
        .finish(&command_id, topic.meta.difficulty, RecordMode::SymbolTyping);

    app.symbol_practice.typing_count += 1;
    app.symbol_practice.typing_accuracy_sum += record.accuracy;
    app.symbol_practice.typing_wpm_sum += record.wpm;

    scorer::update_stats(&mut app.user_stats, &record);
    let _ = app.progress_store.save_stats(&app.user_stats);
    let _ = app.progress_store.append_record(&record);
    app.history.push(record);
}

fn advance_symbol_practice(app: &mut App, topic_index: usize) {
    if app.symbol_practice.current_index + 1 < app.symbol_practice.dictation_indices.len() {
        app.symbol_practice.current_index += 1;
        app.symbol_practice.current_input.clear();
        app.symbol_practice.error_count = 0;
        app.symbol_practice.show_answer = false;
        app.symbol_practice.submitted = false;
        app.symbol_practice.last_correct = None;
        return;
    }

    finish_symbol_practice(app, topic_index);
}

fn finish_symbol_practice(app: &mut App, topic_index: usize) {
    app.symbol_practice.completed = true;
    app.symbol_practice.current_input.clear();
    app.symbol_practice.submitted = false;
    app.symbol_practice.last_correct = None;

    if app.symbol_practice.stats_recorded {
        return;
    }

    if let Some(topic) = app.symbol_topics.get(topic_index) {
        let accuracy = if app.symbol_practice.dictation_count == 0 {
            1.0
        } else {
            app.symbol_practice.dictation_accuracy_sum / app.symbol_practice.dictation_count as f64
        };
        let now_ms = Utc::now().timestamp_millis();
        let record = SessionRecord {
            id: format!("{}", now_ms),
            command_id: format!("symbol:{}", topic.meta.id),
            mode: RecordMode::SymbolPractice,
            keystrokes: Vec::new(),
            started_at: now_ms,
            finished_at: now_ms,
            wpm: 0.0,
            cpm: 0.0,
            accuracy,
            error_count: app
                .symbol_practice
                .dictation_count
                .saturating_sub(app.symbol_practice.dictation_correct_count)
                as u32,
            difficulty: topic.meta.difficulty,
        };
        scorer::update_stats(&mut app.user_stats, &record);
        let _ = app.progress_store.save_stats(&app.user_stats);
        let _ = app.progress_store.append_record(&record);
        app.history.push(record);
        app.symbol_practice.stats_recorded = true;
    }
}

fn handle_symbol_practice_key(
    app: &mut App,
    key: KeyEvent,
    topic_index: usize,
    _symbol_index: usize,
) {
    if app.symbol_practice.completed {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => app.state = AppState::SymbolTopics,
            _ => {}
        }
        return;
    }

    let answers = match app
        .current_symbol_practice_exercise(topic_index)
        .map(|exercise| exercise.answers.clone())
    {
        Some(v) => v,
        None => {
            finish_symbol_practice(app, topic_index);
            return;
        }
    };

    match key.code {
        KeyCode::Esc => app.state = AppState::SymbolTopics,
        KeyCode::Backspace if !app.symbol_practice.submitted => {
            app.symbol_practice.current_input.pop();
        }
        KeyCode::Char(c) if !app.symbol_practice.submitted => {
            app.symbol_practice.current_input.push(c);
        }
        KeyCode::Enter => {
            if app.symbol_practice.submitted {
                if app.symbol_practice.last_correct == Some(true) {
                    advance_symbol_practice(app, topic_index);
                } else {
                    app.symbol_practice.current_input.clear();
                    app.symbol_practice.submitted = false;
                    app.symbol_practice.last_correct = None;
                    app.symbol_practice.show_answer = false;
                }
                return;
            }

            let result = matcher::check(&app.symbol_practice.current_input, &answers);
            match result {
                MatchResult::Exact(_) | MatchResult::Normalized(_) => {
                    app.symbol_practice.dictation_correct_count += 1;
                    app.symbol_practice.dictation_accuracy_sum += 1.0;
                    app.symbol_practice.dictation_count += 1;
                    app.symbol_practice.submitted = true;
                    app.symbol_practice.last_correct = Some(true);
                    app.symbol_practice.show_answer = false;
                }
                MatchResult::NoMatch { .. } => {
                    app.symbol_practice.error_count =
                        app.symbol_practice.error_count.saturating_add(1);
                    app.symbol_practice.submitted = true;
                    app.symbol_practice.last_correct = Some(false);
                    app.symbol_practice.show_answer = true;
                    if app.symbol_practice.error_count >= 3 {
                        app.symbol_practice.dictation_accuracy_sum += 0.0;
                        app.symbol_practice.dictation_count += 1;
                        advance_symbol_practice(app, topic_index);
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_typing_command(exercise: &crate::data::models::Exercise) -> Option<String> {
    if let Some(command) = &exercise.command
        && !command.trim().is_empty()
    {
        return Some(command.clone());
    }

    exercise.answers.first().cloned()
}
