# TODO.md — cmdtyper v0.2 开发检查清单

> 每个 Phase 末尾有调试检查项，必须全部通过才能进入下一 Phase。
> 最后的 Phase F 包含总集成调试。

---

## Phase 0: 项目骨架

### 0.1 项目结构
- [ ] `Cargo.toml` 配置（所有依赖、版本 0.2.0）
- [ ] `.gitignore`（target/、*.swp、.DS_Store）
- [ ] `Dockerfile`（multi-stage build）
- [ ] `docker-compose.yml`（TTY 支持 + 数据卷）
- [ ] 从 v0.1 迁移 `data/commands/` 题库（19 个 TOML 文件）

### 0.2 数据层骨架
- [ ] `src/data/models.rs` — 所有类型定义（§3 完整）
  - [ ] Difficulty, Category, Mode, Importance, PromptStyle 枚举
  - [ ] Command, CommandFile, Token, DictationData, OutputAnnotation
  - [ ] CommandLesson, LessonMeta, LessonOverview, SyntaxInfo, OptionInfo, LessonExample, Gotcha
  - [ ] SymbolTopic, SymbolTopicMeta, SymbolEntry, SymbolExample, Exercise
  - [ ] SystemTopic, SystemTopicMeta, SystemSection, SystemCommand, ConfigFile, ConfigLesson
  - [ ] ReviewData, ReviewGroup, ReviewItem
  - [ ] Keystroke, SessionRecord, CharStat, CommandProgress, DailyStat, UserStats, UserConfig
- [ ] `src/data/mod.rs` — 模块声明
- [ ] `src/data/command_loader.rs` — `load_commands(data_dir) -> Result<Vec<Command>>`
  - [ ] 遍历 `data/commands/*.toml`
  - [ ] 解析为 CommandFile，传播 meta 到每个 Command
  - [ ] v0.2 新增字段兼容（display, summary_short, simulated_output 可选）
- [ ] `src/data/lesson_loader.rs` — `load_lessons(data_dir) -> Result<Vec<CommandLesson>>`
  - [ ] 遍历 `data/lessons/*.toml`
- [ ] `src/data/symbol_loader.rs` — `load_symbol_topics(data_dir) -> Result<Vec<SymbolTopic>>`
  - [ ] 遍历 `data/symbols/*.toml`
- [ ] `src/data/system_loader.rs` — `load_system_topics(data_dir) -> Result<Vec<SystemTopic>>`
  - [ ] 遍历 `data/system/*.toml`
- [ ] `src/data/progress.rs` — ProgressStore（JSON 持久化）
  - [ ] load_stats / save_stats（原子写入）
  - [ ] append_record / load_history
  - [ ] load_config / save_config（含 v0.2 新增 prompt_* 字段）
  - [ ] 损坏恢复（返回 Default）

### 0.3 核心引擎
- [ ] `src/core/mod.rs`
- [ ] `src/core/engine.rs` — TypingEngine
  - [ ] new / input / is_complete / is_error_flashing
  - [ ] current_wpm / current_cpm / current_accuracy / elapsed_secs
  - [ ] finish / reset
- [ ] `src/core/matcher.rs` — Matcher (normalize, check)
- [ ] `src/core/scorer.rs` — Scorer (update_stats, compute_mastery, weak_chars, recommend)
- [ ] `src/core/timer.rs` — Timer (start/pause/resume/elapsed/format_mmss)
- [ ] `src/core/terminal_history.rs` — TerminalHistory
  - [ ] new / push_completed / visible_lines / clear

### 0.4 App 状态机
- [ ] `src/app.rs` — App struct + AppState 枚举（§2.2 完整）
  - [ ] SymbolPhase, SystemPhase, ReviewSource, ReviewPhase 子枚举
  - [ ] App::new() — 加载题库 + 课程 + 统计
  - [ ] App::transition(new_state)
  - [ ] App::tick() — 更新动画状态
  - [ ] App::resize(w, h) — 终端尺寸
- [ ] `src/event.rs` — AppEvent + poll（50ms tick）

### 0.5 UI 骨架
- [ ] `src/ui/mod.rs` — render() 分发
- [ ] `src/ui/widgets.rs` — colors 常量 + format_time
- [ ] `src/ui/home.rs` — 主菜单（5 项：对着打/学习中心/默写/统计/设置）
  - [ ] 上下键导航 + Enter 进入
- [ ] `src/main.rs` — 终端初始化 + 事件循环 + restore

### 0.6 Phase 0 调试
- [ ] `cargo check` 无错误
- [ ] `cargo test` 通过（至少 models 编译）
- [ ] `cargo run` 显示主菜单，Esc/q 退出
- [ ] 终端 raw mode 正常进入和退出（无残留）
- [ ] Ctrl+C 安全退出
- [ ] 终端 resize 后不 panic
- [ ] 主菜单 5 项键盘导航正确
- [ ] 所有 loader 对空目录返回空 Vec，不 panic

---

## Phase A: 对着打改造（终端模拟风格）

### A.1 UI 改造
- [ ] `src/ui/typing.rs` — 完全重写
  - [ ] 去掉顶栏标题
  - [ ] 渲染 prompt（根据 UserConfig.prompt_style）
  - [ ] 已完成行：prompt + 命令文本（绿色）
  - [ ] 当前行：prompt + 三态着色（已打/光标/待打）
  - [ ] 终端历史滚动（TerminalHistory.visible_lines）
  - [ ] 底栏：左侧 summary_short（H 切换）+ 右侧 WPM + 准确率
  - [ ] 多行命令（display 含 `\`）渲染为多行，续行 `> `

### A.2 交互逻辑
- [ ] 最后一个字符正确 → 自动完成当前行
- [ ] 完成后：push_completed → 下一题 → 新 prompt
- [ ] H 键切换含义提示显隐
- [ ] Tab 跳过 / Ctrl+R 重练 / Esc 返回

### A.3 数据增强
- [ ] 现有题库中为关键命令添加 `simulated_output`（至少 20 条）
- [ ] 为长命令添加 `display` 字段（至少 5 条管道/find 命令）

### A.4 Phase A 调试
- [ ] PromptStyle::Full / Simple / Minimal 三种渲染正确
- [ ] 已完成行绿色，不可交互
- [ ] 当前行三态着色正确（白/光标/灰）
- [ ] 错误闪红 ~150ms
- [ ] 多行命令渲染正确（`\` 换行 + `> ` 续行）
- [ ] 终端历史滚动：旧行从顶部消失，不 panic
- [ ] 底栏含义提示 H 键切换
- [ ] 窗口 resize 后布局自适应
- [ ] 连续快速输入不丢键

---

## Phase B: 学习中心骨架 + 命令专题

### B.1 二级菜单
- [ ] `src/ui/learn_hub.rs` — 学习中心菜单
  - [ ] 4 项：命令专题 / 符号专题 / 系统架构 / 专题复习
  - [ ] 上下键导航 + Enter + Esc
- [ ] App 状态转换：Home → LearnHub → CommandTopics / SymbolTopics / ...

### B.2 命令专题列表
- [ ] `src/ui/command_topics.rs` — 按 Category 列表
  - [ ] 显示类别名 + icon + 难度 + 进度（已学/总数）
  - [ ] Enter → 进入该类别的命令列表
  - [ ] 命令列表内上下选择 → Enter 进入学习

### B.3 单命令学习三阶段 UI
- [ ] `src/ui/command_lesson.rs`
  - [ ] `render_overview()` — 概览页
    - [ ] 显示 LessonOverview.explanation
    - [ ] 显示 SyntaxInfo.basic + parts
    - [ ] 显示 OptionInfo 列表
    - [ ] Enter → 进入示例
  - [ ] `render_example()` — 示例页
    - [ ] 显示 LessonExample.command + summary
    - [ ] 显示 token 注释树（从 commands 题库匹配，或 lesson 自带）
    - [ ] 显示模拟输出框（simulated_output）
    - [ ] ←→ 翻页 example_index
    - [ ] Enter → 进入跟打
  - [ ] `render_practice()` — 跟打页
    - [ ] 复用 TypingEngine
    - [ ] 显示 token 注释（简化版）
    - [ ] 打完后显示 simulated_output
    - [ ] WPM / 准确率 / 耗时
    - [ ] ←→ 翻页 / Tab 跳过 / ^R 重练

### B.4 缺少 lesson 文件时的降级
- [ ] 如果 `data/lessons/{command}.toml` 不存在，overview 页显示 summary + tokens
- [ ] 不影响 Practice 阶段（从 commands 题库取数据）

### B.5 模拟输出框渲染组件
- [ ] `src/ui/widgets.rs` 新增 `render_simulated_output(f, area, command, output)`
  - [ ] 终端框边框（DarkGray）
  - [ ] 框内第一行：绿色 `$ ` + 白色命令
  - [ ] 框内后续行：白色输出文本
  - [ ] 限制最大高度（超出截断 + `...`）

### B.6 内容编写（命令讲解 TOML）
- [ ] `data/lessons/ls.toml`
- [ ] `data/lessons/cd.toml`
- [ ] `data/lessons/cat.toml`
- [ ] `data/lessons/grep.toml`
- [ ] `data/lessons/find.toml`
- [ ] `data/lessons/chmod.toml`
- [ ] `data/lessons/cp.toml`
- [ ] `data/lessons/mv.toml`
- [ ] `data/lessons/rm.toml`
- [ ] `data/lessons/mkdir.toml`
- [ ] `data/lessons/ps.toml`
- [ ] `data/lessons/curl.toml`
- [ ] `data/lessons/tar.toml`
- [ ] `data/lessons/awk.toml`
- [ ] `data/lessons/sed.toml`

### B.7 Phase B 调试
- [ ] LearnHub 菜单导航正确
- [ ] CommandTopics 按 category 列表正确
- [ ] 三阶段 Overview → Example → Practice 切换正确
- [ ] 缺少 lesson 文件时 overview 不 panic，显示降级内容
- [ ] 模拟输出框渲染不超出终端宽度
- [ ] 模拟输出在 Practice 打完后出现
- [ ] ←→ 翻页 example/practice 边界正确（首/末不越界）
- [ ] Esc 层层返回：Practice → Example → Overview → CommandTopics → LearnHub → Home
- [ ] 所有 15 个 lesson TOML 解析无错误

---

## Phase C: 符号专题

### C.1 UI
- [ ] `src/ui/symbol_topics.rs` — 符号专题列表
  - [ ] 显示各 topic 名称 + icon + 难度
- [ ] `src/ui/symbol_lesson.rs` — 单符号学习
  - [ ] 讲解页：char_repr + name + explanation
  - [ ] 示例页：command + explanation + simulated_output
  - [ ] 练习页：prompt → 用户输入 → Matcher 匹配判定

### C.2 内容编写
- [ ] `data/symbols/pipe_redirect.toml` — `|` `>` `>>` `2>` `2>&1` `<` `<<` `tee`
- [ ] `data/symbols/wildcards.toml` — `*` `?` `[]` `{}`
- [ ] `data/symbols/quotes_escape.toml` — `'...'` `"..."` `\` `` `...` `` `$(...)`
- [ ] `data/symbols/variables.toml` — `$VAR` `${VAR}` `$()` `$?` `$!` `$#` `$@` `$$`
- [ ] `data/symbols/special_chars.toml` — `;` `&&` `||` `&` `!` `#` `~` `.`
- [ ] `data/symbols/regex_basics.toml` — `.` `*` `+` `?` `^` `$` `[]` `()`

### C.3 Phase C 调试
- [ ] 符号专题列表正确显示
- [ ] 符号讲解多行文本渲染正确
- [ ] 示例 ←→ 翻页正确
- [ ] 练习题 Matcher 匹配逻辑正确
- [ ] 空 exercises 时跳过练习
- [ ] Esc 层层返回正确
- [ ] 所有 6 个 symbol TOML 解析无错误

---

## Phase D: 系统架构专题

### D.1 UI
- [ ] `src/ui/system_topics.rs` — 系统架构专题列表
- [ ] `src/ui/system_lesson.rs` — 单主题学习
  - [ ] Overview 页：ASCII art 总览
  - [ ] Detail 页：章节详细讲解
  - [ ] Commands 页：常用命令 + 模拟输出
  - [ ] ConfigFile 页：配置文件内容 + before/after 对比
  - [ ] 章节间 ←→ 翻页

### D.2 内容编写
- [ ] `data/system/directory_structure.toml` — FHS 目录结构
- [ ] `data/system/filesystem_permissions.toml` — rwx、umask、SUID/SGID
- [ ] `data/system/process_systemd.toml` — 进程生命周期、systemd unit
- [ ] `data/system/network_basics.toml` — ip、DNS、端口、ufw
- [ ] `data/system/package_management.toml` — apt/yum/pacman 对比
- [ ] `data/system/config_files.toml` — bashrc, sshd_config, nginx.conf, fstab

### D.3 Phase D 调试
- [ ] 系统架构列表正确显示
- [ ] ASCII art overview 不错位（中文宽字符）
- [ ] 章节间导航正确
- [ ] 配置文件 before/after 对比渲染正确（高亮差异）
- [ ] 空 config_files 时跳过配置环节
- [ ] Esc 层层返回正确
- [ ] 所有 6 个 system TOML 解析无错误

---

## Phase E: 专题复习

### E.1 UI
- [ ] `src/ui/review.rs`
  - [ ] Summary 页：分组表格展示（ReviewGroup + ReviewItem）
  - [ ] Practice 页：随机抽题
    - [ ] 打字题（TypingEngine）和默写题（Matcher）交替
    - [ ] 显示正确率和进度

### E.2 复习数据生成
- [ ] 从 commands 按 Category 自动生成 ReviewData
- [ ] 从 SymbolTopic 自动生成 ReviewData
- [ ] 从 SystemTopic 自动生成 ReviewData
- [ ] 集中练习随机排序（使用 rand）

### E.3 Phase E 调试
- [ ] 知识梳理表格渲染正确
- [ ] 集中练习随机抽题不重复
- [ ] 打字题和默写题交替出现
- [ ] 练习结束后统计正确率
- [ ] 空 practice_ids 时显示"暂无可练习内容"

---

## Phase F: 设置 + 打磨 + 最终集成调试

### F.1 设置页面
- [ ] `src/ui/settings.rs`
  - [ ] PromptStyle 选择（三选一）
  - [ ] username / hostname 编辑
  - [ ] show_path 开关
  - [ ] target_wpm 数值调整
  - [ ] error_flash_ms 数值调整
  - [ ] show_token_hints 开关
  - [ ] adaptive_recommend 开关
  - [ ] Esc 返回时自动保存

### F.2 UI 打磨
- [ ] 主菜单 ASCII art 标题
- [ ] 菜单项 icon + 描述对齐
- [ ] 统一颜色主题（确认所有页面用 colors 常量）
- [ ] 页面切换无闪烁

### F.3 Docker
- [ ] Dockerfile 更新（含新 data/ 子目录）
- [ ] `docker compose build` 成功
- [ ] `docker compose run --rm cmdtyper` 可交互
- [ ] 数据卷持久化验证

### F.4 最终集成调试
- [ ] **全流程走通**：Home → 对着打 → 练几题 → Esc → 学习中心 → 命令专题 → Overview → Example → Practice → Review → 符号专题 → 系统架构 → 默写 → 统计 → 设置
- [ ] **状态机无泄漏**：每个 Esc 返回路径正确，无死循环
- [ ] **数据持久化**：练习记录保存，重启后统计正确
- [ ] **极端输入**
  - [ ] 空命令处理
  - [ ] 超长命令（>200 字符）
  - [ ] 连续快速输入（>100 WPM）
  - [ ] 非 ASCII 输入过滤
- [ ] **终端兼容**
  - [ ] 小窗口 80×24
  - [ ] 大窗口 200+ 列
  - [ ] tmux / screen 内运行
- [ ] **中文渲染**
  - [ ] unicode-width 对齐验证
  - [ ] 中英混排不错位
  - [ ] token 注释树对齐
- [ ] **性能**
  - [ ] 300+ 命令题库加载 < 200ms
  - [ ] 1000+ 历史记录加载 < 500ms
  - [ ] 渲染帧率 ≥ 20 FPS
- [ ] `cargo clippy` 无 warning
- [ ] `cargo test` 全部通过
- [ ] `cargo build --release` 无错误
- [ ] README.md 更新（新功能 + 截图）

### F.5 集成测试
- [ ] `tests/parse_all.rs` — 所有 TOML 文件解析通过
- [ ] `tests/tokens_consistency.rs` — 所有 tokens 拼接 == command
- [ ] `tests/id_uniqueness.rs` — 全局 ID 唯一

### F.6 发布
- [ ] Git tag v0.2.0
- [ ] CHANGELOG.md
- [ ] GitHub release
