use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::engine::TypingEngine;
use crate::core::matcher::{self, MatchResult};
use crate::core::scorer;
use crate::core::terminal_history::TerminalHistory;
use crate::data::catalog;
use crate::data::command_loader;
use crate::data::lesson_loader;
use crate::data::models::*;
use crate::data::progress::ProgressStore;
use crate::data::symbol_loader;
use crate::data::system_loader;

// ─────────────────────────────────────────────────────────────
// AppState + sub-enums
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Home,
    Typing,
    TypingFilter,
    LearnHub,
    Calendar,
    Scenarios,
    ScenarioPractice,
    PracticeTopics,
    PracticeGroup,
    CommandTopics,
    CommandLessonOverview {
        category_index: usize,
        command_index: usize,
        scroll: usize,
    },
    CommandLessonPractice {
        category_index: usize,
        command_index: usize,
        example_index: usize,
    },
    SymbolTopics,
    SymbolLesson {
        topic_index: usize,
        symbol_index: usize,
        phase: SymbolPhase,
    },
    SystemTopics,
    ReviewTopics,
    SystemLesson {
        topic_index: usize,
        section_index: usize,
        phase: SystemPhase,
        scroll: usize,
    },
    DeepExplanation {
        source: DeepSource,
        scroll: usize,
    },
    Review {
        source: ReviewSource,
        phase: ReviewPhase,
    },
    Dictation,
    Stats,
    Settings,
    Quitting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolPhase {
    Explain,
    Example(usize),
    TypingPractice { exercise_idx: usize },
    Practice,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemPhase {
    Overview,
    Detail,
    TypingPractice { command_idx: usize },
    ConfigFile(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewSource {
    CommandTopic(String),
    CommandCategory(Category),
    SymbolTopic(String),
    SystemTopic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewPhase {
    Summary,
    Practice,
}

const LEARN_HUB_ITEM_COUNT: usize = 6;

#[derive(Debug, Clone, Default)]
pub struct SymbolPracticeState {
    pub current_index: usize,
    pub current_input: String,
    pub error_count: u8,
    pub show_answer: bool,
    pub submitted: bool,
    pub last_correct: Option<bool>,

    pub typing_indices: Vec<usize>,
    pub dictation_indices: Vec<usize>,
    pub total_count: usize,

    pub typing_showing_output: bool,
    pub typing_wpm_sum: f64,
    pub typing_accuracy_sum: f64,
    pub typing_count: usize,

    pub dictation_correct_count: usize,
    pub dictation_accuracy_sum: f64,
    pub dictation_count: usize,

    pub completed: bool,
    pub stats_recorded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewExerciseKind {
    Typing,
    Cloze,
    Dictation,
}

#[derive(Debug, Clone)]
pub struct ReviewExercise {
    pub kind: ReviewExerciseKind,
    pub command_id: String,
    pub command: String,
    pub display: Option<String>,
    pub description: String,
    pub accepted_answers: Vec<String>,
    pub tokens: Vec<Token>,
    pub simulated_output: Option<String>,
    pub difficulty: Difficulty,
    pub cloze_skeleton: Option<String>,
    pub cloze_answer: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ReviewPracticeState {
    pub exercises: Vec<ReviewExercise>,
    pub current_index: usize,
    pub typing_showing_output: bool,
    pub cloze_input: String,
    pub cloze_correct: Option<bool>,
    pub cloze_submitted: bool,
    pub dictation_input: String,
    pub dictation_result: Option<MatchResult>,
    pub dictation_submitted: bool,
    pub accuracy_sum: f64,
    pub total_count: usize,
    pub typing_wpm_sum: f64,
    pub typing_accuracy_sum: f64,
    pub typing_count: usize,
    pub cloze_accuracy_sum: f64,
    pub cloze_count: usize,
    pub dictation_accuracy_sum: f64,
    pub dictation_count: usize,
    pub completed: bool,
    pub stats_recorded: bool,
}

// ─────────────────────────────────────────────────────────────
// App struct
// ─────────────────────────────────────────────────────────────

pub struct App {
    pub state: AppState,

    // Data
    pub commands: Vec<Command>,
    pub command_prompts: std::collections::HashMap<String, String>,
    pub command_training_topics: Vec<CommandTrainingTopic>,
    pub lessons: Vec<CommandLesson>,
    pub symbol_topics: Vec<SymbolTopic>,
    pub system_topics: Vec<SystemTopic>,

    pub sequences: Vec<crate::data::sequence_loader::CommandSequence>,
    pub workflow_state: crate::flow::workflow_flow::WorkflowState,
    pub scenario_catalog: Vec<crate::data::scenario_loader::Scenario>,
    pub scenario_state: crate::data::scenario_loader::ScenarioState,
    pub practice_groups: Vec<crate::data::practice_loader::PracticeGroup>,
    pub practice_state: crate::flow::practice_flow::PracticeState,
    pub calendar_state: crate::ui::calendar::CalendarState,
    pub persistence_error: Option<String>,

    // User data
    pub user_stats: UserStats,
    pub user_config: UserConfig,
    pub progress_store: ProgressStore,
    pub history: Vec<SessionRecord>,

    // Core engines
    pub typing_engine: TypingEngine,
    pub terminal_history: TerminalHistory,

    // Typing mode state
    pub typing_commands: Vec<Command>,
    pub typing_index: usize,
    pub show_hint: bool,
    pub typing_showing_output: bool,
    pub terminal_auto_advance: bool,
    pub typing_mode: TypingDisplayMode,
    pub filter_difficulty: Option<Difficulty>,
    pub filter_category: Option<Category>,
    pub typing_filter_row: usize,
    pub review_topics_index: usize,
    pub topic_training_level: TopicTrainingLevel,

    // Dictation mode state
    pub dictation_commands: Vec<Command>,
    pub dictation_index: usize,
    pub dictation_input: String,
    pub dictation_result: Option<MatchResult>,
    pub dictation_submitted: bool,

    // Symbol practice state
    pub symbol_practice: SymbolPracticeState,

    // System lesson typing phase state
    pub system_typing_showing_output: bool,

    // Review practice state
    pub review_practice: ReviewPracticeState,

    // Menu navigation indices
    pub home_index: usize,
    pub learn_hub_index: usize,
    pub command_topics_index: usize,
    pub command_list_index: usize,
    pub symbol_topics_index: usize,
    pub system_topics_index: usize,
    pub system_section_index: usize,
    pub settings_index: usize,
    pub stats_tab: usize,

    // Current typing round records (for completion summary)
    pub typing_round_records: Vec<SessionRecord>,

    // Lesson command list indices (per category)
    pub lesson_commands_for_category: Vec<CommandLesson>,
}

impl App {
    fn detect_data_dir() -> Result<PathBuf> {
        let home_data_dir = env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .map(|home| home.join(".local/share/cmdtyper/data"));
        let candidates = [
            env::var_os("CMDTYPER_DATA_DIR").map(PathBuf::from),
            home_data_dir,
            Some(PathBuf::from("/usr/local/share/cmdtyper/data")),
            Some(PathBuf::from("./data")),
        ];

        candidates
            .into_iter()
            .flatten()
            .find(|path| Self::is_complete_data_dir(path))
            .ok_or_else(|| {
                anyhow!(
                    "no complete cmdtyper data directory found; expected commands, lessons, symbols, and system"
                )
            })
    }

    fn is_complete_data_dir(path: &Path) -> bool {
        ["commands", "lessons", "symbols", "system"]
            .into_iter()
            .all(|directory| path.join(directory).is_dir())
    }

    pub fn new() -> Result<Self> {
        let data_dir = Self::detect_data_dir()?;
        let command_catalog = command_loader::load_command_catalog(&data_dir)?;
        let mut lessons = lesson_loader::load_lessons(&data_dir)?;
        let mut symbol_topics = symbol_loader::load_symbol_topics(&data_dir)?;
        let mut system_topics = system_loader::load_system_topics(&data_dir)?;
        catalog::hydrate_and_validate_content(
            &command_catalog,
            &mut lessons,
            &mut symbol_topics,
            &mut system_topics,
        )?;
        let command_prompts =
            command_loader::load_command_prompts(&data_dir, &command_catalog.commands)?;
        let sequences =
            crate::data::sequence_loader::load_sequences(&data_dir, &command_catalog.commands)?;
        let aliases =
            crate::data::aliases::CommandAliases::load(&data_dir, &command_catalog.commands)?;
        let practice_groups = crate::data::practice_loader::load_practice_groups(
            &data_dir,
            &command_catalog.commands,
        )?;
        crate::data::practice_loader::validate_practice_sources(
            &practice_groups,
            &lessons,
            &symbol_topics,
            &system_topics,
        )?;
        let mut scenario_catalog =
            crate::data::scenario_loader::load_scenarios(&data_dir.join("scenarios"))?;
        crate::data::scenario_loader::hydrate_scenarios(
            &mut scenario_catalog,
            &command_catalog.commands,
        )?;
        let commands = command_catalog.commands;
        let command_training_topics = command_catalog.topics;

        let progress_store = ProgressStore::new()?;
        aliases.migrate(&progress_store)?;
        let user_stats = progress_store.migrate_stats()?;
        let user_config = progress_store.load_config()?;
        let typing_mode = user_config.typing_mode.clone();
        let history = progress_store.load_history()?;

        let scenario_state = crate::data::scenario_loader::load_state(progress_store.base_dir());
        let mut app = Self {
            state: AppState::Home,
            commands,
            command_prompts,
            command_training_topics,
            lessons,
            symbol_topics,
            system_topics,
            sequences,
            workflow_state: crate::flow::workflow_flow::WorkflowState::default(),
            scenario_catalog,
            scenario_state,
            practice_groups,
            practice_state: Default::default(),
            calendar_state: Default::default(),
            persistence_error: None,
            user_stats,
            user_config,
            progress_store,
            history,
            typing_engine: TypingEngine::new(""),
            terminal_history: TerminalHistory::new(),
            typing_commands: Vec::new(),
            typing_index: 0,
            show_hint: true,
            typing_showing_output: false,
            terminal_auto_advance: false,
            typing_mode,
            filter_difficulty: None,
            filter_category: None,
            typing_filter_row: 0,
            review_topics_index: 0,
            topic_training_level: TopicTrainingLevel::default(),
            dictation_commands: Vec::new(),
            dictation_index: 0,
            dictation_input: String::new(),
            dictation_result: None,
            dictation_submitted: false,
            symbol_practice: SymbolPracticeState::default(),
            system_typing_showing_output: false,
            review_practice: ReviewPracticeState::default(),
            home_index: 0,
            learn_hub_index: 0,
            command_topics_index: 0,
            command_list_index: 0,
            symbol_topics_index: 0,
            system_topics_index: 0,
            system_section_index: 0,
            settings_index: 0,
            stats_tab: 0,
            typing_round_records: Vec::new(),
            lesson_commands_for_category: Vec::new(),
        };
        let resume = app.progress_store.load_resume_state().unwrap_or_default();
        app.apply_resume_state(resume);
        Ok(app)
    }

    pub fn practice_counts(&self, kind: &str, source: &str) -> (usize, usize, usize) {
        let groups: Vec<_> = self
            .practice_groups
            .iter()
            .filter(|g| g.source_kind == kind && g.source_id == source)
            .collect();
        let ids: std::collections::HashSet<_> = groups
            .iter()
            .flat_map(|g| g.exercise_command_ids.iter())
            .collect();
        let completed = ids
            .iter()
            .filter(|id| {
                self.user_stats
                    .command_progress
                    .iter()
                    .any(|p| &p.command_id == **id && p.times_practiced > 0)
            })
            .count();
        (groups.len(), ids.len(), completed)
    }

    pub fn is_follow_typing_screen(&self) -> bool {
        matches!(
            self.state,
            AppState::Typing | AppState::CommandLessonPractice { .. } | AppState::PracticeGroup
        ) || matches!(
            self.state,
            AppState::SymbolLesson {
                phase: SymbolPhase::TypingPractice { .. },
                ..
            } | AppState::SystemLesson {
                phase: SystemPhase::TypingPractice { .. },
                ..
            }
        ) || (matches!(
            self.state,
            AppState::Review {
                phase: ReviewPhase::Practice,
                ..
            }
        ) && self
            .review_practice
            .exercises
            .get(self.review_practice.current_index)
            .is_some_and(|e| e.kind == ReviewExerciseKind::Typing))
            || (self.state == AppState::ScenarioPractice
                && self.scenario_state.phase == crate::data::scenario_loader::ScenarioPhase::Typing)
    }

    /// Persist history first; statistics are a recoverable cache of stable session IDs.
    pub fn persist_record(&mut self, record: SessionRecord) -> bool {
        if self.history.iter().any(|old| old == &record) {
            if self.persistence_error.is_some() {
                self.persistence_error = self
                    .progress_store
                    .save_stats(&self.user_stats)
                    .err()
                    .map(|e| format!("统计保存失败：{e}"));
            }
            return true;
        }
        if let Err(error) = self.progress_store.append_record(&record) {
            self.persistence_error = Some(format!("练习记录保存失败：{error}"));
            return false;
        }
        let previous_progress = self.user_stats.command_progress.clone();
        if let Some(existing) = self.history.iter_mut().find(|old| old.id == record.id) {
            *existing = record;
            self.user_stats = match self.progress_store.stats_for_history(&self.history) {
                Ok(stats) => stats,
                Err(error) => {
                    self.persistence_error = Some(format!("统计恢复失败：{error}"));
                    return true;
                }
            };
            for old in previous_progress {
                if let Some(current) = self
                    .user_stats
                    .command_progress
                    .iter_mut()
                    .find(|p| p.command_id == old.command_id)
                {
                    current.times_practiced = current.times_practiced.max(old.times_practiced);
                    current.best_wpm = current.best_wpm.max(old.best_wpm);
                    current.best_accuracy = current.best_accuracy.max(old.best_accuracy);
                    current.last_practiced = current.last_practiced.max(old.last_practiced);
                    current.mastery = current.mastery.max(old.mastery);
                } else {
                    self.user_stats.command_progress.push(old);
                }
            }
        } else {
            scorer::update_stats(&mut self.user_stats, &record);
            self.history.push(record);
        }
        self.persistence_error = self
            .progress_store
            .save_stats(&self.user_stats)
            .err()
            .map(|error| format!("统计保存失败：{error}"));
        true
    }

    pub fn active_typing_identity(&self) -> Option<(String, Difficulty, RecordMode)> {
        match &self.state {
            AppState::Typing => self
                .current_typing_command()
                .map(|c| (c.id.clone(), c.difficulty, RecordMode::Typing)),
            AppState::CommandLessonPractice {
                category_index,
                command_index,
                example_index,
            } => {
                let categories = self.get_lesson_categories();
                let lessons = self.get_lessons_for_category(*categories.get(*category_index)?);
                let lesson = lessons.get(*command_index)?;
                let example = lesson.examples.get(*example_index)?;
                Some((
                    lesson_example_progress_key(lesson, *example_index),
                    self.effective_record_difficulty(
                        example.command_id.as_deref(),
                        lesson.meta.difficulty,
                    ),
                    RecordMode::LessonPractice,
                ))
            }
            AppState::PracticeGroup if !self.practice_state.done => {
                let command = crate::flow::practice_flow::current_command(self)?;
                let mode = crate::flow::practice_flow::current_record_mode(self)?;
                Some((command.id.clone(), command.difficulty, mode))
            }
            AppState::Review {
                phase: ReviewPhase::Practice,
                ..
            } => self
                .review_practice
                .exercises
                .get(self.review_practice.current_index)
                .filter(|e| e.kind == ReviewExerciseKind::Typing && !self.review_practice.completed)
                .map(|e| (e.command_id.clone(), e.difficulty, RecordMode::ReviewTyping)),
            AppState::SymbolLesson {
                topic_index,
                phase: SymbolPhase::TypingPractice { exercise_idx },
                ..
            } => {
                let topic = self.symbol_topics.get(*topic_index)?;
                let raw = *self.symbol_practice.typing_indices.get(*exercise_idx)?;
                let exercise = topic.exercises.get(raw)?;
                Some((
                    crate::flow::symbol_flow::symbol_exercise_progress_key(topic, exercise, raw),
                    self.effective_record_difficulty(
                        exercise.command_id.as_deref(),
                        topic.meta.difficulty,
                    ),
                    RecordMode::SymbolTyping,
                ))
            }
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase: SystemPhase::TypingPractice { command_idx },
                ..
            } => {
                let topic = self.system_topics.get(*topic_index)?;
                let section = topic.sections.get(*section_index)?;
                let command = section.commands.get(*command_idx)?;
                Some((
                    crate::flow::system_flow::system_command_progress_key(
                        topic,
                        section,
                        command,
                        *section_index,
                        *command_idx,
                    ),
                    self.effective_record_difficulty(
                        command.command_id.as_deref(),
                        topic.meta.difficulty,
                    ),
                    RecordMode::SystemTyping,
                ))
            }
            AppState::ScenarioPractice
                if self.scenario_state.phase
                    == crate::data::scenario_loader::ScenarioPhase::Typing =>
            {
                let scenario = crate::flow::scenario_flow::active_scenario(self)?;
                let step = scenario.steps.get(self.scenario_state.step_index)?;
                Some((
                    step.command_id
                        .clone()
                        .unwrap_or_else(|| format!("scenario:{}:{}", scenario.id, step.id)),
                    scenario.difficulty,
                    RecordMode::ScenarioTyping,
                ))
            }
            _ => None,
        }
    }

    pub fn save_active_typing(&mut self) -> bool {
        if self.typing_engine.has_activity() {
            if let Some((id, difficulty, mode)) = self.active_typing_identity() {
                self.typing_engine.pause();
                let record = self.typing_engine.finish(&id, difficulty, mode);
                return self.persist_record(record);
            }
        }
        true
    }

    pub fn save_resume_state(&self) {
        let _ = self
            .progress_store
            .save_resume_state(&self.current_resume_state());
    }

    fn current_resume_state(&self) -> ResumeState {
        match &self.state {
            AppState::Home => ResumeState {
                screen: ResumeScreen::Home,
                ..ResumeState::default()
            },
            AppState::LearnHub => ResumeState {
                screen: ResumeScreen::LearnHub,
                ..ResumeState::default()
            },
            AppState::CommandTopics => ResumeState {
                screen: ResumeScreen::CommandTopics,
                category_index: self.command_topics_index,
                ..ResumeState::default()
            },
            AppState::CommandLessonOverview {
                category_index,
                command_index,
                scroll,
            } => ResumeState {
                screen: ResumeScreen::CommandLessonOverview,
                category_index: *category_index,
                command_index: *command_index,
                overview_scroll: *scroll,
                lesson_command: self.lesson_command_at(*category_index, *command_index),
                ..ResumeState::default()
            },
            AppState::CommandLessonPractice {
                category_index,
                command_index,
                example_index,
            } => ResumeState {
                screen: ResumeScreen::CommandLessonPractice,
                category_index: *category_index,
                command_index: *command_index,
                example_index: *example_index,
                lesson_command: self.lesson_command_at(*category_index, *command_index),
                ..ResumeState::default()
            },
            AppState::SymbolTopics => ResumeState {
                screen: ResumeScreen::SymbolTopics,
                topic_index: self.symbol_topics_index,
                symbol_topic_id: self
                    .symbol_topics
                    .get(self.symbol_topics_index)
                    .map(|topic| topic.meta.id.clone()),
                ..ResumeState::default()
            },
            AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase: SymbolPhase::Explain,
            } => ResumeState {
                screen: ResumeScreen::SymbolExplain,
                topic_index: *topic_index,
                symbol_index: *symbol_index,
                symbol_topic_id: self
                    .symbol_topics
                    .get(*topic_index)
                    .map(|topic| topic.meta.id.clone()),
                symbol_id: self
                    .symbol_topics
                    .get(*topic_index)
                    .and_then(|topic| topic.symbols.get(*symbol_index))
                    .map(|symbol| symbol.id.clone()),
                ..ResumeState::default()
            },
            AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase: SymbolPhase::Example(example_index),
            } => ResumeState {
                screen: ResumeScreen::SymbolExample,
                topic_index: *topic_index,
                symbol_index: *symbol_index,
                example_index: *example_index,
                symbol_topic_id: self
                    .symbol_topics
                    .get(*topic_index)
                    .map(|topic| topic.meta.id.clone()),
                symbol_id: self
                    .symbol_topics
                    .get(*topic_index)
                    .and_then(|topic| topic.symbols.get(*symbol_index))
                    .map(|symbol| symbol.id.clone()),
                ..ResumeState::default()
            },
            AppState::SymbolLesson { topic_index, .. } => ResumeState {
                screen: ResumeScreen::SymbolTopics,
                topic_index: *topic_index,
                symbol_topic_id: self
                    .symbol_topics
                    .get(*topic_index)
                    .map(|topic| topic.meta.id.clone()),
                ..ResumeState::default()
            },
            AppState::SystemTopics => ResumeState {
                screen: ResumeScreen::SystemTopics,
                topic_index: self.system_topics_index,
                system_topic_id: self
                    .system_topics
                    .get(self.system_topics_index)
                    .map(|topic| topic.meta.id.clone()),
                ..ResumeState::default()
            },
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase,
                ..
            } => ResumeState {
                screen: if matches!(phase, SystemPhase::Overview) {
                    ResumeScreen::SystemOverview
                } else {
                    ResumeScreen::SystemDetail
                },
                topic_index: *topic_index,
                section_index: *section_index,
                system_topic_id: self
                    .system_topics
                    .get(*topic_index)
                    .map(|topic| topic.meta.id.clone()),
                system_section_id: self
                    .system_topics
                    .get(*topic_index)
                    .and_then(|topic| topic.sections.get(*section_index))
                    .map(|section| section.id.clone()),
                ..ResumeState::default()
            },
            AppState::ReviewTopics => {
                self.review_resume_state(self.review_topics_index, ResumeScreen::ReviewTopics)
            }
            AppState::Review { source, .. } => {
                let topic_index = self
                    .review_topic_index_for_source(source)
                    .unwrap_or(self.review_topics_index);
                self.review_resume_state(topic_index, ResumeScreen::ReviewSummary)
            }
            AppState::Dictation => ResumeState {
                screen: ResumeScreen::Dictation,
                ..ResumeState::default()
            },
            AppState::Stats => ResumeState {
                screen: ResumeScreen::Stats,
                ..ResumeState::default()
            },
            AppState::Settings => ResumeState {
                screen: ResumeScreen::Settings,
                ..ResumeState::default()
            },
            _ => ResumeState::default(),
        }
    }

    fn apply_resume_state(&mut self, resume: ResumeState) {
        match resume.screen {
            ResumeScreen::Home => self.state = AppState::Home,
            ResumeScreen::ReviewTopics => {
                self.review_topics_index = self.resolve_review_topic_index(&resume).unwrap_or(0);
                self.topic_training_level = resume.topic_training_level;
                self.state = AppState::ReviewTopics;
            }
            ResumeScreen::ReviewSummary => {
                self.topic_training_level = resume.topic_training_level;
                self.review_practice = ReviewPracticeState::default();
                if let Some(topic_index) = self.resolve_review_topic_index(&resume)
                    && let Some(source) = self.review_source_for_topic_index(topic_index)
                {
                    self.review_topics_index = topic_index;
                    self.state = AppState::Review {
                        source,
                        phase: ReviewPhase::Summary,
                    };
                } else {
                    self.review_topics_index = 0;
                    self.state = AppState::ReviewTopics;
                }
            }
            ResumeScreen::LearnHub => self.state = AppState::LearnHub,
            ResumeScreen::CommandTopics => self.restore_command_topics(resume.category_index),
            ResumeScreen::CommandLessonOverview => {
                if let Some((category_index, command_index)) = self.resolve_lesson_location(&resume)
                {
                    self.prepare_lesson_category(category_index);
                    self.state = AppState::CommandLessonOverview {
                        category_index,
                        command_index,
                        scroll: resume.overview_scroll,
                    };
                } else {
                    let category_index = if resume.lesson_command.is_none() {
                        resume.category_index
                    } else {
                        0
                    };
                    self.restore_command_topics(category_index);
                }
            }
            ResumeScreen::CommandLessonPractice => {
                if let Some((category_index, command_index)) = self.resolve_lesson_location(&resume)
                {
                    self.prepare_lesson_category(category_index);
                    let command =
                        self.get_lesson_categories()
                            .get(category_index)
                            .and_then(|category| {
                                self.get_lessons_for_category(*category)
                                    .get(command_index)
                                    .and_then(|lesson| lesson.examples.get(resume.example_index))
                                    .map(|example| example.command.clone())
                            });
                    if let Some(command) = command {
                        self.typing_engine.reset(&command);
                        self.state = AppState::CommandLessonPractice {
                            category_index,
                            command_index,
                            example_index: resume.example_index,
                        };
                    } else {
                        self.state = AppState::CommandLessonOverview {
                            category_index,
                            command_index,
                            scroll: 0,
                        };
                    }
                } else {
                    let category_index = if resume.lesson_command.is_none() {
                        resume.category_index
                    } else {
                        0
                    };
                    self.restore_command_topics(category_index);
                }
            }
            ResumeScreen::SymbolTopics => self.restore_symbol_topics(&resume),
            ResumeScreen::SymbolExplain | ResumeScreen::SymbolExample => {
                if let Some(topic_index) = self.resolve_symbol_topic(&resume) {
                    self.symbol_topics_index = topic_index;
                    if let Some(symbol_index) = self.resolve_symbol_index(topic_index, &resume) {
                        let phase = if resume.screen == ResumeScreen::SymbolExample
                            && self.symbol_topics[topic_index].symbols[symbol_index]
                                .examples
                                .get(resume.example_index)
                                .is_some()
                        {
                            SymbolPhase::Example(resume.example_index)
                        } else {
                            SymbolPhase::Explain
                        };
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index,
                            phase,
                        };
                    } else {
                        self.state = AppState::SymbolTopics;
                    }
                } else {
                    self.restore_symbol_topics(&resume);
                }
            }
            ResumeScreen::SystemTopics => self.restore_system_topics(&resume),
            ResumeScreen::SystemOverview => {
                if let Some(topic_index) = self.resolve_system_topic(&resume) {
                    self.system_topics_index = topic_index;
                    self.state = AppState::SystemLesson {
                        topic_index,
                        section_index: 0,
                        phase: SystemPhase::Overview,
                        scroll: 0,
                    };
                } else {
                    self.restore_system_topics(&resume);
                }
            }
            ResumeScreen::SystemDetail => {
                if let Some(topic_index) = self.resolve_system_topic(&resume) {
                    self.system_topics_index = topic_index;
                    if let Some(section_index) = self.resolve_system_section(topic_index, &resume) {
                        self.system_section_index = section_index;
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::Detail,
                            scroll: 0,
                        };
                    } else {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index: 0,
                            phase: SystemPhase::Overview,
                            scroll: 0,
                        };
                    }
                } else {
                    self.restore_system_topics(&resume);
                }
            }
            ResumeScreen::Dictation => self.enter_dictation(),
            ResumeScreen::Stats => self.state = AppState::Stats,
            ResumeScreen::Settings => self.state = AppState::Settings,
        }
    }

    fn lesson_command_at(&self, category_index: usize, command_index: usize) -> Option<String> {
        let category = self.get_lesson_categories().get(category_index).copied()?;
        self.get_lessons_for_category(category)
            .get(command_index)
            .map(|lesson| lesson.meta.command.clone())
    }

    fn resolve_lesson_location(&self, resume: &ResumeState) -> Option<(usize, usize)> {
        let categories = self.get_lesson_categories();
        if let Some(command) = resume.lesson_command.as_deref() {
            for (category_index, category) in categories.iter().enumerate() {
                let lessons = self.get_lessons_for_category(*category);
                if let Some(command_index) = lessons
                    .iter()
                    .position(|lesson| lesson.meta.command == command)
                {
                    return Some((category_index, command_index));
                }
            }
            return None;
        }

        let category = categories.get(resume.category_index)?;
        let lessons = self.get_lessons_for_category(*category);
        lessons
            .get(resume.command_index)
            .map(|_| (resume.category_index, resume.command_index))
    }

    fn prepare_lesson_category(&mut self, category_index: usize) {
        if let Some(category) = self.get_lesson_categories().get(category_index).copied() {
            self.command_topics_index = category_index;
            self.lesson_commands_for_category = self
                .lessons
                .iter()
                .filter(|lesson| lesson.meta.category == category)
                .cloned()
                .collect();
        }
    }

    fn restore_command_topics(&mut self, category_index: usize) {
        let count = self.get_lesson_categories().len();
        if count == 0 {
            self.state = AppState::LearnHub;
        } else {
            self.command_topics_index = category_index.min(count - 1);
            self.state = AppState::CommandTopics;
        }
    }

    fn resolve_symbol_topic(&self, resume: &ResumeState) -> Option<usize> {
        match resume.symbol_topic_id.as_deref() {
            Some(id) => self
                .symbol_topics
                .iter()
                .position(|topic| topic.meta.id == id),
            None => self
                .symbol_topics
                .get(resume.topic_index)
                .map(|_| resume.topic_index),
        }
    }

    fn resolve_symbol_index(&self, topic_index: usize, resume: &ResumeState) -> Option<usize> {
        let topic = self.symbol_topics.get(topic_index)?;
        match resume.symbol_id.as_deref() {
            Some(id) => topic.symbols.iter().position(|symbol| symbol.id == id),
            None => topic
                .symbols
                .get(resume.symbol_index)
                .map(|_| resume.symbol_index),
        }
    }

    fn restore_symbol_topics(&mut self, resume: &ResumeState) {
        self.symbol_topics_index = self.resolve_symbol_topic(resume).unwrap_or(0);
        self.state = AppState::SymbolTopics;
    }

    fn resolve_system_topic(&self, resume: &ResumeState) -> Option<usize> {
        match resume.system_topic_id.as_deref() {
            Some(id) => self
                .system_topics
                .iter()
                .position(|topic| topic.meta.id == id),
            None => self
                .system_topics
                .get(resume.topic_index)
                .map(|_| resume.topic_index),
        }
    }

    fn resolve_system_section(&self, topic_index: usize, resume: &ResumeState) -> Option<usize> {
        let topic = self.system_topics.get(topic_index)?;
        match resume.system_section_id.as_deref() {
            Some(id) => topic.sections.iter().position(|section| section.id == id),
            None => topic
                .sections
                .get(resume.section_index)
                .map(|_| resume.section_index),
        }
    }

    fn restore_system_topics(&mut self, resume: &ResumeState) {
        self.system_topics_index = self.resolve_system_topic(resume).unwrap_or(0);
        self.state = AppState::SystemTopics;
    }

    fn review_resume_state(&self, topic_index: usize, screen: ResumeScreen) -> ResumeState {
        let topic_index = topic_index.min(self.command_training_topics.len().saturating_sub(1));
        ResumeState {
            screen,
            topic_index,
            review_topic_id: self
                .command_training_topics
                .get(topic_index)
                .map(|topic| topic.id.clone()),
            topic_training_level: self.topic_training_level,
            ..ResumeState::default()
        }
    }

    fn resolve_review_topic_index(&self, resume: &ResumeState) -> Option<usize> {
        match resume.review_topic_id.as_deref() {
            Some(id) => self
                .command_training_topics
                .iter()
                .position(|topic| topic.id == id),
            None => self
                .command_training_topics
                .get(resume.topic_index)
                .map(|_| resume.topic_index),
        }
    }

    fn review_topic_index_for_source(&self, source: &ReviewSource) -> Option<usize> {
        match source {
            ReviewSource::CommandTopic(id) => self
                .command_training_topics
                .iter()
                .position(|topic| topic.id == *id),
            ReviewSource::CommandCategory(category) => self
                .command_training_topics
                .iter()
                .position(|topic| topic.category == *category),
            _ => None,
        }
    }

    fn review_source_for_topic_index(&self, topic_index: usize) -> Option<ReviewSource> {
        self.command_training_topics
            .get(topic_index)
            .map(|topic| ReviewSource::CommandTopic(topic.id.clone()))
    }

    // ─────────────────────────────────────────────────────────
    // Prompt generation
    // ─────────────────────────────────────────────────────────

    pub fn format_prompt(&self) -> String {
        if self.state == AppState::Typing {
            if let Some(command) = self.current_typing_command() {
                if let Some(prompt) = self.command_prompts.get(&command.id) {
                    return prompt.clone();
                }
            }
        }
        match self.user_config.prompt_style {
            PromptStyle::Full => {
                let path = if self.user_config.show_path { "~" } else { "" };
                format!(
                    "{}@{}:{}$ ",
                    self.user_config.prompt_username, self.user_config.prompt_hostname, path
                )
            }
            PromptStyle::Simple => "$ ".to_string(),
            PromptStyle::Minimal => "> ".to_string(),
        }
    }

    // ─────────────────────────────────────────────────────────
    // Key dispatch
    // ─────────────────────────────────────────────────────────

    pub fn handle_key_and_save_resume(&mut self, key: KeyEvent) {
        self.handle_key(key);
        if self.state != AppState::Quitting {
            self.save_resume_state();
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::ALT)
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && !matches!(key.code, KeyCode::Char('c' | 'r')))
        {
            return;
        }
        if !crate::flow::workflow_flow::handle_key(self, key) {
            self.handle_key_without_workflow(key);
        }
    }

    pub(crate) fn handle_key_without_workflow(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT) {
            match &self.state {
                AppState::Dictation if !self.dictation_submitted => {
                    self.dictation_input.push('\n');
                    return;
                }
                AppState::Review {
                    phase: ReviewPhase::Practice,
                    ..
                } if !self.review_practice.dictation_submitted
                    && self
                        .review_practice
                        .exercises
                        .get(self.review_practice.current_index)
                        .is_some_and(|e| e.kind == ReviewExerciseKind::Dictation) =>
                {
                    self.review_practice.dictation_input.push('\n');
                    return;
                }
                _ => {}
            }
        }
        if key.code == KeyCode::Enter
            && self.is_follow_typing_screen()
            && self.typing_engine.target.get(self.typing_engine.cursor) == Some(&'\n')
        {
            self.typing_engine.input('\n');
            return;
        }
        if key.code == KeyCode::Char('p') {
            let source = match &self.state {
                AppState::CommandLessonOverview {
                    category_index,
                    command_index,
                    ..
                } => self
                    .get_lesson_categories()
                    .get(*category_index)
                    .and_then(|cat| {
                        self.get_lessons_for_category(*cat)
                            .get(*command_index)
                            .map(|l| ("lesson", l.meta.command.clone()))
                    }),
                AppState::SymbolLesson {
                    topic_index,
                    phase: SymbolPhase::Explain,
                    ..
                } => self
                    .symbol_topics
                    .get(*topic_index)
                    .map(|t| ("symbol", t.meta.id.clone())),
                AppState::SystemLesson {
                    topic_index,
                    phase: SystemPhase::Overview | SystemPhase::Detail,
                    ..
                } => self
                    .system_topics
                    .get(*topic_index)
                    .map(|t| ("system", t.meta.id.clone())),
                _ => None,
            };
            if let Some((kind, source)) = source {
                crate::flow::practice_flow::enter(self, Some(kind), Some(&source));
                return;
            }
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Tab)
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c' | 'r')))
        {
            if !self.save_active_typing() {
                return;
            }
        }
        if key.code == KeyCode::Enter
            && self.is_follow_typing_screen()
            && self.typing_engine.is_complete()
        {
            if !self.save_active_typing() {
                return;
            }
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && !matches!(key.code, KeyCode::Char('c' | 'r'))
        {
            return;
        }
        // Global: Ctrl+C saves the current screen before entering Quitting.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.save_resume_state();
            self.state = AppState::Quitting;
            return;
        }

        match self.state.clone() {
            AppState::Home => self.handle_home_key(key),
            AppState::Typing => crate::flow::typing_flow::handle_typing_key(self, key),
            AppState::TypingFilter => self.handle_typing_filter_key(key),
            AppState::LearnHub => self.handle_learn_hub_key(key),
            AppState::Calendar => {
                if key.code == KeyCode::Esc {
                    self.state = AppState::Home;
                } else {
                    self.calendar_state.handle_key(key.code);
                }
            }
            AppState::Scenarios => crate::flow::scenario_flow::handle_topics_key(self, key),
            AppState::ScenarioPractice => {
                crate::flow::scenario_flow::handle_practice_key(self, key)
            }
            AppState::PracticeTopics => crate::flow::practice_flow::handle_topics_key(self, key),
            AppState::PracticeGroup => crate::flow::practice_flow::handle_group_key(self, key),
            AppState::CommandTopics => self.handle_command_topics_key(key),
            AppState::CommandLessonOverview {
                category_index,
                command_index,
                scroll,
            } => crate::flow::lesson_flow::handle_command_lesson_overview_key(
                self,
                key,
                category_index,
                command_index,
                scroll,
            ),
            AppState::CommandLessonPractice {
                category_index,
                command_index,
                example_index,
            } => crate::flow::lesson_flow::handle_command_lesson_practice_key(
                self,
                key,
                category_index,
                command_index,
                example_index,
            ),
            AppState::SymbolTopics => crate::flow::symbol_flow::handle_symbol_topics_key(self, key),
            AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase,
            } => crate::flow::symbol_flow::handle_symbol_lesson_key(
                self,
                key,
                topic_index,
                symbol_index,
                phase,
            ),
            AppState::SystemTopics => crate::flow::system_flow::handle_system_topics_key(self, key),
            AppState::ReviewTopics => self.handle_review_topics_key(key),
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase,
                scroll,
            } => crate::flow::system_flow::handle_system_lesson_key(
                self,
                key,
                topic_index,
                section_index,
                phase,
                scroll,
            ),
            AppState::DeepExplanation { source, scroll } => {
                self.handle_deep_explanation_key(key, source, scroll)
            }
            AppState::Review { source, phase } => {
                crate::flow::review_flow::handle_review_key(self, key, source, phase)
            }
            AppState::Dictation => self.handle_dictation_key(key),
            AppState::Stats => self.handle_stats_key(key),
            AppState::Settings => self.handle_settings_key(key),
            AppState::Quitting => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Home
    // ─────────────────────────────────────────────────────────

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.home_index > 0 {
                    self.home_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.home_index < 5 {
                    self.home_index += 1;
                }
            }
            KeyCode::Enter => match self.home_index {
                0 => {
                    self.typing_filter_row = 0;
                    self.state = AppState::TypingFilter;
                }
                1 => self.state = AppState::LearnHub,
                2 => self.enter_dictation(),
                3 => self.state = AppState::Stats,
                4 => self.state = AppState::Settings,
                5 => self.state = AppState::Calendar,
                _ => {}
            },
            KeyCode::Char('q') => self.state = AppState::Quitting,
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Typing filter
    // ─────────────────────────────────────────────────────────

    fn handle_typing_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => {
                self.typing_filter_row = 1 - self.typing_filter_row;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.typing_filter_row == 0 {
                    self.cycle_filter_difficulty(false);
                } else {
                    self.cycle_filter_category(false);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.typing_filter_row == 0 {
                    self.cycle_filter_difficulty(true);
                } else {
                    self.cycle_filter_category(true);
                }
            }
            KeyCode::Enter => crate::flow::typing_flow::enter_typing_filtered(
                self,
                self.filter_difficulty,
                self.filter_category,
            ),
            _ => {}
        }
    }

    fn cycle_filter_difficulty(&mut self, forward: bool) {
        let options: [Option<Difficulty>; 5] = [
            None,
            Some(Difficulty::Beginner),
            Some(Difficulty::Basic),
            Some(Difficulty::Advanced),
            Some(Difficulty::Practical),
        ];
        let current = options
            .iter()
            .position(|opt| *opt == self.filter_difficulty)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % options.len()
        } else {
            (current + options.len() - 1) % options.len()
        };
        self.filter_difficulty = options[next];
    }

    fn cycle_filter_category(&mut self, forward: bool) {
        let mut options: Vec<Option<Category>> = vec![None];
        options.extend(Category::ALL.into_iter().map(Some));
        let current = options
            .iter()
            .position(|opt| *opt == self.filter_category)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % options.len()
        } else {
            (current + options.len() - 1) % options.len()
        };
        self.filter_category = options[next];
    }

    // ─────────────────────────────────────────────────────────
    // Learn Hub
    // ─────────────────────────────────────────────────────────

    fn handle_learn_hub_key(&mut self, key: KeyEvent) {
        self.learn_hub_index = self.learn_hub_index.min(LEARN_HUB_ITEM_COUNT - 1);
        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.learn_hub_index > 0 {
                    self.learn_hub_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.learn_hub_index + 1 < LEARN_HUB_ITEM_COUNT {
                    self.learn_hub_index += 1;
                }
            }
            KeyCode::Enter => match self.learn_hub_index {
                0 => self.state = AppState::CommandTopics,
                1 => {
                    self.symbol_topics_index = 0;
                    self.state = AppState::SymbolTopics;
                }
                2 => {
                    self.system_topics_index = 0;
                    self.state = AppState::SystemTopics;
                }
                3 => {
                    self.review_topics_index = 0;
                    self.state = AppState::ReviewTopics;
                }
                4 => crate::flow::practice_flow::enter(self, None, None),
                5 => self.state = AppState::Scenarios,
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_review_topics_key(&mut self, key: KeyEvent) {
        let topic_count = self.command_training_topics.len();
        match key.code {
            KeyCode::Esc => self.state = AppState::LearnHub,
            KeyCode::Up | KeyCode::Char('k') => {
                self.review_topics_index = self.review_topics_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.review_topics_index.saturating_add(1) < topic_count {
                    self.review_topics_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(source) = self.review_source_for_topic_index(self.review_topics_index) {
                    self.review_practice = ReviewPracticeState::default();
                    self.state = AppState::Review {
                        source,
                        phase: ReviewPhase::Summary,
                    };
                }
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Command Topics
    // ─────────────────────────────────────────────────────────

    fn handle_command_topics_key(&mut self, key: KeyEvent) {
        // Build list of categories that have lessons
        let categories = self.get_lesson_categories();
        let count = categories.len();
        if count == 0 {
            if key.code == KeyCode::Esc {
                self.state = AppState::LearnHub;
            }
            return;
        }

        match key.code {
            KeyCode::Esc => self.state = AppState::LearnHub,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.command_topics_index > 0 {
                    self.command_topics_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.command_topics_index < count.saturating_sub(1) {
                    self.command_topics_index += 1;
                }
            }
            KeyCode::Enter => {
                if self.command_topics_index < count {
                    let cat = categories[self.command_topics_index];
                    self.lesson_commands_for_category = self
                        .lessons
                        .iter()
                        .filter(|l| l.meta.category == cat)
                        .cloned()
                        .collect();
                    self.command_list_index = 0;
                    if !self.lesson_commands_for_category.is_empty() {
                        self.state = AppState::CommandLessonOverview {
                            category_index: self.command_topics_index,
                            command_index: 0,
                            scroll: 0,
                        };
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_lesson_categories(&self) -> Vec<Category> {
        let mut cats: Vec<Category> = Vec::new();
        for cat in Category::ALL {
            if self.lessons.iter().any(|l| l.meta.category == cat) {
                cats.push(cat);
            }
        }
        cats
    }

    pub fn get_lessons_for_category(&self, category: Category) -> Vec<&CommandLesson> {
        self.lessons
            .iter()
            .filter(|l| l.meta.category == category)
            .collect()
    }

    pub(crate) fn effective_record_difficulty(
        &self,
        command_id: Option<&str>,
        fallback: Difficulty,
    ) -> Difficulty {
        command_id
            .filter(|command_id| !command_id.trim().is_empty())
            .and_then(|command_id| {
                self.commands
                    .iter()
                    .find(|command| command.id == command_id)
            })
            .map(|command| command.difficulty)
            .unwrap_or(fallback)
    }

    // Command lesson flow moved to src/flow/lesson_flow.rs
    // ─────────────────────────────────────────────────────────
    // Command Lesson — Overview / Practice
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // Symbol Topics
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // Symbol Lesson
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // System Topics
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // System Lesson
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // Review
    // ─────────────────────────────────────────────────────────
    // ─────────────────────────────────────────────────────────
    // Dictation
    // ─────────────────────────────────────────────────────────

    fn enter_dictation(&mut self) {
        self.dictation_commands = self.commands.clone();
        self.dictation_index = 0;
        self.dictation_input.clear();
        self.dictation_result = None;
        self.dictation_submitted = false;
        if self.dictation_commands.is_empty() {
            self.state = AppState::Home;
            return;
        }
        self.state = AppState::Dictation;
    }

    fn handle_dictation_key(&mut self, key: KeyEvent) {
        if self.current_dictation_command().is_none() {
            self.state = AppState::Home;
            return;
        }

        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Enter => {
                if self.dictation_submitted {
                    // Go to next question or finish
                    self.dictation_index += 1;
                    if self.dictation_index < self.dictation_commands.len() {
                        self.dictation_input.clear();
                        self.dictation_result = None;
                        self.dictation_submitted = false;
                    } else {
                        self.state = AppState::Home;
                    }
                } else if !self.dictation_input.is_empty() {
                    // Submit
                    let cmd = &self.dictation_commands[self.dictation_index];
                    let result = matcher::check(&self.dictation_input, &cmd.dictation.answers);
                    let accuracy = match result {
                        MatchResult::Exact(_) | MatchResult::Normalized(_) => 1.0,
                        MatchResult::NoMatch { .. } => 0.0,
                    };

                    let now_ms = Utc::now().timestamp_millis();
                    let record = SessionRecord {
                        typing: None,
                        id: format!("{}", now_ms),
                        command_id: cmd.id.clone(),
                        mode: RecordMode::Dictation,
                        keystrokes: Vec::new(),
                        started_at: now_ms,
                        finished_at: now_ms,
                        wpm: 0.0,
                        cpm: 0.0,
                        accuracy,
                        error_count: if accuracy >= 1.0 { 0 } else { 1 },
                        difficulty: cmd.difficulty,
                    };
                    scorer::update_stats(&mut self.user_stats, &record);
                    let _ = self.progress_store.save_stats(&self.user_stats);
                    let _ = self.progress_store.append_record(&record);
                    self.history.push(record);

                    self.dictation_result = Some(result);
                    self.dictation_submitted = true;
                }
            }
            KeyCode::Backspace if !self.dictation_submitted => {
                self.dictation_input.pop();
            }
            KeyCode::Char(c) if !self.dictation_submitted => {
                self.dictation_input.push(c);
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Stats
    // ─────────────────────────────────────────────────────────

    fn handle_stats_key(&mut self, key: KeyEvent) {
        if self.stats_tab == 3
            && matches!(
                key.code,
                KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Home
                    | KeyCode::Char('j' | 'k' | '[' | ']')
            )
        {
            self.calendar_state.handle_key(key.code);
            return;
        }
        if self.stats_tab == 1
            && matches!(
                key.code,
                KeyCode::Char('j' | 'k') | KeyCode::Up | KeyCode::Down
            )
        {
            let code = match key.code {
                KeyCode::Up => KeyCode::Char('k'),
                KeyCode::Down => KeyCode::Char('j'),
                code => code,
            };
            self.calendar_state.handle_key(code);
            return;
        }
        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.stats_tab = (self.stats_tab + 1) % 4;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.stats_tab = if self.stats_tab == 0 {
                    3
                } else {
                    self.stats_tab - 1
                };
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Settings
    // ─────────────────────────────────────────────────────────

    fn handle_settings_key(&mut self, key: KeyEvent) {
        // 7 editable items + 2 read-only display items
        const SETTINGS_COUNT: usize = 7;
        match key.code {
            KeyCode::Esc => {
                let _ = self.progress_store.save_config(&self.user_config);
                self.state = AppState::Home;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_index > 0 {
                    self.settings_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_index < SETTINGS_COUNT - 1 {
                    self.settings_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                self.toggle_setting();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.toggle_setting_reverse();
            }
            _ => {}
        }
    }

    fn toggle_setting(&mut self) {
        match self.settings_index {
            0 => {
                // Prompt style: Full → Simple → Minimal → Full
                self.user_config.prompt_style = match self.user_config.prompt_style {
                    PromptStyle::Full => PromptStyle::Simple,
                    PromptStyle::Simple => PromptStyle::Minimal,
                    PromptStyle::Minimal => PromptStyle::Full,
                };
            }
            1 => {
                // Typing mode: Terminal → Standard → Detailed → Terminal
                self.typing_mode = match self.typing_mode {
                    TypingDisplayMode::Terminal => TypingDisplayMode::Standard,
                    TypingDisplayMode::Standard => TypingDisplayMode::Detailed,
                    TypingDisplayMode::Detailed => TypingDisplayMode::Terminal,
                };
                self.user_config.typing_mode = self.typing_mode.clone();
            }
            2 => {
                // Target WPM +5
                self.user_config.target_wpm = (self.user_config.target_wpm + 5.0).min(200.0);
            }
            3 => {
                // Error flash +50ms
                self.user_config.error_flash_ms = (self.user_config.error_flash_ms + 50).min(500);
            }
            4 => self.user_config.show_token_hints = !self.user_config.show_token_hints,
            5 => self.user_config.adaptive_recommend = !self.user_config.adaptive_recommend,
            6 => self.user_config.show_path = !self.user_config.show_path,
            _ => {}
        }
        let _ = self.progress_store.save_config(&self.user_config);
    }

    fn toggle_setting_reverse(&mut self) {
        match self.settings_index {
            0 => {
                self.user_config.prompt_style = match self.user_config.prompt_style {
                    PromptStyle::Full => PromptStyle::Minimal,
                    PromptStyle::Simple => PromptStyle::Full,
                    PromptStyle::Minimal => PromptStyle::Simple,
                };
            }
            1 => {
                // Reverse cycle: Terminal ← Standard ← Detailed ← Terminal
                self.typing_mode = match self.typing_mode {
                    TypingDisplayMode::Terminal => TypingDisplayMode::Detailed,
                    TypingDisplayMode::Standard => TypingDisplayMode::Terminal,
                    TypingDisplayMode::Detailed => TypingDisplayMode::Standard,
                };
                self.user_config.typing_mode = self.typing_mode.clone();
            }
            2 => {
                self.user_config.target_wpm = (self.user_config.target_wpm - 5.0).max(10.0);
            }
            3 => {
                self.user_config.error_flash_ms =
                    self.user_config.error_flash_ms.saturating_sub(50).max(50);
            }
            4 => self.user_config.show_token_hints = !self.user_config.show_token_hints,
            5 => self.user_config.adaptive_recommend = !self.user_config.adaptive_recommend,
            6 => self.user_config.show_path = !self.user_config.show_path,
            _ => {}
        }
        let _ = self.progress_store.save_config(&self.user_config);
    }

    fn handle_deep_explanation_key(&mut self, key: KeyEvent, source: DeepSource, scroll: usize) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
                self.state = self.deep_source_to_state(&source);
            }
            KeyCode::Right | KeyCode::Char('n') | KeyCode::Char('N') => {
                if let Some(next_source) = self.next_deep_source(&source) {
                    self.state = AppState::DeepExplanation {
                        source: next_source,
                        scroll: 0,
                    };
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state = AppState::DeepExplanation {
                    source,
                    scroll: scroll.saturating_sub(1),
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state = AppState::DeepExplanation {
                    source,
                    scroll: scroll.saturating_add(1),
                };
            }
            KeyCode::PageUp => {
                self.state = AppState::DeepExplanation {
                    source,
                    scroll: scroll.saturating_sub(10),
                };
            }
            KeyCode::PageDown => {
                self.state = AppState::DeepExplanation {
                    source,
                    scroll: scroll.saturating_add(10),
                };
            }
            _ => {}
        }
    }

    fn deep_source_to_state(&self, source: &DeepSource) -> AppState {
        match source {
            DeepSource::LessonExample {
                category_idx,
                command_idx,
                example_idx,
            } => AppState::CommandLessonPractice {
                category_index: *category_idx,
                command_index: *command_idx,
                example_index: *example_idx,
            },
            DeepSource::SymbolExample {
                topic_idx,
                symbol_idx,
                example_idx,
            } => AppState::SymbolLesson {
                topic_index: *topic_idx,
                symbol_index: *symbol_idx,
                phase: SymbolPhase::Example(*example_idx),
            },
            DeepSource::SystemCommand {
                topic_idx,
                section_idx,
                command_idx,
            } => AppState::SystemLesson {
                topic_index: *topic_idx,
                section_index: *section_idx,
                phase: SystemPhase::TypingPractice {
                    command_idx: *command_idx,
                },
                scroll: 0,
            },
        }
    }

    fn next_deep_source(&self, source: &DeepSource) -> Option<DeepSource> {
        match source {
            DeepSource::LessonExample {
                category_idx,
                command_idx,
                example_idx,
            } => {
                let cat = self.get_lesson_categories().get(*category_idx).copied()?;
                let lessons = self.get_lessons_for_category(cat);
                let lesson = lessons.get(*command_idx)?;
                let next = example_idx + 1;
                if next < lesson.examples.len() {
                    Some(DeepSource::LessonExample {
                        category_idx: *category_idx,
                        command_idx: *command_idx,
                        example_idx: next,
                    })
                } else {
                    None
                }
            }
            DeepSource::SymbolExample {
                topic_idx,
                symbol_idx,
                example_idx,
            } => {
                let topic = self.symbol_topics.get(*topic_idx)?;
                let symbol = topic.symbols.get(*symbol_idx)?;
                let next = example_idx + 1;
                if next < symbol.examples.len() {
                    Some(DeepSource::SymbolExample {
                        topic_idx: *topic_idx,
                        symbol_idx: *symbol_idx,
                        example_idx: next,
                    })
                } else {
                    None
                }
            }
            DeepSource::SystemCommand {
                topic_idx,
                section_idx,
                command_idx,
            } => {
                let topic = self.system_topics.get(*topic_idx)?;
                let section = topic.sections.get(*section_idx)?;
                let next = command_idx + 1;
                if next < section.commands.len() {
                    Some(DeepSource::SystemCommand {
                        topic_idx: *topic_idx,
                        section_idx: *section_idx,
                        command_idx: next,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn enter_typing_with_filter(
        &mut self,
        difficulty: Option<Difficulty>,
        category: Option<Category>,
    ) {
        self.filter_difficulty = difficulty;
        self.filter_category = category;
        crate::flow::typing_flow::enter_typing_filtered(self, difficulty, category);
    }

    pub fn filtered_commands(
        &self,
        difficulty: Option<Difficulty>,
        category: Option<Category>,
    ) -> Vec<Command> {
        self.commands
            .iter()
            .filter(|cmd| difficulty.is_none_or(|d| cmd.difficulty == d))
            .filter(|cmd| category.is_none_or(|c| cmd.category == c))
            .cloned()
            .collect()
    }

    pub fn current_filter_match_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|cmd| self.filter_difficulty.is_none_or(|d| cmd.difficulty == d))
            .filter(|cmd| self.filter_category.is_none_or(|c| cmd.category == c))
            .count()
    }

    // ─────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────

    pub fn current_typing_command(&self) -> Option<&Command> {
        self.typing_commands.get(self.typing_index)
    }

    pub fn current_dictation_command(&self) -> Option<&Command> {
        self.dictation_commands.get(self.dictation_index)
    }

    pub fn current_symbol_practice_exercise(&self, topic_index: usize) -> Option<&Exercise> {
        let idx = self
            .symbol_practice
            .dictation_indices
            .get(self.symbol_practice.current_index)
            .copied()
            .unwrap_or(self.symbol_practice.current_index);

        self.symbol_topics
            .get(topic_index)
            .and_then(|topic| topic.exercises.get(idx))
    }

    pub fn current_symbol_typing_exercise(
        &self,
        topic_index: usize,
        exercise_idx: usize,
    ) -> Option<&Exercise> {
        let idx = self
            .symbol_practice
            .typing_indices
            .get(exercise_idx)
            .copied()
            .unwrap_or(exercise_idx);

        self.symbol_topics
            .get(topic_index)
            .and_then(|topic| topic.exercises.get(idx))
    }

    pub fn command_training_topic_for_source(
        &self,
        source: &ReviewSource,
    ) -> Option<&CommandTrainingTopic> {
        match source {
            ReviewSource::CommandTopic(id) => self
                .command_training_topics
                .iter()
                .find(|topic| topic.id == *id),
            ReviewSource::CommandCategory(category) => self
                .command_training_topics
                .iter()
                .find(|topic| topic.category == *category),
            _ => None,
        }
    }

    pub fn command_topic_practiced_count(&self, topic: &CommandTrainingTopic) -> usize {
        topic
            .command_ids
            .iter()
            .filter(|command_id| {
                self.user_stats.command_progress.iter().any(|progress| {
                    progress.command_id == command_id.as_str() && progress.times_practiced > 0
                })
            })
            .count()
    }

    pub fn current_review_exercise(&self) -> Option<&ReviewExercise> {
        self.review_practice
            .exercises
            .get(self.review_practice.current_index)
    }

    pub fn review_accuracy(&self) -> f64 {
        if self.review_practice.total_count == 0 {
            0.0
        } else {
            self.review_practice.accuracy_sum / self.review_practice.total_count as f64
        }
    }

    pub fn typing_is_finished(&self) -> bool {
        self.typing_index >= self.typing_commands.len()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::App;

    const REQUIRED_DATA_DIRS: [&str; 4] = ["commands", "lessons", "symbols", "system"];

    fn environment_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvironmentGuard {
        data_dir: Option<OsString>,
        home: Option<OsString>,
    }

    impl EnvironmentGuard {
        fn set(data_dir: &Path, home: &Path) -> Self {
            let guard = Self {
                data_dir: std::env::var_os("CMDTYPER_DATA_DIR"),
                home: std::env::var_os("HOME"),
            };
            // SAFETY: these tests serialize process-environment mutation with environment_lock.
            unsafe {
                std::env::set_var("CMDTYPER_DATA_DIR", data_dir);
                std::env::set_var("HOME", home);
            }
            guard
        }
    }

    impl Drop for EnvironmentGuard {
        fn drop(&mut self) {
            // SAFETY: these tests serialize process-environment mutation with environment_lock.
            unsafe {
                match &self.data_dir {
                    Some(value) => std::env::set_var("CMDTYPER_DATA_DIR", value),
                    None => std::env::remove_var("CMDTYPER_DATA_DIR"),
                }
                match &self.home {
                    Some(value) => std::env::set_var("HOME", value),
                    None => std::env::remove_var("HOME"),
                }
            }
        }
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
            fs::create_dir_all(&path).expect("create temporary directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn create_data_candidate(path: &Path, omitted: Option<&str>) {
        for directory in REQUIRED_DATA_DIRS {
            if Some(directory) != omitted {
                fs::create_dir_all(path.join(directory)).expect("create data subdirectory");
            }
        }
    }

    #[test]
    fn detect_data_dir_prefers_complete_environment_candidate() {
        let _lock = environment_lock().lock().expect("lock poisoned");
        let root = TempDir::new("cmdtyper-data-detect-env");
        let env_candidate = root.path().join("environment-data");
        let home = root.path().join("home");
        let home_candidate = home.join(".local/share/cmdtyper/data");
        create_data_candidate(&env_candidate, None);
        create_data_candidate(&home_candidate, None);
        let _environment = EnvironmentGuard::set(&env_candidate, &home);

        assert_eq!(
            App::detect_data_dir().expect("complete environment candidate should be accepted"),
            env_candidate
        );
    }

    #[test]
    fn detect_data_dir_rejects_incomplete_environment_candidates_and_uses_home() {
        let _lock = environment_lock().lock().expect("lock poisoned");
        let root = TempDir::new("cmdtyper-data-detect-home-fallback");
        let home = root.path().join("home");
        let home_candidate = home.join(".local/share/cmdtyper/data");
        create_data_candidate(&home_candidate, None);

        for missing_directory in REQUIRED_DATA_DIRS {
            let env_candidate = root.path().join(format!("missing-{missing_directory}"));
            create_data_candidate(&env_candidate, Some(missing_directory));
            let _environment = EnvironmentGuard::set(&env_candidate, &home);

            assert_eq!(
                App::detect_data_dir().expect("complete home candidate should be used as fallback"),
                home_candidate,
                "environment candidate missing {missing_directory} must be rejected"
            );
        }
    }
}
