# Copilot Instructions

## Commands

```sh
# Development (CSS watch + dx serve in parallel)
pnpm run dev

# Build CSS once
pnpm run build:css

# Serve desktop app only
dx serve

# Run tests
cargo test

# Lint
cargo clippy
```

Run a single test:
```sh
cargo test -p ui navigate_date_forward
```

## Architecture

Cargo workspace with three packages under `packages/`:

- **`api`** — Pure data layer. No Dioxus dependency. Owns the SQLite pool (global `OnceCell<SqlitePool>`), sqlx models, and the `Repository` struct. All DB access goes through `Repository::new(api::pool())`.
- **`ui`** — All Dioxus UI. Depends on `api`. Contains the router, pages, components, and shared utilities.
- **`desktop`** — Thin entry point. Calls `api::db::init(db_path)` then `dioxus::launch(ui::App)`. The DB lives at `dirs::data_dir()/timecard-calc/timecard.db`.

**Data flow:** `desktop/main.rs` initializes the DB → `ui/app.rs` loads lookup tables (`LaborCode`, `HourType`, `PayPeriodAnchor`) into global context signals → pages consume them via `use_context`.

**Routing:** `ui/src/routes.rs` defines the `Route` enum. `Layout` wraps all routes with the sidebar; `Outlet<Route>` renders the active page.

**CSS pipeline:** Tailwind v4 reads `packages/ui/assets/tailwind.css` (with `@source "../src/**/*.rs"`) and outputs `packages/ui/assets/app.css`. DaisyUI v5 is the component plugin with the `dark` theme as default. The compiled CSS must be rebuilt when Tailwind classes change in `.rs` files.

## Key Conventions

**Dioxus 0.7** — No `cx`, `Scope`, or `use_state`. Use `use_signal`, `use_memo`, `use_resource`, `use_context`, `use_context_provider`.

**Repository pattern** — Create per-operation: `Repository::new(api::pool())`. Do not hold a `Repository` across an `await` boundary.

**Time zones** — Times are stored in SQLite as UTC ISO 8601. The `api` layer converts Central "HH:MM" + "YYYY-MM-DD" to UTC on write (`central_to_utc`). The `ui` layer converts back with `utils::utc_to_central_hhmm`. Dates (no time component) are stored as plain `YYYY-MM-DD` local strings.

**Two-layer models** — `TimecardEntryRow` is the raw sqlx row (`telework: i64`, times as UTC strings). `TimecardEntryView` is the public type (`telework: bool`, computed `decimal_hours: Option<f64>`). Never expose `TimecardEntryRow` outside the `api` crate.

**Signal borrow rules** — `clippy.toml` forbids holding `GenerationalRef`, `GenerationalRefMut`, or `WriteLock` across `await` points. Drop signal borrows (`.read()` / `.write()`) before any `.await`.

**Global context signals** — `App` provides five context signals: `Signal<Vec<LaborCode>>`, `Signal<Vec<HourType>>`, `Signal<Vec<PayPeriodAnchor>>`, `Signal<String>` (current date `YYYY-MM-DD`), `Signal<String>` (current week-start Monday `YYYY-MM-DD`). Components consume them with `use_context::<Signal<T>>()`.

**Reload pattern** — Components trigger a data refresh by incrementing a `reload: Signal<u32>` that `use_resource` closures capture, causing them to re-run.

**Styling** — Custom utility classes (`pd-stat-value`, `pd-nav-active`, `pd-nav-inactive`, etc.) are defined in `packages/ui/assets/precision-dark.css`. Inline Tailwind classes are used directly in `rsx!`. DaisyUI component classes (`btn`, `alert`, `loading`) are used for interactive elements.

**Assets** — Configured in `Dioxus.toml` with `asset_dir = "packages/ui/assets"`. Reference assets with `asset!("/assets/filename")`.
