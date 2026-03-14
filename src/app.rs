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
    TypingFilter,
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
    Dictation,
}

#[derive(Debug, Clone)]
pub struct ReviewExercise {
    pub kind: ReviewExerciseKind,
    pub command_id: String,
    pub command: String,
    pub description: String,
    pub difficulty: Difficulty,
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
    pub typing_mode: TypingDisplayMode,
    pub filter_difficulty: Option<Difficulty>,
    pub filter_category: Option<Category>,
    pub typing_filter_row: usize,

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
        let typing_mode = user_config.typing_mode.clone();
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
            typing_mode,
            filter_difficulty: None,
            filter_category: None,
            typing_filter_row: 0,
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
            AppState::Typing => crate::flow::typing_flow::handle_typing_key(self, key),
            AppState::TypingFilter => self.handle_typing_filter_key(key),
            AppState::LearnHub => self.handle_learn_hub_key(key),
            AppState::CommandTopics => self.handle_command_topics_key(key),
            AppState::CommandLessonOverview {
                category_index,
                command_index,
            } => crate::flow::lesson_flow::handle_command_lesson_overview_key(
                self,
                key,
                category_index,
                command_index,
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
            AppState::SystemLesson {
                topic_index,
                section_index,
                phase,
            } => crate::flow::system_flow::handle_system_lesson_key(
                self,
                key,
                topic_index,
                section_index,
                phase,
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
                if self.home_index < 4 {
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
        const LEARN_HUB_LAST_INDEX: usize = 7;

        match key.code {
            KeyCode::Esc => self.state = AppState::Home,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.learn_hub_index > 0 {
                    self.learn_hub_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.learn_hub_index < LEARN_HUB_LAST_INDEX {
                    self.learn_hub_index += 1;
                }
            }
            KeyCode::Enter => match self.learn_hub_index {
                0 => self.enter_typing_with_filter(Some(Difficulty::Beginner), None),
                1 => self.enter_typing_with_filter(Some(Difficulty::Basic), None),
                2 => self.enter_typing_with_filter(Some(Difficulty::Advanced), None),
                3 => self.enter_typing_with_filter(Some(Difficulty::Practical), None),
                4 => {
                    self.command_topics_index = 0;
                    self.state = AppState::CommandTopics;
                }
                5 => {
                    self.symbol_topics_index = 0;
                    self.state = AppState::SymbolTopics;
                }
                6 => {
                    self.system_topics_index = 0;
                    self.state = AppState::SystemTopics;
                }
                7 => {
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
            .filter(|cmd| difficulty.map_or(true, |d| cmd.difficulty == d))
            .filter(|cmd| category.map_or(true, |c| cmd.category == c))
            .cloned()
            .collect()
    }

    pub fn current_filter_match_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|cmd| self.filter_difficulty.map_or(true, |d| cmd.difficulty == d))
            .filter(|cmd| self.filter_category.map_or(true, |c| cmd.category == c))
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
