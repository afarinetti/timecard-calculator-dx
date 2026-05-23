# Timecard Calculator — Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `packages/api` — SQLite via sqlx, repository pattern, all CRUD/aggregate/import/export methods — and wire into `packages/desktop` as the app entry point.

**Architecture:** Process-wide `once_cell::sync::OnceCell<SqlitePool>` initialized synchronously before `dioxus::launch`. Migrations embedded at compile time. No HTTP, no IPC — repository methods are plain `async fn`s called directly by Dioxus components via `use_resource`.

**Tech Stack:** Rust 2021, sqlx 0.8 (sqlite, macros, chrono features), chrono 0.4, chrono-tz 0.9, once_cell, dirs, serde/serde_json

---

## File Map

| File | Role |
|---|---|
| `Cargo.toml` | Workspace root — add deps, remove web/mobile |
| `packages/api/Cargo.toml` | API crate deps |
| `packages/api/.cargo/config.toml` | `DATABASE_URL` for sqlx compile-time macros |
| `packages/api/migrations/20260521000001_initial_schema.sql` | All four tables + indexes |
| `packages/api/src/lib.rs` | Public re-exports |
| `packages/api/src/db/mod.rs` | Static pool, `MIGRATOR`, `init()`, `pool()` |
| `packages/api/src/db/models.rs` | All Rust structs (no logic) |
| `packages/api/src/db/repo.rs` | `compute_decimal_hours`, `central_to_utc`, `Repository` + all methods |
| `packages/desktop/Cargo.toml` | Desktop crate deps (Dioxus desktop feature) |
| `packages/desktop/src/main.rs` | `main()` — DB init + `dioxus::launch` |
| `packages/ui/Cargo.toml` | UI crate deps (add rfd, serde_json) |

---

### Task 1: Workspace Cargo.toml — add deps, remove web/mobile

**Files:**
- Modify: `Cargo.toml`
- Modify: `packages/api/Cargo.toml`
- Modify: `packages/desktop/Cargo.toml`
- Modify: `packages/ui/Cargo.toml`

- [ ] **Step 1: Replace `Cargo.toml` (workspace root)**

```toml
[workspace]
resolver = "2"
members = [
    "packages/ui",
    "packages/desktop",
    "packages/api",
]

[workspace.dependencies]
dioxus = { version = "0.7.1" }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "chrono", "macros"] }
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
once_cell = "1"
dirs = "5"
tokio = { version = "1", features = ["full"] }
rfd = "0.15"

ui = { path = "packages/ui" }
api = { path = "packages/api" }
```

- [ ] **Step 2: Delete unused workspace members**

```bash
rm -rf packages/web packages/mobile
```

- [ ] **Step 3: Replace `packages/api/Cargo.toml`**

```toml
[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { workspace = true }
chrono = { workspace = true }
chrono-tz = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
once_cell = { workspace = true }
dirs = { workspace = true }
tokio = { workspace = true }
```

- [ ] **Step 4: Replace `packages/desktop/Cargo.toml`**

```toml
[package]
name = "desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true, features = ["desktop"] }
ui = { workspace = true }
api = { workspace = true }
dirs = { workspace = true }
```

- [ ] **Step 5: Replace `packages/ui/Cargo.toml`**

```toml
[package]
name = "ui"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true, features = ["desktop", "router"] }
api = { workspace = true }
chrono = { workspace = true }
chrono-tz = { workspace = true }
serde_json = { workspace = true }
rfd = { workspace = true }
```

- [ ] **Step 6: Verify workspace parses**

```bash
cargo metadata --no-deps --quiet > /dev/null
```

Expected: exits 0, no output.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml packages/api/Cargo.toml packages/desktop/Cargo.toml packages/ui/Cargo.toml
git rm -r packages/web packages/mobile
git commit -m "chore: switch to Dioxus desktop workspace, remove web/mobile"
```

---

### Task 2: Migrations + sqlx compile config

**Files:**
- Create: `packages/api/.cargo/config.toml`
- Create: `packages/api/migrations/20260521000001_initial_schema.sql`

- [ ] **Step 1: Create `packages/api/.cargo/config.toml`**

```toml
[env]
DATABASE_URL = { value = "sqlite::memory:", force = true }
```

This lets `sqlx::migrate!()` and `sqlx::query!()` macros compile without a live database file.

- [ ] **Step 2: Create `packages/api/migrations/20260521000001_initial_schema.sql`**

```sql
CREATE TABLE IF NOT EXISTS labor_codes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    wbs_number  TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hour_types (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS timecard_entries (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    labor_code_id  INTEGER NOT NULL REFERENCES labor_codes(id),
    hour_type_id   INTEGER NOT NULL REFERENCES hour_types(id),
    telework       INTEGER NOT NULL DEFAULT 0,
    date           TEXT NOT NULL,
    start_time     TEXT NOT NULL,
    end_time       TEXT,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now'))
);

CREATE TABLE IF NOT EXISTS pay_period_anchors (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    start_date TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_timecard_date       ON timecard_entries(date);
CREATE INDEX IF NOT EXISTS idx_timecard_labor_code ON timecard_entries(labor_code_id);
CREATE INDEX IF NOT EXISTS idx_timecard_hour_type  ON timecard_entries(hour_type_id);
```

- [ ] **Step 3: Create the `src/db/` directory structure**

```bash
mkdir -p packages/api/src/db
touch packages/api/src/lib.rs
touch packages/api/src/db/mod.rs
touch packages/api/src/db/models.rs
touch packages/api/src/db/repo.rs
```

- [ ] **Step 4: Commit**

```bash
git add packages/api/
git commit -m "feat: add sqlx migrations and compile-time DATABASE_URL config"
```

---

### Task 3: `compute_decimal_hours` — TDD

**Files:**
- Modify: `packages/api/src/db/repo.rs`
- Modify: `packages/api/src/db/mod.rs` (needed so tests compile)
- Modify: `packages/api/src/lib.rs` (needed so the crate compiles)

- [ ] **Step 1: Write minimal stubs so the crate compiles**

`packages/api/src/lib.rs`:
```rust
pub mod db;
```

`packages/api/src/db/mod.rs`:
```rust
pub mod models;
pub mod repo;
```

`packages/api/src/db/models.rs`:
```rust
// populated in Task 4
```

- [ ] **Step 2: Write the failing tests in `packages/api/src/db/repo.rs`**

```rust
/// Compute decimal hours rounded to nearest 15 minutes.
/// Returns `None` when `end_time` is `None` (entry in progress).
pub fn compute_decimal_hours(_start_time: &str, _end_time: Option<&str>) -> Option<f64> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::compute_decimal_hours;

    #[test]
    fn null_end_returns_none() {
        assert_eq!(compute_decimal_hours("2026-05-21T07:00:00Z", None), None);
    }

    #[test]
    fn exact_15_min_boundary() {
        // 8h 15m = 495m → 8.25
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:15:00Z")),
            Some(8.25)
        );
    }

    #[test]
    fn rounds_up_at_8m() {
        // 8m past boundary → rounds up → 0.25
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T07:08:00Z")),
            Some(0.25)
        );
    }

    #[test]
    fn rounds_down_at_7m() {
        // 7m past boundary → rounds down → 0.0
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T07:07:00Z")),
            Some(0.0)
        );
    }

    #[test]
    fn rounds_up_8h12m() {
        // 8h 12m = 492m → 495m → 8.25
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:12:00Z")),
            Some(8.25)
        );
    }

    #[test]
    fn rounds_down_8h8m() {
        // 8h 8m = 488m → 480m → 8.0
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:08:00Z")),
            Some(8.0)
        );
    }

    #[test]
    fn exact_half_hour() {
        // 4h 30m = 270m → 4.5
        assert_eq!(
            compute_decimal_hours("2026-05-21T08:00:00Z", Some("2026-05-21T12:30:00Z")),
            Some(4.5)
        );
    }

    #[test]
    fn zero_duration_rounds_to_zero() {
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T07:00:00Z")),
            Some(0.0)
        );
    }
}
```

- [ ] **Step 3: Run the tests — expect panics from `todo!()`**

```bash
cargo test -p api compute_decimal_hours 2>&1 | head -20
```

Expected: tests run, all fail with `not yet implemented`.

- [ ] **Step 4: Implement `compute_decimal_hours`**

Replace the `todo!()` stub in `packages/api/src/db/repo.rs`:

```rust
use chrono::NaiveDateTime;

/// Compute decimal hours rounded to nearest 15 minutes.
/// Returns `None` when `end_time` is `None` (entry in progress).
pub fn compute_decimal_hours(start_time: &str, end_time: Option<&str>) -> Option<f64> {
    let end = end_time?;

    let parse = |s: &str| -> Option<NaiveDateTime> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f+00:00")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ"))
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ"))
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
            .ok()
    };

    let start = parse(start_time)?;
    let end = parse(end)?;
    let minutes = (end - start).num_minutes() as f64;
    let rounded = (minutes / 15.0).round() * 15.0;
    Some(rounded / 60.0)
}
```

- [ ] **Step 5: Run tests — expect all pass**

```bash
cargo test -p api compute_decimal_hours
```

Expected:
```
test db::repo::tests::exact_half_hour ... ok
test db::repo::tests::exact_15_min_boundary ... ok
test db::repo::tests::null_end_returns_none ... ok
test db::repo::tests::rounds_down_at_7m ... ok
test db::repo::tests::rounds_up_at_8m ... ok
test db::repo::tests::rounds_up_8h12m ... ok
test db::repo::tests::rounds_down_8h8m ... ok
test db::repo::tests::zero_duration_rounds_to_zero ... ok
test result: ok. 8 passed; 0 failed
```

- [ ] **Step 6: Commit**

```bash
git add packages/api/src/
git commit -m "feat: compute_decimal_hours with full rounding test suite"
```

---

### Task 4: Data models

**Files:**
- Modify: `packages/api/src/db/models.rs`

- [ ] **Step 1: Write all model structs**

```rust
use serde::{Deserialize, Serialize};

// --- Labor Codes ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LaborCode {
    pub id: i64,
    pub wbs_number: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLaborCode {
    pub wbs_number: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLaborCode {
    pub id: i64,
    pub wbs_number: String,
    pub name: String,
}

// --- Hour Types ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HourType {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateHourType {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateHourType {
    pub id: i64,
    pub code: String,
    pub name: String,
}

// --- Timecard Entries ---

/// Raw row returned by sqlx — telework is i64 from SQLite INTEGER.
#[derive(Debug, sqlx::FromRow)]
pub struct TimecardEntryRow {
    pub id: i64,
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: i64,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub wbs_number: String,
    pub labor_code_name: String,
    pub hour_type_code: String,
    pub hour_type_name: String,
}

/// Public view with computed `decimal_hours` and normalized `telework: bool`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimecardEntryView {
    pub id: i64,
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: bool,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub decimal_hours: Option<f64>,
    pub wbs_number: String,
    pub labor_code_name: String,
    pub hour_type_code: String,
    pub hour_type_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTimecardEntry {
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: bool,
    pub date: String,          // YYYY-MM-DD local date
    pub start_time: String,    // HH:MM Central time
    pub end_time: Option<String>, // HH:MM Central time, or None for in-progress
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTimecardEntry {
    pub id: i64,
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: bool,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
}

// --- Pay Periods ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PayPeriodAnchor {
    pub id: i64,
    pub start_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPeriodRange {
    pub start_date: String,
    pub end_date: String,
}

// --- Aggregates ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayAggregate {
    pub date: String,
    pub total_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateRow {
    pub wbs_number: String,
    pub labor_code_name: String,
    pub total_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySummary {
    pub entries: Vec<TimecardEntryView>,
    pub total_hours: f64,
    pub by_labor_code: Vec<AggregateRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekSummary {
    pub entries: Vec<TimecardEntryView>,
    pub total_hours: f64,
    pub by_day: Vec<DayAggregate>,
    pub by_labor_code: Vec<AggregateRow>,
}

// --- Import / Export ---

#[derive(Debug, Clone, Deserialize)]
pub struct ImportLaborCode {
    pub wbs_number: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportHourType {
    pub code: String,
    pub name: String,
}

/// Shared shape for both import (deserialize) and export (serialize).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPayload {
    pub labor_codes: Vec<ImportLaborCode>,
    pub hour_types: Vec<ImportHourType>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported_labor_codes: u64,
    pub imported_hour_types: u64,
}
```

Note: `ImportLaborCode` and `ImportHourType` derive `Serialize` is needed for `ImportPayload` serialization during export. Add `Serialize` to both:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLaborCode { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportHourType { ... }
```

- [ ] **Step 2: Verify compile**

```bash
cargo check -p api
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/api/src/db/models.rs
git commit -m "feat: add all data model structs"
```

---

### Task 5: DB pool initialization

**Files:**
- Modify: `packages/api/src/db/mod.rs`

- [ ] **Step 1: Write `packages/api/src/db/mod.rs`**

```rust
use once_cell::sync::OnceCell;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;

pub mod models;
pub mod repo;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

static POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Returns the global pool. Panics if `init()` has not been called.
pub fn pool() -> &'static SqlitePool {
    POOL.get().expect("DB pool not initialized — call api::db::init() first")
}

/// Synchronously initialize the pool and run pending migrations.
/// Must be called once from `main()` before `dioxus::launch`.
pub fn init(db_path: PathBuf) {
    std::fs::create_dir_all(db_path.parent().unwrap())
        .expect("Failed to create app data directory");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        let opts = SqliteConnectOptions::from_str(
            &format!("sqlite://{}", db_path.display()),
        )
        .expect("Invalid DB path")
        .create_if_missing(true);

        let pool = SqlitePool::connect_with(opts)
            .await
            .expect("Failed to connect to SQLite");

        MIGRATOR.run(&pool).await.expect("Failed to run migrations");

        POOL.set(pool).ok();
    });
}
```

- [ ] **Step 2: Verify compile**

```bash
cargo check -p api
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/api/src/db/mod.rs
git commit -m "feat: add static DB pool with embedded migrations"
```

---

### Task 6: Repository — labor codes + hour types CRUD + tests

**Files:**
- Modify: `packages/api/src/db/repo.rs`

- [ ] **Step 1: Add imports and repository struct to `repo.rs`**

Add to the top of `packages/api/src/db/repo.rs` (keep the existing `compute_decimal_hours` function):

```rust
use chrono::NaiveDateTime;
use chrono_tz::America::Chicago;
use chrono::TimeZone;
use sqlx::SqlitePool;
use super::models::*;

pub struct Repository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> Repository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Convert local Central "HH:MM" + "YYYY-MM-DD" to UTC ISO 8601.
    fn central_to_utc(date: &str, time: &str) -> String {
        let naive = NaiveDateTime::parse_from_str(
            &format!("{}T{}:00", date, time),
            "%Y-%m-%dT%H:%M:%S",
        )
        .expect("Invalid date/time string");
        Chicago
            .from_local_datetime(&naive)
            .single()
            .expect("Ambiguous or invalid Central time")
            .with_timezone(&chrono::Utc)
            .to_rfc3339()
    }

    /// Convert a `TimecardEntryRow` to a `TimecardEntryView` with computed decimal_hours.
    fn row_to_view(r: TimecardEntryRow) -> TimecardEntryView {
        let decimal_hours = compute_decimal_hours(&r.start_time, r.end_time.as_deref());
        TimecardEntryView {
            id: r.id,
            labor_code_id: r.labor_code_id,
            hour_type_id: r.hour_type_id,
            telework: r.telework != 0,
            date: r.date,
            start_time: r.start_time,
            end_time: r.end_time,
            decimal_hours,
            wbs_number: r.wbs_number,
            labor_code_name: r.labor_code_name,
            hour_type_code: r.hour_type_code,
            hour_type_name: r.hour_type_name,
        }
    }
}
```

- [ ] **Step 2: Write failing tests for labor codes and hour types**

Append to the `#[cfg(test)]` module in `repo.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::*;

    // ---- existing compute_decimal_hours tests remain here ----

    // ---- Labor Codes ----

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn create_labor_code_returns_new_record(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let result = repo.create_labor_code(&CreateLaborCode {
            wbs_number: "WBS-001".into(),
            name: "Test Project".into(),
        })
        .await
        .unwrap();
        assert_eq!(result.wbs_number, "WBS-001");
        assert_eq!(result.name, "Test Project");
        assert!(result.id > 0);
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn list_labor_codes_ordered_by_name(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.create_labor_code(&CreateLaborCode { wbs_number: "Z".into(), name: "Zebra".into() }).await.unwrap();
        repo.create_labor_code(&CreateLaborCode { wbs_number: "A".into(), name: "Alpha".into() }).await.unwrap();
        let list = repo.list_labor_codes().await.unwrap();
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zebra");
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn update_labor_code(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let created = repo.create_labor_code(&CreateLaborCode { wbs_number: "WBS-002".into(), name: "Old".into() }).await.unwrap();
        let updated = repo.update_labor_code(&UpdateLaborCode { id: created.id, wbs_number: "WBS-002".into(), name: "New".into() }).await.unwrap();
        assert_eq!(updated.name, "New");
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn delete_labor_code(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let created = repo.create_labor_code(&CreateLaborCode { wbs_number: "WBS-003".into(), name: "Delete Me".into() }).await.unwrap();
        repo.delete_labor_code(created.id).await.unwrap();
        let list = repo.list_labor_codes().await.unwrap();
        assert!(list.is_empty());
    }

    // ---- Hour Types ----

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn create_hour_type_returns_new_record(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let result = repo.create_hour_type(&CreateHourType { code: "REG".into(), name: "Regular".into() }).await.unwrap();
        assert_eq!(result.code, "REG");
        assert_eq!(result.name, "Regular");
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn list_hour_types_ordered_by_code(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.create_hour_type(&CreateHourType { code: "OVT".into(), name: "Overtime".into() }).await.unwrap();
        repo.create_hour_type(&CreateHourType { code: "REG".into(), name: "Regular".into() }).await.unwrap();
        let list = repo.list_hour_types().await.unwrap();
        assert_eq!(list[0].code, "OVT");
        assert_eq!(list[1].code, "REG");
    }
}
```

- [ ] **Step 3: Run — expect compile error (methods not yet defined)**

```bash
cargo test -p api 2>&1 | head -20
```

Expected: compile errors like `no method named 'create_labor_code' found for struct 'Repository'`.

- [ ] **Step 4: Implement labor codes CRUD methods**

Append to `impl<'a> Repository<'a>` in `repo.rs`:

```rust
    // --- Labor Codes ---

    pub async fn list_labor_codes(&self) -> Result<Vec<LaborCode>, sqlx::Error> {
        sqlx::query_as!(LaborCode, "SELECT id, wbs_number, name FROM labor_codes ORDER BY name")
            .fetch_all(self.pool)
            .await
    }

    pub async fn create_labor_code(&self, input: &CreateLaborCode) -> Result<LaborCode, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO labor_codes (wbs_number, name) VALUES ($1, $2)",
            input.wbs_number, input.name,
        )
        .execute(self.pool)
        .await?
        .last_insert_rowid();

        sqlx::query_as!(LaborCode, "SELECT id, wbs_number, name FROM labor_codes WHERE id = $1", id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn update_labor_code(&self, input: &UpdateLaborCode) -> Result<LaborCode, sqlx::Error> {
        sqlx::query!(
            "UPDATE labor_codes SET wbs_number = $1, name = $2 WHERE id = $3",
            input.wbs_number, input.name, input.id,
        )
        .execute(self.pool)
        .await?;

        sqlx::query_as!(LaborCode, "SELECT id, wbs_number, name FROM labor_codes WHERE id = $1", input.id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn delete_labor_code(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM labor_codes WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
```

- [ ] **Step 5: Implement hour types CRUD methods**

Append to `impl<'a> Repository<'a>`:

```rust
    // --- Hour Types ---

    pub async fn list_hour_types(&self) -> Result<Vec<HourType>, sqlx::Error> {
        sqlx::query_as!(HourType, "SELECT id, code, name FROM hour_types ORDER BY code")
            .fetch_all(self.pool)
            .await
    }

    pub async fn create_hour_type(&self, input: &CreateHourType) -> Result<HourType, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO hour_types (code, name) VALUES ($1, $2)",
            input.code, input.name,
        )
        .execute(self.pool)
        .await?
        .last_insert_rowid();

        sqlx::query_as!(HourType, "SELECT id, code, name FROM hour_types WHERE id = $1", id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn update_hour_type(&self, input: &UpdateHourType) -> Result<HourType, sqlx::Error> {
        sqlx::query!(
            "UPDATE hour_types SET code = $1, name = $2 WHERE id = $3",
            input.code, input.name, input.id,
        )
        .execute(self.pool)
        .await?;

        sqlx::query_as!(HourType, "SELECT id, code, name FROM hour_types WHERE id = $1", input.id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn delete_hour_type(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM hour_types WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
```

- [ ] **Step 6: Run tests — expect all pass**

```bash
cargo test -p api 2>&1 | tail -15
```

Expected:
```
test db::repo::tests::create_labor_code_returns_new_record ... ok
test db::repo::tests::list_labor_codes_ordered_by_name ... ok
test db::repo::tests::update_labor_code ... ok
test db::repo::tests::delete_labor_code ... ok
test db::repo::tests::create_hour_type_returns_new_record ... ok
test db::repo::tests::list_hour_types_ordered_by_code ... ok
test result: ok. 14 passed; 0 failed
```

- [ ] **Step 7: Commit**

```bash
git add packages/api/src/db/repo.rs
git commit -m "feat: labor codes and hour types CRUD with tests"
```

---

### Task 7: Repository — timecard entries CRUD + tests

**Files:**
- Modify: `packages/api/src/db/repo.rs`

- [ ] **Step 1: Write failing tests for timecard entries**

Append to `#[cfg(test)]` in `repo.rs`:

```rust
    // ---- Helpers for entry tests ----

    async fn seed_lookup(pool: &sqlx::SqlitePool) -> (i64, i64) {
        let repo = Repository::new(pool);
        let lc = repo.create_labor_code(&CreateLaborCode { wbs_number: "WBS-T".into(), name: "Test".into() }).await.unwrap();
        let ht = repo.create_hour_type(&CreateHourType { code: "REG".into(), name: "Regular".into() }).await.unwrap();
        (lc.id, ht.id)
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn create_entry_returns_view_with_decimal_hours(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        let entry = repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id,
            hour_type_id: ht_id,
            telework: false,
            date: "2026-05-21".into(),
            start_time: "08:00".into(),
            end_time: Some("16:00".into()),
        })
        .await
        .unwrap();
        assert_eq!(entry.date, "2026-05-21");
        assert_eq!(entry.decimal_hours, Some(8.0));
        assert!(!entry.telework);
        assert_eq!(entry.wbs_number, "WBS-T");
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn create_in_progress_entry_has_null_decimal_hours(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        let entry = repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id,
            hour_type_id: ht_id,
            telework: false,
            date: "2026-05-21".into(),
            start_time: "08:00".into(),
            end_time: None,
        })
        .await
        .unwrap();
        assert!(entry.decimal_hours.is_none());
        assert!(entry.end_time.is_none());
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn list_entries_filtered_by_date_range(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        for date in ["2026-05-19", "2026-05-21", "2026-05-22"] {
            repo.create_timecard_entry(&CreateTimecardEntry {
                labor_code_id: lc_id, hour_type_id: ht_id,
                telework: false, date: date.into(),
                start_time: "08:00".into(), end_time: Some("16:00".into()),
            }).await.unwrap();
        }
        let results = repo.list_timecard_entries("2026-05-21", "2026-05-22").await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.date >= "2026-05-21" && e.date <= "2026-05-22"));
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn update_entry_changes_fields(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        let created = repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: false,
            date: "2026-05-21".into(), start_time: "08:00".into(), end_time: Some("16:00".into()),
        }).await.unwrap();
        let updated = repo.update_timecard_entry(&UpdateTimecardEntry {
            id: created.id, labor_code_id: lc_id, hour_type_id: ht_id,
            telework: true, date: "2026-05-21".into(),
            start_time: "09:00".into(), end_time: Some("17:00".into()),
        }).await.unwrap();
        assert!(updated.telework);
        assert_eq!(updated.decimal_hours, Some(8.0));
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn delete_entry_removes_it(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        let entry = repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: false,
            date: "2026-05-21".into(), start_time: "08:00".into(), end_time: Some("16:00".into()),
        }).await.unwrap();
        repo.delete_timecard_entry(entry.id).await.unwrap();
        let list = repo.list_timecard_entries("2026-05-21", "2026-05-21").await.unwrap();
        assert!(list.is_empty());
    }
```

- [ ] **Step 2: Run — expect compile error**

```bash
cargo test -p api 2>&1 | head -10
```

Expected: errors about missing `list_timecard_entries`, `create_timecard_entry`, etc.

- [ ] **Step 3: Implement timecard entry CRUD methods**

Append to `impl<'a> Repository<'a>` in `repo.rs`:

```rust
    // --- Timecard Entries ---

    pub async fn list_timecard_entries(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<TimecardEntryView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            TimecardEntryRow,
            r#"
            SELECT
                te.id, te.labor_code_id, te.hour_type_id,
                te.telework, te.date, te.start_time, te.end_time,
                lc.wbs_number, lc.name AS labor_code_name,
                ht.code AS hour_type_code, ht.name AS hour_type_name
            FROM timecard_entries te
            JOIN labor_codes  lc ON te.labor_code_id = lc.id
            JOIN hour_types   ht ON te.hour_type_id  = ht.id
            WHERE te.date >= $1 AND te.date <= $2
            ORDER BY te.date, te.start_time
            "#,
            date_from,
            date_to,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows.into_iter().map(Self::row_to_view).collect())
    }

    pub async fn create_timecard_entry(
        &self,
        input: &CreateTimecardEntry,
    ) -> Result<TimecardEntryView, sqlx::Error> {
        let utc_start = Self::central_to_utc(&input.date, &input.start_time);
        let utc_end = input.end_time.as_deref().map(|t| Self::central_to_utc(&input.date, t));

        let id = sqlx::query!(
            "INSERT INTO timecard_entries (labor_code_id, hour_type_id, telework, date, start_time, end_time) VALUES ($1, $2, $3, $4, $5, $6)",
            input.labor_code_id,
            input.hour_type_id,
            input.telework as i64,
            input.date,
            utc_start,
            utc_end,
        )
        .execute(self.pool)
        .await?
        .last_insert_rowid();

        self.get_entry_view_by_id(id).await
    }

    pub async fn update_timecard_entry(
        &self,
        input: &UpdateTimecardEntry,
    ) -> Result<TimecardEntryView, sqlx::Error> {
        let utc_start = Self::central_to_utc(&input.date, &input.start_time);
        let utc_end = input.end_time.as_deref().map(|t| Self::central_to_utc(&input.date, t));

        sqlx::query!(
            "UPDATE timecard_entries SET labor_code_id=$1, hour_type_id=$2, telework=$3, date=$4, start_time=$5, end_time=$6, updated_at=strftime('%Y-%m-%dT%H:%M:%f','now') WHERE id=$7",
            input.labor_code_id,
            input.hour_type_id,
            input.telework as i64,
            input.date,
            utc_start,
            utc_end,
            input.id,
        )
        .execute(self.pool)
        .await?;

        self.get_entry_view_by_id(input.id).await
    }

    pub async fn delete_timecard_entry(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM timecard_entries WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    async fn get_entry_view_by_id(&self, id: i64) -> Result<TimecardEntryView, sqlx::Error> {
        let row = sqlx::query_as!(
            TimecardEntryRow,
            r#"
            SELECT
                te.id, te.labor_code_id, te.hour_type_id,
                te.telework, te.date, te.start_time, te.end_time,
                lc.wbs_number, lc.name AS labor_code_name,
                ht.code AS hour_type_code, ht.name AS hour_type_name
            FROM timecard_entries te
            JOIN labor_codes  lc ON te.labor_code_id = lc.id
            JOIN hour_types   ht ON te.hour_type_id  = ht.id
            WHERE te.id = $1
            "#,
            id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(Self::row_to_view(row))
    }
```

- [ ] **Step 4: Run tests — expect all pass**

```bash
cargo test -p api 2>&1 | tail -15
```

Expected: all tests pass including the 5 new timecard entry tests.

- [ ] **Step 5: Commit**

```bash
git add packages/api/src/db/repo.rs
git commit -m "feat: timecard entry CRUD with join view and decimal_hours"
```

---

### Task 8: Repository — pay period anchors + `compute_pay_periods` + tests

**Files:**
- Modify: `packages/api/src/db/repo.rs`

- [ ] **Step 1: Write failing tests**

Append to `#[cfg(test)]` in `repo.rs`:

```rust
    // ---- Pay Period Anchors ----

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn add_and_list_pay_period_anchors(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let a = repo.add_pay_period_anchor("2026-05-06").await.unwrap();
        assert_eq!(a.start_date, "2026-05-06");
        let list = repo.list_pay_period_anchors().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn remove_pay_period_anchor(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let a = repo.add_pay_period_anchor("2026-05-06").await.unwrap();
        repo.remove_pay_period_anchor(a.id).await.unwrap();
        assert!(repo.list_pay_period_anchors().await.unwrap().is_empty());
    }

    // ---- compute_pay_periods (pure, no DB) ----

    #[test]
    fn compute_pay_periods_empty_anchors() {
        let result = Repository::compute_pay_periods(&[], "2026-05-21");
        assert!(result.is_empty());
    }

    #[test]
    fn compute_pay_periods_single_anchor_contains_reference_date() {
        let anchors = vec![PayPeriodAnchor { id: 1, start_date: "2026-05-06".into() }];
        let periods = Repository::compute_pay_periods(&anchors, "2026-05-21");
        assert!(!periods.is_empty());
        let current = periods.iter().find(|p| p.start_date <= "2026-05-21" && p.end_date >= "2026-05-21");
        assert!(current.is_some(), "reference date must fall in some period");
    }

    #[test]
    fn compute_pay_periods_14_day_periods() {
        let anchors = vec![PayPeriodAnchor { id: 1, start_date: "2026-05-06".into() }];
        let periods = Repository::compute_pay_periods(&anchors, "2026-05-21");
        for p in &periods {
            use chrono::NaiveDate;
            let start = NaiveDate::parse_from_str(&p.start_date, "%Y-%m-%d").unwrap();
            let end = NaiveDate::parse_from_str(&p.end_date, "%Y-%m-%d").unwrap();
            assert_eq!((end - start).num_days(), 13, "each period must be exactly 14 days");
        }
    }

    #[test]
    fn compute_pay_periods_are_sorted_and_unique() {
        let anchors = vec![PayPeriodAnchor { id: 1, start_date: "2026-05-06".into() }];
        let periods = Repository::compute_pay_periods(&anchors, "2026-05-21");
        for w in periods.windows(2) {
            assert!(w[0].start_date < w[1].start_date, "periods must be sorted");
        }
    }
```

- [ ] **Step 2: Run — expect compile errors**

```bash
cargo test -p api 2>&1 | head -10
```

Expected: errors for missing `add_pay_period_anchor`, `list_pay_period_anchors`, etc.

- [ ] **Step 3: Implement pay period anchor CRUD**

Append to `impl<'a> Repository<'a>`:

```rust
    // --- Pay Period Anchors ---

    pub async fn list_pay_period_anchors(&self) -> Result<Vec<PayPeriodAnchor>, sqlx::Error> {
        sqlx::query_as!(
            PayPeriodAnchor,
            "SELECT id, start_date FROM pay_period_anchors ORDER BY start_date"
        )
        .fetch_all(self.pool)
        .await
    }

    pub async fn add_pay_period_anchor(&self, start_date: &str) -> Result<PayPeriodAnchor, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO pay_period_anchors (start_date) VALUES ($1)",
            start_date,
        )
        .execute(self.pool)
        .await?
        .last_insert_rowid();

        sqlx::query_as!(
            PayPeriodAnchor,
            "SELECT id, start_date FROM pay_period_anchors WHERE id = $1",
            id
        )
        .fetch_one(self.pool)
        .await
    }

    pub async fn remove_pay_period_anchor(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM pay_period_anchors WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
```

- [ ] **Step 4: Implement `compute_pay_periods` as an associated function**

Append to `impl<'a> Repository<'a>`:

```rust
    /// Pure function — no DB access. Returns sorted, deduplicated 14-day ranges
    /// spanning ±1 year around `reference_date`, derived from `anchors`.
    pub fn compute_pay_periods(anchors: &[PayPeriodAnchor], reference_date: &str) -> Vec<PayPeriodRange> {
        use chrono::{Duration, NaiveDate};

        let ref_date = match reference_date.parse::<NaiveDate>() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        if anchors.is_empty() {
            return Vec::new();
        }

        let window = Duration::days(365);
        let mut periods: Vec<PayPeriodRange> = Vec::new();

        for anchor in anchors {
            let anchor_date = match anchor.start_date.parse::<NaiveDate>() {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Walk backward from anchor
            let mut cur = anchor_date - Duration::days(14);
            while cur >= ref_date - window {
                periods.push(PayPeriodRange {
                    start_date: cur.format("%Y-%m-%d").to_string(),
                    end_date: (cur + Duration::days(13)).format("%Y-%m-%d").to_string(),
                });
                cur -= Duration::days(14);
            }

            // Walk forward from anchor
            let mut cur = anchor_date;
            while cur <= ref_date + window {
                periods.push(PayPeriodRange {
                    start_date: cur.format("%Y-%m-%d").to_string(),
                    end_date: (cur + Duration::days(13)).format("%Y-%m-%d").to_string(),
                });
                cur += Duration::days(14);
            }
        }

        periods.sort_by(|a, b| a.start_date.cmp(&b.start_date));
        periods.dedup_by_key(|p| p.start_date.clone());
        periods
    }
```

- [ ] **Step 5: Run tests — expect all pass**

```bash
cargo test -p api 2>&1 | tail -15
```

Expected: all tests pass including the 6 new pay period tests.

- [ ] **Step 6: Commit**

```bash
git add packages/api/src/db/repo.rs
git commit -m "feat: pay period anchors CRUD and compute_pay_periods"
```

---

### Task 9: Repository — aggregates, import, export

**Files:**
- Modify: `packages/api/src/db/repo.rs`

- [ ] **Step 1: Implement aggregate methods**

Append to `impl<'a> Repository<'a>`:

```rust
    // --- Aggregates ---

    pub async fn get_day_summary(&self, date: &str) -> Result<DaySummary, sqlx::Error> {
        let entries = self.list_timecard_entries(date, date).await?;
        let total_hours: f64 = entries.iter().filter_map(|e| e.decimal_hours).sum();
        let by_labor_code = Self::aggregate_by_labor_code(&entries);
        Ok(DaySummary { entries, total_hours, by_labor_code })
    }

    pub async fn get_week_summary(&self, week_start: &str) -> Result<WeekSummary, sqlx::Error> {
        use chrono::{Duration, NaiveDate};
        let start = week_start.parse::<NaiveDate>().map_err(|_| sqlx::Error::RowNotFound)?;
        let end = (start + Duration::days(6)).format("%Y-%m-%d").to_string();
        let entries = self.list_timecard_entries(week_start, &end).await?;
        let total_hours: f64 = entries.iter().filter_map(|e| e.decimal_hours).sum();
        let by_day = Self::aggregate_by_day(&entries, week_start, &end);
        let by_labor_code = Self::aggregate_by_labor_code(&entries);
        Ok(WeekSummary { entries, total_hours, by_day, by_labor_code })
    }

    pub async fn get_pay_period_summary(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<WeekSummary, sqlx::Error> {
        let entries = self.list_timecard_entries(period_start, period_end).await?;
        let total_hours: f64 = entries.iter().filter_map(|e| e.decimal_hours).sum();
        let by_day = Self::aggregate_by_day(&entries, period_start, period_end);
        let by_labor_code = Self::aggregate_by_labor_code(&entries);
        Ok(WeekSummary { entries, total_hours, by_day, by_labor_code })
    }

    fn aggregate_by_day(entries: &[TimecardEntryView], from: &str, to: &str) -> Vec<DayAggregate> {
        use chrono::{Duration, NaiveDate};
        let mut result = Vec::new();
        let start = match from.parse::<NaiveDate>() { Ok(d) => d, Err(_) => return result };
        let end = match to.parse::<NaiveDate>() { Ok(d) => d, Err(_) => return result };
        let mut cur = start;
        while cur <= end {
            let date_str = cur.format("%Y-%m-%d").to_string();
            let total_hours = entries
                .iter()
                .filter(|e| e.date == date_str)
                .filter_map(|e| e.decimal_hours)
                .sum();
            result.push(DayAggregate { date: date_str, total_hours });
            cur += Duration::days(1);
        }
        result
    }

    fn aggregate_by_labor_code(entries: &[TimecardEntryView]) -> Vec<AggregateRow> {
        use std::collections::HashMap;
        let mut map: HashMap<i64, AggregateRow> = HashMap::new();
        for e in entries {
            let row = map.entry(e.labor_code_id).or_insert_with(|| AggregateRow {
                wbs_number: e.wbs_number.clone(),
                labor_code_name: e.labor_code_name.clone(),
                total_hours: 0.0,
            });
            row.total_hours += e.decimal_hours.unwrap_or(0.0);
        }
        let mut result: Vec<AggregateRow> = map.into_values().collect();
        result.sort_by(|a, b| a.wbs_number.cmp(&b.wbs_number));
        result
    }
```

- [ ] **Step 2: Implement import and export methods**

Append to `impl<'a> Repository<'a>`:

```rust
    // --- Import ---

    pub async fn import_lookup_data(
        &self,
        labor_codes: &[ImportLaborCode],
        hour_types: &[ImportHourType],
    ) -> ImportResult {
        let mut lc_count = 0u64;
        for lc in labor_codes {
            if sqlx::query!(
                "INSERT INTO labor_codes (wbs_number, name) VALUES ($1, $2) ON CONFLICT(wbs_number) DO UPDATE SET name = excluded.name",
                lc.wbs_number, lc.name,
            )
            .execute(self.pool)
            .await
            .is_ok() { lc_count += 1; }
        }
        let mut ht_count = 0u64;
        for ht in hour_types {
            if sqlx::query!(
                "INSERT INTO hour_types (code, name) VALUES ($1, $2) ON CONFLICT(code) DO UPDATE SET name = excluded.name",
                ht.code, ht.name,
            )
            .execute(self.pool)
            .await
            .is_ok() { ht_count += 1; }
        }
        ImportResult { imported_labor_codes: lc_count, imported_hour_types: ht_count }
    }

    // --- Export ---

    /// Returns all labor codes and hour types as an ImportPayload (same shape as import).
    pub async fn export_lookup_data(&self) -> Result<ImportPayload, sqlx::Error> {
        let labor_codes = self.list_labor_codes().await?;
        let hour_types = self.list_hour_types().await?;
        Ok(ImportPayload {
            labor_codes: labor_codes
                .into_iter()
                .map(|lc| ImportLaborCode { wbs_number: lc.wbs_number, name: lc.name })
                .collect(),
            hour_types: hour_types
                .into_iter()
                .map(|ht| ImportHourType { code: ht.code, name: ht.name })
                .collect(),
        })
    }
```

- [ ] **Step 3: Add aggregate test**

Append to `#[cfg(test)]`:

```rust
    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn get_day_summary_totals_correctly(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: false,
            date: "2026-05-21".into(), start_time: "08:00".into(), end_time: Some("16:00".into()),
        }).await.unwrap();
        repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: false,
            date: "2026-05-21".into(), start_time: "16:00".into(), end_time: Some("18:00".into()),
        }).await.unwrap();
        let summary = repo.get_day_summary("2026-05-21").await.unwrap();
        assert_eq!(summary.entries.len(), 2);
        assert_eq!(summary.total_hours, 10.0);
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_upserts_on_conflict(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let payload = vec![ImportLaborCode { wbs_number: "WBS-X".into(), name: "Old".into() }];
        repo.import_lookup_data(&payload, &[]).await;
        let payload2 = vec![ImportLaborCode { wbs_number: "WBS-X".into(), name: "New".into() }];
        repo.import_lookup_data(&payload2, &[]).await;
        let list = repo.list_labor_codes().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "New");
    }
```

- [ ] **Step 4: Run all tests**

```bash
cargo test -p api
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add packages/api/src/db/repo.rs
git commit -m "feat: aggregates, import (upsert), and export"
```

---

### Task 10: `api/lib.rs` + `desktop/src/main.rs`

**Files:**
- Modify: `packages/api/src/lib.rs`
- Create: `packages/desktop/src/main.rs`

- [ ] **Step 1: Write `packages/api/src/lib.rs`**

```rust
pub mod db;

pub use db::models::*;
pub use db::repo::{compute_decimal_hours, Repository};
pub use db::{init, pool};
```

- [ ] **Step 2: Create `packages/desktop/src/main.rs`**

```rust
fn main() {
    let db_path = dirs::data_dir()
        .expect("Cannot locate app data directory")
        .join("timecard-calc")
        .join("timecard.db");

    // Synchronous init — creates file + runs migrations before Dioxus starts.
    api::db::init(db_path);

    dioxus::launch(ui::App);
}
```

- [ ] **Step 3: Create a minimal stub for `packages/ui/src/lib.rs`** so the desktop crate compiles

```rust
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! { div { "Timecard Calc — frontend coming soon" } }
}
```

- [ ] **Step 4: Verify full workspace compiles**

```bash
cargo check
```

Expected: exits 0.

- [ ] **Step 5: Commit**

```bash
git add packages/api/src/lib.rs packages/desktop/src/main.rs packages/ui/src/lib.rs
git commit -m "feat: wire api lib, desktop entry point, ui stub"
```

---

### Task 11: Final verification

- [ ] **Step 1: Run all tests**

```bash
cargo test -p api
```

Expected:
```
test result: ok. N passed; 0 failed; 0 ignored
```

- [ ] **Step 2: Full workspace check**

```bash
cargo check
```

Expected: exits 0, no warnings about unused items.

- [ ] **Step 3: Build the desktop binary**

```bash
cargo build --bin desktop
```

Expected: compiles. The binary is at `target/debug/desktop`.

- [ ] **Step 4: Smoke test — run the app briefly**

```bash
cargo run --bin desktop &
sleep 3
kill %1
```

Expected: app window opens, no crash, `timecard.db` file created at `~/Library/Application Support/timecard-calc/timecard.db` (macOS) or equivalent.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: backend complete — all CRUD, aggregates, import/export, desktop entry"
```

---

## Summary

| Task | Key output |
|---|---|
| 1 | Workspace deps updated, web/mobile deleted |
| 2 | SQLite migration + sqlx compile config |
| 3 | `compute_decimal_hours` + 8 unit tests |
| 4 | All data model structs |
| 5 | DB pool with embedded migrations |
| 6 | Labor codes + hour types CRUD + 6 tests |
| 7 | Timecard entries CRUD + 5 tests |
| 8 | Pay period anchors + `compute_pay_periods` + 6 tests |
| 9 | Aggregates + import + export + 2 tests |
| 10 | `api/lib.rs` + `desktop/main.rs` + UI stub |
| 11 | Final verification |
