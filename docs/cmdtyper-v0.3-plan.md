# cmdtyper v0.3 开发规划

> 文档版本：2026-03-28
> 状态：草稿（待 Socrates 审查）
> 负责人：Alice（统筹） + Codex subagents（开发）

---

## 一、问题陈述

v0.2 交付后，Elara 实际体验发现以下问题，需要在 v0.3 中系统修复：

1. **注意事项多行 Bug**：学习模式中"注意事项"栏目里，多行内容被渲染为单行（换行符失效）
2. **照着打历史区不显示模拟输出**：打完命令后，历史区只显示命令文本，不显示模拟输出
3. **`m`/`M` 键拦截 Bug**：打字过程中按 `m` 会触发模式切换，而不是输入字符
4. **Esc 直跳主界面**：从练习页 Esc 后直接回到主页，而不是学习中心
5. **详解模式字符溢出**：详解模式下 token 描述在窄面板内换行后，末尾字符视觉错位

功能需求：

6. 符号/系统专题加进度记录（与命令专题一致）
7. 符号专题示例加对着打环节（提升参与感）
8. 照着打加底部快捷键提示栏
9. 主页加帮助页面
10. 连续学习课程顺序（命令→符号→系统→复习）
11. 内容一致性验证（符号/系统专题命令均已在命令专题学过）
12. 复习模块重设计 + 大题库

---

## 二、改动清单

### B1：注意事项多行不渲染

**文件**：`src/ui/command_lesson.rs`
**改动**：将 gotcha content 渲染从单行改为逐行迭代

```rust
// 改前（有bug）
lines.push(Line::from(Span::styled(
    format!("    {}", gotcha.content),
    Style::default().fg(DIM),
)));

// 改后（正确）
for content_line in gotcha.content.lines() {
    lines.push(Line::from(Span::styled(
        format!("    {}", content_line),
        Style::default().fg(DIM),
    )));
}
```

**验收标准**：`awk.toml` 的 gotcha `"awk 与 Shell 引号冲突"` 内容在 UI 中正确多行显示

---

### B2：照着打历史区不显示模拟输出

**文件**：`src/data/models.rs`、`src/core/terminal_history.rs`、`src/flow/typing_flow.rs`、`src/ui/typing.rs`

**改动**：

1. `TerminalLine` 结构体加字段：
```rust
pub struct TerminalLine {
    pub prompt: String,
    pub command_display: String,
    pub status: LineStatus,
    pub simulated_output: Option<String>,  // 新增
}
```

2. `push_completed` 签名加参数：
```rust
// 改前
pub fn push_completed(&mut self, prompt: &str, display: &str) { ... }

// 改后
pub fn push_completed(&mut self, prompt: &str, display: &str, output: Option<&str>) { ... }
```

3. `typing_flow.rs` 中两处调用传入 output：
```rust
// typing_finalize_current_command + typing_skip
app.terminal_history.push_completed(&prompt, &display, cmd.simulated_output.as_deref());
```

4. `history_lines()` 改为在每条完成行后插入模拟输出行：
```rust
fn history_lines(app: &App, visible_height: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for tl in app.terminal_history.visible_lines(visible_height) {
        if tl.status == LineStatus::Completed {
            lines.push(render_completed_line(&tl.prompt, &tl.command_display));
            // 新增：输出行
            if let Some(output) = &tl.simulated_output {
                for out_line in output.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", out_line),
                        Style::default().fg(OUTPUT_COLOR),
                    )));
                }
            }
        }
    }
    lines
}
```

5. 定义 `OUTPUT_COLOR` 常量（类似 SUCCESS/WARNING）

**验收标准**：
- 打完 `echo $SHELL` 后，历史区显示命令行 + 模拟输出
- `curl -sSfL` 等无输出命令，历史区只显示命令行

---

### B3：`m`/`M` 键打字时被拦截

**文件**：`src/flow/typing_flow.rs`

**改动**：给 `m`/`M` 加打字中透传保护

```rust
// 改前（有bug）
KeyCode::Char('m') | KeyCode::Char('M')
    if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
{
    cycle_typing_mode(app);
}

// 改后（正确）：只在未开始或已结束时才拦截，打字中走 catch-all Char handler
KeyCode::Char('m') | KeyCode::Char('M')
    if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT
    && (app.typing_engine.start_time.is_none() || app.typing_engine.is_complete()) =>
{
    cycle_typing_mode(app);
}
```

**验收标准**：打字 `mkdir`、`chmod`、`rm` 等含 `m` 的命令时，`m` 正常输入

---

### B4：Esc 直跳主界面

**文件**：`src/app.rs`

**改动**：在 `App` 结构体加 `typing_return_state: Option<AppState>` 字段，记录从哪里进入 Typing 模式；Esc 时回到该状态。

```rust
// App struct 新增字段
pub typing_return_state: Option<AppState>,

// enter_typing_with_filter 改为：
pub fn enter_typing_with_filter(&mut self, difficulty: Option<Difficulty>, category: Option<Category>) {
    self.typing_return_state = Some(AppState::LearnHub);  // 新增：记住从哪来
    self.state = AppState::TypingFilter;
    ...
}

// handle_typing_filter_key Esc：
KeyCode::Esc => {
    if let Some(ret) = self.typing_return_state.take() {
        self.state = ret;
    } else {
        self.state = AppState::Home;
    }
}

// handle_typing_key Esc 同理
```

**验收标准**：
- 从学习中心 → 难度练习 → Esc → 回到学习中心
- 从主页 → 对着打 → Esc → 回到主页

---

### B5：详解模式 token 描述字符溢出

**文件**：`src/ui/typing.rs`（详解模式的 token_panel 渲染）

**改动**：详解模式右侧面板中，当 token 描述宽度超过面板宽度时，启用自动换行：

```rust
// render_token_panel 中，用 Paragraph + Wrap 确保描述不溢出
let panel = Paragraph::new(lines)
    .block(Block::default()
        .title(" 词元详解 ")
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(DIM)))
    .wrap(Wrap { trim: false });  // 启用自动换行，不截断
```

同时加宽度检测，超长描述在面板内截断，避免尾部字符跑到屏幕外。

**验收标准**：`curl -sSfL -o app.tar.gz https://...` 在详解模式右侧面板不出现孤立"向"字

---

### F1：符号/系统专题加进度记录

**文件**：`src/data/models.rs`、`src/ui/symbol_topics.rs`、`src/ui/system_topics.rs`

**改动**：

1. `data/models.rs` 新增进度结构体：
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolTopicProgress {
    pub topic_id: String,
    pub practiced_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemSectionProgress {
    pub topic_id: String,
    pub section_id: String,
    pub practiced_count: usize,
    pub total_count: usize,
}
```

2. `UserStats` 新增字段：
```rust
pub symbol_topic_progress: Vec<SymbolTopicProgress>,
pub system_section_progress: Vec<SystemSectionProgress>,
```

3. `symbol_topics.rs` 仿 `command_topics.rs` 渲染进度条：
```rust
let practiced = app.user_stats.symbol_topic_progress.iter()
    .filter(|p| p.topic_id == topic.meta.id)
    .map(|p| p.practiced_count)
    .sum::<usize>();
lines.push(Line::from(Span::styled(
    format!("  {}/{}", practiced, topic.symbols.len()),
    Style::default().fg(DIM),
)));
```

4. `system_topics.rs` 同理，在章节列表每项后加 `practiced/total`

**验收标准**：符号专题和系统架构专题的列表页显示 `X/Y` 进度（如 `3/6`）

---

### F2：符号专题示例加对着打

**文件**：`src/app.rs`、`src/flow/symbol_flow.rs`、`src/ui/symbol_lesson.rs`、`src/data/models.rs`

**改动**：

1. `SymbolPhase` 枚举新增阶段：
```rust
pub enum SymbolPhase {
    Explain,
    Example(usize),
    ExampleTyping { symbol_idx: usize, example_idx: usize },  // 新增
    TypingPractice { exercise_idx: usize },
    Practice,
}
```

2. `handle_symbol_lesson_key` 中，在 `Example` 阶段按 Enter → 进入 `ExampleTyping`：
```rust
SymbolPhase::Example(idx) => match key.code {
    KeyCode::Enter => {
        if let Some(ex) = topic.examples.get(*idx) {
            app.typing_engine.reset(&ex.command);
            app.state = AppState::SymbolLesson {
                topic_index,
                symbol_index,
                phase: SymbolPhase::ExampleTyping { symbol_idx: symbol_index, example_idx: *idx },
            };
        }
    }
    ...
}
```

3. `ExampleTyping` 阶段按 Enter → 下一条或回到 Example：
```rust
SymbolPhase::ExampleTyping { symbol_idx, example_idx } => match key.code {
    KeyCode::Enter if app.typing_engine.is_complete() => {
        let topic = &app.symbol_topics[topic_index];
        if *example_idx + 1 < topic.symbols[*symbol_idx].examples.len() {
            let next_ex = &topic.symbols[*symbol_idx].examples[*example_idx + 1];
            app.typing_engine.reset(&next_ex.command);
            app.state = AppState::SymbolLesson {
                topic_index, symbol_index,
                phase: SymbolPhase::ExampleTyping { symbol_idx: *symbol_idx, example_idx: *example_idx + 1 },
            };
        } else {
            app.state = AppState::SymbolLesson {
                topic_index, symbol_index,
                phase: SymbolPhase::Example(*example_idx + 1),
            };
        }
    }
    ...
}
```

4. `symbol_lesson.rs` 新增 `render_example_typing` 函数（复用打字引擎渲染）

**验收标准**：符号专题示例页按 Enter 后进入对着打模式，打完显示模拟输出，再按 Enter 翻到下一条

---

### F3：照着打加底部快捷键提示栏

**文件**：`src/ui/typing.rs`

**改动**：Standard/Terminal 模式的底部栏加 hint，类似 `command_lesson.rs` 的 `hint_line`：

```rust
// 在 render_bottom_bar 的 Standard 分支，加一行 hints
let hints = hint_line(&[
    ("M", "切换模式"),
    ("H", "提示"),
    ("Tab", "跳过"),
    ("Ctrl+R", "重试"),
    ("Esc", "返回"),
]);
```

**验收标准**：照着打 Standard 模式底部显示快捷键提示

---

### F4：主页加帮助页面

**文件**：`src/app.rs`、`src/ui/home.rs`、`src/ui/help.rs`（新建）

**改动**：
1. 新建 `src/ui/help.rs`，渲染帮助内容
2. `AppState` 枚举加 `Help` 状态
3. `home.rs` 菜单加"帮助"项（index 4）
4. `handle_home_key` 中按 Enter → `AppState::Help`
5. `handle` 分派加 `AppState::Help => help::render(frame, app)`

**帮助页面内容**：
- 项目介绍
- 各模式说明（对着打/学习中心/默写/复习）
- 快捷键总表
- 版权信息

**验收标准**：主页按 Enter → 显示帮助页，Esc 返回主页

---

### F5：连续学习课程顺序

**文件**：`src/ui/learn_hub.rs`、`src/app.rs`

**改动**：验证学习中心菜单顺序，当前已按命令→符号→系统→复习排列，确认即可（无需代码改动）

当前 learn_hub_index：
- 0-3：难度快速练习（对着打）
- 4：命令专题
- 5：符号专题
- 6：系统架构专题
- 7：复习

**验收标准**：菜单已按命令→符号→系统→复习顺序，无需代码改动

---

### F6：内容一致性验证

**执行方式**：一次性脚本审计，无需代码改动

**验证脚本**：
```bash
# 1. 提取符号专题所有命令
grep -rh "command\s*=" data/symbols/ | sed 's/.*command = "//;s/".*//' | grep -v "^$" | sort -u > /tmp/symbol_cmds.txt

# 2. 提取命令专题所有命令
grep -rh "^command\s*=" data/commands/ | sed 's/.*command = "//;s/".*//' | grep -v "^$" | sort -u > /tmp/lesson_cmds.txt

# 3. 提取系统架构专题所有命令
grep -rh "command\s*=" data/system/ | sed 's/.*command = "//;s/".*//' | grep -v "^$" | sort -u > /tmp/system_cmds.txt

# 4. 检查 gap（符号中命令不在 lessons 中）
comm -23 /tmp/symbol_cmds.txt /tmp/lesson_cmds.txt > /tmp/gap_symbol.txt

# 5. 检查 gap（系统专题命令不在 lessons + symbols 中）
comm -23 /tmp/system_cmds.txt <(cat /tmp/lesson_cmds.txt /tmp/symbol_cmds.txt | sort -u) > /tmp/gap_system.txt
```

**输出**：gap 列表（缺失的命令），用于决定是否补充 lessons 数据

**验收标准**：输出一致性报告（哪些命令缺失），Gap 为 0 则通过

---

### F7：复习模块重设计 + 大题库

**文件**：新增 `data/reviews/` 目录 + `src/flow/review_flow.rs` + `src/ui/review.rs`

**改动范围最大，详见第三部分专项设计**

---

## 三、F7 专项设计：复习模块

### 3.1 当前现状

当前 `build_review_exercises` 直接从 commands/lessons/symbols/system 数据中随机抽取题，题量有限，无独立题库。

### 3.2 新数据结构设计

新增 `data/reviews/` 目录，每专题一个文件（如 `commands_basic.toml`）：

```toml
[[exercises]]
# 每个练习 = 一个独立的复习题
# 每个命令的所有常见用法/选项各 ≥ 3 题
command = "grep -E 'pattern' file"
prompt = "用 grep 过滤含 'pattern' 的行（基础正则）"
difficulty = "basic"
type = "typing"  # typing | dictation
```

### 3.3 题库规模目标

| 专题 | 目标题数 | 说明 |
|------|---------|------|
| 命令·入门 | 30+ | 覆盖 ls/cd/mkdir/cp/mv/rm 等基础命令 |
| 命令·基础 | 45+ | 覆盖 grep/find/sed/awk/cut/sort 等文本处理 |
| 命令·进阶 | 45+ | 覆盖 tar/ssh/ps/kill/systemctl 等 |
| 命令·实战 | 60+ | 管道组合场景 |
| 符号·管道重定向 | 30+ | 每个符号 ≥ 3 题 |
| 符号·通配符 | 20+ | 每个通配符 ≥ 3 题 |
| 系统架构 | 30+ | 每个配置命令 ≥ 3 题 |

### 3.4 UI 重设计

**改前**：随机混合题库 + 通用总结页
**改后**：分专题复习，每专题独立进度

复习页面菜单（`AppState::Review`）：
- 进入专题列表 → 选择专题 → 复习该专题全部题库 → 完成后显示该专题统计

---

## 四、ToDo List

| # | 任务 | 验收标准 | 状态 |
|---|------|---------|------|
| T1 | B1: gotcha 多行渲染 | `awk.toml` gotcha 多行正确显示 | 待开发 |
| T2 | B2: 历史区显示模拟输出 | 打完 `echo $SHELL` 显示输出 | 待开发 |
| T3 | B3: `m`/`M` 键透传 | `mkdir`/`chmod`/`rm` 正常输入 | 待开发 |
| T4 | B4: Esc 返回学习中心 | 学习中心→练习→Esc 回学习中心 | 待开发 |
| T5 | B5: 详解模式面板溢出 | 无孤立"向"字 | 待开发 |
| T6 | F1: 符号/系统专题进度 | 列表页显示 `X/Y` 进度 | 待开发 |
| T7 | F2: 符号示例加对着打 | 示例页按 Enter 进入打字 | 待开发 |
| T8 | F3: 照着打底部提示栏 | Standard 模式底部有快捷键 | 待开发 |
| T9 | F4: 帮助页面 | 主页→帮助→显示内容 | 待开发 |
| T10 | F5: 课程顺序验证 | 菜单已是命令→符号→系统→复习 | 无需开发 |
| T11 | F6: 内容一致性审计 | 输出一致性报告 | 待执行 |
| T12 | ~~B6: ReviewExercise~~ ✅ 无此 bug | 无需改动 | ✅ 完成 |
| T13 | F7: 复习题库数据 | data/reviews/ 下有 ≥ 200 题 | 待开发 |
| T14 | F7: 复习 UI 重设计 | 分专题复习 + 独立进度 | 待开发 |

---

**B6 调研结论（2026-03-28）**：app.rs 只有一处定义，flow 引用路径正确，agent 调研误报，无需改动

---

## 六、风险点

1. **B2（历史输出）**：改动 `TerminalLine` 结构会影响历史序列化；需检查 `ProgressStore` 是否持久化该字段
2. **B4（Esc 返回）**：`typing_return_state` 需要在所有进入 Typing 的入口点正确设置
3. **F2（符号对着打）**：`SymbolPhase` 新增 `ExampleTyping` 变体需更新所有 `matches!` 模式匹配
4. **F7（复习重设计）**：旧 review 进度记录格式需要兼容或做迁移脚本；SM-2 v0.3 暂不做，先做基础分专题大题库
5. **所有改动**：修改 `AppState` 变体时需检查 `ResumeState` 映射是否需要同步更新
