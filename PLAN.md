# cmdtyper 技术规划文档

## 1. 项目概述

**cmdtyper** 是一个基于 Rust + ratatui 的终端 TUI 程序，专注于 Linux 命令行打字练习。

### 1.1 核心功能

| 模式 | 功能 | 核心体验 |
|------|------|----------|
| 学习模式 (Learn) | 逐命令讲解语法 + 跟打 | 每个 token 带中文注释，用户跟着输入 |
| 对着打模式 (Type) | 逐字符匹配打字 | 灰底待输入 → 打对变黑 → 打错闪红不前进 |
| 默写模式 (Dictation) | 看中文描述写命令 | 一题多解，多答案匹配 |
| 统计面板 (Stats) | keybr 风格数据分析 | 速度/准确率/字符分析/类别掌握度/日历 |

### 1.2 技术栈

- **语言**: Rust (edition 2024)
- **TUI 框架**: ratatui 0.29 + crossterm 0.28
- **题库格式**: TOML
- **持久化**: JSON (serde_json)
- **容器化**: Docker (multi-stage build)

---

## 2. 数据接口与类型定义

### 2.1 题库数据结构 (TOML → Rust)

#### TOML 格式规范

```toml
# data/commands/01_beginner.toml

[meta]
category = "文件操作"           # 类别名称
difficulty = "beginner"         # beginner | basic | advanced | practical
description = "基础文件和目录操作命令"

[[commands]]
id = "ls-basic"                 # 唯一标识符，格式: {命令名}-{变体}
command = "ls -la /var/log"     # 完整命令字符串
summary = "显示 /var/log 目录的详细列表（含隐藏文件）"  # 学习模式概述

[[commands.tokens]]
text = "ls"
desc = "列出目录内容的命令"

[[commands.tokens]]
text = "-la"
desc = "-l 详细列表格式 + -a 显示隐藏文件（以 . 开头的文件）"

[[commands.tokens]]
text = "/var/log"
desc = "目标目录：系统日志文件存放位置"

# 默写模式
[commands.dictation]
prompt = "显示 /var/log 目录下所有文件（包含隐藏文件）的详细信息"
answers = [
    "ls -la /var/log",
    "ls -al /var/log",
    "ls --all -l /var/log",
    "ls -l -a /var/log",
]
```

#### Rust 数据结构

```rust
// src/data/models.rs

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

impl Difficulty {
    pub fn label(&self) -> &str {
        match self {
            Self::Beginner  => "入门",
            Self::Basic     => "基础",
            Self::Advanced  => "进阶",
            Self::Practical => "实战",
        }
    }

    pub fn stars(&self) -> &str {
        match self {
            Self::Beginner  => "★☆☆☆",
            Self::Basic     => "★★☆☆",
            Self::Advanced  => "★★★☆",
            Self::Practical => "★★★★",
        }
    }

    /// 掌握度计算所需的目标练习次数
    pub fn target_attempts(&self) -> u32 {
        match self {
            Self::Beginner  => 3,
            Self::Basic     => 5,
            Self::Advanced  => 8,
            Self::Practical => 10,
        }
    }
}

/// 命令类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    FileOps,      // 文件操作
    Permission,   // 权限管理
    TextProcess,  // 文本处理
    Search,       // 搜索查找
    Process,      // 进程管理
    Network,      // 网络
    Archive,      // 压缩归档
    System,       // 系统信息
    Pipeline,     // 管道与重定向
    Scripting,    // 脚本片段
}

impl Category {
    pub fn label(&self) -> &str {
        match self {
            Self::FileOps     => "文件操作",
            Self::Permission  => "权限管理",
            Self::TextProcess => "文本处理",
            Self::Search      => "搜索查找",
            Self::Process     => "进程管理",
            Self::Network     => "网络",
            Self::Archive     => "压缩归档",
            Self::System      => "系统信息",
            Self::Pipeline    => "管道与重定向",
            Self::Scripting   => "脚本片段",
        }
    }
}

/// 单个 token（命令中的一个词元）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub text: String,   // 原始文本，如 "chmod"
    pub desc: String,   // 中文解释，如 "修改文件权限的命令"
}

/// 默写模式数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationData {
    pub prompt: String,             // 中文题目
    pub answers: Vec<String>,       // 所有正确答案
}

/// 单条命令（题库的原子单位）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,                 // 唯一 ID
    pub command: String,            // 完整命令字符串
    pub summary: String,            // 学习模式概述
    pub tokens: Vec<Token>,         // token 级拆分
    pub dictation: DictationData,   // 默写数据
}

/// 题库文件（一个 TOML 文件的顶层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFile {
    pub meta: FileMeta,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub category: Category,
    pub difficulty: Difficulty,
    pub description: String,
}
```

### 2.2 击键记录数据结构

```rust
// src/data/models.rs (续)

/// 单次击键记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keystroke {
    pub expected: char,     // 目标字符
    pub actual: char,       // 实际输入字符（正确时 == expected）
    pub correct: bool,      // 是否一次打对
    pub attempts: u8,       // 该位置总尝试次数（>=1）
    pub latency_ms: u64,    // 距上一个正确击键的时间间隔 (ms)
    pub timestamp_ms: i64,  // Unix 毫秒时间戳
}

/// 练习模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Learn,
    Type,
    Dictation,
}

/// 单次练习会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,                 // UUID
    pub command_id: String,         // 练习的命令 ID
    pub mode: Mode,                 // 练习模式
    pub keystrokes: Vec<Keystroke>, // 所有击键
    pub started_at: i64,            // 开始时间 (Unix ms)
    pub finished_at: i64,           // 结束时间 (Unix ms)
    pub wpm: f64,                   // 本次 WPM
    pub cpm: f64,                   // 本次 CPM
    pub accuracy: f64,              // 本次准确率 (0.0 ~ 1.0)
    pub error_count: u32,           // 总错误次数
}
```

### 2.3 统计聚合数据结构

```rust
// src/data/models.rs (续)

/// 单个字符的聚合统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharStat {
    pub char_key: char,         // 统计的字符
    pub total_correct: u64,     // 累计正确次数
    pub total_errors: u64,      // 累计错误次数
    pub total_samples: u64,     // 累计样本数 (= correct + errors on first attempt)
    pub avg_latency_ms: f64,    // 平均击键间隔
    pub avg_cpm: f64,           // 该字符平均 CPM
    pub accuracy: f64,          // 准确率 (0.0 ~ 1.0)
    pub history: Vec<CharSpeedPoint>, // 速度变化历史
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharSpeedPoint {
    pub session_index: u32,     // 第几次练习
    pub cpm: f64,               // 当次该字符的 CPM
    pub accuracy: f64,          // 当次该字符准确率
}

/// 命令进度
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandProgress {
    pub command_id: String,
    pub times_practiced: u32,       // 练习次数
    pub best_wpm: f64,              // 最佳 WPM
    pub best_accuracy: f64,         // 最佳准确率
    pub last_practiced: Option<i64>, // 上次练习时间
    pub mastery: f64,               // 掌握度 (0.0 ~ 1.0)
}

/// 每日统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,               // "2026-03-08"
    pub sessions_count: u32,        // 练习次数
    pub total_duration_ms: u64,     // 总练习时长
    pub avg_wpm: f64,               // 平均 WPM
    pub avg_accuracy: f64,          // 平均准确率
}

/// 全局用户统计（持久化的顶层结构）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserStats {
    pub total_sessions: u64,
    pub total_keystrokes: u64,
    pub total_duration_ms: u64,
    pub overall_avg_wpm: f64,
    pub overall_avg_accuracy: f64,
    pub best_wpm: f64,
    pub current_streak: u32,        // 当前连续天数
    pub longest_streak: u32,        // 最长连续天数
    pub char_stats: Vec<CharStat>,  // 按字符统计
    pub command_progress: Vec<CommandProgress>,
    pub daily_stats: Vec<DailyStat>,
}
```

### 2.4 持久化文件布局

```
~/.local/share/cmdtyper/
├── history.json          # Vec<SessionRecord> - 所有练习记录
├── stats.json            # UserStats - 聚合统计
└── config.json           # UserConfig - 用户设置
```

#### 用户配置

```rust
// src/data/models.rs (续)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub target_wpm: f64,            // 目标 WPM，默认 40.0
    pub error_flash_ms: u64,        // 错误闪红持续时间，默认 150
    pub show_token_hints: bool,     // 对着打时是否显示 token 注释，默认 true
    pub adaptive_recommend: bool,   // 是否开启智能推荐，默认 true
    pub last_difficulty: Difficulty, // 上次选择的难度
    pub last_category: Option<Category>, // 上次选择的类别
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
```

---

## 3. 核心模块设计

### 3.1 App 状态机 (`src/app.rs`)

```
App 状态转换图：

                    ┌──────────────────────────────┐
                    │         Home (主菜单)          │
                    │  选择模式 / 难度 / 类别        │
                    └──────┬───┬───┬───┬────────────┘
                           │   │   │   │
              ┌────────────┘   │   │   └────────────┐
              ▼                ▼   ▼                ▼
         ┌─────────┐   ┌──────────┐  ┌───────────┐  ┌────────┐
         │  Learn   │   │   Type   │  │ Dictation │  │ Stats  │
         │ 学习模式 │   │ 对着打   │  │ 默写模式  │  │ 统计   │
         └────┬────┘   └────┬─────┘  └─────┬─────┘  └────────┘
              │             │              │
              └─────────────┴──────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ RoundResult  │
                    │ 本轮结果展示 │
                    └──────────────┘

Esc 键：任何模式 → 返回 Home
Tab 键：Learn/Type 模式中跳过当前命令
Ctrl+R：重练当前命令
```

```rust
// src/app.rs

pub enum AppState {
    Home,
    Learn(LearnState),
    Typing(TypingState),
    Dictation(DictationState),
    Stats(StatsTab),
    RoundResult(RoundResultState),
    Quitting,
}

pub struct App {
    pub state: AppState,
    pub commands: Vec<Command>,         // 已加载的题库
    pub user_stats: UserStats,          // 用户统计
    pub user_config: UserConfig,        // 用户配置
    pub selected_difficulty: Difficulty, // 当前选择的难度
    pub selected_category: Option<Category>, // 当前选择的类别
}
```

### 3.2 打字引擎 (`src/core/engine.rs`)

核心逻辑：逐字符匹配，记录每次击键。

```rust
// src/core/engine.rs

pub struct TypingEngine {
    pub target: Vec<char>,          // 目标字符序列
    pub cursor: usize,              // 当前光标位置
    pub keystrokes: Vec<Keystroke>, // 已记录的击键
    pub current_attempts: u8,       // 当前字符的尝试次数
    pub error_flash: Option<Instant>, // 闪红动画开始时间 (None = 无错误)
    pub start_time: Option<Instant>,  // 开始打字的时间
    pub last_correct_time: Option<Instant>, // 上一个正确击键的时间
}

impl TypingEngine {
    /// 创建新引擎实例
    pub fn new(target_str: &str) -> Self;

    /// 处理一次按键输入，返回是否正确
    pub fn input(&mut self, ch: char) -> InputResult;

    /// 是否已完成
    pub fn is_complete(&self) -> bool;

    /// 计算当前实时 WPM
    pub fn current_wpm(&self) -> f64;

    /// 计算当前实时准确率
    pub fn current_accuracy(&self) -> f64;

    /// 生成 SessionRecord
    pub fn finish(&self, command_id: &str, mode: Mode) -> SessionRecord;
}

pub enum InputResult {
    Correct,            // 打对了，光标前进
    Error(char),        // 打错了，返回期望的字符
    AlreadyComplete,    // 已经打完了
}
```

**WPM 计算公式**（与 keybr 一致）：
```
WPM = (正确字符数 / 5.0) / (已用时间秒数 / 60.0)
CPM = 正确字符数 / (已用时间秒数 / 60.0)
准确率 = 一次打对的次数 / 总字符数
```

### 3.3 默写匹配器 (`src/core/matcher.rs`)

```rust
// src/core/matcher.rs

pub struct Matcher;

impl Matcher {
    /// 标准化命令字符串（去多余空格、统一引号）
    pub fn normalize(input: &str) -> String;

    /// 匹配用户输入是否与任一答案等价
    pub fn check(input: &str, answers: &[String]) -> MatchResult;
}

pub enum MatchResult {
    Exact(usize),           // 完全匹配，返回匹配的答案索引
    Normalized(usize),      // 标准化后匹配
    NoMatch {
        closest: String,    // 最接近的正确答案
        diff: Vec<DiffSegment>, // 差异片段（用于高亮显示）
    },
}

pub struct DiffSegment {
    pub text: String,
    pub kind: DiffKind,     // Same / Added / Removed
}

pub enum DiffKind {
    Same,
    Added,
    Removed,
}
```

### 3.4 评分器 (`src/core/scorer.rs`)

```rust
// src/core/scorer.rs

pub struct Scorer;

impl Scorer {
    /// 从 SessionRecord 更新 UserStats
    pub fn update_stats(stats: &mut UserStats, record: &SessionRecord);

    /// 更新单个字符的统计
    fn update_char_stat(stat: &mut CharStat, keystrokes: &[Keystroke]);

    /// 更新命令进度
    fn update_command_progress(
        progress: &mut CommandProgress,
        record: &SessionRecord,
        difficulty: Difficulty,
    );

    /// 计算掌握度
    /// mastery = accuracy × min(1.0, times_practiced / target_attempts)
    fn compute_mastery(accuracy: f64, times: u32, target: u32) -> f64;

    /// 获取薄弱字符 Top N
    pub fn weak_chars(stats: &UserStats, n: usize) -> Vec<&CharStat>;

    /// 获取类别掌握度
    pub fn category_mastery(
        stats: &UserStats,
        commands: &[Command],
        category: Category,
    ) -> f64;

    /// 智能推荐：基于薄弱字符推荐命令
    pub fn recommend_commands(
        stats: &UserStats,
        commands: &[Command],
        n: usize,
    ) -> Vec<&Command>;
}
```

### 3.5 计时器 (`src/core/timer.rs`)

```rust
// src/core/timer.rs

pub struct Timer {
    start: Option<Instant>,
    elapsed: Duration,
    paused: bool,
}

impl Timer {
    pub fn new() -> Self;
    pub fn start(&mut self);
    pub fn pause(&mut self);
    pub fn resume(&mut self);
    pub fn elapsed(&self) -> Duration;
    pub fn elapsed_secs(&self) -> f64;
    pub fn format_mmss(&self) -> String;  // "01:34"
}
```

### 3.6 数据加载 (`src/data/loader.rs`)

```rust
// src/data/loader.rs

pub struct DataLoader {
    data_dir: PathBuf,  // 题库目录 (data/commands/)
}

impl DataLoader {
    pub fn new(data_dir: PathBuf) -> Self;

    /// 加载所有题库文件
    pub fn load_all(&self) -> anyhow::Result<Vec<Command>>;

    /// 按难度筛选
    pub fn load_by_difficulty(
        &self, difficulty: Difficulty
    ) -> anyhow::Result<Vec<Command>>;

    /// 按类别筛选
    pub fn load_by_category(
        &self, category: Category
    ) -> anyhow::Result<Vec<Command>>;
}
```

### 3.7 进度持久化 (`src/data/progress.rs`)

```rust
// src/data/progress.rs

pub struct ProgressStore {
    base_dir: PathBuf,  // ~/.local/share/cmdtyper/
}

impl ProgressStore {
    pub fn new() -> anyhow::Result<Self>;

    /// 加载用户统计
    pub fn load_stats(&self) -> anyhow::Result<UserStats>;

    /// 保存用户统计
    pub fn save_stats(&self, stats: &UserStats) -> anyhow::Result<()>;

    /// 追加练习记录
    pub fn append_record(&self, record: &SessionRecord) -> anyhow::Result<()>;

    /// 加载所有历史记录
    pub fn load_history(&self) -> anyhow::Result<Vec<SessionRecord>>;

    /// 加载用户配置
    pub fn load_config(&self) -> anyhow::Result<UserConfig>;

    /// 保存用户配置
    pub fn save_config(&self, config: &UserConfig) -> anyhow::Result<()>;
}
```

---

## 4. UI 模块设计

### 4.1 UI 渲染架构

每个页面实现统一的 trait：

```rust
// src/ui/mod.rs

pub trait Screen {
    /// 渲染到 Frame
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);

    /// 处理键盘事件，返回可能的状态转换
    fn handle_key(&mut self, key: KeyEvent, app: &mut App) -> Option<AppState>;
}
```

### 4.2 颜色方案

```rust
// src/ui/widgets.rs

pub mod colors {
    use ratatui::style::Color;

    pub const TYPED_CORRECT: Color = Color::White;      // 已正确输入
    pub const TYPED_CORRECT_BG: Color = Color::Reset;   // 黑底白字
    pub const PENDING: Color = Color::DarkGray;          // 待输入（灰色）
    pub const PENDING_BG: Color = Color::Rgb(40,40,40);  // 灰底
    pub const CURSOR: Color = Color::Black;              // 光标字符
    pub const CURSOR_BG: Color = Color::White;           // 光标背景（白底黑字）
    pub const ERROR_FLASH: Color = Color::White;         // 错误闪红
    pub const ERROR_FLASH_BG: Color = Color::Red;        // 红底
    pub const TOKEN_DESC: Color = Color::Cyan;           // token 注释颜色
    pub const TREE_LINE: Color = Color::DarkGray;        // token 树状线颜色
    pub const HEADER: Color = Color::Yellow;             // 标题颜色
    pub const ACCENT: Color = Color::Green;              // 强调色
    pub const WEAK_CHAR: Color = Color::Red;             // 薄弱字符
    pub const STRONG_CHAR: Color = Color::Green;         // 熟练字符
    pub const MEDIUM_CHAR: Color = Color::Yellow;        // 练习中字符
}
```

### 4.3 各页面布局规格

#### 主菜单 (Home)
```
布局: 居中 Block
- 上部: 标题 ASCII art + 版本号
- 中部: 4 个菜单项（上下键选择，高亮当前项）
- 下部: 难度选择器（左右键切换）+ 累计练习统计
- 底栏: 快捷键提示
```

#### 对着打 (Type)
```
布局: 垂直三段
┌─── 顶栏 (3行) ──────────────────────┐
│ 模式标签 | 类别 | 难度 | 进度 n/N   │
├─── 主区 (动态高度) ─────────────────┤
│ 命令文本 + token 注释树             │
│ 输入行（逐字符着色）               │
├─── 底栏 (3行) ──────────────────────┤
│ WPM | 准确率 | 耗时 | 快捷键       │
└─────────────────────────────────────┘
```

#### 统计面板 (Stats)
```
布局: Tab 页 (4个)
[速度总览] [字符分析] [类别掌握] [练习日历]
Tab 键切换子页面
每个子页面有独立的渲染函数
```

---

## 5. 事件循环架构

```rust
// src/event.rs

pub enum AppEvent {
    Key(KeyEvent),
    Tick,           // 每 50ms 触发一次（用于动画、计时器更新）
    Resize(u16, u16),
}

// main loop 伪代码:
loop {
    // 1. 渲染
    terminal.draw(|frame| ui::render(frame, &app))?;

    // 2. 等待事件 (50ms timeout for tick)
    match event::poll(Duration::from_millis(50)) {
        Key(key) => {
            match app.state {
                Home => home.handle_key(key, &mut app),
                Typing(ref mut s) => typing.handle_key(key, &mut app),
                // ...
            }
        }
        Tick => {
            // 更新闪红动画状态
            // 更新实时 WPM 显示
        }
        Resize(w, h) => { /* 重绘 */ }
    }

    if app.state == Quitting { break; }
}
```

---

## 6. 开发阶段规划

### Phase 0: 项目骨架 (P0)
- [x] 项目目录结构
- [x] Cargo.toml 依赖配置
- [x] Git 仓库初始化
- [x] Dockerfile + docker-compose.yml
- [ ] `src/main.rs`: 终端初始化 + 事件循环骨架
- [ ] `src/app.rs`: App 状态机 + 状态枚举
- [ ] `src/event.rs`: 事件处理
- [ ] `src/ui/home.rs`: 主菜单渲染 + 导航
- [ ] 编译通过，显示主菜单

### Phase 1: 打字引擎 (P1)
- [ ] `src/core/engine.rs`: TypingEngine 实现
  - [ ] 逐字符匹配逻辑
  - [ ] 正确击键记录 + 光标前进
  - [ ] 错误击键记录 + 闪红触发 + 不前进
  - [ ] 完成检测
- [ ] `src/core/timer.rs`: Timer 实现
- [ ] `src/ui/typing.rs`: 对着打 UI
  - [ ] 灰底/黑色/闪红三态字符渲染
  - [ ] 实时 WPM + 准确率底栏
  - [ ] 光标动画（闪烁）
- [ ] 单元测试: engine 正确性

### Phase 2: 题库系统 (P2)
- [ ] `src/data/models.rs`: 所有数据结构
- [ ] `src/data/loader.rs`: TOML 解析 + 加载
- [ ] `data/commands/01_beginner.toml`: 入门题库 (~15 条)
- [ ] 按难度/类别筛选
- [ ] 单元测试: loader + 题库格式验证

### Phase 3: 学习模式 (P3)
- [ ] `src/ui/learn.rs`: 学习模式 UI
  - [ ] token 树状注释渲染
  - [ ] 概述文本显示
  - [ ] 跟打输入（复用 TypingEngine）
- [ ] Tab 跳过 + Ctrl+R 重练

### Phase 4: 默写模式 (P4)
- [ ] `src/core/matcher.rs`: 多答案匹配器
  - [ ] 字符串标准化
  - [ ] 精确匹配 + 标准化匹配
  - [ ] 差异计算（简单 diff）
- [ ] `src/ui/dictation.rs`: 默写 UI
  - [ ] 中文提示展示
  - [ ] 自由输入框
  - [ ] 结果判定 + 正确答案展示
- [ ] 单元测试: matcher

### Phase 5: 统计系统 (P5)
- [ ] `src/core/scorer.rs`: 评分器实现
  - [ ] 字符级统计聚合
  - [ ] 掌握度计算
  - [ ] 薄弱字符分析
  - [ ] 类别掌握度
  - [ ] 智能推荐
- [ ] `src/data/progress.rs`: JSON 持久化
- [ ] `src/ui/stats.rs`: 统计面板
  - [ ] Tab 1: 速度总览（WPM 趋势 sparkline + 分布直方图）
  - [ ] Tab 2: 字符分析（表格 + 单字符速度曲线）
  - [ ] Tab 3: 类别掌握度（进度条）
  - [ ] Tab 4: 练习日历（热力图）

### Phase 6: 本轮结果 (P6)
- [ ] `RoundResult` 页面
  - [ ] 本轮 WPM / CPM / 准确率 / 耗时
  - [ ] 错误字符 Top 5
  - [ ] 与历史平均对比（箭头趋势）
  - [ ] [Enter] 下一题 / [R] 重练 / [Esc] 返回

### Phase 7: 题库扩充 + 打磨 (P7)
- [ ] 扩充题库到 ~50 条命令 (~120 条变体)
  - [ ] `02_basic.toml`
  - [ ] `03_advanced.toml`
  - [ ] `04_practical.toml`
- [ ] 语法体系讲解数据 (`data/syntax/`)
- [ ] UI 细节打磨
  - [ ] ASCII art 标题
  - [ ] 过渡动画
  - [ ] 颜色主题

### Phase 8: 集成调试 (P8)
- [ ] 全模式流程走通测试
- [ ] 极端输入测试（空命令、超长命令、Unicode）
- [ ] 终端兼容性（xterm-256color, tmux, 不同尺寸）
- [ ] 中文宽字符对齐验证
- [ ] 持久化读写可靠性（文件损坏恢复）
- [ ] Docker 构建 + 运行测试
- [ ] 性能测试（大题库加载、长历史记录）
- [ ] README.md 编写
- [ ] 发布 v0.1.0 tag

---

## 7. 测试策略

### 7.1 单元测试

| 模块 | 测试内容 |
|------|----------|
| `engine` | 正确输入前进、错误不前进、完成检测、WPM 计算 |
| `matcher` | 精确匹配、标准化匹配、无匹配+diff、边界情况 |
| `scorer` | 字符统计更新、掌握度计算、推荐算法 |
| `loader` | TOML 解析、格式错误处理、筛选逻辑 |
| `progress` | 文件读写、损坏恢复、并发安全 |

### 7.2 集成测试

```rust
// tests/engine_test.rs
#[test]
fn test_typing_full_command() {
    let mut engine = TypingEngine::new("ls -la");
    assert_eq!(engine.input('l'), InputResult::Correct);
    assert_eq!(engine.input('s'), InputResult::Correct);
    assert_eq!(engine.input(' '), InputResult::Correct);
    assert_eq!(engine.input('x'), InputResult::Error('-')); // 打错
    assert_eq!(engine.input('-'), InputResult::Correct);
    // ...
}

#[test]
fn test_matcher_multiple_answers() {
    let answers = vec![
        "ls -la".to_string(),
        "ls -al".to_string(),
    ];
    assert!(matches!(
        Matcher::check("ls -la", &answers),
        MatchResult::Exact(0)
    ));
    assert!(matches!(
        Matcher::check("ls  -al", &answers),  // 多空格
        MatchResult::Normalized(1)
    ));
}
```

---

## 8. 已知技术风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 中文字符宽度 | 终端对齐错误 | 使用 `unicode-width` crate 计算实际显示宽度 |
| ratatui 中文渲染 | 部分终端中文显示异常 | 测试 xterm / gnome-terminal / tmux / kitty |
| 闪红动画帧率 | 闪红太快看不到 | 50ms tick + 150ms flash 持续时间 |
| TOML 大文件解析 | 启动慢 | 题库分文件，按需加载 |
| JSON 持久化损坏 | 用户数据丢失 | 写入时先写 .tmp 再 rename（原子写入） |
