# PLAN.md — cmdtyper v0.2 完整技术规划

> 从"打字练习工具"进化为"Linux 命令行交互式教学系统"
> 本文档是唯一权威技术规范。所有模块开发必须严格遵守此处定义的接口、类型和行为。

---

## 目录

1. [项目概述](#1-项目概述)
2. [模块总览与状态机](#2-模块总览与状态机)
3. [数据类型定义（完整）](#3-数据类型定义完整)
4. [数据文件格式规范（TOML）](#4-数据文件格式规范toml)
5. [模块详细设计](#5-模块详细设计)
6. [事件循环与渲染架构](#6-事件循环与渲染架构)
7. [持久化与文件布局](#7-持久化与文件布局)
8. [源码目录结构](#8-源码目录结构)
9. [调试与测试策略](#9-调试与测试策略)
10. [开发分期与依赖关系](#10-开发分期与依赖关系)
11. [已知风险与缓解](#11-已知风险与缓解)

---

## 1. 项目概述

### 1.1 定位

cmdtyper v0.2 是一个终端 TUI 应用，面向 Linux 初学者到中级用户，提供：

- **对着打模式**：模拟真实终端环境的命令打字练习
- **学习中心**：体系化的命令、符号、系统架构教学，含模拟输出
- **默写模式**：看中文描述写命令
- **统计面板**：练习数据分析

### 1.2 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | edition 2024 (rustc 1.94+) |
| TUI | ratatui + crossterm | 0.29 / 0.28 |
| 数据格式 | TOML (题库/课程) + JSON (持久化) | toml 0.8 / serde_json 1 |
| 序列化 | serde | 1 |
| 随机 | rand | 0.8 |
| Unicode | unicode-width | 0.2 |
| 错误处理 | anyhow | 1 |
| 时间 | chrono | 0.4 |
| 路径 | dirs | 6 |
| 容器化 | Docker (multi-stage) | — |

### 1.3 安全原则

**cmdtyper 永远不执行任何真实命令。** 所有命令输出均为 TOML 预置模拟数据。无 `std::process::Command`、无 shell 调用、无 `system()`。

---

## 2. 模块总览与状态机

### 2.1 功能模块

```
主菜单 (Home)
├── ⌨️  对着打 (Typing)
├── 📖 学习中心 (LearnHub)
│   ├── 命令专题 (CommandTopics)
│   │   └── 单命令学习 (CommandLesson) → 概览/演示/跟打 三阶段
│   ├── 符号专题 (SymbolTopics)
│   │   └── 单符号学习 (SymbolLesson)
│   ├── 系统架构 (SystemTopics)
│   │   └── 单主题学习 (SystemLesson)
│   └── 专题复习 (Review)
├── 📝 默写模式 (Dictation)
├── 📊 统计面板 (Stats)
└── ⚙️  设置 (Settings)
```

### 2.2 AppState 枚举（完整定义）

```rust
/// 应用全局状态
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Home,

    // ── 对着打 ──
    Typing,

    // ── 学习中心 ──
    LearnHub,                               // 二级菜单
    CommandTopics,                          // 命令专题 → 类别列表
    CommandLessonOverview {                  // 阶段1: 命令概览
        category_index: usize,
        command_index: usize,
    },
    CommandLessonExample {                  // 阶段2: 实例演示
        category_index: usize,
        command_index: usize,
        example_index: usize,
    },
    CommandLessonPractice {                 // 阶段3: 跟打练习
        category_index: usize,
        command_index: usize,
        example_index: usize,
    },
    SymbolTopics,                           // 符号专题 → 主题列表
    SymbolLesson {
        topic_index: usize,
        symbol_index: usize,
        phase: SymbolPhase,
    },
    SystemTopics,                           // 系统架构 → 主题列表
    SystemLesson {
        topic_index: usize,
        section_index: usize,
        phase: SystemPhase,
    },
    Review {                                // 专题复习
        source: ReviewSource,
        phase: ReviewPhase,
    },

    // ── 其他 ──
    Dictation,
    Stats,
    Settings,
    RoundResult,
    Quitting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolPhase {
    Explain,                // 符号讲解
    Example(usize),         // 示例演示
    Practice,               // 练习题
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemPhase {
    Overview,               // 总览（ASCII art 等）
    Detail,                 // 详细讲解
    Commands(usize),        // 常用命令 + 模拟输出
    ConfigFile(usize),      // 配置文件讲解
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewSource {
    CommandCategory(Category),
    SymbolTopic(String),
    SystemTopic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewPhase {
    Summary,                // 知识梳理
    Practice(usize),        // 集中练习
}
```

### 2.3 状态转换图

```
Home ─┬─→ Typing ──→ (Esc) ──→ Home
      ├─→ LearnHub ─┬─→ CommandTopics ──→ CommandLessonOverview
      │              │     ──→ CommandLessonExample ──→ CommandLessonPractice
      │              │     ──→ Review (或下一命令)
      │              ├─→ SymbolTopics ──→ SymbolLesson ──→ Review
      │              ├─→ SystemTopics ──→ SystemLesson ──→ Review
      │              └─→ Review（直接进入）
      ├─→ Dictation ──→ RoundResult ──→ Home
      ├─→ Stats ──→ Home
      └─→ Settings ──→ Home

任何状态 + Ctrl+C → Quitting
任何子状态 + Esc → 父状态
```

---

## 3. 数据类型定义（完整）

> 以下所有类型均在 `src/data/models.rs` 中定义。
> `#[serde(skip)]` = 不从 TOML 反序列化，由 loader 运行时填充。
> `#[serde(default)]` = TOML 中可省略，使用 Default 值。

### 3.1 基础枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Beginner,   // 入门 ★☆☆☆
    Basic,      // 基础 ★★☆☆
    Advanced,   // 进阶 ★★★☆
    Practical,  // 实战 ★★★★
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Beginner, Self::Basic, Self::Advanced, Self::Practical];
    pub fn label(&self) -> &str;        // "入门"/"基础"/"进阶"/"实战"
    pub fn stars(&self) -> &str;        // "★☆☆☆" ...
    pub fn target_attempts(&self) -> u32; // 3/5/8/10
    pub fn next(&self) -> Self;
    pub fn prev(&self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    #[default]
    FileOps,        // 文件操作  📁
    Permission,     // 权限管理  🔒
    TextProcess,    // 文本处理  📋
    Search,         // 搜索查找  🔍
    Process,        // 进程管理  ⚙️
    Network,        // 网络      🌐
    Archive,        // 压缩归档  📦
    System,         // 系统信息  💻
    Pipeline,       // 管道重定向 🔀
    Scripting,      // 脚本片段  📜
}

impl Category {
    pub const ALL: [Self; 10] = [...];
    pub fn label(&self) -> &str;
    pub fn icon(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode { Learn, #[default] Type, Dictation }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Importance { #[default] Core, Common, Advanced, Niche }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PromptStyle { #[default] Full, Simple, Minimal }
```

### 3.2 命令题库（对着打 + 默写 共用）

```rust
/// 对应 data/commands/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFile {
    pub meta: CommandFileMeta,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFileMeta {
    pub category: Category,
    pub difficulty: Difficulty,
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub command: String,
    pub summary: String,
    pub tokens: Vec<Token>,
    pub dictation: DictationData,
    #[serde(default)] pub display: Option<String>,
    #[serde(default)] pub summary_short: Option<String>,
    #[serde(default)] pub simulated_output: Option<String>,
    #[serde(default)] pub output_annotations: Vec<OutputAnnotation>,
    #[serde(skip)] pub category: Category,
    #[serde(skip)] pub difficulty: Difficulty,
}

impl Command {
    /// 返回终端显示文本（display 优先，否则 command）
    pub fn display_text(&self) -> &str;
    /// 返回底栏短提示（summary_short 优先，否则 summary）
    pub fn short_summary(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token { pub text: String, pub desc: String }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DictationData { pub prompt: String, pub answers: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAnnotation { pub pattern: String, pub note: String }
```

### 3.3 命令讲解（学习中心 · 命令专题）

```rust
/// 对应 data/lessons/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLesson {
    pub meta: LessonMeta,
    pub overview: LessonOverview,
    pub syntax: SyntaxInfo,
    #[serde(default)] pub options: Vec<OptionInfo>,
    pub examples: Vec<LessonExample>,
    #[serde(default)] pub gotchas: Vec<Gotcha>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonMeta {
    pub command: String,
    #[serde(default)] pub full_name: Option<String>,
    pub category: Category,
    pub difficulty: Difficulty,
    #[serde(default)] pub importance: Importance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonOverview { pub summary: String, pub explanation: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxInfo {
    pub basic: String,
    #[serde(default)] pub parts: Vec<SyntaxPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxPart { pub name: String, pub desc: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionInfo {
    pub flag: String,
    pub name: String,
    #[serde(default)] pub example: Option<String>,
    #[serde(default)] pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonExample {
    pub level: u8,
    pub command: String,
    pub summary: String,
    #[serde(default)] pub display: Option<String>,
    #[serde(default)] pub simulated_output: Option<String>,
    #[serde(default)] pub output_annotations: Vec<OutputAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gotcha { pub title: String, pub content: String }
```

### 3.4 符号专题

```rust
/// 对应 data/symbols/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTopic {
    pub meta: SymbolTopicMeta,
    pub symbols: Vec<SymbolEntry>,
    #[serde(default)] pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTopicMeta {
    pub id: String, pub topic: String, pub description: String,
    pub difficulty: Difficulty,
    #[serde(default)] pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub id: String, pub char_repr: String, pub name: String,
    pub summary: String, pub explanation: String,
    pub examples: Vec<SymbolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolExample {
    pub command: String, pub explanation: String,
    #[serde(default)] pub display: Option<String>,
    #[serde(default)] pub simulated_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise { pub prompt: String, pub answers: Vec<String> }
```

### 3.5 系统架构专题

```rust
/// 对应 data/system/*.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTopic {
    pub meta: SystemTopicMeta,
    #[serde(default)] pub overview: Option<String>,
    pub sections: Vec<SystemSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTopicMeta {
    pub id: String, pub topic: String, pub description: String,
    pub difficulty: Difficulty,
    #[serde(default)] pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSection {
    pub id: String, pub title: String, pub description: String,
    #[serde(default)] pub commands: Vec<SystemCommand>,
    #[serde(default)] pub config_files: Vec<ConfigFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCommand {
    pub command: String, pub summary: String,
    #[serde(default)] pub simulated_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub id: String, pub path: String, pub name: String,
    pub description: String, pub sample_content: String,
    #[serde(default)] pub lessons: Vec<ConfigLesson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLesson {
    pub title: String, pub before: String, pub after: String,
    pub explanation: String,
    #[serde(default)] pub practice_command: Option<String>,
}
```

### 3.6 复习模块

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub topic_id: String, pub topic_name: String,
    pub summary_groups: Vec<ReviewGroup>,
    pub practice_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewGroup { pub name: String, pub items: Vec<ReviewItem> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem { pub command: String, pub brief: String }
```

### 3.7 打字引擎

```rust
// src/core/engine.rs
pub struct TypingEngine {
    pub target: Vec<char>,
    pub cursor: usize,
    pub keystrokes: Vec<Keystroke>,
    pub current_attempts: u8,
    pub error_flash: Option<Instant>,
    pub start_time: Option<Instant>,
    pub last_correct_time: Option<Instant>,
    error_flash_duration: Duration,     // 默认 150ms
}

pub enum InputResult {
    Correct,
    Error { expected: char },
    AlreadyComplete,
}

impl TypingEngine {
    pub fn new(target_str: &str) -> Self;
    pub fn input(&mut self, ch: char) -> InputResult;
    pub fn is_complete(&self) -> bool;
    pub fn is_error_flashing(&self) -> bool;
    pub fn current_wpm(&self) -> f64;       // (正确字符/5) / (秒/60)
    pub fn current_cpm(&self) -> f64;       // 正确字符 / (秒/60)
    pub fn current_accuracy(&self) -> f64;  // 一次正确数 / 总字符数
    pub fn elapsed_secs(&self) -> f64;
    pub fn finish(&self, command_id: &str, mode: Mode) -> SessionRecord;
    pub fn reset(&mut self, target_str: &str);
}
```

### 3.8 终端历史（对着打专用）

```rust
// src/core/terminal_history.rs
pub struct TerminalLine {
    pub prompt: String,
    pub command_display: String,
    pub status: LineStatus,
}

pub enum LineStatus { Completed, Current, Pending }

pub struct TerminalHistory {
    lines: Vec<TerminalLine>,
    max_visible: usize,
}

impl TerminalHistory {
    pub fn new() -> Self;
    pub fn push_completed(&mut self, prompt: &str, display: &str);
    pub fn visible_lines(&self, height: u16) -> &[TerminalLine];
    pub fn clear(&mut self);
}
```

### 3.9 其他核心模块接口

```rust
// Matcher (src/core/matcher.rs) — 与 v0.1 相同
// Scorer (src/core/scorer.rs) — 与 v0.1 相同
// Timer (src/core/timer.rs) — 与 v0.1 相同
// ProgressStore (src/data/progress.rs) — 与 v0.1 相同，UserConfig 扩展新字段
// Keystroke, SessionRecord, CharStat, UserStats 等 — 与 v0.1 相同
```

（完整接口定义见 v0.1 PLAN.md §2-3，此处不重复。仅 UserConfig 新增 prompt_* 和 PromptStyle 字段。）

---

## 4. 数据文件格式规范（TOML）

### 4.1 命令题库 `data/commands/*.toml`

```toml
[meta]
category = "file_ops"       # Category snake_case
difficulty = "beginner"     # Difficulty lowercase
description = "描述"

[[commands]]
id = "ls-la-varlog"         # 必填，全局唯一
command = "ls -la /var/log"  # 必填
summary = "一句话简介"       # 必填
display = "多行显示"         # 可选（默认=command）
summary_short = "短提示"     # 可选（默认=summary）
simulated_output = """...""" # 可选

[[commands.tokens]]          # 至少1个; text拼接==command
text = "ls"
desc = "列出目录内容"

[[commands.output_annotations]]  # 可选
pattern = "drwx"
note = "目录权限标记"

[commands.dictation]         # 必填
prompt = "中文题目"
answers = ["ls -la /var/log"]  # 至少1个
```

### 4.2 命令讲解 `data/lessons/*.toml`

```toml
[meta]
command = "grep"            # 必填
full_name = "..."           # 可选
category = "text_process"   # 必填
difficulty = "basic"        # 必填
importance = "core"         # 可选（默认core）

[overview]
summary = "一句话"           # 必填
explanation = """多行"""     # 必填

[syntax]
basic = "grep [选项] 模式 [文件...]"  # 必填
[[syntax.parts]]             # 可选
name = "模式"
desc = "搜索的字符串"

[[options]]                  # 可选
flag = "-i"
name = "忽略大小写"
example = "grep -i 'err' log"  # 可选
note = "..."                   # 可选

[[examples]]                 # 至少1个
level = 1                    # 必填 1-4
command = "grep 'error' syslog"
summary = "搜索 error"
display = "..."              # 可选
simulated_output = """...""" # 可选

[[gotchas]]                  # 可选
title = "标题"
content = """..."""
```

### 4.3 符号专题 `data/symbols/*.toml`

```toml
[meta]
id = "pipe_redirect"        # 必填
topic = "管道与重定向"
description = "..."
difficulty = "basic"
icon = "🔀"                 # 可选

[[symbols]]
id = "pipe"                  # 必填
char_repr = "|"              # 必填
name = "管道"
summary = "一句话"
explanation = """多行"""

[[symbols.examples]]
command = "cat file | grep x"
explanation = "..."
display = "..."              # 可选
simulated_output = "..."     # 可选

[[exercises]]                # 可选
prompt = "题目"
answers = ["答案1", "答案2"]
```

### 4.4 系统架构 `data/system/*.toml`

```toml
[meta]
id = "directory_structure"
topic = "Linux 目录结构"
description = "..."
difficulty = "beginner"
icon = "🏗️"

overview = """ASCII art..."""  # 可选

[[sections]]
id = "etc"
title = "/etc — 系统配置目录"
description = """多行..."""

[[sections.commands]]
command = "ls /etc/"
summary = "查看配置目录"
simulated_output = """..."""

[[sections.config_files]]    # 可选
id = "sshd-config"
path = "/etc/ssh/sshd_config"
name = "SSH 配置"
description = "..."
sample_content = """..."""

[[sections.config_files.lessons]]
title = "禁用密码登录"
before = "PasswordAuthentication yes"
after = "PasswordAuthentication no"
explanation = """..."""
practice_command = "sudo sed -i '...'"  # 可选
```

### 4.5 跨文件约束

| 约束 | 校验方式 |
|------|----------|
| `command.id` 全局唯一 | 集成测试 `id_uniqueness.rs` |
| `lesson.meta.command` 唯一 | loader 启动时检查 |
| `topic.meta.id` 唯一 | loader 启动时检查 |
| tokens 拼接 == command | 集成测试 `tokens_consistency.rs` |
| TOML 字符串转义正确 | 集成测试 `parse_all.rs` |

---

## 5. 模块详细设计

### 5.1 对着打模式

**UI 布局：**

```
┌─────────────────────────────────────────────────────┐
│ user@cmdtyper:~$ ls -la /var/log                   │ 已完成（绿）
│ user@cmdtyper:~$ grep -r "err█r" /var/log          │ 当前（三态着色）
│                                                     │
│                     （空白区域）                      │
├─────────────────────────────────────────────────────┤
│ 搜索日志中的 error        [H]  WPM: 42  准确: 96%  │ 底栏
└─────────────────────────────────────────────────────┘
```

**交互规则：**

| 输入 | 行为 |
|------|------|
| 正确字符 | 变白，光标右移 |
| 错误字符 | 闪红 150ms，不动 |
| 最后字符正确 | 自动完成当前行 |
| `H` | 切换含义提示 |
| `Tab` | 跳过 |
| `Ctrl+R` | 重练 |
| `Esc` | 返回主菜单 |

**Prompt 生成：**

```rust
fn format_prompt(config: &UserConfig, path: &str) -> String {
    match config.prompt_style {
        Full    => format!("{}@{}:{}$ ", config.prompt_username, config.prompt_hostname, path),
        Simple  => "$ ".to_string(),
        Minimal => "> ".to_string(),
    }
}
// path 默认 "~"
```

**多行命令**：`display` 含 `\` 时渲染续行 `> `。

### 5.2 学习中心 — 命令专题

**三阶段学习流程：**

1. **Overview（概览）** — 显示 `overview.explanation` + `syntax` + `options`
2. **Example（演示）** — 显示 `examples[i]` 的命令 + token 注释 + 模拟输出框
3. **Practice（跟打）** — 用 `TypingEngine` 逐字符打，完成后显示 `simulated_output`

**模拟输出框渲染：**

```
┌──────────────────────────────────────────┐
│ $ grep -rn 'TODO' src/                   │  ← 绿色 prompt + 白色命令
│ src/main.rs:42:    // TODO: add error    │  ← 白色输出文本
│ src/app.rs:156:    // TODO: implement    │
└──────────────────────────────────────────┘
```

边框用 `SIMULATED_BORDER` 颜色，内部 prompt 用 `SIMULATED_PROMPT`。

### 5.3 学习中心 — 符号专题

流程：讲解 → 多个示例（←→翻页）→ 练习题（用 Matcher）

### 5.4 学习中心 — 系统架构专题

流程：总览 → 选章节 → 详细讲解 → 常用命令+模拟输出 → 配置文件（如有）

### 5.5 专题复习

两阶段：**知识梳理**（表格展示）→ **集中练习**（随机抽题，打字+默写混合）

### 5.6 设置页面

可配置项：PromptStyle、username、hostname、show_path、target_wpm、error_flash_ms、show_token_hints、adaptive_recommend。自动保存。

---

## 6. 事件循环与渲染架构

```rust
// src/event.rs
pub enum AppEvent { Key(KeyEvent), Tick, Resize(u16, u16) }

// 主循环: 50ms tick → 渲染 → 处理事件 → 状态转换
// src/ui/mod.rs: render() 根据 app.state 分发到对应 UI 模块
```

**颜色常量**（`src/ui/widgets.rs`）：

| 常量 | 颜色 | 用途 |
|------|------|------|
| TYPED_CORRECT | White | 已正确输入 |
| PENDING / PENDING_BG | DarkGray / Rgb(40,40,40) | 待输入 |
| CURSOR / CURSOR_BG | Black / White | 当前光标 |
| ERROR_FLASH / ERROR_FLASH_BG | White / Red | 错误闪红 |
| COMPLETED | Green | 已完成行 |
| PROMPT | Cyan | prompt 文本 |
| SIMULATED_BORDER | DarkGray | 模拟输出框 |
| SIMULATED_PROMPT | Green | 模拟输出内的 $ |
| HEADER | Yellow | 标题 |
| TOKEN_DESC | Cyan | token 注释 |

---

## 7. 持久化与文件布局

```
~/.local/share/cmdtyper/
├── history.json    # Vec<SessionRecord>
├── stats.json      # UserStats
└── config.json     # UserConfig
```

- 原子写入：写 .tmp → fsync → rename
- 损坏恢复：解析失败返回 Default，不 panic

---

## 8. 源码目录结构

```
src/
├── main.rs
├── app.rs
├── event.rs
├── core/
│   ├── mod.rs
│   ├── engine.rs
│   ├── matcher.rs
│   ├── scorer.rs
│   ├── timer.rs
│   └── terminal_history.rs
├── data/
│   ├── mod.rs
│   ├── models.rs
│   ├── command_loader.rs
│   ├── lesson_loader.rs
│   ├── symbol_loader.rs
│   ├── system_loader.rs
│   └── progress.rs
└── ui/
    ├── mod.rs
    ├── widgets.rs
    ├── home.rs
    ├── typing.rs
    ├── learn_hub.rs
    ├── command_topics.rs
    ├── command_lesson.rs
    ├── symbol_topics.rs
    ├── symbol_lesson.rs
    ├── system_topics.rs
    ├── system_lesson.rs
    ├── review.rs
    ├── dictation.rs
    ├── round_result.rs
    ├── stats.rs
    └── settings.rs

data/
├── commands/       # 命令题库 (现有 19 个 + 增强)
├── lessons/        # 命令讲解 (~15 个)
├── symbols/        # 符号专题 (~6 个)
├── system/         # 系统架构 (~6 个)
└── reviews/        # 复习数据 (可选，可自动生成)

tests/
├── parse_all.rs
├── tokens_consistency.rs
├── id_uniqueness.rs
├── engine_test.rs
├── matcher_test.rs
└── scorer_test.rs
```

---

## 9. 调试与测试策略

### 9.1 单元测试矩阵

| 模块 | 测试内容 |
|------|----------|
| TypingEngine | 正确前进、错误不动、完成检测、WPM/CPM、reset |
| TerminalHistory | push/visible/滚动/clear |
| Matcher | 精确/标准化/NoMatch+diff/空输入 |
| Scorer | 字符统计、掌握度、弱字符、推荐 |
| Timer | start/pause/resume/format |
| CommandLoader | TOML 解析、metadata 传播、缺失目录、v0.2 新增字段 |
| LessonLoader | TOML 解析、可选字段默认值 |
| SymbolLoader | TOML 解析、exercises |
| SystemLoader | TOML 解析、config_files 嵌套 |
| ProgressStore | 读写、原子写入、损坏恢复 |

### 9.2 集成测试

| 文件 | 内容 |
|------|------|
| `parse_all.rs` | 所有 TOML 零错误反序列化 |
| `tokens_consistency.rs` | tokens 拼接 == command |
| `id_uniqueness.rs` | 全局 ID 唯一 |

### 9.3 每阶段调试检查清单

见 TODO.md 各 Phase 末尾的「调试检查」节。

---

## 10. 开发分期与依赖关系

```
Phase 0: 项目骨架
  ↓
Phase A: 对着打改造（终端模拟）  ←── 依赖 Phase 0
  ↓
Phase B: 学习中心骨架 + 命令专题  ←── 依赖 Phase 0
  ↓
Phase C: 符号专题  ←── 依赖 Phase B（复用 UI 组件）
  ↓
Phase D: 系统架构专题  ←── 依赖 Phase B
  ↓
Phase E: 专题复习  ←── 依赖 Phase B/C/D
  ↓
Phase F: 设置 + 打磨 + 最终集成  ←── 依赖全部
```

**Phase A 和 Phase B 可以并行开发**（无代码依赖）。

### 工作量预估

| Phase | 新增/改动行数 | 内容文件 | 复杂度 |
|-------|-------------|----------|--------|
| 0: 骨架 | ~500 | 0 | ★☆☆ |
| A: 对着打 | ~400 | 少量字段 | ★★☆ |
| B: 命令专题 | ~1800 | ~15 lessons | ★★★ |
| C: 符号专题 | ~800 | ~6 symbols | ★★☆ |
| D: 系统架构 | ~1200 | ~6 system | ★★★ |
| E: 复习 | ~600 | 自动生成 | ★★☆ |
| F: 设置+打磨 | ~500 | 0 | ★★☆ |
| **总计** | **~5800** | **~27** | |

---

## 11. 已知风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 中文宽字符对齐 | UI 错位 | unicode-width + 测试 CJK 渲染 |
| TOML 字符串转义 | 解析失败 | 集成测试 parse_all + 文档规范 |
| 模拟输出过长 | 超出终端 | 限制显示行数 + 滚动 |
| 状态机复杂度 | 导航 bug | 严格测试每个 Esc 返回路径 |
| 内容编写量大 | 进度慢 | Codex subagent 并行 + 模板化 |
| JSON 持久化损坏 | 数据丢失 | 原子写入 + 损坏降级 |
