use std::{env, path::PathBuf};

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::seq::SliceRandom;

use crate::core::engine::TypingEngine;
use crate::core::matcher::{self, MatchResult};
use crate::core::scorer;
use crate::core::terminal_history::TerminalHistory;
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
    LearnHub,
    CommandTopics,
    CommandLessonOverview {
        category_index: usize,
        command_index: usize,
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
    SystemLesson {
        topic_index: usize,
        section_index: usize,
        phase: SystemPhase,
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
    Practice,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemPhase {
    Overview,
    Detail,
    Commands(usize),
    ConfigFile(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewSource {
    CommandCategory(Category),
    SymbolTopic(String),
    SystemTopic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewPhase {
    Summary,
    Practice(usize),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolPracticeState {
    pub current_index: usize,
    pub current_input: String,
    pub error_count: u8,
    pub show_answer: bool,
    pub correct_count: usize,
    pub total_count: usize,
    pub submitted: bool,
    pub last_correct: Option<bool>,
    pub completed: bool,
    pub stats_recorded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewExerciseKind {
    Typing,
    Dictation,
}

#[derive(Debug, Clone)]
pub struct ReviewExercise {
    pub kind: ReviewExerciseKind,
    pub command_id: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReviewPracticeState {
    pub exercises: Vec<ReviewExercise>,
    pub current_index: usize,
    pub dictation_input: String,
    pub dictation_result: Option<MatchResult>,
    pub dictation_submitted: bool,
    pub accuracy_sum: f64,
    pub total_count: usize,
    pub typing_wpm_sum: f64,
    pub typing_accuracy_sum: f64,
    pub typing_count: usize,
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
    pub lessons: Vec<CommandLesson>,
    pub symbol_topics: Vec<SymbolTopic>,
    pub system_topics: Vec<SystemTopic>,

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

    // Dictation mode state
    pub dictation_commands: Vec<Command>,
    pub dictation_index: usize,
    pub dictation_input: String,
    pub dictation_result: Option<MatchResult>,
    pub dictation_submitted: bool,

    // Symbol practice state
    pub symbol_practice: SymbolPracticeState,

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
    pub fn new() -> Result<Self> {
        let data_dir = env::var("CMDTYPER_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));
        let commands = command_loader::load_commands(&data_dir)?;
        let lessons = lesson_loader::load_lessons(&data_dir)?;
        let symbol_topics = symbol_loader::load_symbol_topics(&data_dir)?;
        let system_topics = system_loader::load_system_topics(&data_dir)?;

        let progress_store = ProgressStore::new()?;
        let user_stats = progress_store.load_stats()?;
        let user_config = progress_store.load_config()?;
        let history = progress_store.load_history()?;

        Ok(Self {
            state: AppState::Home,
            commands,
            lessons,
            symbol_topics,
            system_topics,
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
            dictation_commands: Vec::new(),
            dictation_index: 0,
            dictation_input: String::new(),
            dictation_result: None,
            dictation_submitted: false,
            symbol_practice: SymbolPracticeState::default(),
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
        })
    }

    // ─────────────────────────────────────────────────────────
    // Prompt generation
    // ─────────────────────────────────────────────────────────

    pub fn format_prompt(&self) -> String {
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global: Ctrl+C → Quitting
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.state = AppState::Quitting;
            return;
        }

        match self.state.clone() {
            AppState::Home => self.handle_home_key(key),
            AppState::Typing => self.handle_typing_key(key),
            AppState::LearnHub => self.handle_learn_hub_key(key),
            AppState::CommandTopics => self.handle_command_topics_key(key),
            AppState::CommandLessonOverview {
                category_index,
                command_index,
            } => self.handle_command_lesson_overview_key(key, category_index, command_index),
            AppState::CommandLessonPractice {
                category_index,
                command_index,
                example_index,
            } => self.handle_command_lesson_practice_key(
                key,
                category_index,
                command_index,
                example_index,
            ),
            AppState::SymbolTopics => self.handle_symbol_topics_key(key),
            AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase,
            } => self.handle_symbol_lesson_key(key, topic_index, symbol_index, phase),
            AppState::SystemTopics => self.handle_system_topics_key(key),
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase,
            } => self.handle_system_lesson_key(key, topic_index, section_index, phase),
            AppState::Review { source, phase } => self.handle_review_key(key, source, phase),
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
                if self.home_index < 4 {
                    self.home_index += 1;
                }
            }
            KeyCode::Enter => match self.home_index {
                0 => self.enter_typing(),
                1 => self.state = AppState::LearnHub,
                2 => self.enter_dictation(),
                3 => self.state = AppState::Stats,
                4 => self.state = AppState::Settings,
                _ => {}
            },
            KeyCode::Char('q') => self.state = AppState::Quitting,
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Typing mode
    // ─────────────────────────────────────────────────────────

    fn enter_typing(&mut self) {
        self.terminal_history.clear();
        self.typing_commands = if self.user_config.adaptive_recommend {
            scorer::recommend_commands(&self.user_stats, &self.commands, self.commands.len())
                .into_iter()
                .cloned()
                .collect()
        } else {
            self.commands.clone()
        };

        if self.typing_commands.is_empty() {
            return;
        }
        self.typing_index = 0;
        self.typing_round_records.clear();
        self.typing_showing_output = false;
        let cmd = &self.typing_commands[0];
        self.typing_engine.reset(&cmd.command);
        self.show_hint = self.user_config.show_token_hints;
        self.state = AppState::Typing;
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.typing_showing_output = false;
                self.state = AppState::Home;
            }
            KeyCode::Enter if self.typing_is_finished() => {
                self.typing_showing_output = false;
                self.state = AppState::Home;
            }
            KeyCode::Enter => self.typing_submit_or_advance(),
            KeyCode::Char('h') | KeyCode::Char('H')
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                // If engine hasn't started or is complete, toggle hint
                if self.typing_engine.start_time.is_none() || self.typing_engine.is_complete() {
                    self.show_hint = !self.show_hint;
                } else if !self.typing_showing_output {
                    // Otherwise it's a regular char input
                    self.typing_char_input(key.code);
                }
            }
            KeyCode::Tab if !self.typing_is_finished() => self.typing_skip(),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.typing_retry();
            }
            KeyCode::Backspace
                if !self.typing_showing_output && !self.typing_engine.is_complete() =>
            {
                self.typing_engine.backspace();
            }
            KeyCode::Char(c) if !self.typing_showing_output => {
                self.typing_char_input(KeyCode::Char(c));
            }
            _ => {}
        }
    }

    fn typing_char_input(&mut self, key: KeyCode) {
        if let KeyCode::Char(c) = key {
            let _ = self.typing_engine.input(c);
        }
    }

    fn typing_submit_or_advance(&mut self) {
        if !self.typing_engine.is_complete() || self.typing_is_finished() {
            return;
        }

        if self.typing_showing_output {
            self.typing_finalize_current_command();
            return;
        }

        let has_output = self
            .typing_commands
            .get(self.typing_index)
            .and_then(|cmd| cmd.simulated_output.as_deref())
            .map(|text| !text.trim().is_empty())
            .unwrap_or(false);

        if has_output {
            self.typing_showing_output = true;
        } else {
            self.typing_finalize_current_command();
        }
    }

    fn typing_finalize_current_command(&mut self) {
        let Some(cmd) = self.typing_commands.get(self.typing_index) else {
            return;
        };

        let prompt = self.format_prompt();
        let display = cmd.display_text().to_string();
        let command_id = cmd.id.clone();
        let difficulty = cmd.difficulty;

        self.terminal_history.push_completed(&prompt, &display);

        // Record session
        let record = self
            .typing_engine
            .finish(&command_id, difficulty, RecordMode::Typing);
        scorer::update_stats(&mut self.user_stats, &record);
        let _ = self.progress_store.save_stats(&self.user_stats);
        let _ = self.progress_store.append_record(&record);
        self.history.push(record.clone());
        self.typing_round_records.push(record);

        // Advance to next command
        self.typing_showing_output = false;
        self.typing_index += 1;
        if self.typing_index < self.typing_commands.len() {
            let next_cmd = &self.typing_commands[self.typing_index];
            self.typing_engine.reset(&next_cmd.command);
        }
    }

    fn typing_skip(&mut self) {
        let cmd = &self.typing_commands[self.typing_index];
        let prompt = self.format_prompt();
        let display = cmd.display_text().to_string();
        self.terminal_history.push_completed(&prompt, &display);

        self.typing_showing_output = false;
        self.typing_index += 1;
        if self.typing_index < self.typing_commands.len() {
            let next_cmd = &self.typing_commands[self.typing_index];
            self.typing_engine.reset(&next_cmd.command);
        }
    }

    fn typing_retry(&mut self) {
        if !self.typing_commands.is_empty() && self.typing_index < self.typing_commands.len() {
            let cmd = &self.typing_commands[self.typing_index];
            self.typing_showing_output = false;
            self.typing_engine.reset(&cmd.command);
        }
    }

    // ─────────────────────────────────────────────────────────
    // Learn Hub
    // ─────────────────────────────────────────────────────────

    fn handle_learn_hub_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.learn_hub_index > 0 {
                    self.learn_hub_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.learn_hub_index < 3 {
                    self.learn_hub_index += 1;
                }
            }
            KeyCode::Enter => match self.learn_hub_index {
                0 => {
                    self.command_topics_index = 0;
                    self.state = AppState::CommandTopics;
                }
                1 => {
                    self.symbol_topics_index = 0;
                    self.state = AppState::SymbolTopics;
                }
                2 => {
                    self.system_topics_index = 0;
                    self.state = AppState::SystemTopics;
                }
                3 => {
                    // Direct review — pick first available category
                    if let Some(cat) = Category::ALL.first() {
                        self.state = AppState::Review {
                            source: ReviewSource::CommandCategory(*cat),
                            phase: ReviewPhase::Summary,
                        };
                    }
                }
                _ => {}
            },
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

    /// Get the command string for a lesson example (cloned to avoid borrow issues).
    fn get_lesson_example_command(
        &self,
        category_index: usize,
        command_index: usize,
        example_index: usize,
    ) -> Option<String> {
        let cats = self.get_lesson_categories();
        let cat = cats.get(category_index)?;
        let lessons = self.get_lessons_for_category(*cat);
        let lesson = lessons.get(command_index)?;
        let example = lesson.examples.get(example_index)?;
        Some(example.command.clone())
    }

    // ─────────────────────────────────────────────────────────
    // Command Lesson — Overview / Practice
    // ─────────────────────────────────────────────────────────

    fn handle_command_lesson_overview_key(
        &mut self,
        key: KeyEvent,
        category_index: usize,
        command_index: usize,
    ) {
        match key.code {
            KeyCode::Esc => self.state = AppState::CommandTopics,
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                let cmd_str = self.get_lesson_example_command(category_index, command_index, 0);
                if let Some(cmd) = cmd_str {
                    self.typing_engine.reset(&cmd);
                    self.state = AppState::CommandLessonPractice {
                        category_index,
                        command_index,
                        example_index: 0,
                    };
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if command_index > 0 {
                    self.state = AppState::CommandLessonOverview {
                        category_index,
                        command_index: command_index - 1,
                    };
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let cats = self.get_lesson_categories();
                if category_index < cats.len() {
                    let lessons = self.get_lessons_for_category(cats[category_index]);
                    if command_index + 1 < lessons.len() {
                        self.state = AppState::CommandLessonOverview {
                            category_index,
                            command_index: command_index + 1,
                        };
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_command_lesson_practice_key(
        &mut self,
        key: KeyEvent,
        category_index: usize,
        command_index: usize,
        example_index: usize,
    ) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::CommandLessonOverview {
                    category_index,
                    command_index,
                };
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let cmd_str =
                    self.get_lesson_example_command(category_index, command_index, example_index);
                if let Some(cmd) = cmd_str {
                    self.typing_engine.reset(&cmd);
                }
            }
            KeyCode::Enter if self.typing_engine.is_complete() => {
                // Save lesson practice stats using lesson difficulty.
                let lesson_meta = {
                    let cats = self.get_lesson_categories();
                    if category_index < cats.len() {
                        let lessons = self.get_lessons_for_category(cats[category_index]);
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
                    let record = self.typing_engine.finish(
                        &command_id,
                        difficulty,
                        RecordMode::LessonPractice,
                    );
                    scorer::update_stats(&mut self.user_stats, &record);
                    let _ = self.progress_store.save_stats(&self.user_stats);
                    let _ = self.progress_store.append_record(&record);
                    self.history.push(record);

                    // Move to next example or back to overview
                    let next_example = example_index + 1;
                    if next_example < example_len {
                        if let Some(cmd) = self.get_lesson_example_command(
                            category_index,
                            command_index,
                            next_example,
                        ) {
                            self.typing_engine.reset(&cmd);
                            self.state = AppState::CommandLessonPractice {
                                category_index,
                                command_index,
                                example_index: next_example,
                            };
                        }
                    } else {
                        self.state = AppState::CommandLessonOverview {
                            category_index,
                            command_index,
                        };
                    }
                }
            }
            KeyCode::Char(c) if !self.typing_engine.is_complete() => {
                self.typing_engine.input(c);
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Symbol Topics
    // ─────────────────────────────────────────────────────────

    fn handle_symbol_topics_key(&mut self, key: KeyEvent) {
        let count = self.symbol_topics.len();
        match key.code {
            KeyCode::Esc => self.state = AppState::LearnHub,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.symbol_topics_index > 0 {
                    self.symbol_topics_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.symbol_topics_index < count.saturating_sub(1) {
                    self.symbol_topics_index += 1;
                }
            }
            KeyCode::Enter => {
                if self.symbol_topics_index < count
                    && !self.symbol_topics[self.symbol_topics_index]
                        .symbols
                        .is_empty()
                {
                    self.state = AppState::SymbolLesson {
                        topic_index: self.symbol_topics_index,
                        symbol_index: 0,
                        phase: SymbolPhase::Explain,
                    };
                }
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Symbol Lesson
    // ─────────────────────────────────────────────────────────

    fn handle_symbol_lesson_key(
        &mut self,
        key: KeyEvent,
        topic_index: usize,
        symbol_index: usize,
        phase: SymbolPhase,
    ) {
        let topic = match self.symbol_topics.get(topic_index) {
            Some(t) => t,
            None => {
                self.state = AppState::SymbolTopics;
                return;
            }
        };

        if matches!(phase, SymbolPhase::Practice) {
            self.handle_symbol_practice_key(key, topic_index, symbol_index);
            return;
        }

        match key.code {
            KeyCode::Esc => self.state = AppState::SymbolTopics,
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
                SymbolPhase::Explain => {
                    let sym = &topic.symbols[symbol_index];
                    if !sym.examples.is_empty() {
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index,
                            phase: SymbolPhase::Example(0),
                        };
                    } else if !topic.exercises.is_empty() {
                        self.start_symbol_practice(topic_index);
                        self.state = AppState::SymbolLesson {
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
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index,
                            phase: SymbolPhase::Example(next),
                        };
                    } else if symbol_index + 1 < topic.symbols.len() {
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index: symbol_index + 1,
                            phase: SymbolPhase::Explain,
                        };
                    } else if !topic.exercises.is_empty() {
                        self.start_symbol_practice(topic_index);
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index,
                            phase: SymbolPhase::Practice,
                        };
                    } else {
                        self.state = AppState::SymbolTopics;
                    }
                }
                SymbolPhase::Practice => {
                    self.state = AppState::SymbolTopics;
                }
            },
            KeyCode::Left | KeyCode::Char('h') => match &phase {
                SymbolPhase::Explain => self.state = AppState::SymbolTopics,
                SymbolPhase::Example(0) => {
                    self.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Explain,
                    };
                }
                SymbolPhase::Example(idx) => {
                    self.state = AppState::SymbolLesson {
                        topic_index,
                        symbol_index,
                        phase: SymbolPhase::Example(idx - 1),
                    };
                }
                SymbolPhase::Practice => {
                    let sym = &topic.symbols[symbol_index];
                    if !sym.examples.is_empty() {
                        self.state = AppState::SymbolLesson {
                            topic_index,
                            symbol_index,
                            phase: SymbolPhase::Example(sym.examples.len() - 1),
                        };
                    } else {
                        self.state = AppState::SymbolLesson {
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

    fn start_symbol_practice(&mut self, topic_index: usize) {
        let total_count = self
            .symbol_topics
            .get(topic_index)
            .map(|topic| topic.exercises.len())
            .unwrap_or(0);
        self.symbol_practice = SymbolPracticeState {
            total_count,
            ..SymbolPracticeState::default()
        };
    }

    fn advance_symbol_practice(&mut self, topic_index: usize) {
        if self.symbol_practice.current_index + 1 < self.symbol_practice.total_count {
            self.symbol_practice.current_index += 1;
            self.symbol_practice.current_input.clear();
            self.symbol_practice.error_count = 0;
            self.symbol_practice.show_answer = false;
            self.symbol_practice.submitted = false;
            self.symbol_practice.last_correct = None;
            return;
        }

        self.symbol_practice.completed = true;
        self.symbol_practice.current_input.clear();
        self.symbol_practice.submitted = false;
        self.symbol_practice.last_correct = None;

        if self.symbol_practice.stats_recorded {
            return;
        }

        if let Some(topic) = self.symbol_topics.get(topic_index) {
            let accuracy = if self.symbol_practice.total_count == 0 {
                1.0
            } else {
                self.symbol_practice.correct_count as f64 / self.symbol_practice.total_count as f64
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
                error_count: (self
                    .symbol_practice
                    .total_count
                    .saturating_sub(self.symbol_practice.correct_count))
                    as u32,
                difficulty: topic.meta.difficulty,
            };
            scorer::update_stats(&mut self.user_stats, &record);
            let _ = self.progress_store.save_stats(&self.user_stats);
            let _ = self.progress_store.append_record(&record);
            self.history.push(record);
            self.symbol_practice.stats_recorded = true;
        }
    }

    fn handle_symbol_practice_key(
        &mut self,
        key: KeyEvent,
        topic_index: usize,
        _symbol_index: usize,
    ) {
        if self.symbol_practice.completed {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.state = AppState::SymbolTopics,
                _ => {}
            }
            return;
        }

        let answers = match self
            .symbol_topics
            .get(topic_index)
            .and_then(|topic| topic.exercises.get(self.symbol_practice.current_index))
            .map(|exercise| exercise.answers.clone())
        {
            Some(v) => v,
            None => {
                self.state = AppState::SymbolTopics;
                return;
            }
        };

        match key.code {
            KeyCode::Esc => self.state = AppState::SymbolTopics,
            KeyCode::Backspace if !self.symbol_practice.submitted => {
                self.symbol_practice.current_input.pop();
            }
            KeyCode::Char(c) if !self.symbol_practice.submitted => {
                self.symbol_practice.current_input.push(c);
            }
            KeyCode::Enter => {
                if self.symbol_practice.submitted {
                    if self.symbol_practice.last_correct == Some(true) {
                        self.advance_symbol_practice(topic_index);
                    } else {
                        self.symbol_practice.current_input.clear();
                        self.symbol_practice.submitted = false;
                        self.symbol_practice.last_correct = None;
                        self.symbol_practice.show_answer = false;
                    }
                    return;
                }

                let result = matcher::check(&self.symbol_practice.current_input, &answers);
                match result {
                    MatchResult::Exact(_) | MatchResult::Normalized(_) => {
                        self.symbol_practice.correct_count += 1;
                        self.symbol_practice.submitted = true;
                        self.symbol_practice.last_correct = Some(true);
                        self.symbol_practice.show_answer = false;
                    }
                    MatchResult::NoMatch { .. } => {
                        self.symbol_practice.error_count =
                            self.symbol_practice.error_count.saturating_add(1);
                        self.symbol_practice.submitted = true;
                        self.symbol_practice.last_correct = Some(false);
                        self.symbol_practice.show_answer = true;
                        if self.symbol_practice.error_count >= 3 {
                            self.advance_symbol_practice(topic_index);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // System Topics
    // ─────────────────────────────────────────────────────────

    fn handle_system_topics_key(&mut self, key: KeyEvent) {
        let count = self.system_topics.len();
        match key.code {
            KeyCode::Esc => self.state = AppState::LearnHub,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.system_topics_index > 0 {
                    self.system_topics_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.system_topics_index < count.saturating_sub(1) {
                    self.system_topics_index += 1;
                }
            }
            KeyCode::Enter => {
                if self.system_topics_index < count {
                    self.system_section_index = 0;
                    self.state = AppState::SystemLesson {
                        topic_index: self.system_topics_index,
                        section_index: 0,
                        phase: SystemPhase::Overview,
                    };
                }
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // System Lesson
    // ─────────────────────────────────────────────────────────

    fn handle_system_lesson_key(
        &mut self,
        key: KeyEvent,
        topic_index: usize,
        section_index: usize,
        phase: SystemPhase,
    ) {
        let topic = match self.system_topics.get(topic_index) {
            Some(t) => t,
            None => {
                self.state = AppState::SystemTopics;
                return;
            }
        };

        match key.code {
            KeyCode::Esc => self.state = AppState::SystemTopics,
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match &phase {
                SystemPhase::Overview => {
                    if !topic.sections.is_empty() {
                        self.state = AppState::SystemLesson {
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
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::Commands(0),
                        };
                    } else if section_index < topic.sections.len()
                        && !topic.sections[section_index].config_files.is_empty()
                    {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::ConfigFile(0),
                        };
                    } else if section_index + 1 < topic.sections.len() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index: section_index + 1,
                            phase: SystemPhase::Detail,
                        };
                    } else {
                        self.state = AppState::SystemTopics;
                    }
                }
                SystemPhase::Commands(idx) => {
                    let section = &topic.sections[section_index];
                    let next = idx + 1;
                    if next < section.commands.len() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::Commands(next),
                        };
                    } else if !section.config_files.is_empty() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::ConfigFile(0),
                        };
                    } else if section_index + 1 < topic.sections.len() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index: section_index + 1,
                            phase: SystemPhase::Detail,
                        };
                    } else {
                        self.state = AppState::SystemTopics;
                    }
                }
                SystemPhase::ConfigFile(idx) => {
                    let section = &topic.sections[section_index];
                    let next = idx + 1;
                    if next < section.config_files.len() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index,
                            phase: SystemPhase::ConfigFile(next),
                        };
                    } else if section_index + 1 < topic.sections.len() {
                        self.state = AppState::SystemLesson {
                            topic_index,
                            section_index: section_index + 1,
                            phase: SystemPhase::Detail,
                        };
                    } else {
                        self.state = AppState::SystemTopics;
                    }
                }
            },
            KeyCode::Up | KeyCode::Char('k') => {
                // Navigate sections
                if matches!(phase, SystemPhase::Detail) && section_index > 0 {
                    self.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index - 1,
                        phase: SystemPhase::Detail,
                    };
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if matches!(phase, SystemPhase::Detail) && section_index + 1 < topic.sections.len()
                {
                    self.state = AppState::SystemLesson {
                        topic_index,
                        section_index: section_index + 1,
                        phase: SystemPhase::Detail,
                    };
                }
            }
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────
    // Review
    // ─────────────────────────────────────────────────────────

    fn handle_review_key(&mut self, key: KeyEvent, source: ReviewSource, phase: ReviewPhase) {
        match phase {
            ReviewPhase::Summary => match key.code {
                KeyCode::Esc => self.state = AppState::LearnHub,
                KeyCode::Enter => {
                    self.start_review_practice(&source);
                    self.state = AppState::Review {
                        source,
                        phase: ReviewPhase::Practice(0),
                    };
                }
                _ => {}
            },
            ReviewPhase::Practice(_) => {
                if self.review_practice.completed {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => self.state = AppState::LearnHub,
                        _ => {}
                    }
                    return;
                }

                let exercise = match self
                    .review_practice
                    .exercises
                    .get(self.review_practice.current_index)
                    .cloned()
                {
                    Some(ex) => ex,
                    None => {
                        self.review_practice.completed = true;
                        self.record_review_stats(&source);
                        return;
                    }
                };

                match exercise.kind {
                    ReviewExerciseKind::Typing => match key.code {
                        KeyCode::Esc => self.state = AppState::LearnHub,
                        KeyCode::Char(c) if !self.typing_engine.is_complete() => {
                            self.typing_engine.input(c);
                        }
                        KeyCode::Enter if self.typing_engine.is_complete() => {
                            let acc = self.typing_engine.current_accuracy();
                            let wpm = self.typing_engine.current_wpm();
                            self.review_practice.typing_count += 1;
                            self.review_practice.typing_accuracy_sum += acc;
                            self.review_practice.typing_wpm_sum += wpm;
                            self.review_practice.accuracy_sum += acc;
                            self.advance_review_practice(&source);
                        }
                        _ => {}
                    },
                    ReviewExerciseKind::Dictation => match key.code {
                        KeyCode::Esc => self.state = AppState::LearnHub,
                        KeyCode::Backspace if !self.review_practice.dictation_submitted => {
                            self.review_practice.dictation_input.pop();
                        }
                        KeyCode::Char(c) if !self.review_practice.dictation_submitted => {
                            self.review_practice.dictation_input.push(c);
                        }
                        KeyCode::Enter => {
                            if self.review_practice.dictation_submitted {
                                self.advance_review_practice(&source);
                            } else {
                                let result = matcher::check(
                                    &self.review_practice.dictation_input,
                                    &vec![exercise.command.clone()],
                                );
                                let acc = match result {
                                    MatchResult::Exact(_) | MatchResult::Normalized(_) => 1.0,
                                    MatchResult::NoMatch { .. } => 0.0,
                                };
                                self.review_practice.dictation_count += 1;
                                self.review_practice.dictation_accuracy_sum += acc;
                                self.review_practice.accuracy_sum += acc;
                                self.review_practice.dictation_result = Some(result);
                                self.review_practice.dictation_submitted = true;
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    fn review_source_key(source: &ReviewSource) -> String {
        match source {
            ReviewSource::CommandCategory(cat) => format!("category:{:?}", cat),
            ReviewSource::SymbolTopic(name) => format!("symbol:{}", name),
            ReviewSource::SystemTopic(name) => format!("system:{}", name),
        }
    }

    fn build_review_exercises(&self, source: &ReviewSource) -> Vec<ReviewExercise> {
        let mut base = Vec::new();
        match source {
            ReviewSource::CommandCategory(category) => {
                for cmd in self.commands.iter().filter(|c| c.category == *category) {
                    base.push(ReviewExercise {
                        kind: ReviewExerciseKind::Typing,
                        command_id: cmd.id.clone(),
                        command: cmd.command.clone(),
                        description: cmd.dictation.prompt.clone(),
                    });
                }
            }
            ReviewSource::SymbolTopic(name) => {
                if let Some(topic) = self
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
                            });
                        }
                    }
                }
            }
            ReviewSource::SystemTopic(name) => {
                if let Some(topic) = self
                    .system_topics
                    .iter()
                    .find(|topic| topic.meta.topic == *name || topic.meta.id == *name)
                {
                    for (sec_idx, section) in topic.sections.iter().enumerate() {
                        for (cmd_idx, command) in section.commands.iter().enumerate() {
                            base.push(ReviewExercise {
                                kind: ReviewExerciseKind::Typing,
                                command_id: format!(
                                    "system:{}:{}:{}",
                                    topic.meta.id, sec_idx, cmd_idx
                                ),
                                command: command.command.clone(),
                                description: command.summary.clone(),
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

    fn start_review_practice(&mut self, source: &ReviewSource) {
        let exercises = self.build_review_exercises(source);
        let total_count = exercises.len();
        self.review_practice = ReviewPracticeState {
            exercises,
            total_count,
            ..ReviewPracticeState::default()
        };

        if let Some(first) = self.review_practice.exercises.first() {
            if matches!(first.kind, ReviewExerciseKind::Typing) {
                self.typing_engine.reset(&first.command);
            }
        }
    }

    fn record_review_stats(&mut self, source: &ReviewSource) {
        if self.review_practice.stats_recorded {
            return;
        }

        let now_ms = Utc::now().timestamp_millis();
        let source_key = Self::review_source_key(source);

        if self.review_practice.typing_count > 0 {
            let acc =
                self.review_practice.typing_accuracy_sum / self.review_practice.typing_count as f64;
            let wpm =
                self.review_practice.typing_wpm_sum / self.review_practice.typing_count as f64;
            let record = SessionRecord {
                id: format!("{}-rt", now_ms),
                command_id: format!("review:{}:typing", source_key),
                mode: RecordMode::ReviewPractice,
                keystrokes: Vec::new(),
                started_at: now_ms,
                finished_at: now_ms,
                wpm,
                cpm: wpm * 5.0,
                accuracy: acc,
                error_count: ((1.0 - acc).max(0.0) * self.review_practice.typing_count as f64)
                    .round() as u32,
                difficulty: Difficulty::Beginner,
            };
            scorer::update_stats(&mut self.user_stats, &record);
            let _ = self.progress_store.append_record(&record);
            self.history.push(record);
        }

        if self.review_practice.dictation_count > 0 {
            let acc = self.review_practice.dictation_accuracy_sum
                / self.review_practice.dictation_count as f64;
            let record = SessionRecord {
                id: format!("{}-rd", now_ms),
                command_id: format!("review:{}:dictation", source_key),
                mode: RecordMode::ReviewPractice,
                keystrokes: Vec::new(),
                started_at: now_ms,
                finished_at: now_ms,
                wpm: 0.0,
                cpm: 0.0,
                accuracy: acc,
                error_count: ((1.0 - acc).max(0.0) * self.review_practice.dictation_count as f64)
                    .round() as u32,
                difficulty: Difficulty::Beginner,
            };
            scorer::update_stats(&mut self.user_stats, &record);
            let _ = self.progress_store.append_record(&record);
            self.history.push(record);
        }

        let _ = self.progress_store.save_stats(&self.user_stats);
        self.review_practice.stats_recorded = true;
    }

    fn advance_review_practice(&mut self, source: &ReviewSource) {
        self.review_practice.current_index += 1;
        self.review_practice.dictation_input.clear();
        self.review_practice.dictation_result = None;
        self.review_practice.dictation_submitted = false;

        if self.review_practice.current_index >= self.review_practice.exercises.len() {
            self.review_practice.completed = true;
            self.record_review_stats(source);
            return;
        }

        if let Some(exercise) = self
            .review_practice
            .exercises
            .get(self.review_practice.current_index)
            && matches!(exercise.kind, ReviewExerciseKind::Typing)
        {
            self.typing_engine.reset(&exercise.command);
        }

        self.state = AppState::Review {
            source: source.clone(),
            phase: ReviewPhase::Practice(self.review_practice.current_index),
        };
    }

    // ─────────────────────────────────────────────────────────
    // Dictation
    // ─────────────────────────────────────────────────────────

    fn enter_dictation(&mut self) {
        self.dictation_commands = self.commands.clone();
        if self.dictation_commands.is_empty() {
            return;
        }
        self.dictation_index = 0;
        self.dictation_input.clear();
        self.dictation_result = None;
        self.dictation_submitted = false;
        self.state = AppState::Dictation;
    }

    fn handle_dictation_key(&mut self, key: KeyEvent) {
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
        // 6 editable items + 2 read-only display items
        const SETTINGS_COUNT: usize = 6;
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
                // Target WPM +5
                self.user_config.target_wpm = (self.user_config.target_wpm + 5.0).min(200.0);
            }
            2 => {
                // Error flash +50ms
                self.user_config.error_flash_ms = (self.user_config.error_flash_ms + 50).min(500);
            }
            3 => self.user_config.show_token_hints = !self.user_config.show_token_hints,
            4 => self.user_config.adaptive_recommend = !self.user_config.adaptive_recommend,
            5 => self.user_config.show_path = !self.user_config.show_path,
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
                self.user_config.target_wpm = (self.user_config.target_wpm - 5.0).max(10.0);
            }
            2 => {
                self.user_config.error_flash_ms =
                    self.user_config.error_flash_ms.saturating_sub(50).max(50);
            }
            3 => self.user_config.show_token_hints = !self.user_config.show_token_hints,
            4 => self.user_config.adaptive_recommend = !self.user_config.adaptive_recommend,
            5 => self.user_config.show_path = !self.user_config.show_path,
            _ => {}
        }
        let _ = self.progress_store.save_config(&self.user_config);
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
        self.symbol_topics
            .get(topic_index)
            .and_then(|topic| topic.exercises.get(self.symbol_practice.current_index))
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
