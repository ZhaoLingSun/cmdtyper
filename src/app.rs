#![allow(dead_code)]

use crate::core::engine::TypingEngine;
use crate::data::models::{
    Category, Command, DictationData, Difficulty, Token, UserConfig, UserStats,
};

/// Application state machine
pub enum AppState {
    Home,
    Learn,
    Typing,
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

/// Main application struct
pub struct App {
    pub state: AppState,
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
}

impl App {
    pub fn new() -> Self {
        let commands = hardcoded_commands();
        Self {
            state: AppState::Home,
            commands,
            user_stats: UserStats::default(),
            user_config: UserConfig::default(),
            selected_difficulty: Difficulty::Beginner,
            selected_category: None,
            menu_index: 0,
            typing_engine: None,
            current_command_index: 0,
            learn_command_index: 0,
            learn_engine: None,
        }
    }

    pub fn current_menu_item(&self) -> MenuItem {
        MenuItem::ALL[self.menu_index]
    }

    pub fn enter_typing_mode(&mut self) {
        if self.commands.is_empty() {
            return;
        }
        self.current_command_index = 0;
        let cmd = &self.commands[self.current_command_index];
        self.typing_engine = Some(TypingEngine::new(&cmd.command));
        self.state = AppState::Typing;
    }

    pub fn enter_learn_mode(&mut self) {
        if self.commands.is_empty() {
            return;
        }
        self.learn_command_index = 0;
        let cmd = &self.commands[self.learn_command_index];
        self.learn_engine = Some(TypingEngine::new(&cmd.command));
        self.state = AppState::Learn;
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
        }
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
                answers: vec![
                    "ls -la /var/log".to_string(),
                    "ls -al /var/log".to_string(),
                ],
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
