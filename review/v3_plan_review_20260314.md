# Socrates Review — cmdtyper v0.3 Plan (2026-03-14)

## 1. Requirements Fidelity Check

### Requirement 1 — 长文本自动换行
**Verdict: ✅**

The plan captures the bug and the intended fix correctly:
- explicitly targets `gotchas`, `explanation`, `output_explanation`, `deep_explanation`
- uses `Wrap { trim: false }`
- scopes the change to the relevant lesson / typing UIs

Minor note: this is already partly true in the current codebase. `src/ui/command_lesson.rs`, `src/ui/symbol_lesson.rs`, `src/ui/system_lesson.rs`, `src/ui/typing.rs`, `src/ui/review.rs`, `src/ui/dictation.rs`, `src/ui/stats.rs` already use wrapping in many places. So this is less a broad feature wave than a targeted audit for missed panels and regressions.

---

### Requirement 2 — 复杂命令“深度解析”页面
**Verdict: ⚠️ Partial / under-specified**

The plan gets the general idea right:
- opt-in detail page via `D`
- scrollable long-form explanation
- return / next navigation
- content curation for only selected commands

But it does **not faithfully model the requirement across all required content types**.

The requirement applies to:
1. command lessons
2. complex symbol-topic example commands
3. system-architecture topic commands

The proposed schema does **not** cover all three cleanly:
- it proposes `deep_explanation` on `Command` / `[[examples]]` conceptually
- but current relevant structs are split across **three different models**:
  - `LessonExample`
  - `SymbolExample`
  - `SystemCommand`
- the plan does not specify schema additions for `SymbolExample` and `SystemCommand`

Further gaps:
- `has_deep_explanation = true` is redundant and dangerous drift-prone metadata; it can disagree with the text field
- “supports Markdown-style markup” is promised, but no rendering strategy is defined; current UI is plain `Paragraph`, not a markdown renderer
- “练完后提示按 D 查看详细解析，再跳回总结页” is not mapped precisely onto the existing lesson flow, which currently returns directly from lesson practice to overview after the last example
- “模拟执行过程” and “数据流可视化” need a content format, not just a long string blob

This feature is present in spirit, but the data model and UX contract are incomplete.

---

### Requirement 3 — 符号专题加入打字训练
**Verdict: ⚠️ Mostly captured, but missing detail**

The plan correctly includes:
- two modes: typing + dictation
- large exercise expansion
- reuse of TypingEngine / Matcher
- preserving deep explanation for complex symbol examples

However:
- current `Exercise` is only `{ prompt, answers }`; the migration path for existing symbol files is not specified
- `kind = "typing" | "dictation"` needs a default for backward compatibility, otherwise all existing symbol TOMLs break or require full migration before code lands
- the plan does not specify what a typing exercise stores beyond `command` and `description`; simulated output / explanations / optional deep explanation linkage are not defined
- “综合统计（打字 WPM + 默写准确率）” is sensible, but record-mode / persistence design is omitted. Current stats logic distinguishes typing-vs-dictation carefully; symbol typing should not be silently dumped into generic `SymbolPractice` without policy decisions

So the feature is directionally right, but not fully designed.

---

### Requirement 4 — 系统架构专题加入打字训练
**Verdict: ⚠️ Captured at feature level, weak at architecture level**

The plan does respond to the requirement:
- system commands become typed, not just read
- simulated output remains
- deep explanation is intended for complex commands

But the plan does not specify:
- how `SystemCommand` is extended for optional token explanations, deep explanation, or output explanation
- whether system-topic typing should update persistent stats like normal typing / lesson practice, or stay local to the lesson
- whether config-file lessons with `practice_command` stay read-only or also become typed

The current system lesson flow is `Overview -> Detail -> Commands(idx) -> ConfigFile(idx)`. Replacing `Commands(idx)` with a typed state is feasible, but the plan does not fully describe the new transition rules.

---

### Requirement 5 — “对着打”模块三档模式
**Verdict: ⚠️ Captured, but the hardest part is under-designed**

The plan correctly captures:
- 3 modes
- settings persistence
- runtime `M` switch

But the difficult part is not described rigorously enough:
- Terminal mode is not just “no WPM popup”; it changes the round lifecycle, output display timing, and completion UX
- Detailed mode says “显示 token_details”, but the main typing corpus (`data/commands/*.toml`) uses `tokens`, while lesson examples use `token_details`; these are different schemas, so the UI source of truth is unclear
- current typing renderer is terminal-centric and summary-centric; adding a side panel or inline explanation view changes layout substantially, especially in narrow terminals

This is implementable, but the plan glosses over the UI/data mapping details.

---

### Requirement 6 — 学习曲线平滑化 / 分级训练模块
**Verdict: ⚠️ Significant mismatch**

This is the biggest fidelity problem after deep-explanation schema scope.

What the requirement asks for:
1. a graded training entry (基础 / 常见 / 复杂 / 实战)
2. a typing-mode difficulty filter
3. **or category filter** (`FileSystem / Network / Docker ...`)

What the plan proposes:
- add a new `tier` field to each command
- show a tier-selection screen before typing

Problems:
- the project **already has** a first-class `Difficulty` enum with exactly four levels: `Beginner / Basic / Advanced / Practical`
- command files are already grouped by file-level difficulty, and the dataset already totals **273 commands across 19 TOML files**
- introducing a second concept `tier = beginner/common/advanced/practical` duplicates existing metadata and even renames `basic -> common`, which creates avoidable inconsistency
- the plan does **not** cover the requested category filter (`FileSystem / Network / Docker ...`)
- “在学习中心或主菜单新增分级入口” is softened into “进入 Typing 前弹出难度选择”, which is weaker than the stated product ask

This requirement should be implemented primarily by reusing or refining the existing `Difficulty` model, not by inventing a near-duplicate axis unless there is a proven semantic difference.

---

## 2. Architectural Concerns

### 2.1 The proposed TOML schema is not yet coherent

#### Good
- The plan intends to make new fields optional, which is the right backward-compatibility direction.

#### Problems
1. **`tier` duplicates `difficulty`**
   - Current `Difficulty` already has: `Beginner`, `Basic`, `Advanced`, `Practical`
   - Plan adds `tier = "beginner" / "common" / "advanced" / "practical"`
   - This produces duplicated truth, migration noise, and inevitable disagreement bugs

2. **Deep-explanation is attached to the wrong abstraction level**
   - Deep explanation belongs to the concrete teachable item being displayed:
     - `LessonExample`
     - `SymbolExample`
     - `SystemCommand`
   - Not one generic field jammed into only one data type

3. **`has_deep_explanation` should not exist**
   - derive availability from `deep_explanation.is_some()`
   - duplicated booleans are how content drift begins

4. **Symbol exercises need an explicit backward-compatible evolution path**
   - Current `Exercise` is minimal
   - If `kind` becomes required, legacy data breaks
   - Better:
     - either default `kind` to dictation
     - or split into `typing_exercises` + `dictation_exercises`

5. **Markdown-style content is promised without a renderer contract**
   - If the UI only renders plain lines, say so
   - If mini-markup is supported, define exactly which tokens are parsed (`#`, `-`, fenced blocks, etc.)

### 2.2 State-machine expansion is currently too casual

Current app state is already large, and `src/app.rs` is **1659 lines**, not ~1100. The plan adds:
- `AppState::DeepExplanation`
- `AppState::TierSelection`
- new symbol phases
- new system phases
- new typing mode enum and runtime switching behavior

The risky part is this field:
- `AppState::DeepExplanation { source: DeepSource, return_state: Box<AppState> }`

Why this is dangerous:
- boxing full `AppState` snapshots makes navigation logic implicit and fragile
- it can preserve stale indices / transient UI assumptions
- “return to current practice page” and “jump to next command” are not purely reversible operations; they should be explicit transitions, not resurrected boxed state blobs

Better design:
- store a **small locator** such as:
  - `DeepSource::LessonExample { topic_idx, lesson_idx, example_idx }`
  - `DeepSource::SymbolExample { topic_idx, symbol_idx, example_idx }`
  - `DeepSource::SystemCommand { topic_idx, section_idx, command_idx }`
- and a **simple return target enum**, not arbitrary boxed app state

### 2.3 `app.rs` maintainability is the real risk

This plan assumes incremental edits to `app.rs` are cheap. They are no longer cheap.

Current evidence:
- `src/app.rs` is 1659 LOC
- it already directly owns state transition logic for typing, lessons, symbols, systems, review, dictation, settings, stats
- Waves 1, 2, 3, 4, 5 all touch app-level transitions

If implemented as proposed, `app.rs` becomes a feature landfill.

Before or during v0.3, the team should extract at least one of these:
- `typing_flow.rs`
- `symbol_flow.rs`
- `system_flow.rs`
- `settings_flow.rs`
- `deep_explanation.rs`

At minimum, move state-specific handlers out of `App` into modules.

### 2.4 TypingEngine reuse is only partially appropriate

**Appropriate uses:**
- command-topic practice
- system-command copy typing
- symbol example copy typing

**Not sufficient by itself for:**
- dictation validation (needs Matcher / answer-set semantics)
- detail-mode token explanations (presentation concern, not typing engine concern)
- content with multiple valid command spellings/orderings unless one canonical command is chosen

So “reuse TypingEngine everywhere” is fine only for exact copy-typing contexts. The plan should say that explicitly.

### 2.5 The plan underestimates renderer/layout complexity

Detailed mode and deep explanation both imply new rendering patterns:
- split panes or sidebars
- long scrollable content
- narrow-terminal fallbacks
- interaction between simulated output and detail prompts

Current renderers are simple full-screen paragraphs and terminal panels. v0.3 needs a small UI architecture plan, not just feature bullets.

---

## 3. Feasibility Assessment

### 3.1 Wave dependencies are not actually safe as written

Planned parallelism is optimistic.

#### Claimed parallel pairs
- Wave 1 + Wave 2 “并行”
- Wave 3 + Wave 4 “并行”
- Wave 5 “独立于其他 Wave”

#### Reality
- **Wave 1 and Wave 2 are not safely parallel**
  - both touch typing flow
  - both touch settings / app entry / typing UX
  - both likely touch `app.rs`, `ui/typing.rs`, possibly menu routing

- **Wave 3 and Wave 4 are not truly isolated**
  - both need `app.rs` state dispatch changes
  - both need `ui/mod.rs` routing changes
  - both may need shared helpers for typed lesson flows

- **Wave 5 is not independent**
  - must touch models, app state, UI dispatch, and multiple lesson/system/symbol renderers
  - if done after Waves 3/4 without prior schema agreement, it causes rework

Conclusion: merge-conflict risk is high. The plan acknowledges only Wave 1/2 conflicts, but that is materially inaccurate.

### 3.2 Time estimates are too aggressive

The schedule says roughly:
- T+0 to T+60: Wave 1 + 2
- T+60 to T+120: Wave 3 + 4
- T+120 to T+210: Wave 5
- T+210 to T+300: test, review, fixes, push

This is not realistic for the full scope.

Why:
- 273 command annotations / classification pass
- 6 symbol topics expanded from current **30 exercises total** to **90–120 total**
- 31 lesson files / **186 lesson examples** need auditing for detail-link candidates
- 6 system-topic files with **129 system commands** need typed-flow decisions
- 30–50 deep explanations are long-form pedagogical writing, not mechanical filler
- app/state/UI refactor overhead is ignored
- QA for educational content is slower than code compilation

A realistic split would be:
1. code scaffolding and UX shells
2. data/schema migration
3. initial content tranche
4. later content expansion

### 3.3 Content volume is achievable, but not in one pass with high quality

#### Tier labels for 273 commands
Achievable, yes. Likely 1 focused pass.

#### 90+ new symbol exercises
Achievable, yes. Current base is low enough that expansion is straightforward.

#### 30–50 deep explanations
Achievable only if staged.

The risk is quality: deep explanations for beginners are not generic prose. They require:
- scenario framing
- token-level decomposition
- realistic outputs
- common pitfalls
- pipeline intermediate states

That is authoring work, not mere schema filling.

### 3.4 Testing scope is under-specified

The plan says `cargo build + test + clippy`, but current tests are mostly loader/logic tests. v0.3 needs more explicit test classes:
- loader compatibility with optional new fields
- state transition tests for new phases
- typing-mode behavior tests (Terminal vs Standard vs Detailed)
- deep explanation availability tests
- filter tests by difficulty / category
- regression tests that old TOML still parses unchanged

---

## 4. Gaps & Risks

1. **The plan does not exploit existing `Difficulty` cleanly**
   - it invents `tier` instead of reusing the current domain model

2. **Category filter from requirement 6.2 is missing**
   - this is a direct fidelity gap

3. **Deep explanation schema is incomplete for symbol and system content**
   - this is the biggest architecture gap

4. **`app.rs` bloat is likely to worsen sharply**
   - without refactor, maintainability will regress

5. **Terminal mode UX is not fully specified**
   - e.g. does it still record per-command stats?
   - what exactly appears after the final command?
   - does `Enter` both accept output and advance, or auto-advance after showing output?

6. **Detailed mode lacks a canonical data source**
   - command typing corpus uses `tokens`
   - lesson examples use `token_details`
   - system commands have neither

7. **Stats policy is not defined for new practice contexts**
   - symbol typing
   - system typing
   - possibly deep-explanation exits and resumptions

8. **Narrow terminal behavior is not addressed**
   - especially for detailed mode side-by-side layouts

9. **Migration / rollout plan is missing**
   - if code lands before content, what is the fallback UX?
   - if some entries lack deep explanation, how is the hint suppressed?

10. **Educational consistency risk**
   - bulk-generated deep explanations can easily hallucinate or teach bad habits if not reviewed carefully

---

## 5. Specific Suggestions

### 5.1 Replace `tier` with existing `Difficulty` unless there is a true semantic gap

Preferred design:
- reuse current `Difficulty` for the four training bands
- if individual commands need override, add optional per-command `difficulty_override: Option<Difficulty>` or simply optional `difficulty: Option<Difficulty>` that falls back to file-level meta
- do **not** invent `common` as a near-synonym of `basic`

### 5.2 Add category filtering explicitly

Requirement 6.2 asks for difficulty **or category** filters.

Minimum viable product:
- first pick difficulty band: All / Beginner / Basic / Advanced / Practical
- optionally pick category: All / FileOps / Network / Archive / ...

This fits the existing `Category` enum cleanly.

### 5.3 Redesign deep explanation around teachable items

Use optional fields on the actual display structs:
- `LessonExample.deep_explanation: Option<String>`
- `SymbolExample.deep_explanation: Option<String>`
- `SystemCommand.deep_explanation: Option<String>`

Optionally add structured helpers later if needed:
- `pipeline_steps: Vec<PipelineStep>`
- `pitfalls: Vec<String>`
- `scenario: Option<String>`

But do not start with a fake generic abstraction that only fits one content type.

### 5.4 Remove `has_deep_explanation`

UI rule should be:
- if `deep_explanation` exists and is non-empty, show the `D` hint
- otherwise do nothing

One source of truth.

### 5.5 Refactor before feature pile-on

Before full v0.3 implementation, perform a short structural cleanup:
- extract typing-mode handlers from `app.rs`
- extract symbol lesson flow
- extract system lesson flow
- centralize route dispatch in `ui/mod.rs` helpers

This will pay for itself immediately.

### 5.6 Introduce a generic “typed lesson item” helper

A lot of flows now want the same pattern:
- show explanation
- copy-type a command
- show simulated output
- maybe allow deep explanation
- next / previous navigation

Abstract the behavior, even if only internally, so symbol/system/command flows do not each invent their own tiny typing lifecycle.

### 5.7 Reorder the waves

Recommended order:

#### Wave A — Architecture prep
- small refactor of `app.rs`
- define final schemas
- define stats policy

#### Wave B — Low-risk UX wins
- wrap audit
- typing 3-mode scaffolding
- settings persistence

#### Wave C — Filtering / learning curve
- reuse `Difficulty`
- add category filter
- add graded entry UI

#### Wave D — Deep explanation shell
- add state + UI + loader support
- seed 3–5 entries only
- validate navigation before bulk content

#### Wave E — Symbol and system typed practice
- build on shared typed-item flow

#### Wave F — Content expansion
- 273 command audit
- 90+ symbol exercises
- 30–50 deep explanations
- human QA pass

This order reduces rework substantially.

### 5.8 Stage content delivery

Do not block the whole release on 30–50 deep explanations.

Possible release slicing:
- v0.3.0: wrap fixes, mode switching, graded entry, symbol/system typing, deep explanation framework + 5 flagship examples
- v0.3.1+: more deep explanations, more symbol exercises

### 5.9 Define stats behavior explicitly

Decide, in writing:
- whether system-topic typing contributes to general typing stats
- whether symbol typing contributes WPM stats
- whether symbol dictation contributes only accuracy
- whether Terminal mode suppresses only summary UI, or also changes persistence

Without this, analytics will drift.

### 5.10 Add backward-compatibility tests first

Before migration work, write loader tests proving:
- old TOML still loads
- missing new fields default safely
- mixed old/new content works

That will keep content rollout decoupled from code rollout.

---

## 6. Verdict

**Revise Plan**

This is a promising product direction, and the plan is strong on intent, but it is **not yet architecturally sound enough to execute as-is**.

The main blockers are:
1. duplicated `tier` vs existing `Difficulty`
2. missing category-filter design
3. incomplete deep-explanation schema across lesson/symbol/system content
4. underestimation of `app.rs` / UI routing impact
5. unrealistic parallelism and timeline assumptions

If the team revises the plan around:
- a single difficulty model,
- explicit category filtering,
- deep explanation attached to concrete teachable items,
- and a modest pre-feature refactor,

then I would expect the implementation to become both feasible and maintainable.
