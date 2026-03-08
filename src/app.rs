#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use chrono::Utc;

use crate::core::engine::TypingEngine;
use crate::core::matcher::{DiffKind, MatchResult, Matcher};
use crate::data::loader;
use crate::data::models::{
    Category, Command, DictationData, Difficulty, Mode, SessionRecord, Token, UserConfig, UserStats,
};

/// Application state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Home,
    Learn,
    Typing,
    Dictation,
    Stats,
    RoundResult,
    Quitting,
}

/// Home screen menu selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Learn,
    Type,
    Dictation,
    Stats,
}

impl MenuItem {
    pub const ALL: [MenuItem; 4] = [
        MenuItem::Learn,
        MenuItem::Type,
        MenuItem::Dictation,
        MenuItem::Stats,
    ];

    pub fn label(&self) -> &str {
        match self {
            Self::Learn => "学习模式 (Learn)",
            Self::Type => "对着打 (Type)",
            Self::Dictation => "默写模式 (Dictation)",
            Self::Stats => "统计面板 (Stats)",
        }
    }

    pub fn desc(&self) -> &str {
        match self {
            Self::Learn => "逐命令讲解语法，跟打学习",
            Self::Type => "逐字符匹配，实时统计 WPM",
            Self::Dictation => "看中文写命令，多答案匹配",
            Self::Stats => "速度、准确率、字符分析",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsTab {
    SpeedOverview,
    CharacterAnalysis,
    CategoryMastery,
    PracticeCalendar,
}

impl StatsTab {
    pub const ALL: [StatsTab; 4] = [
        StatsTab::SpeedOverview,
        StatsTab::CharacterAnalysis,
        StatsTab::CategoryMastery,
        StatsTab::PracticeCalendar,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::SpeedOverview => "速度总览",
            Self::CharacterAnalysis => "字符分析",
            Self::CategoryMastery => "分类掌握",
            Self::PracticeCalendar => "练习日历",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::SpeedOverview => Self::CharacterAnalysis,
            Self::CharacterAnalysis => Self::CategoryMastery,
            Self::CategoryMastery => Self::PracticeCalendar,
            Self::PracticeCalendar => Self::SpeedOverview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::SpeedOverview => Self::PracticeCalendar,
            Self::CharacterAnalysis => Self::SpeedOverview,
            Self::CategoryMastery => Self::CharacterAnalysis,
            Self::PracticeCalendar => Self::CategoryMastery,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationResult {
    pub submitted: String,
    pub evaluation: MatchResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundResultData {
    pub mode: Mode,
    pub command_id: String,
    pub command_text: String,
    pub summary: String,
    pub wpm: f64,
    pub cpm: f64,
    pub accuracy: f64,
    pub elapsed_ms: u64,
    pub error_count: u32,
    pub error_chars: Vec<(char, u32)>,
}

impl RoundResultData {
    fn from_record(mode: Mode, command: &Command, record: &SessionRecord) -> Self {
        let mut error_counts = BTreeMap::new();
        for keystroke in &record.keystrokes {
            let errors = keystroke.attempts.saturating_sub(1) as u32;
            if errors > 0 {
                *error_counts.entry(keystroke.expected).or_insert(0) += errors;
            }
        }

        let mut error_chars = error_counts.into_iter().collect::<Vec<_>>();
        error_chars.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        error_chars.truncate(5);

        Self {
            mode,
            command_id: command.id.clone(),
            command_text: command.command.clone(),
            summary: command.summary.clone(),
            wpm: record.wpm,
            cpm: record.cpm,
            accuracy: record.accuracy,
            elapsed_ms: record.finished_at.saturating_sub(record.started_at) as u64,
            error_count: record.error_count,
            error_chars,
        }
    }
}

/// Main application struct
pub struct App {
    pub state: AppState,
    pub all_commands: Vec<Command>,
    pub commands: Vec<Command>,
    pub user_stats: UserStats,
    pub user_config: UserConfig,
    pub selected_difficulty: Difficulty,
    pub selected_category: Option<Category>,

    // Home screen state
    pub menu_index: usize,

    // Typing mode state
    pub typing_engine: Option<TypingEngine>,
    pub current_command_index: usize,

    // Learn mode state
    pub learn_command_index: usize,
    pub learn_engine: Option<TypingEngine>,

    // Dictation mode state
    pub dictation_command_index: usize,
    pub dictation_input: String,
    pub dictation_cursor: usize,
    pub dictation_result: Option<DictationResult>,
    pub dictation_started_at: Option<Instant>,

    // Stats + results
    pub stats_tab: StatsTab,
    pub round_result: Option<RoundResultData>,
    pending_record: Option<SessionRecord>,
}

impl App {
    pub fn new() -> Self {
        let all_commands = load_command_catalog();
        Self {
            state: AppState::Home,
            all_commands,
            commands: Vec::new(),
            user_stats: UserStats::default(),
            user_config: UserConfig::default(),
            selected_difficulty: Difficulty::Beginner,
            selected_category: None,
            menu_index: 0,
            typing_engine: None,
            current_command_index: 0,
            learn_command_index: 0,
            learn_engine: None,
            dictation_command_index: 0,
            dictation_input: String::new(),
            dictation_cursor: 0,
            dictation_result: None,
            dictation_started_at: None,
            stats_tab: StatsTab::SpeedOverview,
            round_result: None,
            pending_record: None,
        }
    }

    pub fn apply_config(&mut self, config: UserConfig) {
        self.selected_difficulty = config.last_difficulty;
        self.selected_category = config.last_category;
        self.user_config = config;
    }

    pub fn sync_user_config(&mut self) -> bool {
        let mut changed = false;
        if self.user_config.last_difficulty != self.selected_difficulty {
            self.user_config.last_difficulty = self.selected_difficulty;
            changed = true;
        }
        if self.user_config.last_category != self.selected_category {
            self.user_config.last_category = self.selected_category;
            changed = true;
        }
        changed
    }

    pub fn current_menu_item(&self) -> MenuItem {
        MenuItem::ALL[self.menu_index]
    }

    pub fn enter_typing_mode(&mut self) -> Option<AppState> {
        self.commands = self.filtered_commands();
        if self.commands.is_empty() {
            return None;
        }

        self.current_command_index = 0;
        let cmd = &self.commands[self.current_command_index];
        self.typing_engine = Some(TypingEngine::new(&cmd.command));
        self.round_result = None;
        self.state = AppState::Typing;
        Some(AppState::Typing)
    }

    pub fn enter_learn_mode(&mut self) -> Option<AppState> {
        self.commands = self.filtered_commands();
        if self.commands.is_empty() {
            return None;
        }

        self.learn_command_index = 0;
        let cmd = &self.commands[self.learn_command_index];
        self.learn_engine = Some(TypingEngine::new(&cmd.command));
        self.round_result = None;
        self.state = AppState::Learn;
        Some(AppState::Learn)
    }

    pub fn enter_dictation_mode(&mut self) -> Option<AppState> {
        self.commands = self.filtered_commands();
        if self.commands.is_empty() {
            return None;
        }

        self.dictation_command_index = 0;
        self.reset_dictation_prompt_state();
        self.state = AppState::Dictation;
        Some(AppState::Dictation)
    }

    pub fn enter_stats_mode(&mut self) -> AppState {
        self.stats_tab = StatsTab::SpeedOverview;
        self.state = AppState::Stats;
        AppState::Stats
    }

    pub fn current_dictation_command(&self) -> Option<&Command> {
        self.commands.get(self.dictation_command_index)
    }

    pub fn next_typing_command(&mut self) {
        if self.commands.is_empty() {
            return;
        }

        self.current_command_index = (self.current_command_index + 1) % self.commands.len();
        let cmd = &self.commands[self.current_command_index];
        self.typing_engine = Some(TypingEngine::new(&cmd.command));
    }

    pub fn retry_typing_command(&mut self) {
        if let Some(engine) = &mut self.typing_engine {
            engine.reset();
        } else if let Some(cmd) = self.commands.get(self.current_command_index) {
            self.typing_engine = Some(TypingEngine::new(&cmd.command));
        }
    }

    pub fn complete_typing_round(&mut self) -> Option<AppState> {
        let record = {
            let engine = self.typing_engine.as_ref()?;
            let command = self.commands.get(self.current_command_index)?;
            engine.finish(&command.id, Mode::Type, command.difficulty)
        };
        let result = {
            let command = self.commands.get(self.current_command_index)?;
            RoundResultData::from_record(Mode::Type, command, &record)
        };

        self.pending_record = Some(record);
        self.round_result = Some(result);
        self.state = AppState::RoundResult;
        Some(AppState::RoundResult)
    }

    pub fn next_learn_command(&mut self) {
        if self.commands.is_empty() {
            return;
        }

        self.learn_command_index = (self.learn_command_index + 1) % self.commands.len();
        let cmd = &self.commands[self.learn_command_index];
        self.learn_engine = Some(TypingEngine::new(&cmd.command));
    }

    pub fn prev_learn_command(&mut self) {
        if self.commands.is_empty() {
            return;
        }

        if self.learn_command_index == 0 {
            self.learn_command_index = self.commands.len() - 1;
        } else {
            self.learn_command_index -= 1;
        }
        let cmd = &self.commands[self.learn_command_index];
        self.learn_engine = Some(TypingEngine::new(&cmd.command));
    }

    pub fn retry_learn_command(&mut self) {
        if let Some(engine) = &mut self.learn_engine {
            engine.reset();
        } else if let Some(cmd) = self.commands.get(self.learn_command_index) {
            self.learn_engine = Some(TypingEngine::new(&cmd.command));
        }
    }

    pub fn complete_learn_round(&mut self) -> Option<AppState> {
        let record = {
            let engine = self.learn_engine.as_ref()?;
            let command = self.commands.get(self.learn_command_index)?;
            engine.finish(&command.id, Mode::Learn, command.difficulty)
        };
        let result = {
            let command = self.commands.get(self.learn_command_index)?;
            RoundResultData::from_record(Mode::Learn, command, &record)
        };

        self.pending_record = Some(record);
        self.round_result = Some(result);
        self.state = AppState::RoundResult;
        Some(AppState::RoundResult)
    }

    pub fn next_dictation_command(&mut self) {
        if self.commands.is_empty() {
            return;
        }

        self.dictation_command_index = (self.dictation_command_index + 1) % self.commands.len();
        self.reset_dictation_prompt_state();
    }

    pub fn insert_dictation_char(&mut self, ch: char) {
        self.mark_dictation_started();
        let byte_index = byte_index_for_char(&self.dictation_input, self.dictation_cursor);
        self.dictation_input.insert(byte_index, ch);
        self.dictation_cursor += 1;
    }

    pub fn move_dictation_cursor_left(&mut self) {
        self.dictation_cursor = self.dictation_cursor.saturating_sub(1);
    }

    pub fn move_dictation_cursor_right(&mut self) {
        self.dictation_cursor =
            (self.dictation_cursor + 1).min(self.dictation_input.chars().count());
    }

    pub fn move_dictation_cursor_home(&mut self) {
        self.dictation_cursor = 0;
    }

    pub fn move_dictation_cursor_end(&mut self) {
        self.dictation_cursor = self.dictation_input.chars().count();
    }

    pub fn backspace_dictation(&mut self) {
        if self.dictation_cursor == 0 {
            return;
        }

        self.mark_dictation_started();
        let start = byte_index_for_char(&self.dictation_input, self.dictation_cursor - 1);
        let end = byte_index_for_char(&self.dictation_input, self.dictation_cursor);
        self.dictation_input.replace_range(start..end, "");
        self.dictation_cursor -= 1;
    }

    pub fn delete_dictation(&mut self) {
        let char_len = self.dictation_input.chars().count();
        if self.dictation_cursor >= char_len {
            return;
        }

        self.mark_dictation_started();
        let start = byte_index_for_char(&self.dictation_input, self.dictation_cursor);
        let end = byte_index_for_char(&self.dictation_input, self.dictation_cursor + 1);
        self.dictation_input.replace_range(start..end, "");
    }

    pub fn submit_dictation(&mut self) {
        let Some(command) = self.current_dictation_command().cloned() else {
            return;
        };

        let submitted = self.dictation_input.clone();
        let evaluation = Matcher::check(&submitted, &command.dictation.answers);
        let record = self.build_dictation_record(&command, &evaluation);

        self.dictation_result = Some(DictationResult {
            submitted,
            evaluation,
        });
        self.pending_record = Some(record);
    }

    pub fn take_pending_record(&mut self) -> Option<SessionRecord> {
        self.pending_record.take()
    }

    pub fn advance_round_result(&mut self) -> Option<AppState> {
        let mode = self.round_result.as_ref()?.mode;
        self.round_result = None;

        match mode {
            Mode::Type => {
                self.next_typing_command();
                self.state = AppState::Typing;
                Some(AppState::Typing)
            }
            Mode::Learn => {
                self.next_learn_command();
                self.state = AppState::Learn;
                Some(AppState::Learn)
            }
            Mode::Dictation => None,
        }
    }

    pub fn retry_round_result(&mut self) -> Option<AppState> {
        let mode = self.round_result.as_ref()?.mode;
        self.round_result = None;

        match mode {
            Mode::Type => {
                self.retry_typing_command();
                self.state = AppState::Typing;
                Some(AppState::Typing)
            }
            Mode::Learn => {
                self.retry_learn_command();
                self.state = AppState::Learn;
                Some(AppState::Learn)
            }
            Mode::Dictation => None,
        }
    }

    pub fn return_home(&mut self) -> AppState {
        self.typing_engine = None;
        self.learn_engine = None;
        self.dictation_result = None;
        self.dictation_input.clear();
        self.dictation_cursor = 0;
        self.dictation_started_at = None;
        self.round_result = None;
        self.state = AppState::Home;
        AppState::Home
    }

    fn filtered_commands(&self) -> Vec<Command> {
        let mut commands = loader::load_by_difficulty(&self.all_commands, self.selected_difficulty);
        if let Some(category) = self.selected_category {
            commands.retain(|command| command.category == category);
        }
        commands
    }

    fn reset_dictation_prompt_state(&mut self) {
        self.dictation_input.clear();
        self.dictation_cursor = 0;
        self.dictation_result = None;
        self.dictation_started_at = None;
    }

    fn mark_dictation_started(&mut self) {
        if self.dictation_started_at.is_none() {
            self.dictation_started_at = Some(Instant::now());
        }
    }

    fn build_dictation_record(&self, command: &Command, evaluation: &MatchResult) -> SessionRecord {
        let now_ms = Utc::now().timestamp_millis();
        let elapsed_secs = self
            .dictation_started_at
            .map(|started_at| started_at.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        let elapsed_ms = (elapsed_secs * 1000.0) as i64;
        let reference_answer = match evaluation {
            MatchResult::Exact(index) | MatchResult::Normalized(index) => command
                .dictation
                .answers
                .get(*index)
                .map(String::as_str)
                .unwrap_or(command.command.as_str()),
            MatchResult::NoMatch { closest, .. } if !closest.is_empty() => closest.as_str(),
            MatchResult::NoMatch { .. } => command.command.as_str(),
        };
        let char_count = reference_answer.chars().count() as f64;
        let elapsed_minutes = elapsed_secs / 60.0;
        let wpm = if elapsed_minutes > 0.0 {
            (char_count / 5.0) / elapsed_minutes
        } else {
            0.0
        };
        let cpm = if elapsed_minutes > 0.0 {
            char_count / elapsed_minutes
        } else {
            0.0
        };
        let (accuracy, error_count) = dictation_metrics(evaluation);

        SessionRecord {
            id: format!("{now_ms}"),
            command_id: command.id.clone(),
            mode: Mode::Dictation,
            keystrokes: Vec::new(),
            started_at: now_ms - elapsed_ms,
            finished_at: now_ms,
            wpm,
            cpm,
            accuracy,
            error_count,
            difficulty: command.difficulty,
        }
    }
}

fn dictation_metrics(result: &MatchResult) -> (f64, u32) {
    match result {
        MatchResult::Exact(_) | MatchResult::Normalized(_) => (1.0, 0),
        MatchResult::NoMatch { diff, .. } => {
            let same = diff
                .iter()
                .filter(|segment| segment.kind == DiffKind::Same)
                .map(|segment| segment.text.chars().count() as u32)
                .sum::<u32>();
            let changes = diff
                .iter()
                .filter(|segment| segment.kind != DiffKind::Same)
                .map(|segment| segment.text.chars().count() as u32)
                .sum::<u32>();
            let total = same + changes;
            if total == 0 {
                (0.0, 0)
            } else {
                (same as f64 / total as f64, changes)
            }
        }
    }
}

fn byte_index_for_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

fn load_command_catalog() -> Vec<Command> {
    let data_dir = Path::new("data/commands");
    match loader::load_commands(data_dir) {
        Ok(commands) if !commands.is_empty() => commands,
        Ok(_) => hardcoded_commands(),
        Err(err) => {
            eprintln!(
                "warning: failed to load TOML commands from {}: {}; using fallback set",
                data_dir.display(),
                err
            );
            hardcoded_commands()
        }
    }
}

/// Hardcoded test commands for when no TOML files exist
fn hardcoded_commands() -> Vec<Command> {
    vec![
        Command {
            id: "ls-basic".to_string(),
            command: "ls -la /var/log".to_string(),
            summary: "显示 /var/log 目录的详细列表（含隐藏文件）".to_string(),
            category: Category::FileOps,
            difficulty: Difficulty::Beginner,
            tokens: vec![
                Token {
                    text: "ls".to_string(),
                    desc: "列出目录内容".to_string(),
                },
                Token {
                    text: "-la".to_string(),
                    desc: "-l 详细列表 + -a 显示隐藏文件".to_string(),
                },
                Token {
                    text: "/var/log".to_string(),
                    desc: "系统日志目录".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "显示 /var/log 目录下所有文件的详细信息".to_string(),
                answers: vec!["ls -la /var/log".to_string(), "ls -al /var/log".to_string()],
            },
        },
        Command {
            id: "cd-home".to_string(),
            command: "cd ~".to_string(),
            summary: "切换到当前用户的主目录".to_string(),
            category: Category::FileOps,
            difficulty: Difficulty::Beginner,
            tokens: vec![
                Token {
                    text: "cd".to_string(),
                    desc: "切换工作目录".to_string(),
                },
                Token {
                    text: "~".to_string(),
                    desc: "主目录的简写符号".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "切换到主目录".to_string(),
                answers: vec!["cd ~".to_string(), "cd $HOME".to_string()],
            },
        },
        Command {
            id: "mkdir-p".to_string(),
            command: "mkdir -p src/core/utils".to_string(),
            summary: "递归创建多层目录结构".to_string(),
            category: Category::FileOps,
            difficulty: Difficulty::Basic,
            tokens: vec![
                Token {
                    text: "mkdir".to_string(),
                    desc: "创建新目录".to_string(),
                },
                Token {
                    text: "-p".to_string(),
                    desc: "自动创建不存在的父目录".to_string(),
                },
                Token {
                    text: "src/core/utils".to_string(),
                    desc: "目标路径（多层嵌套）".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "递归创建 src/core/utils 目录".to_string(),
                answers: vec![
                    "mkdir -p src/core/utils".to_string(),
                    "mkdir --parents src/core/utils".to_string(),
                ],
            },
        },
        Command {
            id: "grep-rn".to_string(),
            command: "grep -rn \"TODO\" src/".to_string(),
            summary: "在 src 目录下递归搜索包含 TODO 的行".to_string(),
            category: Category::Search,
            difficulty: Difficulty::Basic,
            tokens: vec![
                Token {
                    text: "grep".to_string(),
                    desc: "文本搜索工具".to_string(),
                },
                Token {
                    text: "-rn".to_string(),
                    desc: "-r 递归搜索 + -n 显示行号".to_string(),
                },
                Token {
                    text: "\"TODO\"".to_string(),
                    desc: "要搜索的模式字符串".to_string(),
                },
                Token {
                    text: "src/".to_string(),
                    desc: "搜索的目标目录".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "在 src 目录递归搜索 TODO 并显示行号".to_string(),
                answers: vec![
                    "grep -rn \"TODO\" src/".to_string(),
                    "grep -rn 'TODO' src/".to_string(),
                    "grep -nr \"TODO\" src/".to_string(),
                ],
            },
        },
        Command {
            id: "chmod-755".to_string(),
            command: "chmod 755 deploy.sh".to_string(),
            summary: "设置脚本文件为可执行权限".to_string(),
            category: Category::Permission,
            difficulty: Difficulty::Basic,
            tokens: vec![
                Token {
                    text: "chmod".to_string(),
                    desc: "修改文件权限".to_string(),
                },
                Token {
                    text: "755".to_string(),
                    desc: "rwxr-xr-x 所有者可读写执行，其他人可读执行".to_string(),
                },
                Token {
                    text: "deploy.sh".to_string(),
                    desc: "目标脚本文件".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "给 deploy.sh 设置 755 权限".to_string(),
                answers: vec!["chmod 755 deploy.sh".to_string()],
            },
        },
        Command {
            id: "cat-pipe-grep".to_string(),
            command: "cat /etc/passwd | grep root".to_string(),
            summary: "查看密码文件中包含 root 的行".to_string(),
            category: Category::Pipeline,
            difficulty: Difficulty::Advanced,
            tokens: vec![
                Token {
                    text: "cat".to_string(),
                    desc: "输出文件内容".to_string(),
                },
                Token {
                    text: "/etc/passwd".to_string(),
                    desc: "系统用户账户文件".to_string(),
                },
                Token {
                    text: "|".to_string(),
                    desc: "管道符，将前一命令输出传给后一命令".to_string(),
                },
                Token {
                    text: "grep".to_string(),
                    desc: "文本搜索过滤".to_string(),
                },
                Token {
                    text: "root".to_string(),
                    desc: "要搜索的关键词".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "从 /etc/passwd 中搜索包含 root 的行".to_string(),
                answers: vec![
                    "cat /etc/passwd | grep root".to_string(),
                    "grep root /etc/passwd".to_string(),
                ],
            },
        },
        Command {
            id: "tar-czf".to_string(),
            command: "tar -czf backup.tar.gz /home/user".to_string(),
            summary: "将 /home/user 目录压缩为 tar.gz 归档".to_string(),
            category: Category::Archive,
            difficulty: Difficulty::Advanced,
            tokens: vec![
                Token {
                    text: "tar".to_string(),
                    desc: "归档工具".to_string(),
                },
                Token {
                    text: "-czf".to_string(),
                    desc: "-c 创建 + -z gzip 压缩 + -f 指定文件名".to_string(),
                },
                Token {
                    text: "backup.tar.gz".to_string(),
                    desc: "输出的归档文件名".to_string(),
                },
                Token {
                    text: "/home/user".to_string(),
                    desc: "要归档的目标目录".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "将 /home/user 目录压缩为 backup.tar.gz".to_string(),
                answers: vec![
                    "tar -czf backup.tar.gz /home/user".to_string(),
                    "tar czf backup.tar.gz /home/user".to_string(),
                ],
            },
        },
        Command {
            id: "find-name".to_string(),
            command: "find / -name \"*.log\" -type f".to_string(),
            summary: "在根目录下查找所有 .log 文件".to_string(),
            category: Category::Search,
            difficulty: Difficulty::Practical,
            tokens: vec![
                Token {
                    text: "find".to_string(),
                    desc: "文件查找工具".to_string(),
                },
                Token {
                    text: "/".to_string(),
                    desc: "从根目录开始搜索".to_string(),
                },
                Token {
                    text: "-name \"*.log\"".to_string(),
                    desc: "按文件名模式匹配 .log 后缀".to_string(),
                },
                Token {
                    text: "-type f".to_string(),
                    desc: "只查找普通文件（不含目录）".to_string(),
                },
            ],
            dictation: DictationData {
                prompt: "在整个系统中查找所有 .log 文件".to_string(),
                answers: vec![
                    "find / -name \"*.log\" -type f".to_string(),
                    "find / -type f -name \"*.log\"".to_string(),
                ],
            },
        },
    ]
}
