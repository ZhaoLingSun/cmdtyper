use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, SymbolPhase};
use crate::core::matcher::{self, MatchResult};
use crate::core::scorer;
use crate::data::models::{RecordMode, SessionRecord};

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
                && !app.symbol_topics[app.symbol_topics_index]
                    .symbols
                    .is_empty()
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
    let topic = match app.symbol_topics.get(topic_index) {
        Some(t) => t,
        None => {
            app.state = AppState::SymbolTopics;
            return;
        }
    };

    if matches!(phase, SymbolPhase::Practice) {
        handle_symbol_practice_key(app, key, topic_index, symbol_index);
        return;
    }

    match key.code {
        KeyCode::Esc => app.state = AppState::SymbolTopics,
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
            SymbolPhase::Explain => {
                let sym = &topic.symbols[symbol_index];
                if !sym.examples.is_empty() {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(0),
                    };
                } else if !topic.exercises.is_empty() {
                    start_symbol_practice(app, topic_index);
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Practice,
                    };
                }
            }
            SymbolPhase::Example(idx) => {
                let sym = &topic.symbols[symbol_index];
                let next = idx + 1;
                if next < sym.examples.len() {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(next),
                    };
                } else if symbol_index + 1 < topic.symbols.len() {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index: symbol_index + 1,
                        phase: SymbolPhase::Explain,
                    };
                } else if !topic.exercises.is_empty() {
                    start_symbol_practice(app, topic_index);
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Practice,
                    };
                } else {
                    app.state = AppState::SymbolTopics;
                }
            }
            SymbolPhase::Practice => {
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
            SymbolPhase::Practice => {
                let sym = &topic.symbols[symbol_index];
                if !sym.examples.is_empty() {
                    app.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(sym.examples.len() - 1),
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
    let total_count = app
        .symbol_topics
        .get(topic_index)
        .map(|topic| topic.exercises.len())
        .unwrap_or(0);
    app.symbol_practice = crate::app::SymbolPracticeState {
        total_count,
        ..crate::app::SymbolPracticeState::default()
    };
}

fn advance_symbol_practice(app: &mut App, topic_index: usize) {
    if app.symbol_practice.current_index + 1 < app.symbol_practice.total_count {
        app.symbol_practice.current_index += 1;
        app.symbol_practice.current_input.clear();
        app.symbol_practice.error_count = 0;
        app.symbol_practice.show_answer = false;
        app.symbol_practice.submitted = false;
        app.symbol_practice.last_correct = None;
        return;
    }

    app.symbol_practice.completed = true;
    app.symbol_practice.current_input.clear();
    app.symbol_practice.submitted = false;
    app.symbol_practice.last_correct = None;

    if app.symbol_practice.stats_recorded {
        return;
    }

    if let Some(topic) = app.symbol_topics.get(topic_index) {
        let accuracy = if app.symbol_practice.total_count == 0 {
            1.0
        } else {
            app.symbol_practice.correct_count as f64 / app.symbol_practice.total_count as f64
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
            error_count: (app
                .symbol_practice
                .total_count
                .saturating_sub(app.symbol_practice.correct_count)) as u32,
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
        .symbol_topics
        .get(topic_index)
        .and_then(|topic| topic.exercises.get(app.symbol_practice.current_index))
        .map(|exercise| exercise.answers.clone())
    {
        Some(v) => v,
        None => {
            app.state = AppState::SymbolTopics;
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
                    app.symbol_practice.correct_count += 1;
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
                        advance_symbol_practice(app, topic_index);
                    }
                }
            }
        }
        _ => {}
    }
}
