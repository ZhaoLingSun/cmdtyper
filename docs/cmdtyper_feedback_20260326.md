# cmdtyper 用户反馈记录

**日期**: 2026-03-26  
**反馈人**: Elara  
**记录人**: Alice

---

## Bug #1: 练习完后光标弹回第一个

### 现象
在「学习中心 → 命令专题」列表中，选择某个分类（如「进程管理」）进入学习，练习完成后返回列表时，光标会自动跳回第一项（「文件操作」），而不是停留在之前选择的位置。

### 根因
`src/app.rs` 第 571-573 行：
```rust
4 => {
    self.command_topics_index = 0;  // ❌ 强制重置为 0
    self.state = AppState::CommandTopics;
}
```
每次从学习中心进入「命令专题」时，都会把 `command_topics_index` 强制重置为 0。

### 影响
- 用户体验差：每次练习完都要重新找到之前的位置
- 无法连续练习同一分类下的多个命令

### 建议修复方案
1. **方案 A（推荐）**: 不重置 `command_topics_index`，保持上次选择的位置
2. **方案 B**: 在 `UserStats` 中记录 `last_visited_category`，每次进入时恢复到最近访问的分类

---

## Bug #2: 练习完后进度计数器仍显示 0/7

### 现象
在「命令专题」列表中，每个分类右侧显示进度（如「0/7」），但即使完成了该分类下的命令练习，计数器仍然显示 0，没有任何变化。

### 根因
`command_id` 格式不匹配：

**记录进度时**（`src/flow/lesson_flow.rs` 第 113 行）：
```rust
let command_id = format!("lesson:{}:{}", lesson.meta.command, example_index);
// 结果: "lesson:ssh:0"
```

**统计进度时**（`src/ui/command_topics.rs` 第 52-60 行）：
```rust
let practiced = app.user_stats.command_progress.iter()
    .filter(|p| {
        app.lessons.iter()
            .any(|l| l.meta.category == *cat && l.meta.command == p.command_id)
            //                                    ^^^^^^^^^^^^^^^^ 比对 "ssh"
    })
    .count();
```

`l.meta.command` 是 `"ssh"`，而 `p.command_id` 是 `"lesson:ssh:0"`，两者永远不会匹配。

### 影响
- 进度追踪完全失效
- 用户无法看到自己的学习进度
- 统计面板中的「类别掌握度」也会受影响

### 建议修复方案
1. **方案 A（推荐）**: 统一 `command_id` 格式，去掉 "lesson:" 前缀和示例索引，改为纯命令名 `"ssh"`
2. **方案 B**: 在统计时做字符串解析，提取 "lesson:ssh:0" 中的 "ssh" 部分进行比对
3. **方案 C**: 在 `CommandProgress` 中新增 `base_command` 字段，记录时同时存储完整 ID 和基础命令名

---

## Bug #3: 长页面无法滚动查看完整内容

### 现象
在「命令专题 → SSH 概览」等长页面中，内容超出屏幕高度，但 `↑`/`↓` 键被用来切换不同命令，无法向下滚动查看完整内容。鼠标滚轮在 TUI 中默认也是翻页而非滚动。

### 根因
`AppState::CommandLessonOverview` 没有 `scroll` 字段，`↑`/`↓` 键的行为在 `src/flow/lesson_flow.rs` 第 23-44 行被定义为切换命令：
```rust
KeyCode::Up | KeyCode::Char('k') => {
    if command_index > 0 {
        app.state = AppState::CommandLessonOverview {
            category_index,
            command_index: command_index - 1,  // 切换到上一个命令
        };
    }
}
```

### 影响
- 长内容无法完整查看（如 SSH 的所有选项说明）
- 用户只能看到屏幕可见部分，学习体验受限

### 建议修复方案
1. **方案 A（推荐）**: 为 `CommandLessonOverview` 添加 `scroll` 字段，用 `PageUp`/`PageDown` 或 `Ctrl+↑`/`Ctrl+↓` 滚动内容，保留 `↑`/`↓` 用于切换命令
2. **方案 B**: 改用 `h`/`l` 或 `[`/`]` 切换命令，释放 `↑`/`↓` 用于滚动
3. **方案 C**: 在底部提示中增加 `PgUp`/`PgDn` 滚动提示，并实现对应功能

---

## 其他观察

### 正常功能
- ✅ 深度解析（`D` 键）支持滚动，体验良好
- ✅ 练习模式的打字引擎工作正常
- ✅ 统计面板的 WPM/准确率记录准确

### 待确认
- Resume 功能是否能正确恢复到上次练习的位置？（需要测试退出后重新进入的场景）
- 符号专题和系统架构专题是否有类似的光标重置问题？

---

**记录完成时间**: 2026-03-26 16:00 UTC
