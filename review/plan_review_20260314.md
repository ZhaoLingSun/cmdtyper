# Socrates Plan Review — cmdtyper-v2 Repair Plan (2026-03-14)

## 1. Coverage Matrix

| Original issue | Severity | Plan coverage | Assessment | Commentary |
|---|---|---:|---|---|
| Docker/runtime data path broken by design | Critical | Wave 1.1 + 1.3 | ✅ covered | The plan correctly identifies the root cause: app/runtime path, Docker image layout, and user-data volume semantics must be separated. `CMDTYPER_DATA_DIR` + distinct `CMDTYPER_USER_DIR` is the right direction. |
| Terminal simulation does not behave like README promise (`Enter`, output rendering, `display`, multiline) | Critical | Wave 2.1–2.5 | ⚠️ partially covered | The plan addresses `Enter`, simulated output, `display`, and RoundResult activation. Good. But multiline interaction is still underspecified: “每行独立交互” is an aspiration, not a concrete state/input model. Cursor behavior, backspace across lines, prompt placement, and line-wrapping invariants are not defined. |
| Progress/statistics integrity wrong (difficulty stamped incorrectly; several modes not recorded) | Critical | Wave 1.2 + 1.3 | ⚠️ partially covered | The incorrect difficulty stamping is directly addressed. Stats recording for command lesson, dictation, symbol/system practice is mentioned. However, the plan still does not define the product policy: which modes should count toward mastery, history, streaks, and difficulty analytics? Without a clear analytics contract, implementation may remain inconsistent even if more call sites invoke `finish()`. |
| Placeholder practice surfaces shipped as real features | Critical | Wave 3.1–3.4 | ⚠️ partially covered | Symbol practice and review practice are addressed. Settings UI is addressed. Demo-stage mismatch is acknowledged. But the plan repeatedly says “implement or hide/remove” without forcing a decision up front. That is not execution-ready; it is still a menu of choices. |
| Typing UI multiline support is half-implemented and pedagogically poor | Major | Wave 2.3 | ⚠️ partially covered | The plan recognizes multiline rendering, but not the full interaction contract. It needs explicit behavior for cursor movement, partial completion, per-line validation, and how `display` maps back to `target`. |
| End-of-round state machine inconsistent; RoundResult dead | Major | Wave 2.4 + Wave 5.1 | ✅ covered | The plan correctly spots the need to either activate RoundResult everywhere or delete it. This is the right framing. |
| Command-learning flow does not match documented three-stage model | Major | Wave 3.4 + Wave 5.2 | ⚠️ partially covered | The plan proposes either adding Demo or updating README to two phases, and recommends the latter. Reasonable, but again too noncommittal for an execution plan. Pick one now. |
| Settings UI falsely suggests editability for username/hostname | Major | Wave 3.3 | ✅ covered | Either real text input or visual demotion to read-only are both acceptable resolutions. |
| Data/content completeness materially below product narrative | Major | Wave 4.1–4.4 | ⚠️ partially covered | Lesson coverage targets are good. But command-bank `simulated_output` is only targeted to 80%, which is not enough if the README/product promise remains broad. Also nothing in the plan tackles the uneven quality and risk scaffolding of sensitive examples. |
| Tests miss behavioral correctness | Major | Wave 5.1 | ⚠️ partially covered | The proposed tests are directionally correct, but still too thin for the defects already observed. They do not explicitly cover multiline typing semantics, simulated-output rendering flow, stats inclusion/exclusion policy by mode, or review/symbol practice behavior end-to-end. |

### Bottom line on coverage
The plan **does address every Critical/Major issue at least nominally**, which is good. But several items are only addressed at the level of “we should do something in this area,” not at the level of a locked, testable execution contract. That means the plan is **not yet bulletproof enough** for implementation kickoff.

---

## 2. Gaps & Risks

### A. The plan is still making product decisions during implementation
This is the biggest weakness.

Several items are written as forks rather than decisions:
- RoundResult: implement fully **or** delete dead code
- Settings fields: make editable **or** demote to display-only
- Demo phase: add it **or** rewrite README

Those are valid options during design review, but not inside a repair plan that is supposed to close a Major Revision. A repair plan must resolve ambiguities before execution begins. Otherwise every subagent will make local product decisions and you will get a patchwork result.

### B. Statistics policy is still undefined
The plan fixes bad call sites, but not the underlying contract.

You need an explicit answer to all of these:
- Which modes generate `SessionRecord`?
- Which modes update long-term mastery?
- Which modes contribute to stats page aggregates?
- Are command lessons and review practice equivalent to free typing sessions analytically?
- Should symbol/system exercises share the same WPM/accuracy model as command typing?

Right now the plan assumes “record more things,” but not all learning modes are semantically equivalent. If you do not define this first, the stats page may become more complete yet still conceptually wrong.

### C. Multiline typing remains dangerously vague
This is the most likely place for a “looks fixed, still broken” outcome.

The plan says:
- use `display`
- render multiline properly
- continuation lines get prefixes
- each line is independently interactive

That is not enough. The implementation team needs a concrete model for:
- whether input is stored as a flat buffer or line segments
- whether correctness is evaluated against `target` or `display`
- how escaped newlines / continuation backslashes map to typed characters
- how backspace behaves at line boundaries
- whether the learner presses one final Enter or line-by-line Enter
- how simulated output is sequenced after multiline completion

Without this, Wave 2 risks shipping a cosmetic multiline fix rather than a correct one.

### D. Content coverage target is still too weak for command-bank output
For lessons, the plan is ambitious. For the command bank, it is not.

The plan sets:
- lessons: 100% token details + outputs + explanations
- commands: simulated output to **80%** coverage

That asymmetry is not well justified. If the product still advertises simulated terminal output broadly, leaving ~20% of commands without output means the user experience remains inconsistent and the README remains on shaky ground.

At minimum, the plan should specify one of these:
1. raise command-bank coverage to 100%, or
2. narrow the product claim and clearly define where simulation exists and where it intentionally does not.

### E. High-risk instructional content is not addressed at all
My review explicitly called out risky system-edit examples and thin scaffolding around them. The plan ignores this.

This is not a cosmetic omission. An educational CLI product teaching commands like `sed -i` against SSH/system configs should make stronger pedagogical and safety choices:
- backup-first examples
- validate-before-restart patterns
- safer dry-run sequencing where possible
- explicit “do not run blindly on production” framing

The content hardening wave should include a pass over operationally risky lessons, not just metadata coverage.

### F. Review/symbol practice remediation is feature-level but not state-model-level
“Reuse TypingEngine” and “reuse Matcher” are implementation instincts, not full designs.

The plan does not specify:
- how review exercises are sourced and sequenced
- whether review uses prior weak areas, recent misses, or static topic lists
- whether symbol practice should support partial hints, retries, or answer reveal rules
- how these modes persist progress and appear in history/stats

This matters because placeholder modules often fail not from rendering, but from missing state semantics.

### G. Parallelism introduces merge/conflict risk
Wave 4 content work runs in parallel with code-path changes that may alter schemas, rendering expectations, or validation rules. That is manageable, but only if the schema/acceptance contract is frozen first.

Right now the plan does not say:
- whether any TOML schema fields will change
- who owns validation scripts
- how conflicting edits across multiple subagents will be merged safely
- whether a canonical coverage report is regenerated before merging all content branches

### H. Tool/model choice is sloppy and internally inconsistent
The plan lists “codex (GPT-5.3)” / “codex”, while the review context and review skill standard are explicitly built around GPT-5.4 xhigh for rigorous work. For implementation subagents this is not fatal, but it signals planning looseness.

If you are delegating high-variance tasks like content generation and behavior-sensitive Rust refactors, model selection should be intentional, not hand-wavy.

---

## 3. Execution Critique

### What is good
- The wave decomposition is mostly sensible.
- Separating packaging/stats (Wave 1), typing-loop repair (Wave 2), placeholder-feature repair (Wave 3), content fill (Wave 4), and tests/docs (Wave 5) is the right broad shape.
- Running Wave 4 in parallel is economically sensible because the content surface is large.

### What is weak

#### 3.1 Wave dependencies are not fully correct
The plan says Wave 2 depends on Wave 1 because of the new `finish()` signature. That is too narrow.

Wave 2 also depends on product decisions about:
- whether completed typing goes to RoundResult or directly advances
- how simulated output is staged
- what multiline semantics are

These are not mere coding dependencies; they are behavior dependencies. If they are not fixed before Wave 2 begins, the typing refactor may need rework.

#### 3.2 Wave 3 is over-bundled
Wave 3 groups together:
- symbol practice
- review practice
- settings UI
- command-lesson Demo mismatch

These are not equally coupled. Settings UI is trivial. Review practice is not trivial at all. Bundling them in one wave hides the real risk profile.

I would split Wave 3 into:
- **3A:** remove/deceptive-UI cleanup (settings, README phase mismatch if choosing doc alignment)
- **3B:** true interactive practice implementation (review + symbol), which is materially harder

#### 3.3 Time estimates are optimistic
The quoted total of 2–3 hours is not serious if the goal is to actually satisfy a harsh review rather than merely patch visible defects.

Why this is optimistic:
- multiline typing semantics alone can consume substantial debugging time in TUI apps
- review practice is not a 15-minute feature if done honestly
- content completion across dozens of TOMLs plus QA is tedious and error-prone
- end-to-end behavioral tests in Rust TUI/stateful apps often take longer than expected
- merge/integration across 4–5 subagents always costs overhead

A more credible estimate is **half a day to a full day** for implementation + integration + retest, even if agents parallelize the mechanical content work.

#### 3.4 Validation gates are too weakly specified
“cargo test passes” and “manual flow works” are necessary but not sufficient.

The plan should define explicit acceptance checks such as:
- Docker image launched outside repo root still loads bundled curriculum
- a command with multiline `display` can be fully practiced without hidden target leakage
- stats page changes after dictation / lesson / review according to the chosen policy
- simulated output is shown only at the correct step, then history advances correctly
- unfinished placeholder UI is absent from user-facing navigation

#### 3.5 Final integration ownership is missing
With multiple subagents editing code, tests, content, and docs, somebody must own:
- final merge order
- conflict resolution
- regeneration of coverage metrics
- final README truthfulness audit
- pre-review smoke test checklist

The plan assumes this will somehow happen, but does not assign it.

---

## 4. Specific Suggestions

### Required changes before execution
1. **Resolve all plan forks into decisions.**
   - Choose now: RoundResult implemented or removed.
   - Choose now: settings fields editable or demoted to read-only.
   - Choose now: command lessons remain 2-stage with README corrected, or 3-stage Demo is implemented.
   Do not enter implementation with open product branches.

2. **Add a written analytics contract.**
   Create a short section defining exactly which modes update:
   - session history
   - mastery
   - WPM/accuracy aggregates
   - stats page totals
   - streak/recent activity (if any)

3. **Specify multiline typing behavior concretely.**
   Add an implementation note covering:
   - input buffer model
   - cursor model
   - relation between `display` and `target`
   - Enter semantics for multiline commands
   - backspace behavior across continuation boundaries
   - how correctness is tested and when output appears

4. **Raise or re-scope command output coverage.**
   Either:
   - make command-bank `simulated_output` coverage a 100% target, or
   - explicitly narrow the README and UI promise so missing outputs are honest and unsurprising.

5. **Add a content safety/hardening pass for risky examples.**
   Include explicit review of lessons/examples involving config rewrites, service restarts, SSH hardening, and other foot-gun operations.

6. **Split Wave 3 into low-risk cleanup vs real feature work.**
   This will make scheduling and integration more honest.

7. **Add stronger test cases.**
   Minimum additions:
   - multiline typing end-to-end test
   - simulated-output-before-next-command sequencing test
   - stats inclusion/exclusion by mode policy test
   - review-practice interaction test
   - symbol-practice hide/reveal/answer-check test
   - Docker runtime test from a non-repo working directory assumption

8. **Define final integrator + merge protocol.**
   One owner should merge all waves after:
   - schema freeze
   - content validation script pass
   - full test suite
   - README truth audit
   - manual smoke run

### Recommended restructuring of the plan
A stronger plan would look like this:
- **Wave 0 — Design lock:** resolve open product decisions; define analytics and multiline contracts
- **Wave 1 — Runtime/data/stats plumbing**
- **Wave 2 — Typing-loop semantics + RoundResult/output flow**
- **Wave 3A — Deceptive UI cleanup / README truth alignment**
- **Wave 3B — Real interactive review/symbol practice**
- **Wave 4 — Content completion + risky-content hardening**
- **Wave 5 — Behavioral tests + integration audit**

That ordering is much more likely to survive integration without backtracking.

---

## 5. Verdict

**Revise Plan**

This is a **good first repair plan**, not a final one.

It succeeds in one important respect: it has identified all of the major problem zones from the review and mapped them into concrete work buckets. That means the team understood the critique.

But I do **not** approve it yet, because it is still too ambiguous in the places where ambiguity will cause rework or leave defects half-fixed:
- too many unresolved product choices masquerading as implementation steps
- no explicit analytics contract
- no concrete multiline interaction model
- weak command-bank completeness target
- no risky-content hardening pass
- optimistic timing and under-specified integration ownership

If the team incorporates the required changes above, I would expect the revised plan to be execution-worthy.
