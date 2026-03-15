# cmdtyper

> 🐧 Linux 命令行交互式教学系统 — 终端模拟打字 · 体系化学习 · 分级训练 · 深度解析

**cmdtyper** 是一个基于 Rust 的终端 TUI 应用，面向 Linux 初学者到中级用户，提供沉浸式命令行学习体验。它在终端中模拟真实输入流程，支持多种练习模式、分级筛选、深度讲解与统计追踪，且**不执行任何真实命令**。

## ✨ v0.3 特性总览

- **🖥️ 三档打字模式**：Terminal / Standard / Detailed，训练时按 `M` 一键切换
- **🎯 分级训练**：支持按难度（Beginner/Basic/Advanced/Practical）与类别（Category）过滤命令
- **📺 模拟输出反馈**：输入完成后展示预置终端输出，建立“输入→结果”映射
- **🧠 深度解析框架**：命令示例支持滚动阅读的深度解释，按 `D` 打开/关闭
- **🔣 符号打字训练**：符号专题支持 **Typing + Dictation** 双模式练习
- **💻 系统命令打字**：系统架构专题包含命令跟打与配置理解
- **📊 统计面板**：WPM、准确率、字符分析、类别掌握度、连续练习天数
- **🔒 绝对安全**：纯数据驱动，所有输出均来自 TOML 预置内容

## 📦 内容库（v0.3）

| 类别 | 数量 | 内容 |
|------|------|------|
| 命令题库 | **273 条** | 分布于 `data/commands/*.toml`，覆盖入门到实战 |
| 命令讲解 | **31 个专题** | 含语法、选项、示例、易错点 |
| 符号专题 | **6 个专题** | 管道重定向、通配符、引号转义、变量、特殊字符、正则 |
| 符号练习 | **90 题** | Typing + Dictation 双轨训练 |
| 系统架构专题 | **6 个专题** | 目录结构、权限模型、进程/systemd、网络、包管理、配置文件 |
| 深度解析样例 | **5 个旗舰案例** | 支持滚动查看的详解内容 |

## 🎮 训练与学习模式

### ⌨️ 三档打字模式

在对着打/练习界面可用 `M` 轮换：

1. **Terminal**：更贴近真实终端，节奏更快
2. **Standard**：默认模式，平衡可读性与速度
3. **Detailed**：展示更多解释信息，便于理解命令结构

模式循环顺序：`Standard → Detailed → Terminal → Standard`。

### 🎯 分级训练

支持两个维度筛选：

- **Difficulty**：Beginner / Basic / Advanced / Practical
- **Category**：FileOps / Permission / TextProcess / Search / Process / Network / Archive / System / Pipeline / Scripting

可单独使用任一筛选，也可组合筛选（Difficulty + Category）快速进入精准练习集。

**推荐起步路径**：

1. Beginner + FileOps（先熟悉目录与文件操作）
2. Basic + TextProcess / Search（建立文本与检索能力）
3. Advanced + Pipeline（组合命令）
4. Practical（实战场景巩固）

### 🧠 深度解析

在支持深度讲解的示例中，可按 `D` 打开详细解释面板：

- 展示分步推导与上下文说明
- 支持滚动阅读，适合学习“为什么这样写”
- 适用于命令专题、符号专题与系统命令部分内容

### 🔣 符号打字训练

符号专题新增双模式训练：

- **Typing**：给定命令文本，逐字符跟打
- **Dictation**：给定语义提示，独立回忆并输入命令

训练重点覆盖：`|`, `>`, `>>`, `*`, `?`, `$`, `'"`, `&&`, `||`, 正则等关键符号用法。

### 💻 系统命令打字

系统架构专题不再只讲概念，还可直接进行命令练习：

- 常见运维命令的跟打与理解
- 模拟输出辅助理解命令效果
- 与配置文件讲解联动，形成“命令 + 配置 + 场景”闭环

### 📝 默写模式

只给中文描述，你凭记忆写命令：

- 多答案匹配（如 `ls -la` / `ls -al`）
- 智能标准化（忽略多余空格）
- 错误时展示最接近答案 + diff

### 📖 学习中心

```
📖 学习中心
├── 命令专题
├── 符号专题
├── 系统架构
└── 专题复习
```

每个命令学习通常包含：
1. **Overview**：语法、选项、例子、注意点
2. **Practice**：逐字符跟打并即时反馈

## 🚀 快速开始

### 从源码构建

```bash
git clone https://github.com/ZhaoLingSun/cmdtyper.git
cd cmdtyper
cargo build --release
./target/release/cmdtyper
```

### Docker 运行

```bash
docker compose build
docker compose run --rm cmdtyper
```

### 依赖

- Rust 1.82+（edition 2024）
- 支持 256 色的终端

## ⌨️ 快捷键（按界面）

| 按键 | 操作 |
|------|------|
| `↑` `↓` / `j` `k` | 菜单上下移动（主页/学习中心/专题列表/设置等） |
| `Enter` | 确认进入 / 提交答案 / 继续下一项 |
| `→` / `l` | 在支持页面进入下一步（如学习概览进入练习、统计切标签） |
| `←` / `h` | 在支持页面返回或反向切换 |
| `Tab` | 对着打跳过当前命令；统计页切换标签 |
| `Shift+Tab` | 统计页切换到上一个标签 |
| `M` | 对着打模式切换显示档位（Terminal/Standard/Detailed） |
| `D` | 打开/关闭深度解析面板（在支持页面） |
| `H` | 对着打模式切换词元提示（开始前或完成后） |
| `Ctrl+R` | 重练当前命令（对着打 / 命令学习练习） |
| `Backspace` | 默写模式删除一个字符 |
| `Esc` | 返回上一级/主页 |
| `q` | 仅主页退出 |
| `Ctrl+C` | 全局退出 |

## 📁 项目结构

```
src/
├── main.rs
├── app.rs
├── event.rs
├── core/
│   ├── engine.rs
│   ├── matcher.rs
│   ├── scorer.rs
│   ├── timer.rs
│   └── terminal_history.rs
├── data/
│   ├── models.rs
│   ├── *_loader.rs
│   └── progress.rs
├── flow/
└── ui/

data/
├── commands/
├── lessons/
├── symbols/
└── system/
```

## 🛠️ 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust (edition 2024) |
| TUI | ratatui 0.29 + crossterm 0.28 |
| 数据 | TOML (课程) + JSON (进度) |
| 容器 | Docker (multi-stage build) |

## 🔒 安全

**cmdtyper 不执行任何真实命令。** 所有输出来自预置内容，没有 shell 执行路径。

## 📄 License

MIT
