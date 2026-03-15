# Socrates Re-Audit — cmdtyper-v2 (2026-03-14)

## 1. Fix Verification Matrix

| Original issue | Re-verification | Notes |
|---|---|---|
| 1. Docker/runtime data path broken by design | ✅ Fixed | `App::new()` now reads `CMDTYPER_DATA_DIR` with fallback `./data` (`src/app.rs:194-199`). Docker now bundles course data at `/usr/local/share/cmdtyper/data` and sets `CMDTYPER_DATA_DIR=/usr/local/share/cmdtyper/data`, while user data is separated into `CMDTYPER_USER_DIR=/userdata` (`Dockerfile:16-22`, `src/data/progress.rs:20-29`, `docker-compose.yml`). This closes the original packaging mismatch. |
| 2. Flagship typing flow did not match README (no Enter submit, no simulated output, display ignored) | ✅ Fixed | Typing mode now requires `Enter` to submit completed input (`src/app.rs:371-426`). Simulated output is rendered before advancing when present (`src/ui/typing.rs:24-37`, `56-99`). Active rendering uses `cmd.display_text()` rather than raw command text (`src/ui/typing.rs:43-50`). README now describes the real two-stage lesson flow instead of the old overclaim (`README.md:45-58`). |
| 3. Progress/statistics integrity wrong (difficulty defaulted to Beginner; dictation/lesson practice bypassed stats) | ⚠️ Partially Fixed | `TypingEngine::finish()` now takes explicit `difficulty` + `RecordMode` and preserves both (`src/core/engine.rs:146-186`). Dictation and lesson practice now persist stats/history with the correct difficulty (`src/app.rs:683-720`, `1503-1519`). **But review practice still bypasses `finish()` entirely, synthesizes aggregate records with `difficulty: Beginner`, and does not record per-command progress** (`src/app.rs:1331-1394`). That means the original stats-integrity problem is not fully solved. |
| 4. Symbol/review practice were placeholders | ⚠️ Partially Fixed | Symbol practice is now real: hidden answers, typed input, matcher-based grading, retry loop, 3-error auto-advance, and accuracy recording (`src/app.rs:890-1019`, `src/ui/symbol_lesson.rs:120-212`). Review practice is no longer a stub: it generates actual typing/dictation exercises from source content and collects results (`src/app.rs:1248-1394`, `src/ui/review.rs:103-220`). However, its stats pipeline is still structurally wrong (see issue 3). |
| 5. Typing UI multiline support was cosmetic/poor | ✅ Fixed | Multiline `display` text is now mapped into per-line character positions with a continuous cursor model via `map_display_lines`, and both completed/current commands render across lines correctly (`src/ui/typing.rs:148-209`). This is the right fix for the earlier “only first line active” defect. |
| 6. End-of-round state machine inconsistent; dead RoundResult screen | ✅ Fixed | The old `RoundResult` path is gone. Typing completion now uses `typing_is_finished()` + inline round summary (`src/ui/typing.rs:22-23`, `101-131`), and there are no remaining `RoundResult` references in `src/`, `tests/`, `README.md`, or `data/`. The state machine is cleaner than before, even if the summary is now embedded rather than a separate screen. |
| 7. Command-learning flow did not match documented 3-stage model | ✅ Fixed | This was corrected by bringing documentation back to reality rather than inventing an unnecessary demo state. README now says command lessons are **two stages**: `Overview -> Practice` (`README.md:54-58`), which matches the current implementation. |
| 8. Settings UI falsely suggested username/hostname editability | ✅ Fixed | Username/hostname are now explicitly rendered as read-only (`selectable: false`) and excluded from settings navigation/edit count (`src/ui/settings.rs:67-79`, `82-114`; `src/app.rs:1551-1564`). |
| 9. Data/content completeness far below product narrative | ⚠️ Partially Fixed | Coverage improved dramatically: I measured **31 lesson files**, **186 lesson examples**, **186/186** lesson examples with `simulated_output`, **186/186** with `output_explanation`, and **273/273** command entries with `simulated_output`. But the token-detail claim is still false: only **173/186** lesson examples have `token_details`, and only **26/31** lesson files have token details for all examples. Missing examples remain in `cat.toml`, `find.toml`, `grep.toml`, `ls.toml`, `top.toml`. |
| 10. Tests ignored behavioral correctness | ⚠️ Partially Fixed | New behavioral tests now cover Enter gating, backspace, WPM freeze, env-driven data path, matcher behavior, and WPM policy (`tests/behavior.rs`). Good improvement. But there are still **no behavior tests for review practice**, **no tests for symbol practice transitions**, and therefore the remaining review-stats defect slipped through. |

## 2. New Issues Found

### 2.1 Review-practice statistics are still wrong
This is the main remaining correctness issue.

1. **Hardcoded Beginner difficulty**  
   `record_review_stats()` constructs synthetic `SessionRecord`s with `difficulty: Difficulty::Beginner` regardless of the underlying exercises (`src/app.rs:1362`, `1385`). That directly violates the stated repair goal that stats should use the real difficulty.

2. **Review dictation sessions still pollute WPM aggregation**  
   Both synthesized review records use `mode: RecordMode::ReviewPractice` (`src/app.rs:1353`, `1376`). In `scorer::update_stats()`, `ReviewPractice` is treated as a WPM-bearing mode, so even the dictation aggregate with `wpm = 0.0` increments `total_wpm_sessions` and drags down `overall_avg_wpm` (`src/core/scorer.rs:10-31`). This is analytically wrong.

3. **Review practice does not reinforce per-command mastery**  
   Review records use synthetic IDs like `review:category:...:typing` rather than the underlying command IDs (`src/app.rs:1354`, `1377`). That means review practice improves neither command-level mastery nor difficulty-specific progress for the actual commands being reviewed.

### 2.2 Content-completeness claim is overstated, and a few token explanations are still wrong
The broad coverage pass happened, but not cleanly.

- `data/lessons/ls.toml` contains obviously incorrect token explanations in the `ls -la /etc | grep -i net` example:  
  - token `-i` is explained as “启用交互确认，覆盖/删除前先询问” (an `rm/cp/mv` explanation)  
  - token `net` is explained as “指定私钥文件路径，用该密钥进行 SSH 身份认证”  
  These are copy-paste content defects, not nitpicks.
- `data/lessons/curl.toml` still has generic or incorrect token explanations such as describing `-d` as “通常用于后台运行、延迟间隔或指定目录（看命令上下文）” and describing a URL as a size/parameter field in another example.
- Five lesson files still have missing token details for some examples: `cat.toml`, `find.toml`, `grep.toml`, `ls.toml`, `top.toml`.

### 2.3 Risky SSH-hardening practice commands remain under-guarded
`data/system/config_files.toml` still teaches direct `sed -i ... && systemctl restart sshd` edits for SSH configuration without first validating config syntax or preserving a backup. The prose warns about lockout, but the practice commands still model the riskier operational pattern.

## 3. Content Quality Spot-Check

I spot-checked four representative content files.

### A. `data/lessons/cat.toml` — **Mostly good, but not fully completed**
- Strengths: outputs are plausible; explanations are coherent; pipeline example is pedagogically useful.
- Weakness: examples 1, 2, and 5 still lack `token_details`, so the file is not actually “fully token-detailed.”
- Verdict: **Usable, but incomplete**.

### B. `data/lessons/ls.toml` — **Contains clear factual mistakes**
- Example `ls -la /etc | grep -i net` has broken token explanations:
  - `-i` explanation belongs to interactive file operations, not `grep -i`
  - `net` explanation is unrelated SSH-key text
- This indicates a copy/paste QA failure during the enrichment pass.
- Verdict: **Needs correction before claiming content polish**.

### C. `data/lessons/curl.toml` — **Coverage added, but some explanations are low-fidelity**
- Positive: multiline `display` is present where needed; sample outputs look realistic enough for educational UI use.
- Problems: several token explanations are generic or wrong, e.g. `-d` described with unrelated semantics; one URL token described as a size/parameter field.
- Verdict: **Improved materially, but not yet trustworthy line-by-line**.

### D. `data/system/config_files.toml` — **Technically valuable, operationally under-defended**
- Positive: the file now contains concrete before/after config snippets and practical commands.
- Concern: SSH lessons still normalize risky direct-edit-and-restart patterns (`sed -i ... && systemctl restart sshd`) without `sshd -t`, staged rollout, or backup-first guidance in the command itself.
- Verdict: **Good curriculum ambition, still too cavalier operationally**.

### E. `data/symbols/quotes_escape.toml` — **Strong**
- The symbol explanations are detailed, internally consistent, and the examples are realistic.
- This file is closer to release quality than the weakest command-lesson files.
- Verdict: **Good**.

## 4. Build/Test/Clippy Status

### Verified commands
- `cargo build --release` — **passed**
- `cargo test` — **passed**
  - 80 unit tests in main target passed
  - 9 tests in `tests/behavior.rs` passed
  - parse / ID / token consistency suites passed
- `cargo clippy --all-targets --all-features -- -D warnings` — **passed**

### Notes
- `#![allow(dead_code)]` has indeed been removed from `src/data/models.rs`.
- I attempted a full Docker build as an extra packaging check, but the pull of the Rust base image from Docker Hub on intranet stalled on network/image fetch. Static verification of Dockerfile + compose wiring is sound, and the original data-path defect itself is fixed in code and image configuration.

## 5. Overall Assessment

**Minor Revision**

This is a substantial improvement over the original review. The previous release-blocking issues around Docker data resolution, core typing semantics, simulated output rendering, multiline typing behavior, symbol practice, settings honesty, README drift, and general test coverage have been addressed well enough that the project now looks like a real v0.2 candidate rather than a prototype with inflated claims.

However, I am **not** giving full Accept yet, for two reasons:

1. **Review-practice analytics are still wrong in a way that directly contradicts the repair plan.** The code still synthesizes aggregate review records with fake Beginner difficulty and lets dictation review sessions distort WPM statistics.
2. **The content pass was broad but not fully quality-controlled.** The completeness claim for token details is false, and at least two lesson files (`ls.toml`, `curl.toml`) still contain plainly incorrect token explanations.

This is no longer a “major revision” situation. It is now a focused cleanup pass.

## 6. Remaining Recommendations

1. **Fix review-practice recording properly**  
   Do not synthesize coarse aggregate records with fake difficulty. Record per exercise, or at minimum preserve the real exercise difficulty and split typing vs dictation into modes whose stat semantics match reality.

2. **Stop counting review dictation as WPM-bearing `ReviewPractice`**  
   Either introduce distinct review submodes or make WPM aggregation conditional on exercise kind rather than broad mode.

3. **Make review practice update the underlying command/topic mastery**  
   If a learner reviews `grep`, that should reinforce `grep`, not only a synthetic `review:...` bucket.

4. **Run a content QA sweep specifically for token explanations**  
   Start with `ls.toml` and `curl.toml`, then clean the remaining five incomplete lesson files.

5. **Correct the public completeness claim**  
   Until token details truly cover all 186 lesson examples, do not claim `31/31` lesson completion in that dimension.

6. **Add behavior tests for review/symbol workflows**  
   Minimum needed: review typing record mode/difficulty, review dictation WPM policy, symbol 3-error auto-reveal/advance, and progression/history persistence.

