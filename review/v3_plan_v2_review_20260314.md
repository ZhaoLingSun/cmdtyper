# Socrates Review — cmdtyper v0.3 Plan v2 (2026-03-14)

## 1. Checklist

1. **`tier` replaced with existing `Difficulty`?** — ✅  
   v2 明确锁定“复用现有 `Difficulty` 枚举（Beginner/Basic/Advanced/Practical）”，并删除新增 `tier` 方案。这正面回应了 v1 的核心架构批评。

2. **Category filter added?** — ✅  
   v2 在 Wave C 明确加入 `Difficulty + Category` 双维筛选，并复用现有 `Category` 枚举，符合原始需求 6.2。

3. **Deep explanation on correct structs (`LessonExample`, `SymbolExample`, `SystemCommand`)?** — ✅  
   v2 已将 `deep_explanation: Option<String>` 分别放到 `LessonExample` / `SymbolExample` / `SystemCommand`，这是正确的抽象层级。

4. **`has_deep_explanation` removed?** — ✅  
   v2 明确“不使用”，改由 `deep_explanation.is_some()` 推导，消除了重复真值来源。

5. **DeepExplanation uses locator not `Box<AppState>`?** — ✅  
   v2 使用 `DeepSource` + `DeepReturn` 小定位器/返回目标设计，避免 `Box<AppState>` 快照复活陈旧状态的问题。

6. **`app.rs` refactor planned?** — ✅  
   v2 新增 Wave A 架构预备，计划把 `app.rs` 按 flow 拆到 `typing_flow.rs` / `lesson_flow.rs` / `symbol_flow.rs` / `system_flow.rs` / `review_flow.rs`，这是我在 v1 中要求的关键前置动作。

7. **Wave parallelism fixed (serial execution)?** — ✅  
   v2 明确改为串行 Wave，并解释原因：都改 `app.rs`、`ui/mod.rs` 等共享区域，降低合并风险。这比 v1 的并行估计成熟得多。

8. **Stats policy defined for new contexts?** — ✅  
   v2 给出了新增模式与 `SessionRecord / WPM / Accuracy / Mastery` 的契约表，并明确 `RecordMode` 新增 `SymbolTyping` / `SystemTyping`，Terminal 模式也定义为“照常记录统计”。这比 v1 完整很多。

9. **Backward compatibility tests?** — ✅  
   v2 在 Wave A 明确先写 loader 兼容测试，并在 Wave F 再补行为测试，覆盖“旧 TOML 无新字段仍正常加载”。这一点是合格的。

10. **Content staging plan?** — ✅  
   v2 明确采用 staged rollout：Wave D 先做 5 个旗舰 deep explanation，Wave F 再扩到 30–50 条，并拆分成 `v0.3.0` / `v0.3.1+`。这是我在 v1 中最希望看到的内容生产分期方案。

---

## 2. Decision Quality

### Overall
**Decision quality is substantially improved.**  
相较 v1，这版已经从“方向正确但不可直接执行”提升到“架构上基本可执行、可以进入实施阶段”。多数关键设计决策都变得更克制、更贴近现有代码与数据模型。

### Strong decisions

1. **复用现有枚举，而不是重复造轮子**  
   用 `Difficulty` 和 `Category` 直接实现分级与过滤，是最稳妥、最符合现有数据集的决策。避免了 `tier` / `difficulty` 双轨并存的脏设计。

2. **把 deep explanation 绑在真实教学单元上**  
   这是本次修订中最重要的架构修正。lesson / symbol / system 三类内容现在都能统一支持深度解析，而且不需要虚假的通用字段。

3. **先做 app.rs 拆分，再加功能**  
   这是从“堆功能”转向“可维护演化”的标志。若团队真的按 Wave A 执行，后续 Wave 的实现风险会显著下降。

4. **串行 Wave + staged content rollout**  
   这两个决定一起，基本修复了 v1 最大的执行风险：过度乐观的并行与一次性内容填充。现在的工程节奏更像真实项目，而不是纸面排期。

5. **统计契约被显式写出来**  
   新上下文（SymbolTyping / SystemTyping / Terminal 模式）不再是“实现时再看”，而是前置定义了行为，这能避免后期统计口径漂移。

### Locked decisions that are sound

以下锁定决策我认为**总体 sound**：
- 训练分级复用 `Difficulty`
- 分类过滤复用 `Category`
- `deep_explanation` 分布到三类具体 struct
- 不使用 `has_deep_explanation`
- DeepExplanation 用 locator，而不是 `Box<AppState>`
- `M` / `D` 快捷键选择
- `kind` 缺省默认为 `dictation`
- Terminal 模式“跳过弹窗但保留统计”
- 发布切分为 `v0.3.0`（框架+少量旗舰内容）和 `v0.3.1+`（持续扩充内容）

这些决策都明显比 v1 更成熟，且与原需求相容。

### Timeline realism
**More realistic than v1, but still somewhat optimistic.**  
修订版把总时长从约 300 分钟拉到约 430 分钟，并改成串行，这个方向是对的；但我仍认为：
- Wave A 真正做好模块拆分 + 兼容测试，未必只要 60 min
- Wave E（符号 + 系统打字）同时包含模式扩展、状态机调整、统计接入、题库解析，60 min 偏紧
- Wave F 的内容扩充 + 行为测试 + README + 人工审查，90 min 对 30–50 条 deep explanations 仍偏乐观

所以：**比 v1 真实很多，但仍带一点“理想工时”色彩。**  
如果按高质量交付标准，我更愿意把它理解为：
- `v0.3.0`：1 个完整工作日可达
- `v0.3.1+`：另一个内容迭代窗口继续补全

---

## 3. Remaining Gaps

虽然 v2 已显著改善，但还不是零风险。以下仍是我建议在开工前再明确的点：

### 3.1 `Exercise.kind: Option<String>` 仍然偏弱
当前设计是可行的，但类型不够强：
- `Option<String>` 易出现拼写漂移（`typing` / `typeing` / `dictation`）
- 更稳妥的是 serde 可反序列化到枚举，例如 `ExerciseKind`，并给默认值 `Dictation`

**结论**：不是 blocker，但建议尽快从字符串提升为强类型枚举。

### 3.2 Detailed mode 的 token 来源优先级仍有轻微语义混杂
v2 写了“`token_details` > `tokens` > 无”，这是可实施的，但仍隐含一个问题：
- `token_details` 和 `tokens` 是否真的是同一层语义？
- system command 数据若没有任一字段，Detailed 模式在该场景下会退化成什么样？

当前方案可做，但最好在实现前再明确：
- Detailed 模式是否允许“只有部分命令有 token 解释”
- 对 system commands 是否接受空白右侧面板 / 提示“暂无逐词解释”

### 3.3 DeepExplanation 的 `DeepReturn` 设计还需要和真实导航规则完全对齐
v2 现在有：
- `BackToPractice`
- `NextItem`

这比 `Box<AppState>` 好很多，但还要再核实两个细节：
1. “返回当前练习页”是返回**完成后的该页**，还是返回**总结页前的停留态**？
2. “下一条”在 lesson / symbol / system 三种上下文里，边界行为是否一致？最后一条怎么办？

**结论**：架构方向正确，但导航 contract 还应更细一点。

### 3.4 “纯文本 + 简单约定” 对 deep explanation 内容作者不够自描述
v2 决定不引入 Markdown renderer，这完全合理；但若要批量写 30–50 条 deep explanations，最好再补一页内容 authoring spec，至少规定：
- `###` 标题层级怎么用
- 缩进代码块如何写
- 管道数据流展示的固定模板
- 安全提示的统一写法

否则内容生产阶段会出现格式不统一、渲染风格漂移。

### 3.5 Symbol / System 进入统计体系后，汇总 UI 是否同步更新？
v2 已定义统计契约，这很好；但还要确认现有 stats 页面是否：
- 会展示 `SymbolTyping` / `SystemTyping`
- 这些新记录会不会污染已有 lesson / global typing 统计展示
- mastery 维度中 symbol mastery 与 command mastery 是否会混在一起

**结论**：数据层定义了，但展示层口径还需补一句。

### 3.6 Content volume still needs editorial discipline
v2 已把内容 staged 了，但 30–50 条 deep explanations 依旧不是小工作。风险不在“能不能生成”，而在：
- 是否真的适合完全初学者
- 是否有错误类比或误导性安全建议
- 是否不同作者/agent 输出风格漂移

所以我建议：**Wave F 不是“批量生成完就算完成”，而是“生成 + 人工抽检 + 统一风格修订”。**

---

## 4. Verdict

**Approve Plan**

这次我给 **Approve Plan**，理由如下：
- v1 中我提出的 10 个关键问题，v2 已经 **全部正面回应**
- 核心架构错误（`tier`、`has_deep_explanation`、`Box<AppState>`、并行 Wave、缺失 stats policy）都已修正
- 新版计划开始尊重已有数据模型、已有代码复杂度和真实内容生产成本
- 虽然仍有若干细节待实现时收口，但都属于**可在实现阶段通过小决策解决的二级问题**，不再是“计划本身不能执行”的一级问题

### Approval condition
我的批准是**带审慎条件的通过**：
1. Wave A 必须先落地，不能跳过 app.rs 拆分直接堆功能
2. `Exercise.kind` 最好尽早改成强类型枚举
3. Wave F 必须包含人工内容审查，而不是纯批量灌文本
4. 若 Wave A 或 Wave E 实际复杂度超预期，应允许及时缩 scope，优先保住 `v0.3.0` 的框架完整性

若按上述条件执行，这个 v2 计划已经具备进入实现阶段的质量。
