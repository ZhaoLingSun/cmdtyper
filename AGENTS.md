# Repository Guidelines

## Project Structure & Module Organization
`cmdtyper` is a Rust TUI application. Core runtime code lives in `src/`: `app.rs` owns state, `event.rs` handles input polling, `core/` contains typing and scoring logic, `flow/` drives user journeys, `ui/` renders screens, and `data/` loads models and persisted progress. Integration tests live in `tests/`. Learning content is data-driven under `data/commands/`, `data/lessons/`, `data/symbols/`, and `data/system/`. Keep new content in the matching directory and preserve stable IDs.

## Build, Test, and Development Commands
Use the standard Cargo workflow:

- `cargo run` starts the TUI in the current terminal.
- `cargo build --release` builds the production binary at `target/release/cmdtyper`.
- `cargo test` runs the full integration and behavior suite.
- `cargo test --test parse_all` validates every TOML content file deserializes cleanly.
- `cargo test --test tokens_consistency` checks token text concatenation matches each command string.
- `docker compose build && docker compose run --rm cmdtyper` builds and runs the containerized app.
- `./scripts/install.sh` builds the release binary and installs it to `~/.local/bin/cmdtyper`.

## Coding Style & Naming Conventions
Follow idiomatic Rust with 4-space indentation and `rustfmt` defaults. Use `snake_case` for functions, modules, and tests, `PascalCase` for structs/enums, and short, explicit names for state transitions. Prefer small loader/rendering helpers over large mixed-purpose functions. For content files, use lowercase underscore-separated filenames such as `pipe_redirect.toml`. Keep command/topic IDs unique and stable because tests and saved progress depend on them.

## Testing Guidelines
Add or update integration tests in `tests/` for behavior changes. Test names should describe the expected outcome, for example `typing_enter_is_ignored_when_incomplete`. When changing data files, run `cargo test --test parse_all`, `cargo test --test tokens_consistency`, and `cargo test --test id_uniqueness` before opening a PR. Keep tests deterministic and isolate environment-variable mutations the way current behavior tests do.

## Commit & Pull Request Guidelines
Recent history uses Conventional Commit prefixes: `feat:`, `fix:`, `docs:`, and `chore:`. Keep subjects imperative and specific, for example `fix: preserve resume state after mode switch`. PRs should explain user-visible behavior changes, list affected data directories or modules, and include terminal screenshots or short recordings for TUI updates. Mention the exact test commands you ran.

## Safety & Configuration Notes
This project must remain data-driven and must not execute real shell commands. Respect `CMDTYPER_DATA_DIR` and `CMDTYPER_USER_DIR` when touching loading or persistence logic, and keep Docker defaults aligned with those paths.
