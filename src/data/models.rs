use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// 3.1 基础枚举
// ─────────────────────────────────────────────────────────────

/// 难度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Beginner,
    Basic,
    Advanced,
    Practical,
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Beginner, Self::Basic, Self::Advanced, Self::Practical];

    pub fn label(&self) -> &str {
        match self {
            Self::Beginner => "入门",
            Self::Basic => "基础",
            Self::Advanced => "进阶",
            Self::Practical => "实战",
        }
    }

    pub fn stars(&self) -> &str {
        match self {
            Self::Beginner => "★☆☆☆",
            Self::Basic => "★★☆☆",
            Self::Advanced => "★★★☆",
            Self::Practical => "★★★★",
        }
    }

    pub fn target_attempts(&self) -> u32 {
        match self {
            Self::Beginner => 3,
            Self::Basic => 5,
            Self::Advanced => 8,
            Self::Practical => 10,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Beginner => Self::Basic,
            Self::Basic => Self::Advanced,
            Self::Advanced => Self::Practical,
            Self::Practical => Self::Beginner,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Beginner => Self::Practical,
            Self::Basic => Self::Beginner,
            Self::Advanced => Self::Basic,
            Self::Practical => Self::Advanced,
        }
    }
}

/// 命令类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    #[default]
    FileOps,
    Permission,
    TextProcess,
    Search,
    Process,
    Network,
    Archive,
    System,
    Pipeline,
    Scripting,
}

impl Category {
    pub const ALL: [Self; 10] = [
        Self::FileOps,
        Self::Permission,
        Self::TextProcess,
        Self::Search,
        Self::Process,
        Self::Network,
        Self::Archive,
        Self::System,
        Self::Pipeline,
        Self::Scripting,
    ];

    pub fn label(&self) -> &str {
        match self {
            Self::FileOps => "文件操作",
            Self::Permission => "权限管理",
            Self::TextProcess => "文本处理",
            Self::Search => "搜索查找",
            Self::Process => "进程管理",
            Self::Network => "网络",
            Self::Archive => "压缩归档",
            Self::System => "系统信息",
            Self::Pipeline => "管道与重定向",
            Self::Scripting => "脚本片段",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::FileOps => "📁",
            Self::Permission => "🔒",
            Self::TextProcess => "📋",
            Self::Search => "🔍",
            Self::Process => "⚙️",
            Self::Network => "🌐",
            Self::Archive => "📦",
            Self::System => "💻",
            Self::Pipeline => "🔀",
            Self::Scripting => "📜",
        }
    }
}

/// 练习模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Learn,
    #[default]
    Type,
    Dictation,
}

/// 会话记录模式（统计口径）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecordMode {
    #[default]
    #[serde(alias = "type", alias = "typing")]
    Typing,
    #[serde(alias = "dictation")]
    Dictation,
    #[serde(alias = "learn", alias = "lesson_practice")]
    LessonPractice,
    #[serde(alias = "symbol_practice")]
    SymbolPractice,
    #[serde(alias = "review_practice")]
    ReviewPractice,
}

/// 重要程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    #[default]
    Core,
    Common,
    Advanced,
    Niche,
}

/// 提示符风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PromptStyle {
    #[default]
    Full,
    Simple,
    Minimal,
}

// ─────────────────────────────────────────────────────────────
// 3.2 命令题库（对着打 + 默写 共用）
// ─────────────────────────────────────────────────────────────

/// 题库文件顶层结构 — 对应 data/commands/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFile {
    pub meta: CommandFileMeta,
    pub commands: Vec<Command>,
}

/// 题库文件元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFileMeta {
    pub category: Category,
    pub difficulty: Difficulty,
    pub description: String,
}

/// 单条命令（题库原子单位）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub command: String,
    pub summary: String,
    pub tokens: Vec<Token>,
    pub dictation: DictationData,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub summary_short: Option<String>,
    #[serde(default)]
    pub simulated_output: Option<String>,
    #[serde(default)]
    pub output_annotations: Vec<OutputAnnotation>,
    #[serde(skip)]
    pub category: Category,
    #[serde(skip)]
    pub difficulty: Difficulty,
}

impl Command {
    /// 返回终端显示文本（display 优先，否则 command）
    pub fn display_text(&self) -> &str {
        self.display.as_deref().unwrap_or(&self.command)
    }

    /// 返回底栏短提示（summary_short 优先，否则 summary）
    pub fn short_summary(&self) -> &str {
        self.summary_short.as_deref().unwrap_or(&self.summary)
    }
}

/// 单个 token（命令中的一个词元）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub text: String,
    pub desc: String,
}

/// 默写模式数据
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictationData {
    pub prompt: String,
    pub answers: Vec<String>,
}

/// 模拟输出注释
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAnnotation {
    pub pattern: String,
    pub note: String,
}

// ─────────────────────────────────────────────────────────────
// 3.3 命令讲解（学习中心 · 命令专题）
// ─────────────────────────────────────────────────────────────

/// 命令讲解 — 对应 data/lessons/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLesson {
    pub meta: LessonMeta,
    pub overview: LessonOverview,
    pub syntax: SyntaxInfo,
    #[serde(default)]
    pub options: Vec<OptionInfo>,
    pub examples: Vec<LessonExample>,
    #[serde(default)]
    pub gotchas: Vec<Gotcha>,
}

/// 讲解元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonMeta {
    pub command: String,
    #[serde(default)]
    pub full_name: Option<String>,
    pub category: Category,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub importance: Importance,
}

/// 命令概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonOverview {
    pub summary: String,
    pub explanation: String,
}

/// 语法信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxInfo {
    pub basic: String,
    #[serde(default)]
    pub parts: Vec<SyntaxPart>,
}

/// 语法组成部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxPart {
    pub name: String,
    pub desc: String,
}

/// 选项信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionInfo {
    pub flag: String,
    pub name: String,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// 讲解示例中的词元详解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleTokenDetail {
    pub token: String,
    pub explanation: String,
}

/// 讲解示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonExample {
    pub level: u8,
    pub command: String,
    pub summary: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub simulated_output: Option<String>,
    #[serde(default)]
    pub output_annotations: Vec<OutputAnnotation>,
    #[serde(default)]
    pub token_details: Vec<ExampleTokenDetail>,
    #[serde(default)]
    pub output_explanation: Option<String>,
}

/// 易错点/陷阱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gotcha {
    pub title: String,
    pub content: String,
}

// ─────────────────────────────────────────────────────────────
// 3.4 符号专题
// ─────────────────────────────────────────────────────────────

/// 符号专题 — 对应 data/symbols/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTopic {
    pub meta: SymbolTopicMeta,
    pub symbols: Vec<SymbolEntry>,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

/// 符号专题元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTopicMeta {
    pub id: String,
    pub topic: String,
    pub description: String,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub icon: Option<String>,
}

/// 单个符号条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub id: String,
    pub char_repr: String,
    pub name: String,
    pub summary: String,
    pub explanation: String,
    pub examples: Vec<SymbolExample>,
}

/// 符号示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolExample {
    pub command: String,
    pub explanation: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub simulated_output: Option<String>,
}

/// 练习题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub prompt: String,
    pub answers: Vec<String>,
}

// ─────────────────────────────────────────────────────────────
// 3.5 系统架构专题
// ─────────────────────────────────────────────────────────────

/// 系统架构专题 — 对应 data/system/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTopic {
    pub meta: SystemTopicMeta,
    #[serde(default)]
    pub overview: Option<String>,
    pub sections: Vec<SystemSection>,
}

/// 系统架构元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTopicMeta {
    pub id: String,
    pub topic: String,
    pub description: String,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub icon: Option<String>,
}

/// 系统架构章节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSection {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub commands: Vec<SystemCommand>,
    #[serde(default)]
    pub config_files: Vec<ConfigFile>,
}

/// 系统命令（含模拟输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCommand {
    pub command: String,
    pub summary: String,
    #[serde(default)]
    pub simulated_output: Option<String>,
}

/// 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub id: String,
    pub path: String,
    pub name: String,
    pub description: String,
    pub sample_content: String,
    #[serde(default)]
    pub lessons: Vec<ConfigLesson>,
}

/// 配置文件讲解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLesson {
    pub title: String,
    pub before: String,
    pub after: String,
    pub explanation: String,
    #[serde(default)]
    pub practice_command: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// 3.6 复习模块
// ─────────────────────────────────────────────────────────────

/// 复习数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub topic_id: String,
    pub topic_name: String,
    pub summary_groups: Vec<ReviewGroup>,
    pub practice_ids: Vec<String>,
}

/// 复习分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewGroup {
    pub name: String,
    pub items: Vec<ReviewItem>,
}

/// 复习条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub command: String,
    pub brief: String,
}

// ─────────────────────────────────────────────────────────────
// 3.7 打字引擎相关数据类型
// ─────────────────────────────────────────────────────────────

/// 单次击键记录
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keystroke {
    pub expected: char,
    pub actual: char,
    pub correct: bool,
    pub attempts: u8,
    pub latency_ms: u64,
    pub timestamp_ms: i64,
}

/// 单次练习会话记录
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub command_id: String,
    pub mode: RecordMode,
    pub keystrokes: Vec<Keystroke>,
    pub started_at: i64,
    pub finished_at: i64,
    pub wpm: f64,
    pub cpm: f64,
    pub accuracy: f64,
    pub error_count: u32,
    pub difficulty: Difficulty,
}

// ─────────────────────────────────────────────────────────────
// 3.8 终端历史（对着打专用）
// ─────────────────────────────────────────────────────────────

/// 终端行
#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub prompt: String,
    pub command_display: String,
    pub status: LineStatus,
}

/// 行状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineStatus {
    Completed,
    Current,
    Pending,
}

// ─────────────────────────────────────────────────────────────
// 3.9 统计与持久化数据类型
// ─────────────────────────────────────────────────────────────

/// 单个字符的聚合统计
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharStat {
    pub char_key: char,
    pub total_correct: u64,
    pub total_errors: u64,
    pub total_samples: u64,
    pub avg_latency_ms: f64,
    pub avg_cpm: f64,
    pub accuracy: f64,
    pub history: Vec<CharSpeedPoint>,
}

/// 字符速度数据点
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharSpeedPoint {
    pub session_index: u32,
    pub cpm: f64,
    pub accuracy: f64,
}

/// 命令进度
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandProgress {
    pub command_id: String,
    pub times_practiced: u32,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub last_practiced: Option<i64>,
    pub mastery: f64,
}

/// 每日统计
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub sessions_count: u32,
    pub total_duration_ms: u64,
    pub avg_wpm: f64,
    pub avg_accuracy: f64,
    #[serde(default)]
    pub wpm_sessions_count: u32,
}

/// 全局用户统计
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserStats {
    pub total_sessions: u64,
    pub total_keystrokes: u64,
    pub total_duration_ms: u64,
    pub overall_avg_wpm: f64,
    pub overall_avg_accuracy: f64,
    pub best_wpm: f64,
    #[serde(default)]
    pub total_wpm_sessions: u64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub char_stats: Vec<CharStat>,
    pub command_progress: Vec<CommandProgress>,
    pub daily_stats: Vec<DailyStat>,
}

/// 用户配置（v0.2 扩展 prompt_* 和 PromptStyle 字段）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserConfig {
    pub target_wpm: f64,
    pub error_flash_ms: u64,
    pub show_token_hints: bool,
    pub adaptive_recommend: bool,
    pub last_difficulty: Difficulty,
    pub last_category: Option<Category>,
    #[serde(default = "default_prompt_style")]
    pub prompt_style: PromptStyle,
    #[serde(default = "default_prompt_username")]
    pub prompt_username: String,
    #[serde(default = "default_prompt_hostname")]
    pub prompt_hostname: String,
    #[serde(default = "default_true")]
    pub show_path: bool,
}

fn default_prompt_style() -> PromptStyle {
    PromptStyle::Full
}

fn default_prompt_username() -> String {
    "user".to_string()
}

fn default_prompt_hostname() -> String {
    "cmdtyper".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            target_wpm: 40.0,
            error_flash_ms: 150,
            show_token_hints: true,
            adaptive_recommend: true,
            last_difficulty: Difficulty::Beginner,
            last_category: None,
            prompt_style: PromptStyle::Full,
            prompt_username: "user".to_string(),
            prompt_hostname: "cmdtyper".to_string(),
            show_path: true,
        }
    }
}
