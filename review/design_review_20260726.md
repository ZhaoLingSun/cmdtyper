# cmdtyper v0.3 设计评审与改进建议
- 日期：2026-07-26；面向：项目维护者、内容维护者、测试维护者
- 评审性质：设计与范围裁决，不是实现记录；外部对标：[Linux 入门学习方案对标调研](../docs/linux_learning_landscape_20260726.md)
- 核心约束：继续保持数据驱动，绝不执行真实 shell 命令
> 本文综合内部评审工作流的文档、代码、内容、新手路径、进阶机制、 内容计划、范围裁决与对抗式核查结果。 数字与结论优先采用最后一轮 critique 的复核口径。 外部对标文档只作为课程编排、交互模式、内容来源和许可证依据， 本文不重复其完整调研过程。

## 一、执行摘要

### 1.1 总结论
cmdtyper 的技术方向成立，且已经拥有三个难得的基础资产：
1. 内容由 TOML 驱动，运行时不执行真实命令，安全边界清楚；2. `AppState`、按键分派、只读 UI 渲染三层关系总体规整。
3. 命令、词元、预置输出、lesson、统计与历史记录已经形成可扩展底座。
但当前工作区不能被视为“v0.3 功能基本完成，只差打磨”。 更准确的判断是：
- 现有打字与专题学习主干可用；ReviewTopics 只有入口与旧复习引擎完成了半接线；
- 新 reviews 题库、`review_loader`、帮助页均未进入编译或运行链路；启动恢复映射存在会改变首屏的严重错误；
- Dictation 恢复存在可推导的空数组索引崩溃路径；LearnHub 存在明确的 off-by-one；
- 跟打与完整默写之间没有认知脚手架；当前掌握度更接近“历史最佳打字表现”，不能直接充当记忆调度器；
- 内容数量不少，但新手课程拓扑、提示质量与内容门禁仍不均衡。
因此，v0.3 的主题应收敛为： **先修正确性，再打通最小的五级学习梯度，最后让复习入口诚实可用。**
不建议在 v0.3 同时完成以下所有事项：
- 重写并接入 350-450 道独立 reviews 题库；完整 SM-2/FSRS；
- 大规模状态机重构；符号与系统专题的全量进度体系；
- 10 篇以上 lesson 与百题级扩库；帮助页、持久化性能、统计重构等所有积压项。
这些事项彼此存在 schema、状态与测试依赖，强行塞入 v0.3 会重复本轮 “文件写了但未接线、计划显示完成但产品不可用”的问题。

### 1.2 v0.3 发布门槛
建议把以下六项设为 v0.3 的发布门槛：
1. 修复 Home/ReviewTopics resume 映射与 Dictation resume 崩溃路径；2. 修复 LearnHub 边界、matcher 大小写语义和复习多答案不一致。
3. 将默写改为可筛选、短会话、可重试、可放弃、错题会重现；4. 实现 L1-L5 五级梯度的最小闭环，不再从全文跟打直跳全文默写。
5. ReviewTopics 改用现有主命令库和最小进度模型，不接当前孤儿 reviews 题库；6. 为上述路径补齐状态恢复、行为、兼容性和内容门禁测试。

### 1.3 对孤儿资产的裁决
`src/data/review_loader.rs:1`、`src/ui/help.rs:1` 与以 `data/reviews/commands_basic.toml:1` 为代表的 reviews 题库不能继续停留在“看起来做过”的状态。 本评审建议：
- v0.3 不接当前 `data/reviews/`；v0.3 的复习先复用 `data/commands/` 与稳定 `command_id`；
- 当前 `review_loader` 不应直接接线，后续按统一数据路径与新 schema 重写；`help.rs` 顺延 v0.3.x，正式开发时再接线；
- 发布分支不得把未声明进模块树的 Rust 文件计为已完成功能。
这是对内部方案冲突的明确裁决。

## 二、当前产品与架构总评

### 2.1 产品定位
cmdtyper 不应被定义为单纯的“Linux 命令打字速度工具”。 它更有价值的定位是：
**面向中文 Linux 新手的、安全模拟式、词元级命令学习与回忆训练 TUI。** 这个定位与仓库现状是相容的：
- `Command` 已同时保存命令串、摘要、tokens、默写提示和预置输出， 见 `src/data/models.rs:376`；每个 token 已有文本、中文描述和可选 kind， 见 `src/data/models.rs:410`。
- lesson 已有 overview、syntax、options、examples、gotchas， 见 `src/data/models.rs:445`；所有运行时内容从数据目录加载，不需要执行用户输入， 见 `src/data/command_loader.rs:8`。
外部对标也支持这一路线：成熟学习产品通过逐步减少已给信息， 而不是不断更换题目，来完成“示范到独立”的迁移， 见 `docs/linux_learning_landscape_20260726.md:72`。

### 2.2 当前优点

#### 优点 A：安全边界清楚
产品的模拟输出不是妥协，而是差异化能力。 它允许项目安全呈现：
- `rm`、`chmod`、`sudo` 等危险命令；权限错误、包管理锁、网络失败等报错场景；
- 前一条命令输出成为后一题上下文的情景串；不同发行版或服务器场景的教学输出。
只要继续坚持“输入只判分、输出只读 TOML”， 扩充教学深度不会扩大真实 shell 执行风险。

#### 优点 B：状态机主干清晰
`AppState` 明确枚举屏幕状态，见 `src/app.rs:22`。 按键统一从 `App::handle_key` 分派，见 `src/app.rs:383`。
UI 统一从 `ui::render` 分派，见 `src/ui/mod.rs:22`。 渲染函数普遍只读 `&App`，这一纪律值得保留。
它使 v0.3 可以把五级练习折叠进少量现有屏幕， 而不是为每一级新增一套 AppState、handler 和 renderer。

#### 优点 C：持久化基础可靠
现有进度写入使用临时文件加 rename 的原子替换策略， 见 `src/data/progress.rs:104`。 `CommandProgress` 已有练习次数、最佳速度、最佳准确率、 最近练习时间和 mastery，见 `src/data/models.rs:799`。
`UserStats` 已按命令保存进度，见 `src/data/models.rs:822`。 这意味着五级梯度不需要另建数据库， 只需要为记忆阶段添加少量带默认值的字段。

#### 优点 D：内容底座厚
内部工作流在受审快照上核实了：
- 273 条命令；31 篇 lesson；
- 6 个符号专题、90 道符号练习；6 个系统专题；
- 词元拼接、解析与 ID 唯一性已有门禁。
lesson 的 gotcha，尤其危险操作、权限和平台差异， 是当前内容中最成熟的部分。 预置输出总体也比一般教程的示意文本更可信。
这些资产意味着 v0.3 不需要先追求“更多命令”， 而应优先让已有内容形成学习闭环。

### 2.3 当前缺陷

#### 缺陷 A：产品旅程按功能分区，而不是按学习进程组织
主页把“对着打”“学习中心”“默写”并列， 见 `src/app.rs:472`。 这会把应当连续发生的三个教学阶段变成互不相干的入口。
用户完成一条跟打后，不会自然进入填空或提示默写； 进入默写时，也不会优先看到刚学过或刚错过的命令。

#### 缺陷 B：默写对新手是悬崖
当前 `enter_dictation` 直接复制全部命令， 见 `src/app.rs:721`。 没有难度筛选、类别筛选、会话长度、洗牌、重试或错题重现。
空输入不能提交，见 `src/app.rs:747`。 答错后只能进入下一题，见 `src/app.rs:737`。
内部走查还核实：受审快照中全局第 38 题已经进入 awk 题， 原因是 loader 按文件名排序，见 `src/data/command_loader.rs:18`。

#### 缺陷 C：判分可信度不足
matcher 将输入全部转为小写，见 `src/core/matcher.rs:36`。 因此 `-r` 与 `-R` 可能被当成同一答案。
这不是友好容错，而是 Linux 语义错误。 与之相反，复习默写只传一个 canonical command， 见 `src/flow/review_flow.rs:82`， 又会把主默写接受的合法 answers 变体判错。

#### 缺陷 D：学习内容数量与课程拓扑脱节
内部受审快照中，难度与类别基本形成对角矩阵：
- beginner 只覆盖 file_ops；permission、process、system 等从 basic 才开始；
- network、text_process 多从 advanced 才开始；pipeline 集中在 practical。
这使“beginner”更像文件操作过滤器， 而不是完整的新手第一阶段。 外部教材的共同顺序则是导航、文件操作、求助、 通配符与管道、权限、进程、软件包， 见 `docs/linux_learning_landscape_20260726.md:41`。

#### 缺陷 E：计划状态与代码事实失配
v0.3 计划列出 B1-B5 与 F1-F7， 见 `docs/cmdtyper-v0.3-plan.md:9`。 但计划中的状态不能作为当前完成度依据：
- B3 已实现；ReviewTopics 入口已实现但题源仍是旧逻辑；
- F4/F7 有未跟踪文件，却没有进入模块树；B1/B2/B4 仍可从代码看到未完成路径；
- B5 只能判定“未完成复现与验收”，不能武断写成仍存在。
v0.3 后续必须以可执行验收项，而不是“文件已经生成”， 作为计划状态更新依据。

## 三、已核实的代码、接线与测试现状

### 3.1 ReviewTopics：准确结论是“半接线”
ReviewTopics 不是完全不可达，也不是完整落地。 已接通的部分包括：
1. `AppState::ReviewTopics` 已存在，见 `src/app.rs:46`；2. `handle_key` 已分派到 ReviewTopics handler，见 `src/app.rs:430`。
3. `ui::render` 已分派到 topics renderer，见 `src/ui/mod.rs:52`；4. LearnHub 索引 7 已进入 ReviewTopics，见 `src/app.rs:591`。
5. Enter 已能构造 `ReviewSource` 并进入 Review Summary， 见 `src/app.rs:606`。
尚未接通或接错的部分包括：
1. UI 数组中的 `commands_basic`、`commands_advanced`、`symbols` 这些 ID 没有参与数据加载，见 `src/ui/review.rs:291`；2. 第 0 项实际上硬编码为 `Category::ALL[0]`，即 FileOps， 见 `src/app.rs:608` 与 `src/data/models.rs:244`。
3. 第 1 项 UI 名称准确说是“压缩归档”，不是“命令·进阶”； 但说明文字承诺 awk/sed/tar/ssh/systemctl， 实际硬编码 `Category::ALL[6]`，仍只得到 Archive， 见 `src/ui/review.rs:293` 与 `src/app.rs:614`；4. 第 2 项符号映射本身是通的，不能写成三个专题全部错位； 它取第一个 symbol topic，见 `src/app.rs:619`。
5. `ReviewSource::SystemTopic` 有分支，但当前入口没有构造它， 见 `src/app.rs:82` 与 `src/flow/review_flow.rs:144`。
所以，ReviewTopics 的准确状态是： **屏幕、导航、摘要与旧练习引擎可运行；专题 ID、文案、题源、 独立题库和掌握度调度尚未形成一致链路。**

### 3.2 实际复习题源仍是旧引擎
`build_review_exercises` 仍从内存里的 commands、symbols、systems 即时构造练习，见 `src/flow/review_flow.rs:111`。 命令专题按 Category 过滤主命令库，见 `src/flow/review_flow.rs:114`。
然后题目被洗牌，并把约 30% 就地改成 Dictation， 见 `src/flow/review_flow.rs:169`。 这个比例与命令掌握度、近期错误、练习阶段和到期时间无关。
因此当前“70% 跟打 + 30% 默写”只是随机混合， 不是渐进学习，也不是间隔复习。

### 3.3 `review_loader.rs` 是孤儿文件
`src/data/mod.rs:1` 只声明了七个 data 模块， 没有 `review_loader`。 当前加载器文件也没有运行时调用方。
即使直接声明进模块树，它仍有一个发布级问题： `data_dir()` 使用编译期 `CARGO_MANIFEST_DIR` 拼接 data， 见 `src/data/review_loader.rs:25`。
这绕过了主应用的解析顺序：
1. `CMDTYPER_DATA_DIR`；2. 系统安装路径；
3. 当前目录 data；
主路径实现见 `src/app.rs:224`。 所以当前 loader 不能“加一行 pub mod 就算接线”。
正确做法是让 loader 接收 `&Path`， 由 `App::new` 使用同一个已解析 `data_dir` 传入。

### 3.4 当前 `data/reviews/` 不适合直接接入
对抗式核查确认当前三份 reviews 文件共 1390 题：
- `commands_basic` 418；`commands_advanced` 402；
- `symbols` 570。
更重要的是质量口径：
- 1390 题全部是 typing，0 条 dictation；`command_id` 100% 缺失；
- 222 条 `answer != command`；“执行命令:”前缀 170 条；
- “键入:”前缀 426 条；两类模板合计 596 条，占约 43%，不是 170 条，也不是约三分之二；
- prompt 含完整命令原文的约 600 条，同样约 43%。
文件开头已经注明自动生成，见 `data/reviews/commands_basic.toml:1`。 同一文件中可直接看到提示泄题和展示串/答案错位， 见 `data/reviews/commands_basic.toml:11`。
在这些问题修正前接线，只会把不可用的题从“孤儿数据” 变成“用户可见的坏题”。

### 3.5 `help.rs` 是第二个孤儿文件
`src/ui/help.rs:1` 已有完整渲染函数。 但：
- `src/ui/mod.rs:1` 没有声明 help；`AppState` 没有 Help variant，见 `src/app.rs:22`；
- Home 只有五个索引与对应动作，见 `src/app.rs:460`；没有 `?` 或其他入口；
- 没有 Esc 返回处理。
因此 F4 当前没有进入编译和产品行为。

### 3.6 启动 resume 映射是严重问题
`ResumeScreen::Home` 是默认值，见 `src/data/models.rs:891`。 但是 `apply_resume_state` 将 `Home | ReviewTopics` 统一映射成 `AppState::ReviewTopics`，见 `src/app.rs:322`。
与此同时，`current_resume_state` 没有 ReviewTopics 正向分支， 其他未覆盖状态落回默认 Home，见 `src/app.rs:304`。 净效果是：
- 新用户没有 resume 文件时，默认 Home 被恢复成 ReviewTopics；一些本应保存为默认 Home 的状态，下次也会落到 ReviewTopics；
- 保存与恢复不是互逆映射。
这是首屏级严重错误，优先级高于任何新功能。

### 3.7 Dictation resume 有崩溃路径
应用初始化时 `dictation_commands` 是空 Vec， 见 `src/app.rs:273`。 恢复 Dictation 时只切 `state`，没有调用 `enter_dictation()`， 见 `src/app.rs:355`。
之后输入任意字符再按 Enter，会索引当前命令， 见 `src/app.rs:749`。 在空 Vec 上执行该索引可导致 panic。
恢复逻辑必须恢复会话数据，或明确把不可恢复的进行中会话 降级回安全入口，不能只恢复屏幕枚举。

### 3.8 LearnHub 存在明确 off-by-one
LearnHub renderer 只有 8 项，合法索引是 0-7， 见 `src/ui/learn_hub.rs:7`。 handler 却把最后索引写为 8，见 `src/app.rs:560`。
光标可移动到不可见的第 9 项；此时无高亮，Enter 无动作。 之前评审把“常量更新为 8”当成修复， 见 `review/socrates_review_20260329.md:17`。
这说明边界必须由数据长度推导， 不能继续靠人工复核魔法数字。

### 3.9 其他已核实的接线与行为债
- `ReviewPhase::Practice(usize)` 的 usize 没有被实际使用， 见 `src/app.rs:89` 与 `src/ui/review.rs:64`；复习跟打路径不处理 Backspace，见 `src/flow/review_flow.rs:47`。
- 复习默写只认单一 command，见 `src/flow/review_flow.rs:82`；symbol review 只取 `answers.first()`，见 `src/flow/review_flow.rs:125`。
- ReviewTopics 不支持 j/k，见 `src/app.rs:601`；ReviewTopics Esc 回 Home，而 Review 内部 Esc 回 LearnHub， 见 `src/app.rs:603` 与 `src/flow/review_flow.rs:13`。
- `handle_key` 每次按键 clone 整个 state，见 `src/app.rs:390`；app.rs 内仍有 9 个 handler，而不是 8 个；这是重构债，不是 v0.3 blocker。
- `ReviewSource::SystemTopic`、部分 Review model 仍有死分支或死数据；每次按键都保存 resume，见 `src/main.rs:49`。

### 3.10 原计划 Bug 的当前判断

| 项 | 当前判断 | 证据与处理 |
|---|---|---|
| B1 gotcha 多行 | 未完成 | 仍把完整 content 放进一个 Line，见 `src/ui/command_lesson.rs:128` |
| B2 历史区模拟输出 | 未完成 | `TerminalHistory` 仍未保存 output；按原计划验收，见 `docs/cmdtyper-v0.3-plan.md:58` |
| B3 m/M 拦截 | 已完成 | handler 已限制切换时机；已有相关行为测试 |
| B4 Esc 返回来源 | 未完成 | Typing Esc 仍回 Home，见 `src/flow/typing_flow.rs:52` |
| B5 详解溢出 | 存疑 | renderer 已启用 Wrap，见 `src/ui/typing.rs:149`；必须先复现再定修复 |
不能把 B5 写成“确定仍有 bug”， 也不能因为存在 `Wrap` 就写成“已经修好”。

### 3.11 测试现状与快照边界
内部代码审查在其受审快照上实际运行了：
- `cargo check` 通过；`cargo test` 共 118 项通过；
- parse_all、tokens_consistency、id_uniqueness 等均通过。
但这些“全绿”不能证明 v0.3 新增路径正确：
1. `tests/parse_all.rs:13` 只解析 commands、lessons、symbols、system；2. reviews 没有进入 parse_all。
3. tests 中没有 ReviewTopics 全流程与 resume 恢复用例；4. 正好是未覆盖区域出现了首屏劫持与 Dictation panic。
5. `tests/v03_behavior.rs:145` 仍把总命令数硬编码为 273。
此外，当前工作区在内部工作流之后出现了未跟踪的 `20_*.toml` 到 `34_*.toml` 内容文件；首尾可见 `data/commands/20_beginner_help_rescue.toml:1` 与 `data/commands/34_beginner_zh_locale.toml:1`。 只读统计显示当前目录已是 34 个 command 文件、542 条命令。
这些新增文件没有包含在 critique 的 19 文件、273 题质量审计口径中， 本评审也没有运行会写入 `target/` 的新一轮 Cargo 测试。 因此：
- 273、难度矩阵、192 条短描述等数字均应标注为“受审快照”；当前 542 条内容需要独立重跑解析、ID、token、提示和输出审计；
- 硬编码 273 的测试必须改为基于实际 loader 结果或内容不变量断言。

## 四、跟打到默写的五级进阶梯度

### 4.1 设计原则
五级梯度只改变“给用户多少信息”，不频繁更换命令。 同一条命令从识别、线索回忆逐步走向自由回忆。
这一原则与外部平台的共同模式一致， 见 `docs/linux_learning_landscape_20260726.md:74`。 最适合 cmdtyper 的现成机制是 token cloze、ghost text、 分层提示、错题队列和简化间隔调度， 见 `docs/linux_learning_landscape_20260726.md:78`。

### 4.2 五级定义

| 级别 | 名称 | 屏幕给出的信息 | 用户任务 | 主要目标 |
|---|---|---|---|---|
| L1 | 全显跟打 | 完整命令、逐字符反馈、token 中文解释 | 原样输入完整命令 | 建立结构认识与肌肉记忆 |
| L2 | 渐隐跟打 | 命令名可见，flag/argument 以 ghost text 或遮罩显示 | 仍逐字符输入 | 从视觉识别过渡到主动提取 |
| L3 | 词元填空 | 显示命令骨架，只挖 1 个，再挖 2-3 个关键 token | 输入缺失 token 或完整串 | 强线索回忆 |
| L4 | 提示默写 | 只给中文任务；允许逐级揭示结构、命令名、关键 flag | 行编辑后提交 | 弱线索自由回忆 |
| L5 | 完整默写 | 只给唯一指向的任务提示 | 独立输入完整命令 | 无辅助自由回忆 |
“混合复习”不是第六级。 它是一个调度层：根据每条命令当前 stage， 在 L1-L5 中选择合适题型。

### 4.3 L1：全显跟打
L1 复用现有 TypingEngine，不需要新引擎。 应保留：
- 逐字符正确/错误反馈；Backspace；
- token 解释；预置输出；
- M 显示档位；练习完成后的准确率与速度。
需要修正：
- H 只在开始前或完成后生效，见 `src/flow/typing_flow.rs:72`；打字中按 h 会被当成输入字符；
- 底栏只显示 `[H]`，见 `src/ui/typing.rs:566`。
v0.3 不应再用普通字母键抢占正在输入的命令。 提示功能应改用不与 ASCII 命令字符冲突的按键， 并在底栏完整标注动作。

### 4.4 L2：渐隐跟打
L2 保持 TypingEngine 的目标串和判定完全不变。 只在渲染层改变未输入部分：
- 命令名保持可见；flag 和参数使用暗色 ghost text；
- 达到晋级条件后，ghost text 进一步变为等宽遮罩；已正确输入的字符立即显现；
- 连续多次错误时，可临时揭示当前 token，但记为使用帮助。
遮罩区间应由 `Command.tokens` 的拼接位置计算。 token 拼接等于 command 已有专门测试， 见 `tests/tokens_consistency.rs:9`。
不建议在 v0.3 给 `RecordMode` 增加 Cloze enum variant， 因为旧版本读取新 enum variant 可能把 history 当成损坏文件。 更稳妥的办法是在 SessionRecord 增加可选数字字段， 详见第七节。

### 4.5 L3：词元填空
L3 复用 tokens 建骨架，例如：

```text
tar ____ logs.tar.gz
```

第一次只挖一个高信息量 token：
- flag；子命令；
- 路径或模式参数；管道右侧关键命令。
第二次可挖 2-3 个 token。 不得按字符随机挖空，否则会把 Linux 语义单位切碎。
默认 mask 策略可使用 `TokenKind` 与文本启发式， 但内容作者应可通过可选字段覆盖高歧义题。 外部对标中，Linux Journey quiz 与 navi 变量槽都证明 token 级填空是成本最低的中间台阶， 见 `docs/linux_learning_landscape_20260726.md:80`。

### 4.6 L4：提示默写
L4 使用现有 Dictation 式行编辑，不做逐字符即时判错。 初始只显示中文任务描述。
用户可逐级揭示：
1. 命令结构，如“命令 + 组合选项 + 路径”；2. 命令名或首字母骨架；
3. 关键 flag 与对应 token 中文解释。
按键建议：
- `F1`：展开下一层提示；`F2`：显示答案并记为放弃；
- `Enter`：提交；`Backspace`：编辑；
- 结果页 `r`：重试当前题；`Esc`：返回来源页。
明确不要使用 `Ctrl+H`： 多数终端会把它编码成 Backspace，无法可靠区分。
也不要依赖 `Ctrl+S`： 它可能被终端 IXON 流控或复用器吞掉。
这是对内部进阶方案的关键更正。

### 4.7 L5：完整默写
L5 只显示经过审校、唯一指向的中文任务。 判分必须：
- 大小写敏感；使用完整 `answers` 集；
- 空白可做有限归一；exact 与明确列出的等价写法可判对；
- 不在 v0.3 自动推断复杂 shell 等价性；不执行命令验证输出。
v0.3 先修“大小写”和“多答案统一”两个正确性问题。 token 级部分得分可作为后续反馈增强， 但不应在 v0.3 直接改写 `SessionRecord.accuracy` 的既有统计语义。
可新增独立 `recall_score`，避免历史准确率、mastery、 推荐排序一起发生隐式语义迁移。

### 4.8 输入法与中文新手
题面是中文、答案是 ASCII，中文输入法切换是实际学习成本。 v0.3 至少需要以下防护：
- 行编辑接受 Unicode 时不 panic；非 ASCII 输入不能破坏 cursor 或 diff；
- 输入区明确保持等宽；提交非 ASCII 答案时给出温和、可恢复的反馈；
- 不在输入过程中拦截普通 `h`、`r`、`s` 等字母；`r` 重试只在结果状态生效。
输入法问题不需要应用控制系统 IME， 但必须被纳入手工验收，而不是假设所有用户始终处于英文输入状态。

### 4.9 最小掌握度模型
现有 mastery 公式是： `best_accuracy * min(times / target, 1)`， 见 `src/core/scorer.rs:124`。
它有三个局限：
- 使用历史最佳准确率；随练习次数单调上升并最终饱和；
- 没有时间衰减、到期日或失败次数。
因此 v0.3 不应直接重定义现有 `mastery` 字段。 建议在 `CommandProgress` 增加最小记忆状态：

```rust
#[serde(default)] pub learning_stage: u8,        // 0..=4 => L1..L5
#[serde(default)] pub stage_streak: u8,         // 当前级连续通过次数
#[serde(default)] pub recent_recall: f64,       // EWMA，独立于 best_accuracy
#[serde(default)] pub lapse_count: u32,         // 发生降级的次数
#[serde(default)] pub next_due_at: Option<i64>, // 简化复习到期时间
```

五个字段已经足够支持最小闭环。 不要在 v0.3 同时加入 ease、FSRS stability、difficulty、 retrievability 等完整算法字段。

### 4.10 评分与 mastery 计算
每次练习产生独立的 `recall_score`：

| 结果 | recall_score |
|---|---:|
| L1-L3 首次完成且准确率 >= 95% | 1.0 |
| L1-L3 完成且准确率 90%-95% | 0.8 |
| L4/L5 无提示命中任一合法答案 | 1.0 |
| L4 使用一级提示后答对 | 0.8 |
| L4 使用二级提示后答对 | 0.6 |
| L4 使用三级提示后答对 | 0.4 |
| 显示答案、跳过或答错 | 0.0 |
近期回忆采用简单 EWMA：

```text
recent_recall = 0.7 * old + 0.3 * recall_score
```

首次记录直接取本次 `recall_score`。 展示或排序用的记忆掌握度可临时派生：

```text
recall_mastery = 0.6 * (learning_stage / 4) + 0.4 * recent_recall
```

这个派生值不落盘，也不替换现有 mastery。

### 4.11 晋级规则
满足以下条件时晋一级：
1. 当前级连续两次 `recall_score >= 0.8`；2. 第二次通过后 `recent_recall >= 0.8`；
3. 当前级不是 L5；4. 两次记录不能来自同一次“显示答案后重输”。
晋级后：
- `learning_stage += 1`；`stage_streak = 0`；
- 保留 `recent_recall`；下次正常练习使用新级别。
必须保留“手动挑战下一等级”的入口。 手动挑战失败不立即降级，避免出现 keybr 式长期卡关， 相关外部教训见 `docs/linux_learning_landscape_20260726.md:93`。

### 4.12 降级规则
一次答错不立即持久化降级。 先做会话内降压：
- L5 答错，当前题重排为 L4；L4 答错，轮尾重排为 L3；
- L3 答错，轮尾重排为 L2；L2/L1 答错，保持本级并显示必要解释。
满足以下任一条件才持久化降一级：
1. 同一级连续两次 `recall_score < 0.6`；2. 到期复习中答错两次；
3. `recent_recall < 0.5` 且本次失败。
降级后：
- `learning_stage = learning_stage.saturating_sub(1)`；`stage_streak = 0`；
- `lapse_count += 1`；`next_due_at` 设为次日。
这里的“L5 错后变 L4”仍使用行编辑， 不会在 Dictation 屏临时嵌入 TypingEngine。 L4 到 L3 的转换放到统一 Practice/Review 会话轮尾， 利用 Review 屏本来就有的 typing 与 dictation 双路径， 避免复制一套复杂 handler。

### 4.13 错题复现最小模型
错题复现分两层。

#### 会话内
每轮维护一个去重的错题队列：
- 首次答错立即进入队列；正常题完成后，错题在轮尾再出现一次；
- 重现时自动降低一级；第二次仍错则展示答案并结束本题，不无限循环；
- 一轮最多 10-15 个原始题，避免错题队列无限膨胀。

#### 跨会话
只用固定间隔，不上完整 SM-2：

```text
通过：1 天 -> 3 天 -> 7 天 -> 14 天
失败：次日
```

`next_due_at <= now` 的命令进入“今日复习”。 `lapse_count > 0 && recent_recall < 0.8` 的命令进入“错题重练”。
调度层从命令当前 `learning_stage` 选题型， 不再随机把 30% 题目变为 Dictation。

## 五、v0.3 必做与 v0.3.x 顺延

### 5.1 范围裁决原则
本节解决内部设计方案的两个主要冲突：
1. reviews 题库是 v0.3 接线，还是顺延；2. 掌握度字段是 v0.3 落地，还是全部顺延。
裁决如下：
- 当前 reviews 题库与 loader 顺延；五级梯度所需的最小五字段进度模型留在 v0.3；
- 完整 SRS、独立 review schema 与大题库留到 v0.3.x。
这样既不切断梯度的持久化前置， 也不把高风险题库重写塞进同一个发布周期。

### 5.2 v0.3 必做

#### A. P0 正确性修复
1. Home resume 必须恢复到 Home；2. ReviewTopics 必须有正向 save 与独立 restore 映射。
3. Dictation 恢复必须重建会话，或安全回到 Dictation 入口；4. LearnHub 边界由可见项目数量推导。
5. matcher 不再全局小写化；6. Review command/symbol 默写统一使用完整 answers。
7. 复习跟打补 Backspace。

#### B. 默写体验底线
1. 进入默写前可选难度与类别；2. 每轮默认 10 题，可配置 5/10/15。
3. 会话内洗牌，但测试可注入确定性 RNG；4. 允许空输入触发“不会/看答案”。
5. 提供重试、显示答案、错题轮尾复现；6. Esc 返回来源屏，不丢失已完成记录。
7. 结果反馈区分大小写错误、缺 token 和多余 token。

#### C. 五级梯度最小闭环
1. L1 复用现有跟打；2. L2 实现渲染遮罩与 ghost text。
3. L3 实现 token 级 cloze；4. L4 实现 F1 分层提示与 F2 放弃。
5. L5 使用大小写敏感、多答案 matcher；6. 落地五字段 CommandProgress 扩展。
7. 落地晋级、降级与会话内错题队列；8. 落地固定 1/3/7/14 天的最小到期调度。

#### D. ReviewTopics 诚实化
v0.3 不读 `data/reviews/`。 ReviewTopics 建议改为：
1. 今日复习；2. 错题重练；
3. 分类练习；4. 符号专题。
前两项来自 `CommandProgress`。 第三项复用主命令库 Category。
第四项复用现有 symbol topics。 菜单项、边界和 source 必须来自同一描述数组， 不能再由 UI 一套 ID、handler 另一套整数映射。

#### E. 原计划的短平快体验修复
1. B1 gotcha 按真实换行拆成多个 Line；2. B2 历史区保存并渲染预置输出。
3. B4 Typing 保存明确 return destination；4. B5 先用窄终端复现；只有复现失败时才关闭。
5. F3 底栏显示完整动作名称，不只显示 `[H]`。

#### F. 内容质量门禁与小批内容
1. reviews 暂不接入运行时，但必须明确标记为未发布资产；2. 新增内容先通过 parse、ID、token、提示唯一性、泄题与输出自洽检查。
3. 优先审校 12-20 个核心命令，不以总题数作为 v0.3 成功指标；4. 优先补“求助/自救、apt、tar、权限、进程、网络”新手路径。
5. 修复已知 docker format 与 ping 输出问题。

### 5.3 v0.3.x 顺延
以下事项明确顺延：
1. 当前 `data/reviews/` 清洗、重生成与正式接线；2. 新 review schema 的 350-450 道独立优质题。
3. `review_loader` 重写、专题发现、外部扩展题源；4. 完整 SM-2 或 FSRS。
5. `ease`、`stability`、`retrievability` 等高级调度字段；6. `help.rs` 正式接线与 About/许可证页面。
7. F1 符号/系统专题完整进度体系；8. F2 符号示例额外跟打阶段。
9. matcher 的 flag 重排、引号等价、复杂 shell 等价推断；10. token 级连续部分分数全面接入统计。
11. history JSONL、按月分片与 resume 仅状态变化时保存；12. app.rs 九个 handler 迁移到 flow/。
13. 菜单索引、AppState 参数、ResumeState 的全面单一来源重构；14. lesson scenario/comparison 新字段的全量内容生产。
15. token_details 从 6% 提升至 30% 的大规模工程；16. system 六专题的全量 quiz。

### 5.4 不纳入任何版本的做法
以下做法应明确禁止：
- 通过执行真实 shell 命令判分；未经许可证核查直接复制外部正文；
- 只生成文件、不声明模块、不加入口，却标记功能完成；用硬编码索引模拟数据驱动；
- 为每个学习级别新增独立顶层 AppState；自动推断所有 shell 命令等价性；
- 用题量替代题目质量与学习闭环验收。

## 六、题库与讲解扩充策略

### 6.1 先修拓扑，再堆数量
受审快照已有 273 条命令，当前工作区又出现大批未跟踪扩充。 继续追求题量之前，应先回答：
1. 新手第一小时依次学什么；2. 每个主题的前置是什么；
3. 哪些命令需要完整五级练习；4. 哪些长尾命令只需要讲解和跟打；
5. 哪些题必须有预置输出或报错；6. 哪些题的 prompt 能唯一指向 answers 集合。
外部对标建议保持金字塔：
- 核心 12-20 个命令做深；长尾命令做浅；
- 每个模块有 quiz；
见 `docs/linux_learning_landscape_20260726.md:142`。

### 6.2 建议的新手主线
建议采用以下 12 段主线：
1. shell 与提示符：什么是命令、参数、当前目录；2. 导航：pwd、cd、ls。
3. 文件操作：touch、mkdir、cp、mv、rm、ln；4. 求助与自救：man、--help、history、Tab、type、which。
5. apt 全流程：update、install、remove、search、常见报错；6. tar/zip：先查看、再解压、再打包。
7. 通配符、重定向、管道、tee；8. 权限：ls -l、chmod、chown、sudo、为什么不要 777。
9. PATH、export、source、.bashrc；10. find/grep、df/du、ps/kill、端口排查。
11. vim 生存、ssh/scp、systemctl/journalctl/crontab；12. 报错急救与中文用户专题：换源、乱码、WSL/远程环境。
这与外部调研推荐顺序一致， 见 `docs/linux_learning_landscape_20260726.md:53`。

### 6.3 内容来源与许可证
优先来源：

| 来源 | 许可 | 建议用途 |
|---|---|---|
| jaywcjlove/linux-command | MIT | 中文 lesson、token_details 的底料与术语参考 |
| cmdchallenge | MIT | 60 道任务题、排障情景、多解与 expected_failures 思路 |
| tldr pages | CC-BY 4.0 | 高频示例、任务式默写提示；必须署名并注明修改 |
| Linux Upskill Challenge | CC BY 4.0 | 21 天课程顺序与可改写任务；必须署名 |
| srsudar/eg | MIT | 具体示例优先的 lesson 样式 |
| cheat/cheatsheets | CC0，待再次留证 | 补充速查底料 |
许可证与覆盖说明见 `docs/linux_learning_landscape_20260726.md:12`。 不得直接复制正文的来源包括 GFDL、CC-BY-SA、GPL、NC/ND 和无明确开放许可的站点，见 `docs/linux_learning_landscape_20260726.md:30`。

### 6.4 内容生产管线
建议生产流程：
1. 先从外部允许来源抽取事实、命令和结构；2. 生成 TOML 草稿，不直接合并。
3. 把占位符实例化为安全、连贯的虚拟路径；4. 手写或复核 simulated_output。
5. 逐 token 补中文解释与记忆钩子；6. 编写唯一指向的 dictation.prompt。
7. 枚举有限、明确的 accepted answers；8. 补常见错误答案与针对性解释。
9. 运行结构门禁与内容审计；10. 人工抽审至少 10%，核心命令 100% 审校。
tldr 的格式很适合生成草稿， 但它没有预置输出，不能全自动变成可发布题， 见 `docs/linux_learning_landscape_20260726.md:258`。

### 6.5 单条命令的讲解模板
每条核心命令建议固定为：
1. 场景：为什么现在需要它；2. 第一条可完整输入的命令。
3. 语法骨架与 token 拆解；4. 2-4 个高频示例及可信预置输出。
5. 1-2 个易错直觉与危险提示；6. 等价写法、边界条件与相关命令。
7. 一道 L3 填空、一道 L4/L5 默写。
外部调研建议示例占讲解约一半， 并坚持“具体示例在前、抽象通式在后”， 见 `docs/linux_learning_landscape_20260726.md:97`。

### 6.6 prompt 质量标准
每条默写提示必须满足：
- 场景先行；唯一指向；
- 不直接泄露命令与完整参数；与 answers 集合语义一致；
- 区分易混命令，如 df 与 du；中文白话，但保留英文命令、flag 与必要术语；
- 不出现“执行命令: X”“键入: X”这类伪题。
内部 critique 更正后的基线是：
- 29 条主命令 prompt 含命令首词，不是 28；13 是归一化后冲突的答案字符串数量；
- 按 ID 对计是 8 组冲突；冲突清单不能漏掉 cd home 那一组；
- 受审快照中真正零覆盖的是 exit 与 more；passwd 和 tee 已有条目，不能继续写成零覆盖。
由于当前工作区已有新增内容， 这些数字只能作为历史缺陷基线，必须重新审计后再更新。

### 6.7 内容优先级
P0 内容主题：
1. 求助与自救；2. apt 与 dpkg 报错；
3. tar/unzip/gzip；4. chmod/chown/sudo；
5. 文件操作安全；6. 重定向、管道与 tee；
7. PATH 与 shell 环境；8. find/grep；
9. df/du；10. ps/kill/端口；
11. ssh/scp；12. vim 生存。
这些主题与社区高票任务分布一致， 见 `docs/linux_learning_landscape_20260726.md:122`。 P1 内容主题：
- systemctl/journalctl/crontab；useradd/usermod/groups/passwd；
- sed/cut/sort/uniq/wc；符号与引用；
- 中文换源、locale、乱码；Permission denied、command not found、dpkg lock 等报错卡。
不优先进入打字题库：
- 磁盘分区；RAID/LVM；
- 内核编译；完整开机流程；
- 与命令输入关系弱的发行版选择和驱动问题。

## 七、数据模型、状态机与兼容性落地建议

### 7.1 用统一 PracticeSession 承载五级练习
不要新增 `AppState::GhostTyping`、`AppState::Cloze`、 `AppState::HintedDictation` 等顶层状态。 建议在现有 Typing/Review 会话下引入领域对象：

```rust
pub struct PracticeSession {
    pub source: PracticeSource,
    pub exercises: Vec<PracticeExercise>,
    pub current_index: usize,
    pub wrong_queue: Vec<PracticeExercise>,
    pub return_to: ReturnDestination,
}
```

每道题携带：

```rust
pub struct PracticeExercise {
    pub content_id: String,
    pub level: u8,
    pub command: String,
    pub answers: Vec<String>,
    pub prompt: String,
    pub masked_tokens: Vec<usize>,
    pub hint_level: u8,
}
```

`level` 用 1-5 的数值持久化， 避免旧版本遇到未知 enum variant。 运行时可以转换为内部 enum， 但 history/stats JSON 保持可宽容读取的字段。

### 7.2 稳定 ID 是进度主键
命令直接使用 `Command.id`。 现有 command ID 唯一性门禁见 `tests/id_uniqueness.rs:10`。
symbol/system 练习不能再使用 TOML 顺序索引生成 ID， 因为重排内容会让历史进度指向别的题。 建议：
- symbol exercise 增稳定 `id`；system command 增稳定 `id`；
- review 扩展题必须有独立 `id` 和 `command_id`；已发布 ID 不改名、不复用；
- 删除内容后允许保留孤儿进度键。

### 7.3 ResumeState 必须是可恢复快照，不只是屏幕名
当前问题的根源是“恢复屏幕”与“重建会话”混为一谈。 建议把屏幕分三类：
1. 可直接恢复：Home、LearnHub、Topics、Stats、Settings；2. 可由稳定 ID 重建：lesson、symbol、system 的阅读位置。
3. 不应直接恢复：进行中的 Typing/Dictation/Review 会话， 除非同时保存题目 ID、索引、输入和队列。
v0.3 的保守方案：
- 对第 3 类只保存其安全入口与过滤条件；启动时重新构建一轮；
- 不保存正在输入的半条命令；save/restore 对每个 ResumeScreen 做互逆单测；
- 未知或越界索引回退安全上级，不 panic。

### 7.4 ReviewTopic 必须数据驱动
建议定义统一描述：

```rust
pub struct ReviewTopicDescriptor {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source: ReviewSource,
    pub enabled: bool,
}
```

renderer、导航边界与 Enter handler 共用同一 Vec。 这样可以同时消灭：
- UI ID 无用途；描述与实际 source 错位；
- `.min(2)`；增加菜单项后忘记改边界；
- 空 topic 仍可进入。

### 7.5 Loader 统一接收数据根目录
所有 loader 签名应遵循：

```rust
fn load_xxx(data_dir: &Path) -> Result<Vec<T>>
```

主应用只做一次目录探测。 loader 不得再次读取环境变量， 也不得使用 `env!("CARGO_MANIFEST_DIR")`。
reviews 在 v0.3.x 接线时， 必须沿用 `src/app.rs:224` 的目录选择结果。

### 7.6 stats.json 兼容策略
所有新增 `CommandProgress` 字段使用 `#[serde(default)]`。 建议默认语义：
- `learning_stage = 0`；`stage_streak = 0`；
- `recent_recall = 0.0`；`lapse_count = 0`；
- `next_due_at = None`。
旧 stats.json 加载后等价于“尚未进入新梯度”， 不需要一次性迁移脚本。 已有 `best_accuracy` 与 mastery 不回填新模型， 避免把打字准确率误当命令回忆证据。

### 7.7 history.json 兼容策略
建议给 `SessionRecord` 添加可选字段：

```rust
#[serde(default)] pub learning_level: Option<u8>,
#[serde(default)] pub hint_count: u8,
#[serde(default)] pub recall_score: Option<f64>,
```

保持现有 `RecordMode` variants 不变。 L1-L3 仍可记录为 typing 类模式， L4-L5 仍为 dictation 类模式， 具体阶段由 `learning_level` 区分。
这样：
- 新版本能读旧历史；旧版本通常会忽略未知字段；
- 不会因未知 enum variant 把整份 history 当损坏数据；Stats 可以逐步选择是否展示新字段。

### 7.8 matcher 的渐进改造边界
v0.3：
- 保留 trim 与空白折叠；删除全局 lowercasing；
- exact 优先；answers 列表逐个匹配；
- 大小写仅差异时返回明确错误类型；diff 仍可基于 LCS。
v0.3.x：
- 明确配置短 flag 簇等价；明确配置 flag 顺序是否无关；
- 支持 expected_failures 与针对性反馈；研究引号、变量展开与子命令上下文；
- 不尝试实现完整 shell parser 或执行判等。
matcher 自带测试目前明确断言 lowercase 行为， 见 `src/core/matcher.rs:215`。 修复时必须同步这些单测， 不能只改 integration tests。

### 7.9 当前 mastery 与推荐算法的关系
现有推荐算法把弱字符与低 mastery 合成排序， 见 `src/core/scorer.rs:183`。 五级梯度上线时必须明确：
- 键盘字符弱点用于打字练习排序；`recent_recall` 与 `next_due_at` 用于记忆复习排序；
- 两类分数不相互覆盖；`adaptive_recommend` 关闭时仍允许手动选级；
- 未掌握的稀有 token 要有最低出场频率。
否则“打得慢”和“记不住”会继续被混成一个 mastery。

### 7.10 v0.3.x reviews 新 schema 建议
未来独立题库至少应包含：

```toml
id = "review-ls-all-long"
command_id = "ls-all-long"
kind = "dictation"
prompt = "列出 /var/log 中包含隐藏项的详细信息"
command = "ls -la /var/log"
answers = ["ls -la /var/log", "ls -al /var/log"]
difficulty = "beginner"
```

质量不变量：
- `id` 唯一且稳定；`command_id` 必须回链；
- typing 的目标串与提交答案一致；dictation 的 answers 非空；
- prompt 不泄露完整 command；kind 不能全部是 typing；
- loader 使用统一 data root；parse_all、ID、回链、提示和答案一致性全部纳入测试。

## 八、验收测试清单

### 8.1 P0 启动与恢复
- [ ] 无 resume 文件时启动进入 Home。<br>- [ ] `ResumeScreen::Home` 恢复为 `AppState::Home`。
- [ ] ReviewTopics 保存后恢复为 ReviewTopics，索引不越界。<br>- [ ] LearnHub 保存后恢复为 LearnHub。
- [ ] Dictation resume 不会出现 `1/0` 题目状态。<br>- [ ] Dictation resume 后输入并提交不会 panic。
- [ ] 不可重建的会话安全回到入口页。<br>- [ ] 损坏 resume JSON 回退 Home。
- [ ] 旧 resume JSON 缺新字段仍可加载。<br>- [ ] 每个 ResumeScreen 都有 save/restore 互逆测试。

### 8.2 LearnHub 与菜单边界
- [ ] LearnHub 8 项只能选择索引 0-7。<br>- [ ] 在索引 7 按 Down 不会失去高亮。
- [ ] 菜单为空时 Up/Down/Enter 不 panic。<br>- [ ] ReviewTopics 边界来自 descriptor Vec 长度。
- [ ] Home 边界不再硬编码 4。<br>- [ ] ReviewTopics 支持 Up/Down 与 j/k。
- [ ] ReviewTopics Esc 返回 LearnHub。<br>- [ ] Review 内所有 Summary/Practice/Completed 的 Esc 语义一致。
- [ ] disabled topic 不可进入，并有可测试状态。

### 8.3 ReviewTopics 与题源
- [ ] “今日复习”只包含已到期 command_id。<br>- [ ] “错题重练”只包含符合 lapse/recent_recall 条件的命令。
- [ ] “分类练习”显示的标题与实际 Category 一致。<br>- [ ] 符号专题 source 与选中 topic 一致，而非永远取 first。
- [ ] 无到期题时显示空状态，不进入空练习。<br>- [ ] v0.3 运行时不调用孤儿 review_loader。
- [ ] ReviewSource::SystemTopic 若无入口则删除或明确顺延。<br>- [ ] `ReviewPhase::Practice` 不再携带未使用索引。
- [ ] Review typing 支持 Backspace。<br>- [ ] Review dictation 使用完整 answers。
- [ ] Symbol review 使用完整 answers，而不是 first。

### 8.4 matcher 正确性
- [ ] `ls -R src` 与 `ls -r src` 不等价。<br>- [ ] `grep -H` 与 `grep -h` 不等价。
- [ ] 大小写仅差异返回明确 CaseMismatch。<br>- [ ] 首尾空白与连续空白可归一。
- [ ] exact answer 优先于 normalized answer。<br>- [ ] 主默写和复习对同一 answers 集给出同一结果。
- [ ] answers 为空时返回安全 NoMatch，不 panic。<br>- [ ] 中文 IME 字符输入不破坏 diff。
- [ ] LCS 反馈可重建输入与目标。<br>- [ ] matcher 原有 lowercase 单测已更新为大小写敏感语义。

### 8.5 L1-L3 跟打与遮罩
- [ ] L1 显示完整目标。<br>- [ ] L2 只遮计划中的 flag/argument token。
- [ ] L3 按 token 挖空，不按随机字符挖空。<br>- [ ] token 区间拼接与 UTF-8 字符索引不越界。
- [ ] 已输入字符会从 ghost/遮罩状态显现。<br>- [ ] Backspace 后遮罩恢复正确。
- [ ] Ctrl+R 重置题目、遮罩和提示计数。<br>- [ ] 当前 token 多次错误后的揭示会记录 hint_count。
- [ ] 窄终端下遮罩不改变布局宽度。<br>- [ ] Ratatui TestBackend 快照验证遮罩位置与高亮。

### 8.6 L4-L5 默写交互
- [ ] F1 依次展示三层提示。<br>- [ ] F1 不与 Backspace 冲突。
- [ ] F2 显示答案并记录 recall_score=0。<br>- [ ] 不依赖可能被流控吞掉的 Ctrl+S。
- [ ] 空输入可以明确选择“不会”。<br>- [ ] 结果页 r 才触发重试；输入阶段 r 正常输入字符。
- [ ] 每题开始时 hint_count 清零。<br>- [ ] L5 不显示任何答案片段。
- [ ] L4/L5 都使用完整 accepted answers。<br>- [ ] 10 题会话完成后显示汇总并保存每题记录。

### 8.7 晋级、降级与错题复现
- [ ] 当前级连续两次达标才晋级。<br>- [ ] 一次失败不立即持久化降级。
- [ ] 两次失败触发降级并增加 lapse_count。<br>- [ ] stage 永远限制在 0-4。
- [ ] `recent_recall` 按 EWMA 更新。<br>- [ ] 使用不同提示层得到正确 recall_score 折扣。
- [ ] L5 错题会话内以 L4 重现。<br>- [ ] L4 错题轮尾以 L3 重现。
- [ ] 同一错题在一轮内去重。<br>- [ ] 第二次仍错不会无限循环。
- [ ] 通过后 next_due 按 1/3/7/14 天推进。<br>- [ ] 失败后 next_due 设为次日。
- [ ] 手动挑战下一等级失败不强制降级。<br>- [ ] 固定时钟注入保证调度测试可重复。

### 8.8 数据模型与兼容性
- [ ] 旧 stats.json 缺五个新字段仍可加载。<br>- [ ] 旧 CommandProgress 默认进入 L1。
- [ ] 新 stats.json 保存后重新加载值不丢失。<br>- [ ] 旧 history.json 缺 learning_level/hint_count/recall_score 可加载。
- [ ] 新 history JSON 被旧字段解析器忽略未知字段的行为有夹具验证。<br>- [ ] 不新增会让旧版本拒绝整份历史的 RecordMode variant。
- [ ] 损坏 stats/history 仍走现有安全回退。<br>- [ ] command_id 改名或缺失时不会 panic。
- [ ] symbol/system 使用稳定 ID，不以 Vec 索引持久化。<br>- [ ] 兼容测试覆盖 `CMDTYPER_USER_DIR` 隔离。

### 8.9 数据目录与安装形态
- [ ] `CMDTYPER_DATA_DIR` 优先级正确。<br>- [ ] 系统安装目录可加载 commands/lessons。
- [ ] Docker 路径可加载全部已发布数据。<br>- [ ] loader 不使用构建机 `CARGO_MANIFEST_DIR`。
- [ ] 从非仓库当前目录启动仍能找到安装数据。<br>- [ ] reviews 在 v0.3 未发布时不影响启动。
- [ ] v0.3.x 接线后 reviews 缺失给出可诊断错误或禁用专题。<br>- [ ] 单个 reviews TOML 损坏不会静默变成“0 题成功”。

### 8.10 内容门禁
- [ ] 所有 command TOML 可解析。<br>- [ ] 所有 lesson TOML 可解析。
- [ ] 所有 symbol/system TOML 可解析。<br>- [ ] tokens 拼接严格等于 command。
- [ ] 所有稳定 ID 全局唯一。<br>- [ ] dictation answers 非空。
- [ ] prompt 归一化重复与答案冲突被报告。<br>- [ ] prompt 含完整 command 或命令首词时被报告并支持白名单。
- [ ] typing 题 answer 必须等于可见目标串。<br>- [ ] review command_id 必须回链主内容。
- [ ] 大小写碰撞答案被报告。<br>- [ ] `ping -c N` 输出行数与统计自洽。
- [ ] docker format 模板引号平衡。<br>- [ ] 静默命令允许无输出，但讲解说明如何确认成功。
- [ ] 核心命令人工审校率 100%。<br>- [ ] 外部改写条目标记来源与许可证。

### 8.11 UI 与手工 TUI 验收
- [ ] 80x24 下 Home、LearnHub、ReviewTopics 无重叠。<br>- [ ] 100 列以下 Detailed 模式正确降级或换行。
- [ ] gotcha 内嵌换行逐行显示。<br>- [ ] 历史区模拟输出按真实换行显示。
- [ ] 长 prompt 与长 path 不越界。<br>- [ ] 五级提示不改变固定布局高度。
- [ ] 中文、英文、标点混排不破坏光标位置。<br>- [ ] 中文输入法开启时可恢复到英文输入并继续答题。
- [ ] F1/F2 在常见终端、tmux、ssh 场景可识别。<br>- [ ] Esc 从 Home 来源回 Home，从 LearnHub 来源回 LearnHub。
- [ ] 所有屏幕底栏只显示当前状态实际可用的键。

### 8.12 安全验收
- [ ] 全仓库不存在运行用户 command 字符串的调用。<br>- [ ] 新 matcher 只做字符串解析与比较。
- [ ] 情景题输出全部来自预置数据。<br>- [ ] 外部题库转换脚本只生成草稿，不执行样例命令。
- [ ] 危险命令有显式警告与安全虚拟路径。<br>- [ ] 测试不在真实用户目录写入进度。

## 九、事实核查与存疑点

### 9.1 已核实事实
以下可以作为当前代码事实：
- ReviewTopics 的 state、render、handler、LearnHub 入口已接通；ReviewTopics 的两个命令类项目映射与说明不一致。
- 符号项目映射可用，不能说三个项目都错；实际复习题源仍是 `build_review_exercises`。
- `review_loader.rs` 与 `help.rs` 未进入模块树；review loader 使用 `CARGO_MANIFEST_DIR`。
- Home 默认 resume 被错误恢复为 ReviewTopics；Dictation resume 未重建 commands，存在空 Vec 索引路径。
- LearnHub 的 8 项菜单配了最后索引 8；matcher 全局小写化。
- Review dictation 与 symbol review 没有使用完整 answers；受审快照的 118 项测试全绿，但没有 resume/review 主路径测试。

### 9.2 已采用的 critique 更正
本文已按 critique 修正以下说法：
1. reviews 泄题模板是 596/1390，约 43%，不是 170 条，也不是约三分之二；2. 29 条主命令 prompt 含命令首词；“28 条”仅是近似旧口径。
3. 13 指冲突答案字符串，按 ID 对是 8 组，并包含 cd home；4. 受审快照真正零覆盖的是 exit、more；passwd、tee 并非零覆盖。
5. ReviewTopics 第 1 项标题实际是“压缩归档”；6. 实际错位的是两个命令项目；符号项目不是错位项。
7. app.rs 内待迁移 handler 是 9 个，不是 8 个；8. 不采用 Ctrl+H 与 Ctrl+S 作为核心提示键。
9. 不声称新增 RecordMode variant“向后兼容无风险”；10. matcher 大小写修复必须同步其模块内单测。

### 9.3 快照与当前工作区的差异
内部工作流的内容数字基于 19 个 command 文件、273 条命令。 当前工作区另有 15 个未跟踪 command 文件， 只读统计合计为 34 文件、542 条命令。
这意味着：
- 原难度矩阵可能已经变化；exit/more 等缺口可能已经被新文件补上；
- lesson 覆盖率、短描述数、重复提示数需要重算；`tests/v03_behavior.rs:145` 的 273 硬编码已经过时；
- 不能把 118 项全绿自动外推到当前未跟踪内容集合。
本文没有把这些新增内容的质量写成已验证事实。

### 9.4 尚待复现或决策的点

#### B5 详解面板溢出
当前 renderer 已有 Wrap，见 `src/ui/typing.rs:149`。 但没有本轮窄终端截图或确定性渲染测试。
结论只能是“待复现”，不能写成已修或未修。

#### 新增 20-34 内容文件
这些文件明显对应外部对标中的求助、apt、tar、管道、环境、 find/grep、磁盘、进程、SSH、systemd、vim、用户权限主题。 但它们未被本轮内部 critique 逐题核验。
合并前仍需检查：
- ID；token 拼接；
- 答案等价形；prompt 泄题与唯一性；
- 输出可信度；难度与类别；
- 与旧题重复；外部来源和署名。

#### 五级梯度的 UI 细节
F1/F2 是本评审推荐键位，不是当前代码事实。 实现前应在 Linux 原生终端、tmux、SSH 与常见终端模拟器中验证。

#### 最小间隔调度
1/3/7/14 天是范围可控的产品建议，不是经本项目用户数据验证的最优参数。 v0.3 应记录使用数据；v0.3.x 再评估是否升级算法。

#### token 级部分得分
它有教学价值，但会改变准确率、mastery 和统计解释。 本文因此建议新增独立 `recall_score`， 并把完整 token tokenizer 与统计 UI 重构顺延。

### 9.5 最终维护建议
v0.3 完成后，计划文档应只保留三种状态：
- 已完成：有代码、入口、测试和验收证据；部分完成：明确列出缺失链路；
- 顺延：有目标版本，不保留伪完成文件。
其中“生成了文件”“有一个 renderer”“测试总数全绿” 都不能单独证明功能完成。 v0.3 的成功标准也不应是题库达到某个整数， 而应是一个基础薄弱的新用户能够：
1. 从正确的主页开始；2. 学会一条命令的用途与 token；
3. 从全显跟打逐步走到完整默写；4. 答错后得到提示并在本轮再次遇到；
5. 隔天在“今日复习”中再次看到；6. 始终在安全的预置输出环境里学习；
7. 不因恢复、边界或判分错误失去对产品的信任。
做到这七点，cmdtyper 才真正从“有很多内容的打字 TUI” 进入“有学习闭环的 Linux 命令教学工具”。
