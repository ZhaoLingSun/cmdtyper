# cmdtyper 开发检查清单

## Phase 0: 项目骨架
- [x] 项目目录结构创建
- [x] Cargo.toml 依赖配置
- [x] Git 仓库初始化
- [x] .gitignore 配置
- [x] Dockerfile (multi-stage build)
- [x] docker-compose.yml
- [x] PLAN.md 技术规划文档
- [x] TODO.md 检查清单
- [ ] `src/main.rs` — 终端初始化 + raw mode + alternate screen + 事件循环
- [ ] `src/app.rs` — App struct + AppState 枚举 + 状态转换方法
- [ ] `src/event.rs` — AppEvent 枚举 + 50ms tick 事件生成
- [ ] `src/ui/mod.rs` — Screen trait 定义 + 顶层 render 分发
- [ ] `src/ui/home.rs` — 主菜单渲染（标题 + 4 选项 + 难度选择 + 快捷键）
- [ ] `src/ui/home.rs` — 上下键导航 + Enter 进入模式 + 左右键切换难度
- [ ] 编译通过，`cargo run` 显示主菜单，Esc/q 退出

### P0 调试检查
- [ ] 终端 raw mode 正常进入和退出（无残留）
- [ ] Ctrl+C 能安全退出
- [ ] 终端 resize 后布局正常
- [ ] 主菜单键盘导航响应正确

---

## Phase 1: 对着打核心引擎
- [ ] `src/data/models.rs` — Keystroke, SessionRecord, Mode 结构体
- [ ] `src/core/mod.rs` — 模块声明
- [ ] `src/core/timer.rs` — Timer struct (start/pause/resume/elapsed/format_mmss)
- [ ] `src/core/engine.rs` — TypingEngine struct 定义
- [ ] `src/core/engine.rs` — `new(target_str)` 构造函数
- [ ] `src/core/engine.rs` — `input(ch) -> InputResult` 核心逻辑
  - [ ] 正确：记录 Keystroke(correct=true) + cursor++ + 更新 last_correct_time
  - [ ] 错误：记录 attempts++ + 设置 error_flash = Some(Instant::now())
  - [ ] 首字符启动计时器
- [ ] `src/core/engine.rs` — `is_complete()` 完成检测
- [ ] `src/core/engine.rs` — `current_wpm()` 实时 WPM 计算
- [ ] `src/core/engine.rs` — `current_accuracy()` 实时准确率
- [ ] `src/core/engine.rs` — `finish()` 生成 SessionRecord
- [ ] `src/ui/typing.rs` — 对着打页面布局（顶栏 + 主区 + 底栏）
- [ ] `src/ui/typing.rs` — 逐字符渲染（三态着色：已完成/光标/待输入）
- [ ] `src/ui/typing.rs` — 错误闪红动画（150ms 红底 → 恢复）
- [ ] `src/ui/typing.rs` — token 注释树渲染（├─ └─ 格式）
- [ ] `src/ui/typing.rs` — 底栏实时 WPM + 准确率 + 耗时
- [ ] `src/ui/typing.rs` — Esc 返回 / Tab 跳过 / Ctrl+R 重练

### P1 调试检查
- [ ] 正确字符输入后光标前进一位，字符变白
- [ ] 错误输入后光标不动，字符闪红 ~150ms
- [ ] 连续快速输入不丢键
- [ ] WPM 计算在第一个字符后开始（不含等待时间）
- [ ] 全部打完后自动进入 RoundResult
- [ ] 空格、特殊字符（`|`, `>`, `~`, `$`, `-`）都能正常匹配
- [ ] 中文注释不影响对齐

---

## Phase 2: 题库系统
- [ ] `src/data/models.rs` — Command, Token, DictationData, CommandFile, FileMeta
- [ ] `src/data/models.rs` — Difficulty, Category 枚举 + label()/stars() 方法
- [ ] `src/data/loader.rs` — DataLoader struct
- [ ] `src/data/loader.rs` — `load_all()` 遍历 data/commands/*.toml
- [ ] `src/data/loader.rs` — `load_by_difficulty()` 筛选
- [ ] `src/data/loader.rs` — `load_by_category()` 筛选
- [ ] `data/commands/01_beginner.toml` — 入门题库 15 条
  - [ ] ls 系列 (ls, ls -la, ls -lhS, ls -R)
  - [ ] cd 系列 (cd, cd .., cd ~, cd -)
  - [ ] 基础操作 (pwd, mkdir -p, touch, cat, echo, cp, mv, rm)
- [ ] TOML 格式校验脚本 (`scripts/validate_toml.sh`)
- [ ] 将 DataLoader 集成到 App，主菜单显示可用命令数

### P2 调试检查
- [ ] TOML 解析无报错
- [ ] 全部 token.text 拼接 == command（一致性验证）
- [ ] 题库文件缺失时优雅报错
- [ ] 格式错误时给出明确行号和字段提示

---

## Phase 3: 学习模式
- [ ] `src/ui/learn.rs` — 页面布局（命令概述 + 语法讲解 + 输入区）
- [ ] `src/ui/learn.rs` — 完整 token 树状注释展示
- [ ] `src/ui/learn.rs` — summary 文本显示
- [ ] `src/ui/learn.rs` — 跟打输入（复用 TypingEngine，但 error 处理更宽松）
- [ ] `src/ui/learn.rs` — 命令间导航（←→ 上一条/下一条）
- [ ] Tab 跳过 + Ctrl+R 重练

### P3 调试检查
- [ ] 注释树对齐正确（├─ └─ 格式，含中文宽字符）
- [ ] 输入区与注释区不重叠
- [ ] 跟打完成后显示"已完成"提示 + 自动翻页

---

## Phase 4: 默写模式
- [ ] `src/core/matcher.rs` — `normalize()` 字符串标准化
  - [ ] 去除首尾空格
  - [ ] 连续空格合并为单空格
  - [ ] 统一单双引号（可选）
- [ ] `src/core/matcher.rs` — `check()` 多答案匹配
  - [ ] 精确匹配路径
  - [ ] 标准化后匹配路径
  - [ ] 无匹配 → 计算最接近答案 + diff
- [ ] `src/core/matcher.rs` — DiffSegment 差异计算（简单 LCS diff）
- [ ] `src/ui/dictation.rs` — 题目展示（中文提示 + 难度标签）
- [ ] `src/ui/dictation.rs` — 自由输入框（支持退格、光标移动）
- [ ] `src/ui/dictation.rs` — Enter 提交 → 判定结果
- [ ] `src/ui/dictation.rs` — 正确：绿色 ✅ + 展示匹配的答案
- [ ] `src/ui/dictation.rs` — 错误：红色 ❌ + 展示所有正确答案 + 差异高亮
- [ ] `src/ui/dictation.rs` — Space/Enter 下一题

### P4 调试检查
- [ ] `"ls  -la"` (双空格) 匹配 `"ls -la"` → Normalized
- [ ] `"ls -al"` 匹配 answers 中的 `"ls -al"` → Exact
- [ ] 完全不匹配时 diff 高亮正确
- [ ] 退格键正常工作
- [ ] 光标在输入框内正常显示

---

## Phase 5: 统计系统
- [ ] `src/data/models.rs` — CharStat, CharSpeedPoint, CommandProgress, DailyStat, UserStats
- [ ] `src/data/models.rs` — UserConfig + Default impl
- [ ] `src/core/scorer.rs` — `update_stats()` 从 SessionRecord 更新 UserStats
- [ ] `src/core/scorer.rs` — `update_char_stat()` 字符级统计
- [ ] `src/core/scorer.rs` — `update_command_progress()` 命令进度
- [ ] `src/core/scorer.rs` — `compute_mastery()` 掌握度公式
- [ ] `src/core/scorer.rs` — `weak_chars(n)` 薄弱字符 Top N
- [ ] `src/core/scorer.rs` — `category_mastery()` 类别掌握度
- [ ] `src/core/scorer.rs` — `recommend_commands()` 智能推荐
- [ ] `src/data/progress.rs` — ProgressStore 构造（~/.local/share/cmdtyper/）
- [ ] `src/data/progress.rs` — load_stats / save_stats
- [ ] `src/data/progress.rs` — append_record / load_history
- [ ] `src/data/progress.rs` — load_config / save_config
- [ ] `src/data/progress.rs` — 原子写入（.tmp → rename）
- [ ] `src/ui/stats.rs` — Tab 切换框架 (4 个子页面)
- [ ] `src/ui/stats.rs` — Tab 1 速度总览
  - [ ] 四个统计卡片（平均WPM / 最高WPM / 准确率 / 练习次数）
  - [ ] WPM 趋势 sparkline（最近 50 次）
  - [ ] 速度分布直方图
- [ ] `src/ui/stats.rs` — Tab 2 字符分析
  - [ ] 字符表格（字符 / 速度 / 准确率 / 样本数 / 状态 emoji）
  - [ ] 选中字符的速度变化曲线
- [ ] `src/ui/stats.rs` — Tab 3 类别掌握度
  - [ ] 各类别进度条 + 百分比 + 状态 emoji
- [ ] `src/ui/stats.rs` — Tab 4 练习日历
  - [ ] 月历网格（▓/░ 热力图）
  - [ ] 累计/本月/连续天数统计

### P5 调试检查
- [ ] 练习一轮后统计数据正确更新
- [ ] stats.json 文件格式正确可手动验证
- [ ] 重启程序后历史数据正确加载
- [ ] sparkline 在不同终端宽度下自适应
- [ ] 0 条历史数据时统计面板不 panic
- [ ] 字符表格按准确率排序正确

---

## Phase 6: 本轮结果页
- [ ] `src/ui/mod.rs` — RoundResult 状态定义
- [ ] `src/ui/typing.rs` / `learn.rs` / `dictation.rs` — 完成后跳转 RoundResult
- [ ] RoundResult 页面渲染
  - [ ] 本轮 WPM / CPM / 准确率 / 耗时
  - [ ] 错误字符 Top 5（闪红色）
  - [ ] 与历史平均对比（↑↓ 箭头）
- [ ] [Enter] 下一题 / [R] 重练 / [Esc] 返回主菜单
- [ ] 数据自动保存（调用 progress.append_record + scorer.update_stats）

### P6 调试检查
- [ ] 从对着打完成自动跳转正确
- [ ] 从默写完成自动跳转正确
- [ ] Enter 进入下一题时题目正确切换
- [ ] R 重练时引擎状态正确重置

---

## Phase 7: 题库扩充 + UI 打磨
- [ ] `data/commands/02_basic.toml` — 基础题库
  - [ ] chmod 系列 (chmod 755, chmod u+x, chmod -R)
  - [ ] grep 系列 (grep, grep -r, grep -i, grep -n)
  - [ ] find 系列 (find . -name, find -type, find -mtime)
  - [ ] tar 系列 (tar czf, tar xzf, tar tf)
  - [ ] ps, kill, top, bg, fg, jobs
- [ ] `data/commands/03_advanced.toml` — 进阶题库
  - [ ] awk 系列 (awk '{print}', awk -F, awk 'NR==')
  - [ ] sed 系列 (sed 's///', sed -i, sed -n)
  - [ ] xargs 系列
  - [ ] 管道组合 (grep | sort | uniq -c | sort -rn)
  - [ ] 重定向 (>, >>, 2>&1, tee)
- [ ] `data/commands/04_practical.toml` — 实战题库
  - [ ] 运维场景命令链
  - [ ] 日志分析组合
  - [ ] 系统诊断命令
- [ ] 题库总量验证 ≥ 50 命令 / ≥ 120 变体
- [ ] UI 打磨
  - [ ] ASCII art 标题 ("cmdtyper")
  - [ ] 主菜单过渡效果
  - [ ] 统一颜色主题
- [ ] README.md 编写
  - [ ] 项目介绍 + 截图 / 录屏 GIF
  - [ ] 安装方法 (cargo install / docker)
  - [ ] 使用说明
  - [ ] 题库贡献指南

### P7 调试检查
- [ ] 新增题库 TOML 格式全部通过校验
- [ ] 全部命令 tokens 拼接一致性 100%
- [ ] 每个难度级别至少 10 条命令

---

## Phase 8: 集成调试 + 发布
- [ ] 全流程走通：主菜单 → 各模式 → 完成 → 结果 → 统计
- [ ] 模式间自由切换无状态泄漏
- [ ] 极端输入测试
  - [ ] 空命令处理
  - [ ] 超长命令（>200 字符）
  - [ ] 连续快速输入（打字速度 > 100 WPM）
  - [ ] 非 ASCII 输入（中文、emoji）过滤
- [ ] 终端兼容性
  - [ ] xterm-256color
  - [ ] tmux / screen
  - [ ] 小窗口 (80x24 最小尺寸)
  - [ ] 大窗口 (200+ 列)
- [ ] 中文渲染
  - [ ] unicode-width 宽字符对齐验证
  - [ ] token 注释 + 底栏中英混排对齐
- [ ] 持久化鲁棒性
  - [ ] stats.json 损坏 → 优雅降级（重置统计，不 panic）
  - [ ] 磁盘满 → 错误提示
  - [ ] 并发实例写入安全（file lock 或提示）
- [ ] Docker 测试
  - [ ] `docker compose build` 成功
  - [ ] `docker compose run --rm cmdtyper` 可交互
  - [ ] 数据卷持久化验证
- [ ] 性能测试
  - [ ] 500+ 命令题库加载 < 100ms
  - [ ] 10000+ 历史记录加载 < 500ms
  - [ ] 渲染帧率稳定 ≥ 20 FPS
- [ ] `cargo clippy` 无 warning
- [ ] `cargo test` 全部通过
- [ ] `cargo build --release` 无错误
- [ ] Git tag v0.1.0
- [ ] CHANGELOG.md
