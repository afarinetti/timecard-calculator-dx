# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
just lint          # cargo clippy --all-targets --all-features
just test          # cargo test
dx serve           # run the desktop app with hot-reload (from repo root)
just sqlx-prepare  # regenerate sqlx offline query cache after schema changes
```

Run a single test by name:
```bash
cargo test <test_name>              # e.g. cargo test compute_pay_periods
cargo test -p api <test_name>       # target a specific crate
```

## Architecture

Cargo workspace with three crates:

- **`packages/desktop`** — entry point only; calls `api::db::init(db_path)` then `dioxus::launch(ui::App)`. The SQLite database is stored at `{data_dir}/timecard-calc/timecard.db`.
- **`packages/ui`** — all Dioxus UI: components, pages, routing, and client-side utilities. No database access — calls `api::pool()` directly via `Repository::new(api::pool())` inside `use_resource` closures.
- **`packages/api`** — SQLite persistence via sqlx. Exports `Repository`, all model types, and `db::{init, pool}`.

## Key Patterns

**SQLx offline mode:** `packages/api/.cargo/config.toml` forces `SQLX_OFFLINE=true`. Any time you add or modify a `sqlx::query!` / `sqlx::query_as!` macro call, run `just sqlx-prepare` to update the cache in `packages/api/.sqlx/`. Without this, compilation will fail.

**Time handling:** Times are stored as UTC ISO 8601 in SQLite. The `Repository` converts "HH:MM" strings (Central time, `America/Chicago`) to UTC on write and back on export (`export_entries`). The UI's `utils::utc_to_central_hhmm` uses the local system timezone for display — these will differ if the machine is not in Central time.

**Two-layer entry model:** `TimecardEntryRow` is the raw sqlx row (telework as `i64`). `TimecardEntryView` is the public type with `telework: bool` and the computed `decimal_hours: Option<f64>` (rounded to nearest 15 minutes, `None` for in-progress entries with no `end_time`).

**Context store newtypes:** `CurrentDateSig` and `CurrentWeekSig` are newtype wrappers around `Signal<String>` so both can coexist in Dioxus's context store without TypeId collisions. Use `use_context::<CurrentDateSig>().0` to access the inner signal.

**Signal borrow rules:** Clippy is configured (via `clippy.toml`) to reject `GenerationalRef`, `GenerationalRefMut`, and `WriteLock` held across await points. Derive stat values from signals before `rsx!` so borrows drop before any async code.

## Dioxus Version

This project uses **Dioxus 0.7**. The `cx`/`Scope`/`use_state` API is gone. Use `use_signal`, `use_memo`, `use_resource`, `use_context_provider`, and `use_context`. See `AGENTS.md` for the full Dioxus 0.7 reference.
