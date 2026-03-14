use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};
use rand::seq::SliceRandom;

use crate::app::{App, AppState, ReviewExercise, ReviewExerciseKind, ReviewPhase, ReviewSource};
use crate::core::matcher::{self, MatchResult};
use crate::core::scorer;
use crate::data::models::{RecordMode, SessionRecord};

pub fn handle_review_key(app: &mut App, key: KeyEvent, source: ReviewSource, phase: ReviewPhase) {
    match phase {
        ReviewPhase::Summary => match key.code {
            KeyCode::Esc => app.state = AppState::LearnHub,
            KeyCode::Enter => {
                start_review_practice(app, &source);
                app.state = AppState::Review {
                    source,
                    phase: ReviewPhase::Practice(0),
                };
            }
            _ => {}
        },
        ReviewPhase::Practice(_) => {
            if app.review_practice.completed {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => app.state = AppState::LearnHub,
                    _ => {}
                }
                return;
            }

            let exercise = match app
                .review_practice
                .exercises
                .get(app.review_practice.current_index)
                .cloned()
            {
                Some(ex) => ex,
                None => {
                    app.review_practice.completed = true;
                    record_review_stats(app, &source);
                    return;
                }
            };

            match exercise.kind {
                ReviewExerciseKind::Typing => match key.code {
                    KeyCode::Esc => app.state = AppState::LearnHub,
                    KeyCode::Char(c) if !app.typing_engine.is_complete() => {
                        app.typing_engine.input(c);
                    }
                    KeyCode::Enter if app.typing_engine.is_complete() => {
                        let acc = app.typing_engine.current_accuracy();
                        let wpm = app.typing_engine.current_wpm();
                        app.review_practice.typing_count += 1;
                        app.review_practice.typing_accuracy_sum += acc;
                        app.review_practice.typing_wpm_sum += wpm;
                        app.review_practice.accuracy_sum += acc;
                        record_review_exercise_result(
                            app,
                            &exercise,
                            RecordMode::ReviewTyping,
                            wpm,
                            acc,
                        );
                        advance_review_practice(app, &source);
                    }
                    _ => {}
                },
                ReviewExerciseKind::Dictation => match key.code {
                    KeyCode::Esc => app.state = AppState::LearnHub,
                    KeyCode::Backspace if !app.review_practice.dictation_submitted => {
                        app.review_practice.dictation_input.pop();
                    }
                    KeyCode::Char(c) if !app.review_practice.dictation_submitted => {
                        app.review_practice.dictation_input.push(c);
                    }
                    KeyCode::Enter => {
                        if app.review_practice.dictation_submitted {
                            advance_review_practice(app, &source);
                        } else {
                            let result = matcher::check(
                                &app.review_practice.dictation_input,
                                std::slice::from_ref(&exercise.command),
                            );
                            let acc = match result {
                                MatchResult::Exact(_) | MatchResult::Normalized(_) => 1.0,
                                MatchResult::NoMatch { .. } => 0.0,
                            };
                            app.review_practice.dictation_count += 1;
                            app.review_practice.dictation_accuracy_sum += acc;
                            app.review_practice.accuracy_sum += acc;
                            record_review_exercise_result(
                                app,
                                &exercise,
                                RecordMode::ReviewDictation,
                                0.0,
                                acc,
                            );
                            app.review_practice.dictation_result = Some(result);
                            app.review_practice.dictation_submitted = true;
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

fn build_review_exercises(app: &App, source: &ReviewSource) -> Vec<ReviewExercise> {
    let mut base = Vec::new();
    match source {
        ReviewSource::CommandCategory(category) => {
            for cmd in app.commands.iter().filter(|c| c.category == *category) {
                base.push(ReviewExercise {
                    kind: ReviewExerciseKind::Typing,
                    command_id: cmd.id.clone(),
                    command: cmd.command.clone(),
                    description: cmd.dictation.prompt.clone(),
                    difficulty: cmd.difficulty,
                });
            }
        }
        ReviewSource::SymbolTopic(name) => {
            if let Some(topic) = app
                .symbol_topics
                .iter()
                .find(|topic| topic.meta.topic == *name || topic.meta.id == *name)
            {
                for (idx, exercise) in topic.exercises.iter().enumerate() {
                    if let Some(answer) = exercise.answers.first() {
                        base.push(ReviewExercise {
                            kind: ReviewExerciseKind::Dictation,
                            command_id: format!("symbol:{}:{}", topic.meta.id, idx),
                            command: answer.clone(),
                            description: exercise.prompt.clone(),
                            difficulty: topic.meta.difficulty,
                        });
                    }
                }
            }
        }
        ReviewSource::SystemTopic(name) => {
            if let Some(topic) = app
                .system_topics
                .iter()
                .find(|topic| topic.meta.topic == *name || topic.meta.id == *name)
            {
                for (sec_idx, section) in topic.sections.iter().enumerate() {
                    for (cmd_idx, command) in section.commands.iter().enumerate() {
                        base.push(ReviewExercise {
                            kind: ReviewExerciseKind::Typing,
                            command_id: format!("system:{}:{}:{}", topic.meta.id, sec_idx, cmd_idx),
                            command: command.command.clone(),
                            description: command.summary.clone(),
                            difficulty: topic.meta.difficulty,
                        });
                    }
                }
            }
        }
    }

    if base.is_empty() {
        return base;
    }

    let mut rng = rand::thread_rng();
    base.shuffle(&mut rng);

    let total = base.len();
    let mut dictation_count = ((total as f64) * 0.3).round() as usize;
    if total >= 3 {
        dictation_count = dictation_count.clamp(1, total.saturating_sub(1));
    }

    for (idx, exercise) in base.iter_mut().enumerate() {
        exercise.kind = if idx < dictation_count {
            ReviewExerciseKind::Dictation
        } else {
            ReviewExerciseKind::Typing
        };
    }
    base.shuffle(&mut rng);
    base
}

fn start_review_practice(app: &mut App, source: &ReviewSource) {
    let exercises = build_review_exercises(app, source);
    let total_count = exercises.len();
    app.review_practice = crate::app::ReviewPracticeState {
        exercises,
        total_count,
        ..crate::app::ReviewPracticeState::default()
    };

    if let Some(first) = app.review_practice.exercises.first()
        && matches!(first.kind, ReviewExerciseKind::Typing)
    {
        app.typing_engine.reset(&first.command);
    }
}

fn record_review_exercise_result(
    app: &mut App,
    exercise: &ReviewExercise,
    mode: RecordMode,
    wpm: f64,
    accuracy: f64,
) {
    let now_ms = Utc::now().timestamp_millis();
    let error_count = if accuracy >= 1.0 { 0 } else { 1 };
    let record = SessionRecord {
        id: format!("{}-{}", now_ms, exercise.command_id),
        command_id: exercise.command_id.clone(),
        mode,
        keystrokes: Vec::new(),
        started_at: now_ms,
        finished_at: now_ms,
        wpm,
        cpm: wpm * 5.0,
        accuracy,
        error_count,
        difficulty: exercise.difficulty,
    };
    scorer::update_stats(&mut app.user_stats, &record);
    let _ = app.progress_store.append_record(&record);
    app.history.push(record);
}

fn record_review_stats(app: &mut App, _source: &ReviewSource) {
    if app.review_practice.stats_recorded {
        return;
    }

    let _ = app.progress_store.save_stats(&app.user_stats);
    app.review_practice.stats_recorded = true;
}

fn advance_review_practice(app: &mut App, source: &ReviewSource) {
    app.review_practice.current_index += 1;
    app.review_practice.dictation_input.clear();
    app.review_practice.dictation_result = None;
    app.review_practice.dictation_submitted = false;

    if app.review_practice.current_index >= app.review_practice.exercises.len() {
        app.review_practice.completed = true;
        record_review_stats(app, source);
        return;
    }

    if let Some(exercise) = app
        .review_practice
        .exercises
        .get(app.review_practice.current_index)
        && matches!(exercise.kind, ReviewExerciseKind::Typing)
    {
        app.typing_engine.reset(&exercise.command);
    }

    app.state = AppState::Review {
        source: source.clone(),
        phase: ReviewPhase::Practice(app.review_practice.current_index),
    };
}
