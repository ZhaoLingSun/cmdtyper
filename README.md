# cmdtyper

> 🐧 Terminal-based Linux command typing practice — learn, type, dictate, and track your progress.

**cmdtyper** is a TUI (Terminal User Interface) application built with Rust for practicing Linux commands through typing. It features multiple practice modes, a rich command database with Chinese annotations, and detailed performance tracking — all running directly in your terminal.

## ✨ Features

- **4 Practice Modes** — Learn, Type, Dictation, and Stats
- **273 Commands** across 19 topic files, from beginner `ls` to advanced `awk` pipelines
- **Character-by-character matching** with real-time error highlighting
- **Dictation mode** with multi-answer matching and intelligent diff
- **Performance tracking** — WPM, accuracy, character-level analysis, streaks
- **Persistent progress** saved locally as JSON
- **No command execution** — all commands are text-only practice material (safe by design)
- **Docker support** for containerized usage

## 📦 Modes

### 🎓 Learn

Displays each command with token-by-token annotations explaining every flag and argument. You follow along by typing, building muscle memory with understanding.

```
Command: ls -la /var/log

  ls      ─ 列出目录内容的命令
  -la     ─ -l 详细列表格式 + -a 显示隐藏文件
  /var/log ─ 目标目录：常见的系统日志目录
```

### ⌨️ Type

Classic typing practice: the command is shown, and you type it character by character. Correct characters advance the cursor; errors flash red without advancing — just like a real typing trainer.

- Real-time WPM / accuracy / elapsed time
- Token annotations visible as hints

### 📝 Dictation

Only a Chinese description is shown (e.g., *"显示 /var/log 目录下所有文件的详细信息"*). You write the command from memory.

- Multiple accepted answers per question
- Normalized matching (extra spaces, quote style)
- On wrong answers: shows the closest correct answer with diff highlighting

### 📊 Stats

A keybr-style analytics dashboard:

- **Speed overview** — average/best WPM, accuracy, session count
- **Character analysis** — per-character speed and error rate
- **Category mastery** — progress bars per topic
- **Practice calendar** — daily activity heatmap and streaks

## 🗂️ Command Database

Commands are stored as TOML files under `data/commands/`:

| File | Topic | Difficulty |
|------|-------|------------|
| `01_beginner.toml` | File & directory basics | ★☆☆☆ |
| `02_basic.toml` | Permissions, search, system | ★★☆☆ |
| `03_advanced.toml` | Text processing, pipelines | ★★★☆ |
| `04_practical.toml` | Real-world DevOps combos | ★★★★ |
| `05`–`19` | Extended topics (git, curl, firewall, networking, etc.) | Mixed |

**Total: 273 commands** across 4 difficulty levels and 10+ categories.

<details>
<summary>TOML format example</summary>

```toml
[meta]
category = "file_ops"
difficulty = "beginner"
description = "基础文件和目录操作命令"

[[commands]]
id = "ls-all-long"
command = "ls -la /var/log"
summary = "显示 /var/log 目录的详细列表（包含隐藏文件）"

[[commands.tokens]]
text = "ls"
desc = "列出目录内容的命令"

[[commands.tokens]]
text = "-la"
desc = "-l 详细列表格式 + -a 显示隐藏文件"

[[commands.tokens]]
text = "/var/log"
desc = "目标目录：常见的系统日志目录"

[commands.dictation]
prompt = "显示 /var/log 目录下所有文件（包含隐藏文件）的详细信息"
answers = [
    "ls -la /var/log",
    "ls -al /var/log",
    "ls --all -l /var/log",
]
```

</details>

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.82+ (edition 2024)
- A terminal with 256-color support

### Build from source

```bash
git clone https://github.com/ZhaoLingSun/cmdtyper.git
cd cmdtyper
cargo build --release
./target/release/cmdtyper
```

### Run with Docker

```bash
docker compose build
docker compose run --rm cmdtyper
```

Progress data is persisted in a Docker volume (`cmdtyper_data`).

### Install via Cargo (once published)

```bash
cargo install cmdtyper
```

## ⌨️ Keybindings

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate menu / select items |
| `←` `→` | Switch difficulty / navigate |
| `Enter` | Confirm selection / submit answer / next command |
| `Tab` | Skip current command |
| `Ctrl+R` | Retry current command |
| `Esc` | Back to main menu |
| `q` | Quit |

## 📁 Data Storage

User progress is saved locally:

```
~/.local/share/cmdtyper/
├── history.json    # All practice session records
├── stats.json      # Aggregated statistics
└── config.json     # User preferences
```

## 🔒 Safety

**cmdtyper does not execute any commands.** All Linux commands in the database are treated purely as text for typing practice. There is no shell invocation, no `Command::new()`, no `system()` call — by design.

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (edition 2024) |
| TUI framework | [ratatui](https://github.com/ratatui/ratatui) 0.29 + [crossterm](https://github.com/crossterm-rs/crossterm) 0.28 |
| Data format | TOML (questions) + JSON (progress) |
| Serialization | serde + serde_json + toml |
| Unicode handling | unicode-width |
| Containerization | Docker (multi-stage build) |

## 🤝 Contributing

### Adding commands

1. Create or edit a TOML file under `data/commands/`
2. Follow the format in existing files
3. Ensure every `tokens[].text` concatenated with spaces equals the `command` field
4. Provide at least one `dictation.answers` entry

### Development

```bash
cargo test          # Run tests
cargo clippy        # Lint
cargo run           # Run in dev mode
```

## 📄 License

MIT
