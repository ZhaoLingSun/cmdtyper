use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{
    App, AppState, ReviewExercise, ReviewExerciseKind, ReviewPhase, ReviewPracticeState,
    ReviewSource,
};
use crate::core::matcher::{self, MatchResult};
use crate::data::models::{
    Command, Exercise, RecordMode, SessionRecord, SymbolTopic, SystemSection, SystemTopic, Token,
    TokenKind, TopicTrainingLevel,
};

pub fn handle_review_key(app: &mut App, key: KeyEvent, source: ReviewSource, phase: ReviewPhase) {
    match phase {
        ReviewPhase::Summary => handle_summary_key(app, key, source),
        ReviewPhase::Practice => handle_practice_key(app, key, source),
    }
}

fn handle_summary_key(app: &mut App, key: KeyEvent, source: ReviewSource) {
    match key.code {
        KeyCode::Esc => return_to_topics(app, &source),
        KeyCode::Left | KeyCode::Char('h') => cycle_training_level(app, false),
        KeyCode::Right | KeyCode::Char('l') => cycle_training_level(app, true),
        KeyCode::Enter => {
            start_review_practice(app, &source);
            app.state = AppState::Review {
                source,
                phase: ReviewPhase::Practice,
            };
        }
        _ => {}
    }
}

fn handle_practice_key(app: &mut App, key: KeyEvent, source: ReviewSource) {
    if key.code == KeyCode::Esc {
        app.review_practice = ReviewPracticeState::default();
        app.state = AppState::Review {
            source,
            phase: ReviewPhase::Summary,
        };
        return;
    }

    if app.review_practice.completed {
        if key.code == KeyCode::Enter {
            app.review_practice = ReviewPracticeState::default();
            app.state = AppState::Review {
                source,
                phase: ReviewPhase::Summary,
            };
        }
        return;
    }

    let exercise = match app
        .review_practice
        .exercises
        .get(app.review_practice.current_index)
        .cloned()
    {
        Some(exercise) => exercise,
        None => {
            complete_review_practice(app, &source);
            return;
        }
    };

    if key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && key.code == KeyCode::Char('r')
    {
        if exercise.kind == ReviewExerciseKind::Typing {
            reset_typing_engine_for_current_exercise(app);
            app.review_practice.typing_showing_output = false;
        }
        return;
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return;
    }

    match exercise.kind {
        ReviewExerciseKind::Typing => handle_typing_key(app, key, &source, &exercise),
        ReviewExerciseKind::Cloze => handle_cloze_key(app, key, &source, &exercise),
        ReviewExerciseKind::Dictation => handle_dictation_key(app, key, &source, &exercise),
    }
}

fn handle_typing_key(
    app: &mut App,
    key: KeyEvent,
    source: &ReviewSource,
    exercise: &ReviewExercise,
) {
    match key.code {
        KeyCode::Char(_) if app.review_practice.typing_showing_output => {
            advance_review_practice(app, source);
            if !app.review_practice.completed {
                handle_practice_key(app, key, source.clone());
            }
        }
        KeyCode::Backspace if !app.review_practice.typing_showing_output => {
            app.typing_engine.backspace();
        }
        KeyCode::Char(c)
            if !app.review_practice.typing_showing_output && !app.typing_engine.is_complete() =>
        {
            app.typing_engine.input(c);
        }
        KeyCode::Enter if app.review_practice.typing_showing_output => {
            advance_review_practice(app, source);
        }
        KeyCode::Enter if app.typing_engine.is_complete() => {
            let record = app.typing_engine.finish(
                &exercise.command_id,
                exercise.difficulty,
                RecordMode::ReviewTyping,
            );
            app.review_practice.typing_count += 1;
            app.review_practice.typing_accuracy_sum += record.accuracy;
            app.review_practice.typing_wpm_sum += record.wpm;
            app.review_practice.accuracy_sum += record.accuracy;
            persist_review_record(app, record);
            app.review_practice.typing_showing_output = true;
        }
        _ => {}
    }
}

fn handle_cloze_key(
    app: &mut App,
    key: KeyEvent,
    source: &ReviewSource,
    exercise: &ReviewExercise,
) {
    match key.code {
        KeyCode::Backspace if !app.review_practice.cloze_submitted => {
            app.review_practice.cloze_input.pop();
        }
        KeyCode::Char(c) if !app.review_practice.cloze_submitted => {
            app.review_practice.cloze_input.push(c);
        }
        KeyCode::Enter if app.review_practice.cloze_submitted => {
            advance_review_practice(app, source);
        }
        KeyCode::Enter => {
            let correct = exercise
                .cloze_answer
                .as_deref()
                .is_some_and(|answer| app.review_practice.cloze_input.trim() == answer.trim());
            let accuracy = if correct { 1.0 } else { 0.0 };
            app.review_practice.cloze_count += 1;
            app.review_practice.cloze_accuracy_sum += accuracy;
            app.review_practice.accuracy_sum += accuracy;
            record_review_exercise_result(app, exercise, RecordMode::ReviewCloze, 0.0, accuracy);
            app.review_practice.cloze_correct = Some(correct);
            app.review_practice.cloze_submitted = true;
        }
        _ => {}
    }
}

fn handle_dictation_key(
    app: &mut App,
    key: KeyEvent,
    source: &ReviewSource,
    exercise: &ReviewExercise,
) {
    match key.code {
        KeyCode::Backspace if !app.review_practice.dictation_submitted => {
            app.review_practice.dictation_input.pop();
        }
        KeyCode::Char(c) if !app.review_practice.dictation_submitted => {
            app.review_practice.dictation_input.push(c);
        }
        KeyCode::Enter if app.review_practice.dictation_submitted => {
            advance_review_practice(app, source);
        }
        KeyCode::Enter => {
            let result = matcher::check(
                &app.review_practice.dictation_input,
                &exercise.accepted_answers,
            );
            let accuracy = match result {
                MatchResult::Exact(_) | MatchResult::Normalized(_) => 1.0,
                MatchResult::NoMatch { .. } => 0.0,
            };
            app.review_practice.dictation_count += 1;
            app.review_practice.dictation_accuracy_sum += accuracy;
            app.review_practice.accuracy_sum += accuracy;
            record_review_exercise_result(
                app,
                exercise,
                RecordMode::ReviewDictation,
                0.0,
                accuracy,
            );
            app.review_practice.dictation_result = Some(result);
            app.review_practice.dictation_submitted = true;
        }
        _ => {}
    }
}

fn cycle_training_level(app: &mut App, forward: bool) {
    let levels = TopicTrainingLevel::ALL;
    let current = levels
        .iter()
        .position(|level| *level == app.topic_training_level)
        .unwrap_or(0);
    let next = if forward {
        (current + 1).min(levels.len() - 1)
    } else {
        current.saturating_sub(1)
    };
    app.topic_training_level = levels[next];
}

fn return_to_topics(app: &mut App, source: &ReviewSource) {
    if let ReviewSource::CommandTopic(id) = source
        && let Some(index) = app
            .command_training_topics
            .iter()
            .position(|topic| topic.id == *id)
    {
        app.review_topics_index = index;
    }
    app.review_practice = ReviewPracticeState::default();
    app.state = AppState::ReviewTopics;
}

pub fn build_review_exercises(app: &App, source: &ReviewSource) -> Vec<ReviewExercise> {
    match source {
        ReviewSource::SymbolTopic(name) => build_symbol_review_exercises(app, name),
        ReviewSource::SystemTopic(name) => build_system_review_exercises(app, name),
        ReviewSource::CommandTopic(_) | ReviewSource::CommandCategory(_) => {
            let mut commands = canonical_commands_for_source(app, source);
            commands.sort_by_key(|command| command_times_practiced(app, &command.id));
            commands.truncate(10);
            commands
                .into_iter()
                .map(|command| exercise_from_command(command, app.topic_training_level))
                .collect()
        }
    }
}

fn canonical_commands_for_source<'a>(app: &'a App, source: &ReviewSource) -> Vec<&'a Command> {
    match source {
        ReviewSource::CommandTopic(id) => app
            .command_training_topics
            .iter()
            .find(|topic| topic.id == *id)
            .map(|topic| {
                topic
                    .command_ids
                    .iter()
                    .filter_map(|command_id| {
                        app.commands
                            .iter()
                            .find(|command| command.id == *command_id)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        ReviewSource::CommandCategory(category) => app
            .commands
            .iter()
            .filter(|command| command.category == *category)
            .collect(),
        ReviewSource::SymbolTopic(_) | ReviewSource::SystemTopic(_) => Vec::new(),
    }
}

fn build_symbol_review_exercises(app: &App, name: &str) -> Vec<ReviewExercise> {
    let Some(topic) = app
        .symbol_topics
        .iter()
        .find(|topic| topic.meta.id == name || topic.meta.topic == name)
    else {
        return Vec::new();
    };

    let mut exercises: Vec<ReviewExercise> = topic
        .exercises
        .iter()
        .enumerate()
        .filter_map(|(index, exercise)| {
            symbol_review_exercise(app, topic, exercise, index, app.topic_training_level)
        })
        .collect();
    exercises.sort_by_key(|exercise| command_times_practiced(app, &exercise.command_id));
    exercises.truncate(10);
    exercises
}

fn symbol_review_exercise(
    app: &App,
    topic: &SymbolTopic,
    exercise: &Exercise,
    exercise_index: usize,
    level: TopicTrainingLevel,
) -> Option<ReviewExercise> {
    if let Some(command_id) = exercise.command_id.as_deref()
        && let Some(command) = app.commands.iter().find(|command| command.id == command_id)
    {
        return Some(exercise_from_command(command, level));
    }

    let command = exercise
        .command
        .as_deref()
        .filter(|command| !command.trim().is_empty())
        .or_else(|| exercise.answers.first().map(String::as_str))?
        .to_string();
    let accepted_answers = if exercise.answers.is_empty() {
        vec![command.clone()]
    } else {
        exercise.answers.clone()
    };
    let command_id =
        crate::flow::symbol_flow::symbol_exercise_progress_key(topic, exercise, exercise_index);

    Some(exercise_from_inline(
        command_id,
        command,
        exercise.prompt.clone(),
        accepted_answers,
        exercise.simulated_output.clone(),
        topic.meta.difficulty,
        level,
    ))
}

fn build_system_review_exercises(app: &App, name: &str) -> Vec<ReviewExercise> {
    let Some(topic) = app
        .system_topics
        .iter()
        .find(|topic| topic.meta.id == name || topic.meta.topic == name)
    else {
        return Vec::new();
    };

    let mut exercises: Vec<ReviewExercise> = topic
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section_index, section)| {
            section
                .commands
                .iter()
                .enumerate()
                .filter_map(move |(command_index, command)| {
                    system_review_exercise(
                        app,
                        topic,
                        section,
                        command,
                        section_index,
                        command_index,
                        app.topic_training_level,
                    )
                })
        })
        .collect();
    exercises.sort_by_key(|exercise| command_times_practiced(app, &exercise.command_id));
    exercises.truncate(10);
    exercises
}

fn system_review_exercise(
    app: &App,
    topic: &SystemTopic,
    section: &SystemSection,
    command: &crate::data::models::SystemCommand,
    section_index: usize,
    command_index: usize,
    level: TopicTrainingLevel,
) -> Option<ReviewExercise> {
    if let Some(command_id) = command.command_id.as_deref()
        && let Some(canonical) = app
            .commands
            .iter()
            .find(|canonical| canonical.id == command_id)
    {
        return Some(exercise_from_command(canonical, level));
    }

    if command.command.trim().is_empty() {
        return None;
    }

    let command_id = crate::flow::system_flow::system_command_progress_key(
        topic,
        section,
        command,
        section_index,
        command_index,
    );
    Some(exercise_from_inline(
        command_id,
        command.command.clone(),
        command.summary.clone(),
        vec![command.command.clone()],
        command.simulated_output.clone(),
        topic.meta.difficulty,
        level,
    ))
}

fn exercise_from_inline(
    command_id: String,
    command: String,
    description: String,
    accepted_answers: Vec<String>,
    simulated_output: Option<String>,
    difficulty: crate::data::models::Difficulty,
    level: TopicTrainingLevel,
) -> ReviewExercise {
    let (kind, cloze_skeleton, cloze_answer) = match level {
        TopicTrainingLevel::L1 => (ReviewExerciseKind::Typing, None, None),
        TopicTrainingLevel::L3 => {
            let (skeleton, answer) = build_cloze(&command, &[]);
            (ReviewExerciseKind::Cloze, Some(skeleton), Some(answer))
        }
        TopicTrainingLevel::L5 => (ReviewExerciseKind::Dictation, None, None),
    };

    ReviewExercise {
        kind,
        command_id,
        command,
        display: None,
        description,
        accepted_answers,
        tokens: Vec::new(),
        simulated_output,
        difficulty,
        cloze_skeleton,
        cloze_answer,
    }
}

fn command_times_practiced(app: &App, command_id: &str) -> u32 {
    app.user_stats
        .command_progress
        .iter()
        .find(|progress| progress.command_id == command_id)
        .map(|progress| progress.times_practiced)
        .unwrap_or(0)
}

fn exercise_from_command(command: &Command, level: TopicTrainingLevel) -> ReviewExercise {
    let (kind, cloze_skeleton, cloze_answer) = match level {
        TopicTrainingLevel::L1 => (ReviewExerciseKind::Typing, None, None),
        TopicTrainingLevel::L3 => {
            let (skeleton, answer) = build_cloze(&command.command, &command.tokens);
            (ReviewExerciseKind::Cloze, Some(skeleton), Some(answer))
        }
        TopicTrainingLevel::L5 => (ReviewExerciseKind::Dictation, None, None),
    };

    ReviewExercise {
        kind,
        command_id: command.id.clone(),
        command: command.command.clone(),
        display: command.display.clone(),
        description: command.dictation.prompt.clone(),
        accepted_answers: command.dictation.answers.clone(),
        tokens: command.tokens.clone(),
        simulated_output: command.simulated_output.clone(),
        difficulty: command.difficulty,
        cloze_skeleton,
        cloze_answer,
    }
}

fn build_cloze(command: &str, tokens: &[Token]) -> (String, String) {
    let selected = tokens
        .iter()
        .enumerate()
        .min_by_key(|(index, token)| (cloze_priority(cloze_token_kind(*index, token)), *index));

    let Some((selected_index, selected_token)) = selected else {
        return (mask_token(command), command.trim().to_string());
    };

    let skeleton = tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            if index == selected_index {
                mask_token(&token.text)
            } else {
                token.text.clone()
            }
        })
        .collect::<String>();
    (skeleton, selected_token.text.trim().to_string())
}

fn cloze_token_kind(index: usize, token: &Token) -> TokenKind {
    token.kind.unwrap_or_else(|| {
        if index == 0 {
            TokenKind::Command
        } else {
            TokenKind::infer(&token.text)
        }
    })
}

fn cloze_priority(kind: TokenKind) -> u8 {
    match kind {
        TokenKind::Subcommand => 0,
        TokenKind::ShortOption | TokenKind::ShortOptionBundle | TokenKind::LongOption => 1,
        TokenKind::Operator | TokenKind::Pipe => 2,
        TokenKind::Redirection => 3,
        TokenKind::Pattern | TokenKind::Regex => 4,
        TokenKind::Path | TokenKind::Filename | TokenKind::Directory => 5,
        TokenKind::OptionValue
        | TokenKind::Literal
        | TokenKind::QuotedExpr
        | TokenKind::Variable
        | TokenKind::Substitution
        | TokenKind::Placeholder
        | TokenKind::PermissionMode
        | TokenKind::Number
        | TokenKind::ServiceName
        | TokenKind::Url => 6,
        TokenKind::Other => 7,
        TokenKind::Command => 8,
    }
}

fn mask_token(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let leading = chars.iter().take_while(|ch| ch.is_whitespace()).count();
    if leading == chars.len() {
        return text.to_string();
    }
    let trailing = chars
        .iter()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();
    let content_end = chars.len().saturating_sub(trailing);
    let mut masked = chars[..leading].iter().collect::<String>();
    masked.push_str("____");
    masked.extend(chars[content_end..].iter());
    masked
}

fn start_review_practice(app: &mut App, source: &ReviewSource) {
    let exercises = build_review_exercises(app, source);
    let total_count = exercises.len();
    app.review_practice = ReviewPracticeState {
        exercises,
        total_count,
        completed: total_count == 0,
        ..ReviewPracticeState::default()
    };

    reset_typing_engine_for_current_exercise(app);
}

fn record_review_exercise_result(
    app: &mut App,
    exercise: &ReviewExercise,
    mode: RecordMode,
    wpm: f64,
    accuracy: f64,
) {
    let now_ms = Utc::now().timestamp_millis();
    let record = SessionRecord {
        typing: None,
        id: format!("{}-{}", now_ms, exercise.command_id),
        command_id: exercise.command_id.clone(),
        mode,
        keystrokes: Vec::new(),
        started_at: now_ms,
        finished_at: now_ms,
        wpm,
        cpm: wpm * 5.0,
        accuracy,
        error_count: u32::from(accuracy < 1.0),
        difficulty: exercise.difficulty,
    };
    persist_review_record(app, record);
}

fn persist_review_record(app: &mut App, record: SessionRecord) {
    app.persist_record(record);
}

fn record_review_stats(app: &mut App) {
    if app.review_practice.stats_recorded {
        return;
    }

    let _ = app.progress_store.save_stats(&app.user_stats);
    app.review_practice.stats_recorded = true;
}

fn complete_review_practice(app: &mut App, source: &ReviewSource) {
    app.review_practice.completed = true;
    record_review_stats(app);
    app.state = AppState::Review {
        source: source.clone(),
        phase: ReviewPhase::Practice,
    };
}

fn advance_review_practice(app: &mut App, source: &ReviewSource) {
    app.review_practice.current_index += 1;
    app.review_practice.typing_showing_output = false;
    app.review_practice.cloze_input.clear();
    app.review_practice.cloze_correct = None;
    app.review_practice.cloze_submitted = false;
    app.review_practice.dictation_input.clear();
    app.review_practice.dictation_result = None;
    app.review_practice.dictation_submitted = false;

    if app.review_practice.current_index >= app.review_practice.exercises.len() {
        complete_review_practice(app, source);
        return;
    }

    reset_typing_engine_for_current_exercise(app);
    app.state = AppState::Review {
        source: source.clone(),
        phase: ReviewPhase::Practice,
    };
}

fn reset_typing_engine_for_current_exercise(app: &mut App) {
    if let Some(exercise) = app
        .review_practice
        .exercises
        .get(app.review_practice.current_index)
        && matches!(exercise.kind, ReviewExerciseKind::Typing)
    {
        app.typing_engine.reset(&exercise.command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(text: &str) -> Token {
        Token {
            text: text.to_string(),
            desc: String::new(),
            kind: None,
        }
    }

    #[test]
    fn l3_cloze_masks_trimmed_prioritized_token_instead_of_find() {
        let command = "find ~/Documents -type f -name '*.pdf'";
        let tokens = vec![
            token("find"),
            token(" ~/Documents"),
            token(" -type f"),
            token(" -name"),
            token(" '*.pdf'"),
        ];

        let (skeleton, answer) = build_cloze(command, &tokens);

        assert_eq!(answer, "-name");
        assert_ne!(answer, "find");
        assert_eq!(skeleton, "find ~/Documents -type f ____ '*.pdf'");
        assert_eq!(skeleton.replacen("____", &answer, 1), command);
    }

    #[test]
    fn l3_cloze_treats_unannotated_first_token_as_command() {
        let command = "customcmd argument";
        let tokens = vec![token("customcmd"), token(" argument")];

        let (skeleton, answer) = build_cloze(command, &tokens);

        assert_eq!(answer, "argument");
        assert_eq!(skeleton, "customcmd ____");
    }

    #[test]
    fn l5_dictation_preserves_whitespace_inside_quotes() {
        let mut command = Command {
            id: "echo-build-complete".to_string(),
            command: "echo 'build complete'".to_string(),
            ..Command::default()
        };
        command.dictation.answers = vec![command.command.clone()];
        let exercise = exercise_from_command(&command, TopicTrainingLevel::L5);

        assert_eq!(exercise.kind, ReviewExerciseKind::Dictation);
        assert_eq!(
            matcher::check("echo  'build complete'", &exercise.accepted_answers),
            MatchResult::Normalized(0)
        );
        assert!(matches!(
            matcher::check("echo 'build  complete'", &exercise.accepted_answers),
            MatchResult::NoMatch { .. }
        ));
    }
}
