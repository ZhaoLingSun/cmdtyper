#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 难度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Beginner,
    Basic,
    Advanced,
    Practical,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Beginner
    }
}

impl Difficulty {
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

    pub const ALL: [Difficulty; 4] = [
        Difficulty::Beginner,
        Difficulty::Basic,
        Difficulty::Advanced,
        Difficulty::Practical,
    ];

    pub fn next(&self) -> Difficulty {
        match self {
            Self::Beginner => Self::Basic,
            Self::Basic => Self::Advanced,
            Self::Advanced => Self::Practical,
            Self::Practical => Self::Beginner,
        }
    }

    pub fn prev(&self) -> Difficulty {
        match self {
            Self::Beginner => Self::Practical,
            Self::Basic => Self::Beginner,
            Self::Advanced => Self::Basic,
            Self::Practical => Self::Advanced,
        }
    }
}

/// 命令类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
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

impl Default for Category {
    fn default() -> Self {
        Self::FileOps
    }
}

impl Category {
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

    pub const ALL: [Category; 10] = [
        Category::FileOps,
        Category::Permission,
        Category::TextProcess,
        Category::Search,
        Category::Process,
        Category::Network,
        Category::Archive,
        Category::System,
        Category::Pipeline,
        Category::Scripting,
    ];
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

/// 单条命令（题库的原子单位）
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub command: String,
    pub summary: String,
    pub tokens: Vec<Token>,
    pub dictation: DictationData,
    #[serde(skip)]
    pub category: Category,
    #[serde(skip)]
    pub difficulty: Difficulty,
}

/// 题库文件（一个 TOML 文件的顶层结构）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandFile {
    pub meta: FileMeta,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileMeta {
    pub category: Category,
    pub difficulty: Difficulty,
    pub description: String,
}

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

/// 练习模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Learn,
    Type,
    Dictation,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Type
    }
}

/// 单次练习会话记录
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub command_id: String,
    pub mode: Mode,
    pub keystrokes: Vec<Keystroke>,
    pub started_at: i64,
    pub finished_at: i64,
    pub wpm: f64,
    pub cpm: f64,
    pub accuracy: f64,
    pub error_count: u32,
    pub difficulty: Difficulty,
}

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
    pub current_streak: u32,
    pub longest_streak: u32,
    pub char_stats: Vec<CharStat>,
    pub command_progress: Vec<CommandProgress>,
    pub daily_stats: Vec<DailyStat>,
}

/// 用户配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserConfig {
    pub target_wpm: f64,
    pub error_flash_ms: u64,
    pub show_token_hints: bool,
    pub adaptive_recommend: bool,
    pub last_difficulty: Difficulty,
    pub last_category: Option<Category>,
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
        }
    }
}
