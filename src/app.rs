use std::{env, path::PathBuf};

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

    // Dictation mode state
    pub dictation_commands: Vec<Command>,
    pub dictation_index: usize,
    pub dictation_input: String,
    pub dictation_result: Option<MatchResult>,
    pub dictation_submitted: bool,

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
        let data_dir = Path::new("data");
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
            dictation_commands: Vec::new(),
            dictation_index: 0,
            dictation_input: String::new(),
            dictation_result: None,
            dictation_submitted: false,
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
        let cmd = &self.typing_commands[0];
        self.typing_engine.reset(&cmd.command);
        self.show_hint = self.user_config.show_token_hints;
        self.state = AppState::Typing;
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Home;
            }
            KeyCode::Enter if self.typing_is_finished() => {
                self.state = AppState::Home;
            }
            KeyCode::Char('h') | KeyCode::Char('H')
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                // If engine hasn't started or is complete, toggle hint
                if self.typing_engine.start_time.is_none() || self.typing_engine.is_complete() {
                    self.show_hint = !self.show_hint;
                } else {
                    // Otherwise it's a regular char input
                    self.typing_char_input(key.code);
                }
            }
            KeyCode::Tab => self.typing_skip(),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.typing_retry();
            }
            KeyCode::Char(c) => {
                self.typing_char_input(KeyCode::Char(c));
            }
            _ => {}
        }
    }

    fn typing_char_input(&mut self, key: KeyCode) {
        if let KeyCode::Char(c) = key {
            let result = self.typing_engine.input(c);
            if self.typing_engine.is_complete() {
                self.typing_complete_line();
            }
            let _ = result;
        }
    }

    fn typing_complete_line(&mut self) {
        let cmd = &self.typing_commands[self.typing_index];
        let prompt = self.format_prompt();
        let display = cmd.display_text().to_string();
        self.terminal_history.push_completed(&prompt, &display);

        // Record session
        let record = self
            .typing_engine
            .finish(&cmd.id, cmd.difficulty, RecordMode::Typing);
        scorer::update_stats(&mut self.user_stats, &record);
        let _ = self.progress_store.save_stats(&self.user_stats);
        let _ = self.progress_store.append_record(&record);
        self.history.push(record.clone());
        self.typing_round_records.push(record);

        // Advance to next command
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

        self.typing_index += 1;
        if self.typing_index < self.typing_commands.len() {
            let next_cmd = &self.typing_commands[self.typing_index];
            self.typing_engine.reset(&next_cmd.command);
        }
    }

    fn typing_retry(&mut self) {
        if !self.typing_commands.is_empty() && self.typing_index < self.typing_commands.len() {
            let cmd = &self.typing_commands[self.typing_index];
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
                let cats = self.get_lesson_categories();
                if category_index < cats.len() {
                    let lessons = self.get_lessons_for_category(cats[category_index]);
                    if command_index < lessons.len() {
                        let lesson = lessons[command_index];
                        let command_id = format!(lesson:{}:{}, lesson.meta.command, example_index);
                        let record = self.typing_engine.finish(
                            &command_id,
                            lesson.meta.difficulty,
                            RecordMode::LessonPractice,
                        );
                        scorer::update_stats(&mut self.user_stats, &record);
                        let _ = self.progress_store.save_stats(&self.user_stats);
                        let _ = self.progress_store.append_record(&record);
                        self.history.push(record);

                        // Move to next example or back to overview
                        let next_example = example_index + 1;
                        if next_example < lesson.examples.len() {
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
        match key.code {
            KeyCode::Esc => self.state = AppState::LearnHub,
            KeyCode::Enter => match phase {
                ReviewPhase::Summary => {
                    self.state = AppState::Review {
                        source,
                        phase: ReviewPhase::Practice(0),
                    };
                }
                ReviewPhase::Practice(_) => {
                    self.state = AppState::LearnHub;
                }
            },
            _ => {}
        }
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
                        id: format!({}, now_ms),
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

    pub fn typing_is_finished(&self) -> bool {
        self.typing_index >= self.typing_commands.len()
    }
}
