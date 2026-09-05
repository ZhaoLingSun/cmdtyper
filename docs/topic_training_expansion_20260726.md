# cmdtyper v0.3 专题训练扩充实现总结

- 日期：2026-07-26
- 状态：内容、专题训练、质量门禁、安装与 Docker/TUI 包装验收全部完成
- 题库审计：[`question_bank_expansion_20260726.md`](question_bank_expansion_20260726.md)
- 前期设计评审：[`review/design_review_20260726.md`](../review/design_review_20260726.md)
- 外部学习方案调研：[`linux_learning_landscape_20260726.md`](linux_learning_landscape_20260726.md)

## 1. 背景与范围裁决

前期评审指出，cmdtyper 虽然已经拥有打字引擎、命令 lesson、符号专题、系统专题和独立默写，但这些能力分散在不同入口中；新手从“看着完整命令输入”跳到“只看中文提示默写”时，中间缺少可控的回忆台阶。评审还发现，旧的 ReviewTopics 入口使用硬编码分类，并没有与真实内容专题形成同一条数据链。

本轮没有照搬评审中建议的完整 L1–L5 与间隔调度方案，而是收敛到当前可验证的三档产品：

- L1：完整命令跟打；
- L3：遮住一个关键 token 的填空；
- L5：只给中文任务提示的独立默写。

L2 渐隐跟打、L4 分层提示、错题到期与间隔重复明确顺延。这样既补上了最重要的中间台阶，又避免把尚未完成的调度算法写成用户承诺。

内容侧同时完成了 16 个命令专题、44 个新 lesson、2 个符号专题、5 个系统专题，以及终端会话恢复专题。所有新增能力继续遵守项目的硬边界：**应用只展示、输入和判分，不执行任何真实 shell 命令。**

## 2. 最终规模

| 指标 | 最终值 |
|---|---:|
| 命令文件 | **35** |
| 规范命令 | **554** |
| 数据驱动命令专题 | **16** |
| 专题映射命令 | **283** |
| Lesson 文件 | **75** |
| 新增 v0.3 Lesson | **44** |
| Lesson 链接覆盖 | **301/554（54%）** |
| 符号专题 / 练习 | **8 / 120** |
| 系统专题 / 章节 | **11 / 52** |
| canonical command 重复组 | **0** |
| Rust 最终测试基线 | **178 passed / 0 failed** |

554 条命令由 01–19 号文件中的 271 条非专题命令和 20–35 号文件中的 283 条专题命令组成。相对最初 273 条基线，最终净增加 281 条；差额来自两条既有 tmux 命令从旧文件迁入第 35 专题，而不是在两处重复保留。

## 3. 数据驱动架构

### 3.1 `[meta.topic]` 让专题跟随内容文件

每个专题命令文件在既有 `[meta]` 下增加一个可选 `[meta.topic]`：

```toml
[meta]
category = "process"
difficulty = "practical"
description = "tmux 会话观察、终端关系诊断与 reptyr 任务接管的安全恢复流程"

[meta.topic]
id = "terminal_session_recovery"
title = "终端会话与任务接管"
order = 16
```

加载器仍把所有命令汇总为扁平规范题库，同时为带 `[meta.topic]` 的文件生成专题记录。专题记录保存：

- 稳定专题 ID；
- 标题、图标、顺序与简介；
- 文件级难度和类别；
- 该文件内全部规范 `command_id`。

因此专题列表、显示顺序、命令数量和训练题源来自同一份 TOML，而不是 UI 中的一组硬编码名称和 handler 中的另一组整数映射。专题 ID 和 `order` 重复、标题为空或专题没有命令都会在加载阶段失败。

### 3.2 `command_id` hydration 消除复制漂移

Lesson 示例、符号示例/练习和系统命令可以只保存稳定 `command_id`，无需重复粘贴完整命令数据。应用加载顺序为：

1. 读取规范命令 catalog；
2. 读取 lesson、symbol、system 内容；
3. 按 `command_id` 查找规范命令；
4. 补全命令串、摘要、提示、答案、模拟输出、token 解释等字段；
5. 若引用内容显式填写了与规范值冲突的字段，则拒绝启动，而不是静默覆盖。

这条 hydration 链带来三项直接收益：

- 一条规范命令只在 `data/commands/` 维护；
- lesson、符号和系统内容能共享相同答案与模拟输出；
- 内容审计可以验证引用是否悬空、重复或与规范命令不一致。

44 个新增 lesson 的全部 283 个示例引用与 283 条专题命令恰好一一对应。

## 4. 16 个命令专题

| 顺序 | 专题 ID | 中文标题 | 难度 | 命令数 |
|---:|---|---|---|---:|
| 1 | `help_rescue` | 求助自救与终端急救 | Beginner | 19 |
| 2 | `apt_workflow` | APT 软件管理工作流 | Basic | 18 |
| 3 | `tar_zip` | 打包压缩与解压 | Basic | 19 |
| 4 | `fileops_safety` | 文件操作与安全边界 | Beginner | 18 |
| 5 | `redirect_pipe` | 重定向与管道基础 | Basic | 18 |
| 6 | `env_shell` | 环境变量与 Shell 配置 | Basic | 17 |
| 7 | `find_grep` | find 与 grep 搜索 | Basic | 18 |
| 8 | `disk_space` | 磁盘空间排查 | Basic | 18 |
| 9 | `process_port` | 进程与端口排查 | Basic | 16 |
| 10 | `ssh_remote` | SSH 远程与文件传输 | Basic | 17 |
| 11 | `systemd_cron` | systemd 服务与定时任务 | Basic | 20 |
| 12 | `vim_survival` | Vim 生存与编辑器 | Beginner | 18 |
| 13 | `users_permission` | 用户、组与最小权限 | Basic | 17 |
| 14 | `text_toolkit` | 文本处理工具箱 | Basic | 18 |
| 15 | `zh_locale` | 中文环境与 Locale | Beginner | 18 |
| 16 | `terminal_session_recovery` | 终端会话与任务接管 | Practical | 14 |
|  |  | **合计** |  | **283** |

学习中心入口统一称为“**专题训练**”。专题列表显示难度、命令数和已练覆盖，并在空间允许时展示文件级简介。

## 5. L1 / L3 / L5 用户体验

### 5.1 公共会话规则

用户选择专题后进入训练摘要页，可以用 `←` / `→` 或 `h` / `l` 在 L1、L3、L5 之间切换。每轮最多 10 题，按该命令的历史练习次数从少到多选择；次数相同时保持题库顺序。当前策略是“优先覆盖练得少的命令”，不是间隔重复调度。

退出练习会回到当前专题摘要；专题 ID 与级别会写入恢复状态。完成后按级别展示相应统计。

### 5.2 L1：完整输入

L1 显示完整目标命令，复用现有逐字符打字引擎：

- 正确字符、当前光标和待输入字符分色；
- 支持 Backspace；
- 完成后记录准确率与 WPM；
- 按 Enter 展示预设模拟输出，再进入下一题。

L1 的目标是认识结构和形成完整输入动作，不声称测量命令回忆能力。

### 5.3 L3：单 token 填空

L3 使用规范命令的 token 列表建立骨架，只把一个 token 替换为 `____`。选择优先级倾向于：

1. 子命令；
2. 短/长选项；
3. 管道与操作符；
4. 重定向；
5. 模式、路径和参数；
6. 最后才考虑命令名。

用户只输入缺失 token，而不是重新输入整条命令。答案按 `trim` 后精确比较，严格区分大小写。对于没有 token 元数据的内联练习，回退为遮住完整命令，但规范专题命令均有 token 数据。

L3 是当前唯一的中间脚手架，明确不是 L2 渐隐显示，也不是多空 cloze。

### 5.4 L5：独立默写

L5 不显示命令骨架，只显示规范命令的中文任务提示。提交时使用该命令完整的 `answers` 集合判定：

- 精确答案优先；
- 允许首尾空白和连续空白差异；
- 命令、选项、变量和路径仍区分大小写；
- 错误时显示最接近答案和字符 diff。

L5 没有分层提示按钮、自动降级或到期安排，因此文档只称“独立默写”，不称“智能记忆调度”。

## 6. 44 个 Lesson 批次

新增 lesson 用 16 个批次覆盖所有专题命令：

| 批次 | Lesson 数 | 主要内容 |
|---:|---:|---|
| 20 | 3 | 手册求助、命令发现、历史与作业急救 |
| 21 | 3 | APT 索引/搜索、安装/升级、dpkg 修复与清理 |
| 22 | 3 | tar 创建/查看、安全提取、zip/gzip/xz |
| 23 | 3 | 文件创建复制移动、删除前盘点、链接与元数据 |
| 24 | 3 | 标准流、管道/tee、流程控制 |
| 25 | 3 | 环境变量、PATH 诊断、alias/source 与 Shell 配置 |
| 26 | 3 | find 名称/类型、grep 上下文/正则、locate 与命令发现 |
| 27 | 3 | 容量/inode、du 排查、块设备/挂载身份 |
| 28 | 2 | 进程观察、端口/信号/作业 |
| 29 | 3 | SSH 主机与密钥、SCP/rsync、网络与下载 |
| 30 | 3 | systemd 服务、journal 日志、cron 调度 |
| 31 | 3 | Vim 导航编辑、恢复/只读、nano 生存 |
| 32 | 3 | 用户身份/组、权限所有权、sudo 审计 |
| 33 | 3 | 字段/排序/去重、计数/head/tail、连接与变换 |
| 34 | 2 | Locale 变量/编码、日期/时区/APT 源观察 |
| 35 | 1 | tmux、TTY、会话关系与谨慎 reptyr 接管 |
|  | **44** |  |

每课围绕一个可讲清楚的任务簇组织，而不是简单把一个文件中的 18–20 条命令塞进单个超长页面。示例通过 `command_id` 共享规范 token 与输出，gotcha 则补充覆盖风险、权限边界、版本差异和停止条件。

## 7. 符号与系统专题扩充

### 7.1 新增符号专题

既有 6 个符号专题扩展为 8 个，共 120 道练习。本轮新增：

| 专题 | 练习数 | 重点 |
|---|---:|---|
| 条件测试与流程判断 | 15 | `test`、`[ ]`、`[[ ]]`、文件/字符串/整数测试、`&&`、`||` |
| 分组、替换与展开顺序 | 15 | `( )`、`{ ; }`、`$( )`、`${...}`、花括号展开、引用与展开顺序 |

新增内容同时区分 POSIX `[ ]` 与 Bash/Ksh `[[ ]]`，解释短路链不是完整事务，也说明子 Shell 与当前 Shell 分组对环境状态的不同影响。

### 7.2 新增系统专题

既有 6 个系统专题扩展为 11 个，共 52 节。本轮新增 5 个专题，每个 4 节：

| 专题 | 章节数 | 重点 |
|---|---:|---|
| 终端、Shell、TTY 与会话恢复 | 4 | PID/PPID/SID/TTY、tmux、reptyr 安全、断线恢复场景 |
| 标准流、重定向与退出状态 | 4 | fd 0/1/2、重定向顺序、管道、退出状态 |
| 存储、文件系统与挂载 | 4 | 块设备、文件系统、挂载树、容量与 inode |
| 用户、组、sudo 与登录会话 | 4 | UID/GID、补充组、sudo 边界、登录与进程上下文 |
| Locale、时间与字符编码 | 4 | LANG/LC_*、UTF-8、时区、乱码与时间偏差排查 |

每个系统专题都包含“概念层 → 观察命令 → 多步诊断场景”，并反复声明命令和输出是静态教材。

## 8. tmux / reptyr 安全设计

终端会话恢复专题采用“预防优先、事后接管谨慎”的结构。

### 8.1 tmux 路径

推荐在长期任务开始前使用 tmux。恢复时按以下层次观察：

1. `list-sessions` 核对当前用户的会话名和 attached/detached 状态；
2. `list-windows` 检查窗口；
3. `list-panes -F ...` 核对 pane ID、PID、TTY 和当前命令；
4. `capture-pane` 只读恢复最近输出；
5. 确认无误后 attach 或 detach。

`detach-client` 只断开客户端视图；`kill-session` 会结束目标会话及全部窗格。两者在 lesson 中明确区分，后者要求所有权、任务状态、其他连接者和输出保存情况都已核实。

### 8.2 reptyr 路径

reptyr 被定位为裸进程遗留后的补救工具，而不是 tmux 的等价替代。普通接管前要求依次确认：

- 当前 TTY；
- 目标 USER、PID、PPID、SID、TTY、STAT 与完整命令行；
- PID 没有被复用；
- 目标属于当前同一用户且获得授权；
- reptyr 来源可信并可从 PATH 找到；
- Yama `ptrace_scope` 和其他安全策略是否允许附加。

失败时应停止并保留诊断，不写 `/proc`、不降低 Yama、不随手 sudo，也不扩大 capability。

### 8.3 明确排除 `reptyr -T`

`reptyr -T` 影响的是整条控制 TTY，范围可能包含 Shell、前台任务和其他相关进程。本轮只在说明文字中解释风险，并提供只读的 `ps -t TTY -o ...` 范围盘点；**题库没有可执行的 `reptyr -T` 命令记录，也没有把它作为普通模式失败后的升级答案。**

## 9. 进度与恢复兼容性

### 9.1 稳定进度键

规范命令继续使用 `Command.id` 作为进度主键。引用规范命令的 lesson、symbol、system 和专题训练都尽量复用同一 `command_id`，所以用户在不同入口练同一条规范命令时不会生成互相无关的进度身份。

新增内联内容若不属于主命令库，则使用显式稳定 ID 和带命名空间的进度键。旧 lesson 中没有示例 ID 的内联记录保留历史 lesson key，避免升级后让旧进度失联。

### 9.2 ResumeState 的宽容演进

专题训练恢复状态保存：

- `review_topic_id`：稳定专题 ID；
- `topic_training_level`：L1、L3 或 L5；
- 旧有索引字段作为兼容回退。

新增字段使用 serde 默认值：旧 resume JSON 缺少这些字段时仍能读取，训练级别默认 L1。恢复时优先按稳定 ID 找专题；若内容被重排，索引变化不会把用户带到另一个专题。无法解析或已删除的 ID 会回退到安全上级，而不是索引越界。

旧 stats/history/resume 文件继续由兼容测试覆盖；本轮没有新增会让旧 JSON 因未知枚举值整体失效的 L2/L4 记录模式。

## 10. 安装与包装策略

发布数据只包含运行时真正加载的四类目录：

- `data/commands/`
- `data/lessons/`
- `data/symbols/`
- `data/system/`

`data/reviews/` 仍不是当前运行时题源，因此安装脚本和 Docker 镜像都明确排除它，避免把未发布资产误包装成可用功能。

### 10.1 用户安装

`./scripts/install.sh` 构建 release 二进制，并安装到：

```text
~/.local/bin/cmdtyper
~/.local/share/cmdtyper/data/{commands,lessons,symbols,system}
```

数据目录采用临时 staging、备份和替换流程，失败时尽量恢复旧数据，而不是留下半套安装。

### 10.2 数据目录发现

应用按以下顺序寻找完整数据根目录：

1. `CMDTYPER_DATA_DIR`；
2. `~/.local/share/cmdtyper/data`；
3. `/usr/local/share/cmdtyper/data`；
4. 当前目录下的 `./data`。

候选目录必须同时包含 commands、lessons、symbols、system 四个子目录，避免只找到一部分内容后以空专题继续运行。

### 10.3 Docker

Docker 镜像把四个发布目录复制到 `/usr/local/share/cmdtyper/data`，并设置：

```text
CMDTYPER_DATA_DIR=/usr/local/share/cmdtyper/data
CMDTYPER_USER_DIR=/userdata
```

进度数据与只读课程数据分离。最终使用 `docker build --network=host -t cmdtyper:v0.3 .` 构建成功；镜像内包含 35 个 command 文件、75 个 lesson、8 个 symbol 专题和 11 个 system 专题，不含 `data/reviews/`。Docker TUI 在真实 tmux 终端中显示 `v0.3.0` 并 clean exit。

## 11. 质量审计与测试

### 11.1 内容不变量

当前自动门禁覆盖：

- command、lesson、symbol、system TOML 使用真实 Rust 类型解析；
- 35 个命令文件和 554 条命令数量锁定；
- 16 个专题 ID 与顺序锁定，专题命令总数锁定为 283；
- token 文本拼接严格等于规范命令串；
- `answers[0]` 等于规范命令；
- command ID、专题 ID/顺序、lesson/symbol/system 局部 ID 唯一；
- `command_id` 不悬空，并与规范数据一致；
- 44 个 v0.3 lesson 恰好覆盖 283 条专题命令，每条一次；
- canonical command 重复组为 0；
- 运行时 Rust 代码静态检查不存在 shell/process 执行路径。

### 11.2 Token/Lesson 审计

当前审计结果：

```text
命令：554
token：2003
唯一 token 文本：855
lesson 示例：469
lesson token_details：607
内联命令链接：19
command_id 链接：283
唯一 lesson 覆盖：301/554（54%）
悬空 command_id：0
重复 command_id 引用：0
空泛模板描述：0
```

`python3 scripts/audit_tokens.py` 当前通过。显式 token kind 仍为 0/2003，短描述少于 8 字的记录有 191 条；运行时已有推断逻辑，因此它们是后续内容精修项，不是本轮发布阻断项。

### 11.3 最终测试与包装基线

2026-07-26 最终验收结果：

```text
cargo test --no-fail-fast：178 passed / 0 failed（11 suites）
cargo clippy --all-targets --locked -- -D warnings：0 warnings
规范命令与 accepted answers 的 bash -n：1288 checks / 0 failures
符号、系统与 kill lesson 补充 bash -n：574 checks / 0 failures
git diff --check：通过
最终对抗性内容复核：no P0/P1 content findings remain
```

安装脚本在真实 HOME 完成最终发布，在隔离临时 HOME 中也验证了成功安装、sibling JSON 保留，以及 staging copy、data publish、binary publish、SIGHUP 四类失败回滚。安装后的二进制和四类数据目录与工作树一致，`data/reviews/` 未发布。

已安装 TUI 在真实 tmux 终端中显示 `v0.3.0`，可进入学习中心、16 个专题、“终端会话与任务接管”以及 L1/L3/L5 选择，并 clean exit。Docker 镜像构建、数据清单检查和 TUI clean-exit smoke 同样通过。所有教材命令只做 TOML/字符串审计或 `bash -n` 语法解析，没有被执行。

## 12. 明确顺延的能力

以下能力不属于当前完成范围：

### L2 渐隐跟打

尚未实现 ghost text、按掌握度逐步遮挡或从 token 到字符的渐隐逻辑。L1 完整输入不会自动晋级为 L2。

### L4 分层提示默写

尚未实现提示层级、提示次数折扣、F1/F2 提示/放弃流程或 L5 答错后自动降到 L4。当前 L5 是无辅助独立默写。

### 间隔重复与到期调度

尚未实现 SM-2、FSRS、固定 1/3/7/14 天间隔、`next_due_at`、错题跨会话队列或“今日复习”。当前选题仅按历史练习次数从少到多排序。

这些顺延项仍可参考前期设计评审的方案，但上线前必须重新评估数据模型、统计语义、按键兼容和旧 JSON 兼容性。设计建议见 [`review/design_review_20260726.md`](../review/design_review_20260726.md)，外部机制比较见 [`linux_learning_landscape_20260726.md`](linux_learning_landscape_20260726.md)。

## 13. 来源与许可证策略

本轮内容采用“事实核对与独立表达分离”的方式：

- 命令字符串、选项语义和系统行为属于事实层，通过 Debian manpages、GNU Bash、OpenSSH、systemd、APT、tmux、procfs/Yama 与 reptyr 文档交叉核对；
- 中文 summary、prompt、token 解释、模拟输出、诊断步骤和 gotcha 由项目独立编写；
- 外部课程用于比较学习顺序、题型和新手痛点，不直接复制不兼容许可证正文。

可改写或借鉴内容的优先来源包括：

| 来源 | 许可证 | 策略 |
|---|---|---|
| jaywcjlove/linux-command | MIT | 中文术语、命令覆盖线索与讲解结构参考 |
| cmdchallenge | MIT | 任务式题面、多答案与排障场景机制参考 |
| srsudar/eg | MIT | “具体示例优先”的组织方式参考 |
| cheat/cheatsheets | CC0 | 命令覆盖与速查线索 |
| tldr-pages | CC-BY 4.0 | 高频任务线索；若直接改编必须署名并注明修改 |
| Linux Upskill Challenge | CC BY 4.0 | 课程顺序与任务节奏参考；直接改编须署名 |

ArchWiki/GNU 文档的 GFDL 正文、Ubuntu/Linux Journey 的 CC-BY-SA 正文、GPL 教材、NC/ND 内容和无明确开放许可的站点仅用于事实核对或结构研究，不把文字并入 MIT 仓库。更完整的许可证调查见 [`linux_learning_landscape_20260726.md`](linux_learning_landscape_20260726.md)。

## 14. 最终验收清单

### 已完成

- [x] 35 个命令文件、554 条规范命令可加载
- [x] 16 个 `[meta.topic]` 专题按稳定顺序发现
- [x] 283 条专题命令互不重复并回链规范命令
- [x] 学习中心使用“专题训练”名称
- [x] L1 完整输入可记录准确率与 WPM，并展示预设输出
- [x] L3 只遮一个关键 token，答案区分大小写
- [x] L5 只给中文提示并使用完整 accepted answers
- [x] 文档明确不包含 L2、L4 与间隔调度
- [x] 75 个 lesson 中新增 44 个专题 lesson
- [x] 44 个新 lesson 恰好覆盖 283 条专题命令
- [x] 符号内容扩到 8 个专题、120 题
- [x] 系统内容扩到 11 个专题、52 节
- [x] tmux/reptyr 内容说明所有权、TTY、Yama 与停止边界
- [x] 可执行 `reptyr -T` 练习被明确排除
- [x] canonical command 重复组归零
- [x] Lesson 链接覆盖达到 301/554（54%）
- [x] Token/Lesson 审计通过
- [x] 运行时无真实 shell/process 执行路径
- [x] 旧进度与恢复 JSON 通过默认字段和稳定 ID 保持兼容
- [x] 包装范围只包含 commands、lessons、symbols、system
- [x] Rust edition 2024 的最低工具链要求记录为 Rust 1.88+
- [x] 最终 Rust 测试基线为 178 passed / 0 failed
- [x] 严格 Clippy 与 1862 项 shell 语法解析通过
- [x] `./scripts/install.sh` 成功安装并核对二进制与四个数据目录
- [x] 事务安装的 copy/data/binary/HUP 失败注入均完整回滚
- [x] fresh Bash/Zsh 指向当前安装二进制和数据目录
- [x] Docker build 成功，镜像加载 35/75/8/11 发布清单
- [x] 安装包与镜像均不包含未发布的 `data/reviews/`
- [x] 已安装 TUI 与 Docker TUI 在真实终端中启动并 clean exit

## 15. 结论

本轮把 cmdtyper 从“按功能分散的题库与练习入口”推进到一套可解释的数据链：命令文件声明专题，专题持有稳定命令 ID，lesson/符号/系统内容按 ID 引用并在加载时补全，专题训练再用同一规范命令生成 L1、L3、L5 练习。

最终产品事实是 554 条命令、16 个专题、75 个 lesson、8 个符号专题、11 个系统专题，以及 301/554 的 lesson 链接覆盖。最重要的边界同样清楚：当前只有 L1/L3/L5，没有 L2/L4 和间隔调度；终端恢复内容不会执行命令，`reptyr -T` 也不会作为练习答案出现。Rust、内容审计、事务安装、环境变量、Docker 和真实 TUI 验收现已共同构成 v0.3 扩充训练的发布基线。
