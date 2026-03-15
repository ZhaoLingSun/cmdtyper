# cmdtyper-v2 v0.3 Socrates Engineering Review (2026-03-14)

## 1. Feature Verification Matrix

| Feature | Status | Evidence | Verdict |
|---|---|---|---|
| Wave A: `app.rs` refactor + flow modules + schema extension | ⚠️ | `src/flow/*.rs` cleanly owns typing/lesson/symbol/system/review flows; `src/data/models.rs` adds `TypingDisplayMode`, `ExerciseKind`, `deep_explanation`, `RecordMode` variants | Refactor is directionally good, but `src/app.rs` is still **1022 lines** and remains the central god-object with a very large mutable state surface |
| Three-mode typing (`Terminal / Standard / Detailed`, `M`) | ⚠️ | Mode cycle implemented in `src/flow/typing_flow.rs:93-101`; rendering split in `src/ui/typing.rs` | Mode switching exists, but **Terminal mode output is not actually visible** due to immediate finalize-after-sleep logic; wrapped `display` strings also desync cursor/highlight mapping |
| Text wrap fix | ⚠️ | `map_display_lines()` + multi-line rendering in `src/ui/typing.rs:385-452` | Better than single-line-only, but still incorrect when `display != command` (inserted backslashes/indentation consume cursor mapping) |
| Difficulty + Category dual filter | ✅ | `filtered_commands()` and `current_filter_match_count()` in `src/app.rs:942-960`; filter UI in `src/ui/filter_select.rs` | Implementation is straightforward and correct |
| Learn Hub graded quick-start entries | ✅ | `src/ui/learn_hub.rs:7-16`, `src/app.rs:488-493` | Beginner/Basic/Advanced/Practical shortcuts correctly jump into filtered typing |
| Deep explanation framework (`D`, scrolling viewer, formatting) | ⚠️ | `DeepSource` model + viewer in `src/ui/deep_explanation.rs`, openers in lesson/symbol/system flows | Framework works and data loads, but scroll is unclamped and content is currently only populated in command lessons (5 cases) |
| 5 flagship deep explanations | ✅ | Count verified from `data/lessons/*.toml`; present in `cat/cp/find/sed/xargs` | Quantity matches claim |
| Symbol typing practice (Typing + Dictation) | ✅ | `start_symbol_practice()` splits by `ExerciseKind`; typing/dictation flows in `src/flow/symbol_flow.rs` | Flow integration is sound; per-typing stats are recorded with WPM-bearing `RecordMode::SymbolTyping` |
| System command typing practice | ✅ | `enter_system_typing()` and `handle_system_typing_key()` in `src/flow/system_flow.rs` | Integrated cleanly; per-command stats use `RecordMode::SystemTyping` |
| 60 new symbol exercises (90 total) | ⚠️ | Count verified: 90 total across 6 symbol files | Count is correct, but at least one answer variant is syntactically invalid and some content still needs QA polish |
| 12 new behavior tests | ⚠️ | `tests/v03_behavior.rs` has 12 tests and all pass | Tests cover parsing/defaults/filtering/scorer policy, but they miss the main interaction regressions introduced in v0.3 |
| README update | ⚠️ | `README.md` documents v0.3 features well | Mostly aligned, but package metadata still says `version = "0.2.0"` in `Cargo.toml`, and README overstates readiness because clippy still fails |
| Build / Test / Clippy | ⚠️ | `cargo build` ✅, `cargo test` ✅, `cargo clippy --all-targets --all-features -- -D warnings` ❌ | Release gate not fully green |

## 2. Architecture Assessment

### What improved

1. **Flow extraction was the right move.**
   - `src/flow/typing_flow.rs`
   - `src/flow/lesson_flow.rs`
   - `src/flow/symbol_flow.rs`
   - `src/flow/system_flow.rs`
   - `src/flow/review_flow.rs`

   These modules now hold the phase-specific key handling and progression logic. That is materially better than keeping all navigation/state transitions inside a monolithic `app.rs`.

2. **Schema evolution is coherent.**
   `src/data/models.rs` cleanly adds:
   - `TypingDisplayMode`
   - `ExerciseKind`
   - `deep_explanation` on lesson/symbol/system content
   - new `RecordMode` variants for symbol/system/review stats

   Backward compatibility was considered via `#[serde(default)]` and enum aliases.

3. **State machine shape is understandable.**
   `AppState`, `SymbolPhase`, `SystemPhase`, `ReviewPhase`, and `DeepSource` are explicit and readable. This is good TUI architecture for a project of this size.

### What is still weak

1. **`src/app.rs` is still too large and too stateful.**
   It is not ~700 lines now; it is **1022 lines** in the reviewed tree. The file still mixes:
   - app construction
   - global dispatch
   - home/filter/learn hub handlers
   - deep explanation navigation
   - filtering helpers
   - various state accessors

   The refactor reduced risk, but did not really finish the decomposition.

2. **`App` remains a god-struct.**
   It owns all domain data, all UI indices, all practice-state variants, typing engine state, persistence handles, and ephemeral round data. This is manageable today but brittle for v0.4+.

3. **Cross-module coupling still routes through `App` internals.**
   Flow modules mutate many fields directly. That is acceptable in Rust for a small TUI, but the next refactor step should be extracting dedicated sub-state structs (`TypingSession`, `FilterState`, `LessonNavState`, `SymbolPracticeState`, `SystemLessonState`) and giving flows narrower APIs.

### Architecture verdict

**Minor revision, not rejection.**
The refactor is clearly better than the pre-v0.3 monolith, but `app.rs` is not yet genuinely “small and maintainable.” It is “less bad,” not “done.”

## 3. Content Quality Spot-Check

### Deep explanations spot-check

#### A. `data/lessons/find.toml` — `find /tmp -type f -mtime +30 -exec rm -v {} \;`
**Verdict: Good**
- Explains the real-world scenario well
- Walks through `find` predicate logic clearly
- Includes safety framing (“remove `-exec` first to confirm match range”)
- This is the strongest of the sampled deep explanations

#### B. `data/lessons/sed.toml` — `sed -i.bak 's/localhost/db.production.internal/g' config.yml`
**Verdict: Good deep explanation, but adjacent example metadata is polluted**
- The deep explanation itself is sensible and practical
- However, nearby token explanations in the same lesson are wrong (see Issues section), which undermines trust in the lesson as a whole

#### C. `data/lessons/xargs.toml` — `cat urls.txt | xargs -n 1 -P 4 wget -q`
**Verdict: Good concept coverage**
- Correctly explains stdin → argv transformation, batching, and parallelism
- Practical enough for intermediate users
- Could be improved with a stronger warning about idempotency / server load / retries when using `-P`

### Symbol exercise spot-check

I spot-checked 10 exercises across `pipe_redirect.toml`, `quotes_escape.toml`, and `variables.toml`.

#### Strong examples
1. `cat auth.log | grep -i failed | head -5`
2. `dmesg | grep -i error >> kernel.err`
3. `./deploy.sh > deploy.log 2>&1`
4. `ping -c 3 8.8.8.8 | tee ping.out`
5. `some_command 2>> err.log`
6. `sort < list.txt > result.txt`
7. `echo '$HOME'`
8. `echo "user: $USER"`
9. `echo "$USER:$HOME"`
10. `TODAY=$(date +%F) && echo $TODAY`

These are generally practical, appropriately scoped, and consistent with the teaching goals.

#### Content defects found
1. **Invalid answer variant** in `data/symbols/variables.toml`:
   - `sleep 300 &; echo $!`
   - This is invalid shell syntax (`&;`)

2. **Lesson token-detail quality regression** in multiple command lessons:
   - `sed -i` described as “启用交互确认，覆盖/删除前先询问”
   - sed substitution body described as SSH private key path
   - Same pattern reappears in `xargs ... sed -i 's/.../.../g'`

This suggests some token-level explanations were mass-generated/copied without command-specific review.

### Overall content verdict

**Concept coverage is strong; fine-grained QA is not finished.**
The new content is genuinely useful, but the token-detail defects are serious because they directly teach the wrong thing.

## 4. New Issues Found

### 1) Terminal mode simulated output is effectively invisible
**Severity:** High

**Code path:**
- `src/flow/typing_flow.rs:122-128`
- `src/ui/typing.rs:63-72`

In Terminal mode, when a command has simulated output:
1. `app.typing_showing_output = true`
2. `thread::sleep(Duration::from_millis(450))`
3. `typing_finalize_current_command(app)` immediately advances index and clears state

But the UI only renders output while:
- `typing_showing_output == true`
- current command is still the just-finished one

Because finalize happens in the same event cycle, the output screen is never actually presented to the user. The code claims “auto continue,” but in practice it auto-skips the output.

**Impact:** one of the headline v0.3 features is not functioning in Terminal mode.

---

### 2) Wrapped `display` text breaks cursor/token alignment
**Severity:** High

**Code path:**
- `src/ui/typing.rs:385-445`

`render_current_command_lines()` maps the **display string** character-by-character onto the typing engine target length. That works only when `display == command` modulo raw newlines.

It fails when `display` inserts presentation-only characters, e.g.:
- `data/commands/04_practical.toml`:
  - command: `tail -f /var/log/nginx/access.log | grep 500`
  - display: `tail -f /var/log/nginx/access.log | \
  grep 500`
- many lesson examples also insert `\` and indentation for pretty wrapping

Those visual backslashes/spaces are not part of the typed target, but they still consume mapping indices. Result: cursor/highlight/progress becomes desynchronized.

**Impact:** the “wrap fix” is incomplete and especially undermines Detailed mode, where visual precision matters most.

---

### 3) Token explanations are factually wrong in some flagship lessons
**Severity:** High

**Examples:**
- `data/lessons/sed.toml:144-150`
  - `-i` is explained as interactive confirm (wrong; that sounds like `rm -i`)
  - `s/PasswordAuthentication yes/.../` is explained as SSH private key path (nonsense)
- `data/lessons/xargs.toml:333-343`
  - repeated sed-token corruption (`-i`, substitution body)

**Impact:** directly teaches incorrect command semantics.

---

### 4) `cargo clippy -D warnings` fails
**Severity:** Medium

**Findings:**
- `src/app.rs:949,950,958,959` → `unnecessary_map_or`
- `src/ui/filter_select.rs:130` → `needless_range_loop`

This is small to fix, but release gating is not green.

---

### 5) Version metadata not updated for v0.3
**Severity:** Medium

**Evidence:** `Cargo.toml` still says:
```toml
version = "0.2.0"
```
while README and feature set are clearly branding this as v0.3.

---

### 6) v0.3 tests miss the real failure modes
**Severity:** Medium

`tests/v03_behavior.rs` is useful as compatibility insurance, but it does **not** test:
- Terminal mode output visibility
- wrapped display cursor alignment
- Detailed mode token-source correctness
- `D` key deep explanation opening/return/navigation
- Learn Hub quick-start routing
- symbol/system flow progression via key events
- symbol/system stat recording persistence

This is why the most important regressions survived.

---

### 7) Deep explanation scrolling is not clamped
**Severity:** Low

`src/app.rs:816-831` allows unlimited `scroll.saturating_add(...)` without content-height clamp. This is harmless but sloppy; users can scroll into blank space indefinitely.

---

### 8) Filter UX lacks explicit empty-result handling
**Severity:** Low

`enter_typing_filtered()` returns early if the filtered set is empty, but gives no dedicated warning state. The match count is shown, so this is not a correctness bug—just a UX rough edge.

## 5. Build / Test / Clippy Status

### Build
```bash
cargo build
```
**Status:** ✅ Pass

### Tests
```bash
cargo test
```
**Status:** ✅ Pass

Highlights:
- core/unit tests: pass
- compatibility tests: pass
- parse-all tests: pass
- `tests/v03_behavior.rs` 12/12 pass

### Clippy
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
**Status:** ❌ Fail

Current blockers:
- `clippy::unnecessary_map_or` in `src/app.rs`
- `clippy::needless_range_loop` in `src/ui/filter_select.rs`

## 6. Overall Assessment

**Decision: Minor Revision**

This is **not** a reject. The v0.3 work is substantial and mostly real:
- modularization improved the project
- filtering works
- symbol/system practice is integrated
- deep-explanation framework exists
- data volume and coverage are materially better
- build/tests are green

But this is also **not yet acceptable as a clean gate pass** because two user-facing quality issues are too central:
1. Terminal mode’s output feedback is effectively broken
2. Some lesson token explanations are simply wrong

And release engineering is incomplete because clippy still fails and Cargo metadata still advertises v0.2.0.

## 7. Recommendations

### Must fix before final v0.3 sign-off
1. **Fix Terminal mode rendering lifecycle**
   - Do not sleep-and-finalize in one handler pass
   - Instead introduce an intermediate “show output until next tick / deadline / next event” state
   - Render at least one frame with the output visible before advancing

2. **Fix wrapped display mapping**
   - Separate *presentation layout* from *target character indexing*
   - Presentation-only characters (`\`, indentation padding) must not consume engine cursor indices
   - Add a regression test for a wrapped command with `display != command`

3. **Correct polluted token explanations**
   - Audit at least `sed.toml` and `xargs.toml`
   - Search for suspicious generic explanations such as:
     - “启用交互确认”
     - “指定私钥文件路径”
   - Manually review all newly added token-detail blocks, not just the flagged examples

4. **Make clippy pass**
   - This is easy and should be mandatory for a release tag

5. **Update package version to v0.3.0**
   - `Cargo.toml`
   - any lockstep docs/version badges if present

### Strongly recommended
6. **Add real interaction tests** for:
   - M-key cycle persistence
   - D-key open/close and source return
   - Terminal mode output visibility
   - wrapped display cursor correctness
   - symbol/system progression + record persistence

7. **Finish the refactor**
   - push more non-core handlers out of `app.rs`
   - reduce direct field mutation from flow modules where practical

8. **Run a content QA sweep on all new exercises/examples**
   - one invalid shell answer (`sleep 300 &; echo $!`) already slipped through
   - token-details clearly need human review

9. **Clamp deep-explanation scroll**
   - small polish, but worth doing

---

## Final Gate Summary

- **Engineering direction:** good
- **Feature completeness:** mostly real
- **Teaching quality:** promising but not fully trustworthy yet
- **Release readiness:** **not final until Minor Revision items are fixed**
