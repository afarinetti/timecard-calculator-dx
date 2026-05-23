# Timecard Calculator — Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `packages/api` data layer with SQLite (sqlx), repository pattern, and all async CRUD methods. Set up `packages/desktop` entry point with DB initialization and Dioxus launch.

**Architecture:** Repository pattern — all SQL queries in `packages/api/src/db/repo.rs`. No Tauri IPC, no server functions. Components call repository methods directly via `use_resource`. Pool stored in a `once_cell::sync::OnceCell` initialized at startup.

**Tech Stack:** Rust 2021, Dioxus 0.7 (desktop), sqlx 0.8 (sqlite feature), chrono 0.4, chrono-tz, once_cell, dirs

---

### Task 1: Update Cargo.toml files

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `packages/api/Cargo.toml`
- Modify: `packages/desktop/Cargo.toml`
- Modify: `packages/ui/Cargo.toml`

- [ ] **Step 1: Update workspace `Cargo.toml`**

Remove `packages/web` and `packages/mobile` from the workspace members. Add shared dependencies:

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

# workspace crates
ui = { path = "packages/ui" }
api = { path = "packages/api" }
```

- [ ] **Step 2: Delete unused packages**

```bash
rm -rf packages/web packages/mobile
```

- [ ] **Step 3: Update `packages/api/Cargo.toml`**

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

- [ ] **Step 4: Update `packages/desktop/Cargo.toml`**

```toml
[package]
name = "desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true, features = ["desktop"] }
ui = { workspace = true }
api = { workspace = true }
```

- [ ] **Step 5: Update `packages/ui/Cargo.toml`**

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
```

- [ ] **Step 6: Verify `cargo check` passes**

```bash
cargo check
```

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml packages/api/Cargo.toml packages/desktop/Cargo.toml packages/ui/Cargo.toml
git rm -r packages/web packages/mobile
git commit -m "deps: switch to Dioxus desktop + sqlx, remove web/mobile packages"
```

---

### Task 2: Create sqlx migrations

**Files:**
- Create: `packages/api/migrations/20260521000001_initial_schema.sql`
- Create: `packages/api/.cargo/config.toml`

- [ ] **Step 1: Write `migrations/20260521000001_initial_schema.sql`**

```sql
CREATE TABLE IF NOT EXISTS labor_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wbs_number TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hour_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS timecard_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    labor_code_id INTEGER NOT NULL REFERENCES labor_codes(id),
    hour_type_id INTEGER NOT NULL REFERENCES hour_types(id),
    telework INTEGER NOT NULL DEFAULT 0,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now'))
);

CREATE TABLE IF NOT EXISTS pay_period_anchors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_date TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_timecard_date ON timecard_entries(date);
CREATE INDEX IF NOT EXISTS idx_timecard_labor_code ON timecard_entries(labor_code_id);
CREATE INDEX IF NOT EXISTS idx_timecard_hour_type ON timecard_entries(hour_type_id);
```

- [ ] **Step 2: Create `packages/api/.cargo/config.toml`**

```toml
[env]
DATABASE_URL = { value = "sqlite::memory:", force = true }
```

This allows `sqlx::embed_migrations!` and `sqlx::query!` macros to compile without a live database.

- [ ] **Step 3: Commit**

```bash
git add packages/api/migrations/ packages/api/.cargo/
git commit -m "feat: add sqlx migrations for initial schema"
```

---

### Task 3: Create `db/models.rs`

**Files:**
- Create: `packages/api/src/db/models.rs`

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
// Raw DB row — used only for sqlx::FromRow mapping.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct TimecardEntryRow {
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

// Public view — decimal_hours computed after DB fetch.
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

impl From<TimecardEntryRow> for TimecardEntryView {
    fn from(r: TimecardEntryRow) -> Self {
        let decimal_hours = crate::db::repo::compute_decimal_hours(
            &r.start_time,
            r.end_time.as_deref(),
        );
        Self {
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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTimecardEntry {
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: bool,
    pub date: String,          // local date YYYY-MM-DD
    pub start_time: String,    // local Central time HH:MM
    pub end_time: Option<String>, // local Central time HH:MM, nullable
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

// --- Pay Period Anchors ---
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

// --- Import ---
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

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported_labor_codes: u64,
    pub imported_hour_types: u64,
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/api/src/db/models.rs
git commit -m "feat: add model structs for all DB entities"
```

---

### Task 4: Create `db/mod.rs` — pool initialization

**Files:**
- Create: `packages/api/src/db/mod.rs`

- [ ] **Step 1: Write `db/mod.rs`**

```rust
use once_cell::sync::OnceCell;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;

pub mod models;
pub mod repo;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
static POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Returns the global pool. Panics if `init()` has not been called.
pub fn pool() -> &'static SqlitePool {
    POOL.get().expect("DB pool not initialized — call api::db::init() first")
}

/// Initialize the DB pool synchronously. Must be called once before `dioxus::launch`.
/// Creates the database file if it does not exist, then runs all pending migrations.
pub fn init(db_path: PathBuf) {
    std::fs::create_dir_all(db_path.parent().unwrap()).expect("Failed to create app data dir");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
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

- [ ] **Step 2: Commit**

```bash
git add packages/api/src/db/mod.rs
git commit -m "feat: add db module with static pool and sqlx migrations"
```

---

### Task 5: Create `db/repo.rs` — repository with all CRUD methods

**Files:**
- Create: `packages/api/src/db/repo.rs`

- [ ] **Step 1: Write repository struct and helpers**

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

    /// Convert local Central time "HH:MM" + date "YYYY-MM-DD" to UTC ISO 8601.
    fn central_to_utc(date: &str, time: &str) -> String {
        let naive = NaiveDateTime::parse_from_str(
            &format!("{}T{}:00", date, time),
            "%Y-%m-%dT%H:%M:%S",
        )
        .expect("Invalid date/time format");
        Chicago
            .from_local_datetime(&naive)
            .single()
            .expect("Ambiguous or invalid Central time")
            .with_timezone(&chrono::Utc)
            .to_rfc3339()
    }

    /// Compute decimal hours rounded to nearest 15 minutes.
    /// Returns None if end_time is None (entry in progress).
    pub fn compute_decimal_hours(start_time: &str, end_time: Option<&str>) -> Option<f64> {
        let end = end_time?;
        let parse = |s: &str| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f+00:00")
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ"))
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
                .ok()
        };
        let start = parse(start_time)?;
        let end = parse(end)?;
        let minutes = (end - start).num_minutes() as f64;
        let rounded = (minutes / 15.0).round() * 15.0;
        Some(rounded / 60.0)
    }

    /// Fetch a single entry view by id (after insert/update).
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
            JOIN labor_codes lc ON te.labor_code_id = lc.id
            JOIN hour_types ht ON te.hour_type_id = ht.id
            WHERE te.id = $1
            "#,
            id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(row.into())
    }
}
```

- [ ] **Step 2: Add labor code CRUD methods**

```rust
impl<'a> Repository<'a> {
    pub async fn list_labor_codes(&self) -> Result<Vec<LaborCode>, sqlx::Error> {
        sqlx::query_as!(
            LaborCode,
            "SELECT id, wbs_number, name FROM labor_codes ORDER BY name"
        )
        .fetch_all(self.pool)
        .await
    }

    pub async fn create_labor_code(&self, input: &CreateLaborCode) -> Result<LaborCode, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO labor_codes (wbs_number, name) VALUES ($1, $2)",
            input.wbs_number,
            input.name,
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
            input.wbs_number,
            input.name,
            input.id,
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
}
```

- [ ] **Step 3: Add hour type CRUD methods**

```rust
impl<'a> Repository<'a> {
    pub async fn list_hour_types(&self) -> Result<Vec<HourType>, sqlx::Error> {
        sqlx::query_as!(
            HourType,
            "SELECT id, code, name FROM hour_types ORDER BY code"
        )
        .fetch_all(self.pool)
        .await
    }

    pub async fn create_hour_type(&self, input: &CreateHourType) -> Result<HourType, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO hour_types (code, name) VALUES ($1, $2)",
            input.code,
            input.name,
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
            input.code,
            input.name,
            input.id,
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
}
```

- [ ] **Step 4: Add timecard entry CRUD methods**

```rust
impl<'a> Repository<'a> {
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
            JOIN labor_codes lc ON te.labor_code_id = lc.id
            JOIN hour_types ht ON te.hour_type_id = ht.id
            WHERE te.date >= $1 AND te.date <= $2
            ORDER BY te.date, te.start_time
            "#,
            date_from,
            date_to,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
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
            "UPDATE timecard_entries SET labor_code_id = $1, hour_type_id = $2, telework = $3, date = $4, start_time = $5, end_time = $6, updated_at = strftime('%Y-%m-%dT%H:%M:%f','now') WHERE id = $7",
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
}
```

- [ ] **Step 5: Add pay period anchor methods**

```rust
impl<'a> Repository<'a> {
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

        sqlx::query_as!(PayPeriodAnchor, "SELECT id, start_date FROM pay_period_anchors WHERE id = $1", id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn remove_pay_period_anchor(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM pay_period_anchors WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Pure function: compute pay periods from anchors.
    /// Returns a sorted, deduplicated list of 14-day ranges spanning ±1 year around reference_date.
    pub fn compute_pay_periods(
        anchors: &[PayPeriodAnchor],
        reference_date: &str,
    ) -> Vec<PayPeriodRange> {
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
            let mut cur = anchor_date;
            loop {
                cur -= Duration::days(14);
                if cur < ref_date - window {
                    break;
                }
                periods.push(PayPeriodRange {
                    start_date: cur.format("%Y-%m-%d").to_string(),
                    end_date: (cur + Duration::days(13)).format("%Y-%m-%d").to_string(),
                });
            }

            // Walk forward from anchor
            let mut cur = anchor_date;
            loop {
                if cur > ref_date + window {
                    break;
                }
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
}
```

- [ ] **Step 6: Add aggregate methods**

```rust
impl<'a> Repository<'a> {
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
        let mut totals: HashMap<i64, (String, String, f64)> = HashMap::new();
        for e in entries {
            let entry = totals.entry(e.labor_code_id).or_insert_with(|| {
                (e.wbs_number.clone(), e.labor_code_name.clone(), 0.0)
            });
            entry.2 += e.decimal_hours.unwrap_or(0.0);
        }
        let mut result: Vec<AggregateRow> = totals
            .into_values()
            .map(|(wbs, name, hours)| AggregateRow {
                wbs_number: wbs,
                labor_code_name: name,
                total_hours: hours,
            })
            .collect();
        result.sort_by(|a, b| a.wbs_number.cmp(&b.wbs_number));
        result
    }

    pub async fn import_lookup_data(
        &self,
        labor_codes: &[ImportLaborCode],
        hour_types: &[ImportHourType],
    ) -> ImportResult {
        let mut lc_count = 0u64;
        for lc in labor_codes {
            if sqlx::query!(
                "INSERT INTO labor_codes (wbs_number, name) VALUES ($1, $2) ON CONFLICT(wbs_number) DO UPDATE SET name = excluded.name",
                lc.wbs_number,
                lc.name,
            )
            .execute(self.pool)
            .await
            .is_ok()
            {
                lc_count += 1;
            }
        }
        let mut ht_count = 0u64;
        for ht in hour_types {
            if sqlx::query!(
                "INSERT INTO hour_types (code, name) VALUES ($1, $2) ON CONFLICT(code) DO UPDATE SET name = excluded.name",
                ht.code,
                ht.name,
            )
            .execute(self.pool)
            .await
            .is_ok()
            {
                ht_count += 1;
            }
        }
        ImportResult { imported_labor_codes: lc_count, imported_hour_types: ht_count }
    }
}
```

- [ ] **Step 7: Commit**

```bash
git add packages/api/src/db/repo.rs
git commit -m "feat: implement repository with all CRUD, aggregates, and import"
```

---

### Task 6: Create `api/src/lib.rs` + `desktop/src/main.rs`

**Files:**
- Create/modify: `packages/api/src/lib.rs`
- Create/modify: `packages/desktop/src/main.rs`

- [ ] **Step 1: Write `packages/api/src/lib.rs`**

```rust
pub mod db;
pub use db::models::*;
pub use db::repo::Repository;
pub use db::{init, pool};
```

- [ ] **Step 2: Write `packages/desktop/src/main.rs`**

```rust
fn main() {
    let db_path = dirs::data_dir()
        .expect("No app data directory found")
        .join("timecard-calc")
        .join("timecard.db");

    api::db::init(db_path);

    dioxus::launch(ui::App);
}
```

- [ ] **Step 3: Commit**

```bash
git add packages/api/src/lib.rs packages/desktop/src/main.rs
git commit -m "feat: wire api lib and desktop entry point"
```

---

### Task 7: Add Rust unit tests

**Files:**
- Create: `packages/api/src/db/tests.rs`

- [ ] **Step 1: Write rounding tests**

```rust
#[cfg(test)]
mod tests {
    use crate::db::repo::Repository;

    #[test]
    fn rounding_exact_15() {
        // 8h 15m → 8.25
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:15:00Z")),
            Some(8.25)
        );
    }

    #[test]
    fn rounding_up() {
        // 8h 12m = 492m → 495m → 8.25
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:12:00Z")),
            Some(8.25)
        );
    }

    #[test]
    fn rounding_down() {
        // 8h 8m = 488m → 480m → 8.0
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:08:00Z")),
            Some(8.0)
        );
    }

    #[test]
    fn rounding_null_end() {
        assert_eq!(Repository::compute_decimal_hours("2026-05-21T07:00:00Z", None), None);
    }

    #[test]
    fn rounding_7m_rounds_down() {
        // 7m → 0m → 0.0
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T07:07:00Z")),
            Some(0.0)
        );
    }

    #[test]
    fn rounding_8m_rounds_up() {
        // 8m → 15m → 0.25
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T07:08:00Z")),
            Some(0.25)
        );
    }

    #[test]
    fn rounding_exact_half() {
        // 4h 30m → 4.5
        assert_eq!(
            Repository::compute_decimal_hours("2026-05-21T08:00:00Z", Some("2026-05-21T12:30:00Z")),
            Some(4.5)
        );
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p api
```

Expected: all 7 tests pass.

- [ ] **Step 3: Commit**

```bash
git add packages/api/src/db/tests.rs
git commit -m "test: add rounding unit tests for repository"
```

---

### Task 8: Final verification

- [ ] **Step 1: Run `cargo check`**

```bash
cargo check
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

- [ ] **Step 3: Build desktop**

```bash
dx build --platform desktop
```

Expected: compiles, `timecard.db` created in app data directory on first run, migrations applied.

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: complete backend — repository, DB init, desktop entry point"
```

---

## Plan Summary

| Task | File(s) | Status |
|------|---------|--------|
| 1. Update Cargo.toml files | workspace, api, desktop, ui | - [ ] |
| 2. Create migrations | `packages/api/migrations/`, `.cargo/config.toml` | - [ ] |
| 3. Create db/models.rs | `packages/api/src/db/models.rs` | - [ ] |
| 4. Create db/mod.rs | `packages/api/src/db/mod.rs` | - [ ] |
| 5. Create db/repo.rs | `packages/api/src/db/repo.rs` | - [ ] |
| 6. Wire api/lib.rs + desktop/main.rs | `packages/api/src/lib.rs`, `packages/desktop/src/main.rs` | - [ ] |
| 7. Add unit tests | `packages/api/src/db/tests.rs` | - [ ] |
| 8. Final verification | cargo check, cargo test, dx build | - [ ] |

**After this plan is complete, proceed to the frontend implementation plan.**
