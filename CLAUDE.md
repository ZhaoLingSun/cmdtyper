# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

cmdtyper is a Rust TUI app (ratatui + crossterm) that teaches the Linux command line through typing practice, dictation, and structured lessons. UI text and learning content are in Chinese.

**Hard safety invariant: the app never executes real shell commands.** All terminal "output" shown to the user is preset content from TOML data files. Do not introduce any shell-execution path.

## Commands

```bash
cargo run                              # start the TUI (needs a real terminal)
cargo build --release                  # production binary at target/release/cmdtyper
cargo test                             # full suite
cargo test --test parse_all            # every TOML content file deserializes cleanly
cargo test --test tokens_consistency   # token text concatenation == command string
cargo test --test id_uniqueness        # command/topic IDs are unique
cargo test --test behavior test_name   # single test in one integration test file
./scripts/install.sh                   # build release + install to ~/.local/bin/cmdtyper
python3 scripts/audit_tokens.py        # audit token/lexicon coverage of content files
docker compose build && docker compose run --rm cmdtyper
```

Tests load data via the relative path `data/`, so run them from the repo root. When changing anything under `data/`, run `parse_all`, `tokens_consistency`, and `id_uniqueness` before opening a PR.

Formatting is rustfmt defaults. Commits use Conventional Commit prefixes (`feat:`, `fix:`, `docs:`, `chore:`).

## Architecture

The app is a single state machine keyed on the `AppState` enum in `src/app.rs`. Three dispatch points switch on it, and all three must be updated together when adding or changing a screen:

1. `AppState` enum variant (screen identity + per-screen params like indices/phase, `src/app.rs`)
2. `App::handle_key` in `src/app.rs` → delegates to a handler in `src/flow/*`
3. `ui::render` in `src/ui/mod.rs` → delegates to a renderer in `src/ui/*`

Module roles:

- `src/app.rs` — `App` struct holds *all* mutable state (loaded content, user stats/config, per-mode practice state, menu indices). Flow handlers mutate `App` directly.
- `src/flow/` — key-event handlers per user journey (typing, lesson, symbol, system, review). Business logic for state transitions lives here, not in `ui/`.
- `src/ui/` — pure render functions `fn render(frame, app, ...)`; read-only over `App`.
- `src/core/` — mode-independent logic: `TypingEngine` (per-char typing state), `matcher` (dictation answer matching/normalization/diff), `scorer` (WPM/accuracy), `timer`, `terminal_history`.
- `src/data/` — serde models (`models.rs`), one TOML loader per content type (commands, lessons, symbols, system, practice groups, sequences, scenarios), `progress.rs` (JSON persistence), and `lexicon.rs` (static built-in token descriptions used as fallback when a lesson lacks `token_details`).
- `src/main.rs` — terminal setup/teardown and the event loop; `src/event.rs` polls with a 50ms tick.

### Data and persistence paths

- Content dir resolution (`App::detect_data_dir`): `CMDTYPER_DATA_DIR` → `~/.local/share/cmdtyper/data` → `/usr/local/share/cmdtyper/data` → `./data`. A candidate must contain `commands/`, `lessons/`, `symbols/`, and `system/`.
- User data (`ProgressStore`): `CMDTYPER_USER_DIR`, else `~/.local/share/cmdtyper/`. Files include `stats.json`, `history.json`, `config.json`, `resume_state.json`, and `scenario_progress.json`. Typing records go through `App::persist_record`: durable history first, then a rebuildable aggregate cache. A failed history write must prevent navigation that discards the active session. Corrupt history is an error during migration or update; never replace it with an empty default. Migrations publish complete backups atomically before changing originals; preserve old fields and stable IDs. See `docs/statistics.md` and compatibility/migration tests.
- The Dockerfile sets both env vars (`/usr/local/share/cmdtyper/data`, `/userdata` volume); keep Docker defaults aligned when touching load/persist logic.

### Content files (`data/`)

- `commands/*.toml` — typing/dictation question bank; `lessons/*.toml` — per-command lessons; `symbols/*.toml` — symbol topics; `system/*.toml` — system-architecture topics; `reviews/<topic>.toml` — v0.3 review exercises; `syntax/` — token/syntax reference data.
- `practice/` defines one teaching command and three exercises per foundational use; `sequences/` defines ordered steps with contextual output, explanation and prompt; `scenarios/` defines cases with evidence and decisions. `command_aliases.toml` migrates historical IDs, and `command_contexts.toml` provides SQL prompts. Shared canonical command definitions belong in `commands/`; references retain their own contextual teaching text.
- `workflow_flow` wraps the full-typing entrypoints to submit each step and exclude output reading from timing while retaining one stable session. Update its behavior tests when changing parent screen transitions.
- Command and topic IDs must stay unique and stable — saved user progress and tests depend on them. Filenames are lowercase underscore-separated.
- In command entries, the concatenated token texts must exactly reproduce the command string (`tokens_consistency` enforces this).

### Testing conventions

Integration tests live in `tests/` (`behavior.rs`, `v03_behavior.rs` for key-handling behavior; `compat.rs` for backward compatibility of data/config formats). Name tests after the expected outcome, e.g. `typing_enter_is_ignored_when_incomplete`. Keep tests deterministic and isolate env-var mutations the way existing behavior tests do (they set/restore `CMDTYPER_USER_DIR` around temp dirs).

## PR expectations

Explain user-visible behavior changes, list affected data directories or modules, include terminal screenshots or recordings for TUI changes, and mention the exact test commands run.
