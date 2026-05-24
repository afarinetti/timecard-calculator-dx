use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};

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
    pub date: NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub wbs_number: String,
    pub labor_code_name: String,
    pub hour_type_code: String,
    pub hour_type_name: String,
}

/// Public view with computed `decimal_hours` and normalized `telework: bool`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimecardEntryView {
    pub id: i64,
    pub labor_code_id: i64,
    pub hour_type_id: i64,
    pub telework: bool,
    pub date: NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLaborCode {
    pub wbs_number: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// --- Entry Export / Import ---

/// Human-readable timecard entry for export / import (uses WBS number and
/// hour type code so the file is portable across DB instances).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub wbs_number: String,
    pub hour_type_code: String,
    pub telework: bool,
    pub date: String,           // YYYY-MM-DD
    pub start_time: String,     // HH:MM
    pub end_time: Option<String>, // HH:MM or None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntriesPayload {
    pub entries: Vec<ExportEntry>,
}
