# Timecard Calculator — Design Spec

**Date:** 2026-05-21 (adapted 2026-05-23)
**Status:** Draft

## Overview

Desktop timecard calculator app. Users enter daily time against labor codes and hour types, view aggregated hours rounded to the nearest 15 minutes, and navigate across day/week/pay period views.

## Stack

| Layer | Technology |
|---|---|
| UI framework | Dioxus 0.7 (desktop feature) |
| Styling | TailwindCSS v4 + DaisyUI v5 |
| Routing | Dioxus Router (`#[derive(Routable)]` enum) |
| State | `use_signal` + Context API (`use_context_provider`) |
| Data layer | `packages/api` — repository pattern, direct async SQLite access |
| Database | SQLite via sqlx 0.8 (sqlite, chrono, macros features) |
| Date/time | chrono 0.4 + chrono-tz |
| Build | `dx` CLI (Dioxus CLI) |

## Workspace Layout

`packages/web` and `packages/mobile` are removed — this is a desktop-only app.

```
packages/
  api/                        -- data layer (no server functions, no HTTP)
    src/
      lib.rs                  -- pub use db::*, re-exports all repository types
      db/
        mod.rs                -- static pool, init(), migration runner
        repo.rs               -- Repository struct + all async CRUD methods
        models.rs             -- Rust structs for DB rows / DTOs
    migrations/
      20260521000001_initial_schema.sql
    .cargo/config.toml        -- DATABASE_URL=sqlite::memory: for sqlx macros
  ui/                         -- all Dioxus components and pages
    src/
      lib.rs                  -- pub use App
      app.rs                  -- root App component, context providers, startup load
      routes.rs               -- Route enum
      pages/
        dashboard.rs
        settings.rs
      components/
        layout.rs
        entry_form.rs
        entry_table.rs
    assets/
      tailwind.css            -- @import "tailwindcss"; @plugin "daisyui";
  desktop/
    src/
      main.rs                 -- calls api::db::init(), then dioxus::launch(ui::App)
```

## Database Schema

All tables in a single SQLite file at the platform app data directory.

### `labor_codes`

| Column | Type | Constraints |
|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `wbs_number` | TEXT | NOT NULL, UNIQUE |
| `name` | TEXT | NOT NULL |

### `hour_types`

| Column | Type | Constraints |
|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `code` | TEXT | NOT NULL, UNIQUE (3-char, e.g. "REG", "OVT") |
| `name` | TEXT | NOT NULL |

### `timecard_entries`

| Column | Type | Constraints |
|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `labor_code_id` | INTEGER | NOT NULL, FK → `labor_codes(id)` |
| `hour_type_id` | INTEGER | NOT NULL, FK → `hour_types(id)` |
| `telework` | INTEGER | NOT NULL, DEFAULT 0 (0/1 boolean) |
| `date` | TEXT | NOT NULL (local date `YYYY-MM-DD` for range queries) |
| `start_time` | TEXT | NOT NULL (UTC ISO 8601) |
| `end_time` | TEXT | NULLABLE (UTC ISO 8601) — NULL means in-progress |
| `created_at` | TEXT | NOT NULL, DEFAULT now() |
| `updated_at` | TEXT | NOT NULL, DEFAULT now() |

Indexes: `timecard_entries(date)`, `timecard_entries(labor_code_id)`, `timecard_entries(hour_type_id)`.

Entries never cross midnight. User creates separate entries for each calendar day.

### `pay_period_anchors`

| Column | Type | Constraints |
|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `start_date` | TEXT | NOT NULL, UNIQUE (local date `YYYY-MM-DD`) |

Each anchor defines the start of a 2-week pay period. Periods cascade forward in 14-day increments from each anchor until the next anchor overrides the cadence.

## Hour Rounding

Computed at read time (not stored). Rounds to the **nearest** 15 minutes:

```
minutes = (end_time - start_time).total_seconds() / 60
rounded_minutes = round(minutes / 15) * 15
decimal_hours = rounded_minutes / 60.0
```

Examples:
- 8h 12m = 492m → round to 495m → 8.25
- 4h 30m = 270m → stays 270m → 4.50
- 8h 8m = 488m → round to 480m → 8.00
- NULL end_time → decimal_hours is null; UI shows live elapsed time

## Backend Architecture

### Data Access Pattern

No Tauri IPC, no server functions, no HTTP. Components call `api` repository functions directly inside `use_resource`. The pool is initialized before app launch and stored as a process-wide static in `api::db`.

```rust
// In a Dioxus component:
let pool = use_context::<&'static SqlitePool>();
let entries = use_resource(move || async move {
    api::db::repo::list_timecard_entries(pool, &date.read(), &date.read()).await
});
```

### Repository Pattern

All SQL queries live in `packages/api/src/db/repo.rs`. The `Repository` struct holds a `&'static SqlitePool` obtained via `api::db::pool()` and exposes typed async methods for each operation.

### Repository API

**Labor Codes**
- `list_labor_codes()` → `Result<Vec<LaborCode>, sqlx::Error>`
- `create_labor_code(CreateLaborCode)` → `Result<LaborCode, sqlx::Error>`
- `update_labor_code(UpdateLaborCode)` → `Result<LaborCode, sqlx::Error>`
- `delete_labor_code(id: i64)` → `Result<(), sqlx::Error>`

**Hour Types**
- `list_hour_types`, `create_hour_type`, `update_hour_type`, `delete_hour_type` — same pattern

**Timecard Entries**
- `list_timecard_entries(date_from, date_to)` → `Result<Vec<TimecardEntryView>, sqlx::Error>` — joined view with computed `decimal_hours`
- `create_timecard_entry(CreateTimecardEntry)` → `Result<TimecardEntryView, sqlx::Error>`
- `update_timecard_entry(UpdateTimecardEntry)` → `Result<TimecardEntryView, sqlx::Error>`
- `delete_timecard_entry(id: i64)` → `Result<(), sqlx::Error>`

**Pay Periods**
- `list_pay_period_anchors()` → `Result<Vec<PayPeriodAnchor>, sqlx::Error>`
- `add_pay_period_anchor(start_date)` → `Result<PayPeriodAnchor, sqlx::Error>`
- `remove_pay_period_anchor(id)` → `Result<(), sqlx::Error>`
- `compute_pay_periods(anchors, reference_date)` → `Vec<PayPeriodRange>` (pure, no DB)

**Aggregates**
- `get_day_summary(date)` → `Result<DaySummary, sqlx::Error>`
- `get_week_summary(week_start)` → `Result<WeekSummary, sqlx::Error>`
- `get_pay_period_summary(period_start, period_end)` → `Result<WeekSummary, sqlx::Error>`

**Import**
- `import_lookup_data(labor_codes, hour_types)` → `ImportResult`

### Timezone Handling

- User inputs local Central time strings (`HH:MM`)
- `repo.rs` converts Central → UTC using `chrono-tz::America::Chicago` before storing
- DB stores UTC ISO 8601; read methods return UTC
- Display layer converts UTC → Central using chrono in `ui::components`

### DB Initialization

The pool is initialized in `desktop/src/main.rs` before launching Dioxus:

```rust
fn main() {
    let db_path = dirs::data_dir()
        .unwrap()
        .join("timecard-calc")
        .join("timecard.db");
    api::db::init(db_path);   // blocking: creates DB file, runs migrations
    dioxus::launch(ui::App);
}
```

`api::db::init()` stores the pool in a `once_cell::sync::OnceCell<SqlitePool>`. All repository methods retrieve it via `api::db::pool()`.

## Frontend Architecture

### DaisyUI Styling Convention

Use DaisyUI's **semantic/structural classes** for visual styling (`btn`, `card`, `table`, `modal-box`, `tabs`, etc.) but **never use DaisyUI CSS-state classes** (`dropdown-open`, `drawer-toggle`, `collapse-open`, `modal-open`, `tab-active` via checkbox, etc.). These classes work via CSS sibling-selectors and checkbox hacks that conflict with Dioxus's reactive rendering.

Instead, drive all open/closed/active state with `use_signal`:

```rust
// Good — Dioxus signal controls the class
let open = use_signal(|| false);
rsx! {
    dialog { class: if open() { "modal modal-open" } else { "modal" }, ... }
}

// Avoid — DaisyUI checkbox/sibling-selector pattern
rsx! {
    input { r#type: "checkbox", class: "modal-toggle" }
}
```

This applies to: modals, dropdowns, drawers, collapses, tab-active, and any other DaisyUI component that normally relies on CSS `:checked` or adjacent-sibling state.

### Routes

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(Layout)]
        #[route("/")]
        Dashboard {},
        #[route("/settings")]
        Settings {},
}
```

### Global State (Context API)

Provided at app root via `use_context_provider` in `app.rs`:

- `Signal<Vec<LaborCode>>` — loaded once on startup
- `Signal<Vec<HourType>>` — loaded once on startup
- `Signal<Vec<PayPeriodAnchor>>`
- `Signal<String>` — current date (`YYYY-MM-DD`)
- `Signal<String>` — current week start (`YYYY-MM-DD`)
- `Signal<Option<PayPeriodRange>>` — current pay period

### Dashboard Page (Tabs)

DaisyUI `tabs tabs-lifted` with four panels: Day, Week, Pay Period, History.

- **Day**: entry table for current day. DaisyUI `btn` navigation ±1 day. Add button → DaisyUI `modal` with entry form. Footer shows total hours.
- **Week**: Mon–Fri by default. DaisyUI `toggle` for Sat/Sun (smart: columns hidden if no entries exist for that day regardless of toggle; if entries exist, toggle controls visibility). Navigation ±1 week. Daily subtotals.
- **Pay Period**: 14-day window from current pay period. Navigation ±1 period. Daily + period subtotals.
- **History**: DaisyUI `select` to pick a previous pay period by range label. Read-only table view.

### Entry Form (Modal)

DaisyUI `modal` (`<dialog>` element with `.modal-open`) containing:
- `<select>` for Labor Code (displays `wbs_number — name`)
- `<select>` for Hour Type (displays `code — name`)
- DaisyUI `toggle` for Telework
- `<input type="time">` for Start Time (required)
- `<input type="time">` for End Time (optional — leaves entry in-progress)
- Mode toggle buttons: **Time Inputs** (default) ↔ **Duration** (`<input type="number">` in decimal hours; end time computed from start + duration)

### Settings Page

- **Pay Period Anchors**: list of anchor dates with add/remove. Shows computed period ranges for preview.
- **Labor Codes**: DaisyUI `table` with inline edit/delete, plus "Add" button opening an inline form row.
- **Hour Types**: same pattern.
- **Import JSON**: `<input type="file">` + button. Expects `{ "labor_codes": [...], "hour_types": [...] }`. Upserts on import.

## Error Handling

- Repository errors propagated as `String` to components
- DaisyUI `alert alert-error` shown inline for data errors
- Form validation: required fields (labor code, hour type, start time) checked before submit; show DaisyUI `label` error text
- Delete protection: return error string with reference count if FK constraint would be violated

## Testing

- Rust: unit tests in `packages/api` crate using in-memory SQLite (`sqlite::memory:`)
  - Rounding edge cases (0m, 7m, 8m, 14m, 15m, 22m, 23m)
  - Pay period anchor cascade logic
  - FK constraint enforcement
- UI: manual verification via `dx serve` (desktop build)
