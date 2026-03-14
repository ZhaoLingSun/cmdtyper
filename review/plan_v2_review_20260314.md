# Socrates Plan Review — cmdtyper-v2 Repair Plan v2 (2026-03-14)

## 1. Required Changes Checklist

| Required change from v1 review | Status | Evaluation |
|---|---:|---|
| 1. All plan forks resolved into concrete decisions | ✅ | v2 now locks the key product forks up front in Wave 0: RoundResult is deleted, username/hostname become read-only, command lessons stay two-stage, and command-bank `simulated_output` coverage is raised to 100%. This is the biggest improvement versus v1. |
| 2. Written analytics contract added | ✅ | v2 adds a mode-by-mode contract covering `SessionRecord`, mastery, WPM/accuracy aggregation, and Stats-page visibility. That was missing before and is now present. |
| 3. Multiline typing behavior specified concretely | ✅ | v2 defines the core model: `target` remains the flat truth, `display` drives multiline rendering, line breaks are precomputed, cursor is mapped back into the display, continuation prefixes are render-only, Backspace operates on the flat buffer, and Enter only submits after completion. That is concrete enough to implement. |
| 4. Command output coverage raised or re-scoped | ✅ | v2 chooses the stronger path: 100% command-bank `simulated_output` coverage instead of the earlier 80% target. That aligns the plan with the README promise. |
| 5. Content safety / hardening pass added | ✅ | v2 adds explicit safety rules for risky examples (`sed -i`, `systemctl restart sshd`, `rm -rf`, `chmod 777`, firewall changes) and threads them into Wave 4 acceptance criteria. Good. |
| 6. Wave 3 split into low-risk cleanup vs real feature work | ✅ | v2 cleanly separates **Wave 3A** (deceptive UI / README / dead-code cleanup) from **Wave 3B** (real interactive symbol/review practice). This is much more honest about complexity. |
| 7. Stronger test cases specified | ✅ | v2 adds a materially stronger behavior-test section: state machine, typing semantics, stats policy, Docker path behavior, symbol practice, and review practice. This is meaningfully better than the previous thin test list. |
| 8. Final integrator + merge protocol defined | ✅ | v2 explicitly assigns Alice as integrator, freezes schema, defines merge order, and requires validation after each merge plus a final README truth audit. That closes a major planning gap. |

### Bottom line on checklist
The team addressed **all eight required changes** from my v1 plan review. On that dimension, v2 is a successful revision.

---

## 2. Decision Quality

### Overall judgment
The locked decisions are **mostly sound** and materially better than the v1 plan. The plan is now much closer to execution-grade rather than exploratory.

### What is notably good

#### A. The design-lock wave is the right move
Putting all contentious product decisions into **Wave 0** is exactly what the prior revision needed. This sharply reduces the chance of subagents making inconsistent local product calls during implementation.

#### B. The packaging/runtime decision is correct
Separating `CMDTYPER_DATA_DIR` from `CMDTYPER_USER_DIR`, and aligning Docker image layout with app runtime expectations, directly addresses the release-blocking packaging flaw from the project review. This is the correct fix, not a band-aid.

#### C. The command-output stance is finally honest
Raising command-bank `simulated_output` to **100% coverage** is the right call if the product continues to promise simulated terminal output broadly. Anything weaker would have left the README/product contract shaky.

#### D. The multiline model is workable
The chosen multiline model is not fancy, but it is implementable and coherent:
- one flat truth source (`target`)
- one display-layer mapping (`display` + `line_breaks`)
- no per-line submission semantics
- Enter only after full completion

That is a sensible “minimal correct model” for a TUI educational tool. It avoids the worst trap from v1, namely vague talk about “independent multiline interaction” without a data model.

#### E. The Wave 3 split is strong
Separating deceptive-UI cleanup from real feature implementation is excellent. Settings/README/dead-code cleanup are cheap and low-risk. Symbol/review practice are real product work. Those should never have been blended into a single bland “Wave 3.”

#### F. The test plan is now much more credible
The new behavior-test emphasis is finally pointed at the actual user-visible bug surface rather than only schema validity.

### Analytics contract quality
The analytics contract is **substantially improved**, but not perfect.

What it gets right:
- It distinguishes typing vs non-typing modes.
- It explicitly limits WPM to true typing interactions.
- It makes reading-only system lessons non-analytic, which is correct.
- It states that accuracy-driven modes still update mastery/stats, which fixes the earlier conceptual ambiguity.

What is still slightly weak:
- `ReviewPractice` is described as “与源模式一致,” but the implementation section introduces a single `RecordMode::ReviewPractice`. Those two ideas do not fully match. If review sessions can contain a mixed stream of typing and dictation questions, the code path must preserve enough detail to avoid muddling WPM aggregation semantics.
- The contract says “所有判定类交互都计算 accuracy 并更新 mastery,” which is broadly fine, but mastery semantics across commands vs symbols are still somewhat lumped together at the prose level. That may be acceptable if the data model already separates them, but the plan does not say so explicitly.

This is not a plan-blocking flaw, but it is a real internal-consistency point the team should be careful with during implementation.

### Time-estimate realism
The estimates are **more realistic than v1**, but still a little aggressive.

v1’s “2–3 hours” was unserious. v2’s ~190-minute parallelized timeline is far better because it:
- acknowledges merge overhead,
- separates implementation from integration,
- includes a real behavior-test phase.

That said, three places may still slip:
1. multiline TUI debugging,
2. review-practice state-machine work,
3. Wave 4 content QA across a large TOML surface.

So my judgment is: **improved and plausible for a well-coordinated multi-agent push, but still optimistic on the tail risk**. I would mentally budget closer to half a day if you include integration friction and retesting.

### Model-choice note
The “`codex` (GPT-5.3)” note is still a bit sloppy. It is not plan-fatal, but it remains a minor quality blemish. If the team is delegating behavior-sensitive refactors, model choice should be described more cleanly and consistently.

---

## 3. Remaining Gaps

These are now **secondary gaps**, not primary blockers.

### 1. ReviewPractice analytics semantics remain slightly under-specified
The plan says ReviewPractice should behave “like the source mode,” but its implementation sketch introduces a single `ReviewPractice` record mode. If a review round mixes typing and dictation items, the implementation must not accidentally:
- assign WPM to non-typing review items,
- or collapse all review items into a uniform analytics bucket.

This is fixable in code, but the contract language should be followed carefully.

### 2. Wave 4 safety remediation is explanation-heavy, not example-heavy
The safety pass mostly emphasizes `output_explanation` warnings and hints. That is good, but for some risky examples the stronger pedagogical move is not just “warn harder”; it is also to ensure the example sequence itself models safer behavior where possible. For instance, backup-first or validate-before-restart patterns should be reflected in the educational framing, not only appended as after-the-fact caution text.

### 3. Acceptance criteria for review practice could still be sharper
The plan now defines real interaction, which is a major improvement. But the acceptance bar is still a little light on:
- exercise sourcing policy from weak areas / recent mistakes / topic pools,
- deterministic testability of the mixed question generator,
- progress persistence semantics when a review session is interrupted.

Again: not fatal, but worth watching.

### 4. The plan does not explicitly mention a final coverage-report artifact
It requires running a coverage script, which is good. I would still prefer the integrator to save a concrete final report (before/after counts) so the content-completion claims are auditable rather than merely asserted.

### 5. Schedule remains mildly optimistic
As noted above, the plan is no longer fantasy-level, but it still underestimates the probability that TUI state-machine work and content QA bleed beyond the nominal timeline.

### New gaps introduced by v2?
No serious new architectural gaps were introduced. The revision mostly improves specificity rather than creating fresh risk. The only minor new risk is that the plan now coordinates **many concurrent streams**, so integration discipline becomes more important. Fortunately, v2 also adds a merge protocol, which partly offsets that risk.

---

## 4. Verdict

**Approve Plan**

This revision clears the bar that v1 missed.

Why I approve it now:
- the plan resolves the major product forks instead of punting them into implementation,
- it adds an explicit analytics contract,
- it defines a workable multiline model,
- it raises command-output completeness to a level consistent with the product promise,
- it adds a safety/hardening pass,
- it separates low-risk cleanup from real feature work,
- it substantially strengthens the test strategy,
- and it assigns final integration ownership.

This is no longer just a map of concern areas; it is an execution-capable plan.

My approval is **not** a claim that the implementation will be easy. The team still needs to be disciplined around:
- mixed-mode review analytics,
- risky-content pedagogy,
- and final integration / QA.

But those are execution cautions, not plan-level blockers.

Proceed. Then bring the actual implementation back for review.
