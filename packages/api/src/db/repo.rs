use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::America::Chicago;
use chrono::TimeZone;
use sqlx::SqlitePool;
use crate::db::models::*;

pub struct Repository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> Repository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Convert local Central "HH:MM" + "YYYY-MM-DD" to UTC ISO 8601.
    fn central_to_utc(date: &str, time: &str) -> Result<String, sqlx::Error> {
        let naive = NaiveDateTime::parse_from_str(
            &format!("{}T{}:00", date, time),
            "%Y-%m-%dT%H:%M:%S",
        )
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date/time: {}", e)))?;
        let local = Chicago
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| sqlx::Error::Protocol("Ambiguous or invalid Central time".into()))?;
        Ok(local.with_timezone(&chrono::Utc).to_rfc3339())
    }

    /// Convert a `TimecardEntryRow` to a `TimecardEntryView` with computed decimal_hours.
    fn row_to_view(r: TimecardEntryRow) -> TimecardEntryView {
        let decimal_hours = compute_decimal_hours(r.start_time, r.end_time);
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

    // --- Hour Types ---

    pub async fn list_hour_types(&self) -> Result<Vec<HourType>, sqlx::Error> {
        sqlx::query_as!(HourType, r#"SELECT id as "id!", code, name FROM hour_types ORDER BY code"#)
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

        sqlx::query_as!(HourType, r#"SELECT id as "id!", code, name FROM hour_types WHERE id = $1"#, id)
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

        sqlx::query_as!(HourType, r#"SELECT id as "id!", code, name FROM hour_types WHERE id = $1"#, input.id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn delete_hour_type(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM hour_types WHERE id = $1", id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

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
                te.id as "id!: i64",
                te.labor_code_id as "labor_code_id!: i64",
                te.hour_type_id as "hour_type_id!: i64",
                te.telework as "telework!: i64",
                te.date as "date!: NaiveDate",
                te.start_time as "start_time!: DateTime<Utc>",
                te.end_time as "end_time?: DateTime<Utc>",
                lc.wbs_number as "wbs_number!: String",
                lc.name AS "labor_code_name!: String",
                ht.code AS "hour_type_code!: String",
                ht.name AS "hour_type_name!: String"
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
        let utc_start = Self::central_to_utc(&input.date, &input.start_time)?;
        let utc_end = input.end_time.as_deref().map(|t| Self::central_to_utc(&input.date, t)).transpose()?;
        let telework: i64 = input.telework as i64;

        let id = sqlx::query!(
            "INSERT INTO timecard_entries (labor_code_id, hour_type_id, telework, date, start_time, end_time) VALUES ($1, $2, $3, $4, $5, $6)",
            input.labor_code_id,
            input.hour_type_id,
            telework,
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
        let utc_start = Self::central_to_utc(&input.date, &input.start_time)?;
        let utc_end = input.end_time.as_deref().map(|t| Self::central_to_utc(&input.date, t)).transpose()?;
        let telework: i64 = input.telework as i64;

        sqlx::query!(
            "UPDATE timecard_entries SET labor_code_id=$1, hour_type_id=$2, telework=$3, date=$4, start_time=$5, end_time=$6, updated_at=strftime('%Y-%m-%dT%H:%M:%f','now') WHERE id=$7",
            input.labor_code_id,
            input.hour_type_id,
            telework,
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

    // --- Pay Period Anchors ---

    pub async fn list_pay_period_anchors(&self) -> Result<Vec<PayPeriodAnchor>, sqlx::Error> {
        sqlx::query_as!(
            PayPeriodAnchor,
            r#"SELECT id as "id!: i64", start_date FROM pay_period_anchors ORDER BY start_date"#
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
            r#"SELECT id as "id!: i64", start_date FROM pay_period_anchors WHERE id = $1"#,
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
                .filter(|e| e.date == cur)
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

    // --- Import ---

    pub async fn import_lookup_data(
        &self,
        labor_codes: &[ImportLaborCode],
        hour_types: &[ImportHourType],
    ) -> Result<ImportResult, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut lc_count = 0u64;
        for lc in labor_codes {
            sqlx::query!(
                "INSERT INTO labor_codes (wbs_number, name) VALUES ($1, $2) ON CONFLICT(wbs_number) DO UPDATE SET name = excluded.name",
                lc.wbs_number, lc.name,
            )
            .execute(&mut *tx)
            .await?;
            lc_count += 1;
        }
        let mut ht_count = 0u64;
        for ht in hour_types {
            sqlx::query!(
                "INSERT INTO hour_types (code, name) VALUES ($1, $2) ON CONFLICT(code) DO UPDATE SET name = excluded.name",
                ht.code, ht.name,
            )
            .execute(&mut *tx)
            .await?;
            ht_count += 1;
        }
        tx.commit().await?;
        Ok(ImportResult { imported_labor_codes: lc_count, imported_hour_types: ht_count })
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

    // --- Entry Export / Import ---

    /// Convert UTC DateTime back to Central "HH:MM" for export.
    fn utc_to_central_hhmm(dt: DateTime<Utc>) -> String {
        dt.with_timezone(&Chicago).format("%H:%M").to_string()
    }

    pub async fn export_entries(&self,
    ) -> Result<ExportEntriesPayload, sqlx::Error> {
        let rows = sqlx::query_as!(
            TimecardEntryRow,
            r#"
            SELECT
                te.id as "id!: i64",
                te.labor_code_id as "labor_code_id!: i64",
                te.hour_type_id as "hour_type_id!: i64",
                te.telework as "telework!: i64",
                te.date as "date!: NaiveDate",
                te.start_time as "start_time!: DateTime<Utc>",
                te.end_time as "end_time?: DateTime<Utc>",
                lc.wbs_number as "wbs_number!: String",
                lc.name AS "labor_code_name!: String",
                ht.code AS "hour_type_code!: String",
                ht.name AS "hour_type_name!: String"
            FROM timecard_entries te
            JOIN labor_codes  lc ON te.labor_code_id = lc.id
            JOIN hour_types   ht ON te.hour_type_id  = ht.id
            ORDER BY te.date, te.start_time
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        let entries = rows
            .into_iter()
            .map(|r| ExportEntry {
                wbs_number:      r.wbs_number,
                hour_type_code:  r.hour_type_code,
                telework:        r.telework != 0,
                date:            r.date.to_string(),
                start_time:      Self::utc_to_central_hhmm(r.start_time),
                end_time:        r.end_time.map(Self::utc_to_central_hhmm),
            })
            .collect();

        Ok(ExportEntriesPayload { entries })
    }

    pub async fn import_entries(
        &self,
        entries: &[ExportEntry],
    ) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut count = 0u64;

        for entry in entries {
            // Resolve WBS number → labor_code_id
            let labor_code_id: i64 = sqlx::query_as!(
                LaborCode,
                r#"SELECT id as "id!: i64", wbs_number, name FROM labor_codes WHERE wbs_number = $1"#,
                entry.wbs_number,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| sqlx::Error::Protocol(
                format!("Labor code not found: {}", entry.wbs_number)
            ))?
            .id;

            // Resolve hour type code → hour_type_id
            let hour_type_id: i64 = sqlx::query_as!(
                HourType,
                r#"SELECT id as "id!", code, name FROM hour_types WHERE code = $1"#,
                entry.hour_type_code,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| sqlx::Error::Protocol(
                format!("Hour type not found: {}", entry.hour_type_code)
            ))?
            .id;

            let utc_start = Self::central_to_utc(&entry.date, &entry.start_time)?;
            let utc_end = entry.end_time.as_deref()
                .map(|t| Self::central_to_utc(&entry.date, t))
                .transpose()?;
            let telework: i64 = entry.telework as i64;

            sqlx::query!(
                "INSERT INTO timecard_entries (labor_code_id, hour_type_id, telework, date, start_time, end_time) VALUES ($1, $2, $3, $4, $5, $6)",
                labor_code_id,
                hour_type_id,
                telework,
                entry.date,
                utc_start,
                utc_end,
            )
            .execute(&mut *tx)
            .await?;

            count += 1;
        }

        tx.commit().await?;
        Ok(count)
    }

    async fn get_entry_view_by_id(&self, id: i64) -> Result<TimecardEntryView, sqlx::Error> {
        let row = sqlx::query_as!(
            TimecardEntryRow,
            r#"
            SELECT
                te.id as "id!: i64",
                te.labor_code_id as "labor_code_id!: i64",
                te.hour_type_id as "hour_type_id!: i64",
                te.telework as "telework!: i64",
                te.date as "date!: NaiveDate",
                te.start_time as "start_time!: DateTime<Utc>",
                te.end_time as "end_time?: DateTime<Utc>",
                lc.wbs_number as "wbs_number!: String",
                lc.name AS "labor_code_name!: String",
                ht.code AS "hour_type_code!: String",
                ht.name AS "hour_type_name!: String"
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
}

/// Compute decimal hours rounded to nearest 15 minutes.
/// Returns `None` when `end_time` is `None` (entry in progress).
pub fn compute_decimal_hours(start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Option<f64> {
    let end = end_time?;
    let minutes = (end - start_time).num_minutes() as f64;
    if minutes < 0.0 {
        return None;
    }
    let rounded = (minutes / 15.0).round() * 15.0;
    Some(rounded / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn dt(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn null_end_returns_none() {
        assert_eq!(compute_decimal_hours(dt("2026-05-21T07:00:00Z"), None), None);
    }

    #[test]
    fn exact_15_min_boundary() {
        // 8h 15m = 495m → 8.25
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T15:15:00Z"))),
            Some(8.25)
        );
    }

    #[test]
    fn rounds_up_at_8m() {
        // 8m past boundary → rounds up → 0.25
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T07:08:00Z"))),
            Some(0.25)
        );
    }

    #[test]
    fn rounds_down_at_7m() {
        // 7m past boundary → rounds down → 0.0
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T07:07:00Z"))),
            Some(0.0)
        );
    }

    #[test]
    fn rounds_up_8h12m() {
        // 8h 12m = 492m → 495m → 8.25
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T15:12:00Z"))),
            Some(8.25)
        );
    }

    #[test]
    fn rounds_down_8h7m() {
        // 8h 7m = 487m → nearest 15-min boundary is 480m → 8.0
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T15:07:00Z"))),
            Some(8.0)
        );
    }

    #[test]
    fn exact_half_hour() {
        // 4h 30m = 270m → 4.5
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T08:00:00Z"), Some(dt("2026-05-21T12:30:00Z"))),
            Some(4.5)
        );
    }

    #[test]
    fn zero_duration_rounds_to_zero() {
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T07:00:00Z"), Some(dt("2026-05-21T07:00:00Z"))),
            Some(0.0)
        );
    }

    #[test]
    fn end_before_start_returns_none() {
        assert_eq!(
            compute_decimal_hours(dt("2026-05-21T08:00:00Z"), Some(dt("2026-05-21T07:00:00Z"))),
            None
        );
    }

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
        assert_eq!(entry.date.to_string(), "2026-05-21");
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
        assert!(results.iter().all(|e| e.date.to_string().as_str() >= "2026-05-21" && e.date.to_string().as_str() <= "2026-05-22"));
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
        let current = periods.iter().find(|p| p.start_date.as_str() <= "2026-05-21" && p.end_date.as_str() >= "2026-05-21");
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
        repo.import_lookup_data(&payload, &[]).await.unwrap();
        let payload2 = vec![ImportLaborCode { wbs_number: "WBS-X".into(), name: "New".into() }];
        repo.import_lookup_data(&payload2, &[]).await.unwrap();
        let list = repo.list_labor_codes().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "New");
    }

    // ---- Entry Export / Import ----

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn export_entries_empty_db(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        let payload = repo.export_entries().await.unwrap();
        assert!(payload.entries.is_empty());
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn export_entries_returns_wbs_and_code(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: true,
            date: "2026-05-21".into(), start_time: "08:00".into(), end_time: Some("16:00".into()),
        }).await.unwrap();

        let payload = repo.export_entries().await.unwrap();
        assert_eq!(payload.entries.len(), 1);
        let e = &payload.entries[0];
        assert_eq!(e.wbs_number, "WBS-T");
        assert_eq!(e.hour_type_code, "REG");
        assert!(e.telework);
        assert_eq!(e.date, "2026-05-21");
        // Times should be Central HH:MM (round-trip from create → export)
        assert_eq!(e.start_time, "08:00"); // 08:00 CDT → UTC → 08:00 CDT
        assert_eq!(e.end_time, Some("16:00".into())); // 16:00 CDT → UTC → 16:00 CDT
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn export_entries_includes_in_progress(pool: sqlx::SqlitePool) {
        let (lc_id, ht_id) = seed_lookup(&pool).await;
        let repo = Repository::new(&pool);
        repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: lc_id, hour_type_id: ht_id, telework: false,
            date: "2026-05-21".into(), start_time: "08:00".into(), end_time: None,
        }).await.unwrap();

        let payload = repo.export_entries().await.unwrap();
        assert_eq!(payload.entries.len(), 1);
        assert!(payload.entries[0].end_time.is_none());
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_entries_creates_records(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        // Seed lookup data first
        repo.import_lookup_data(
            &[ImportLaborCode { wbs_number: "WBS-A".into(), name: "Alpha".into() }],
            &[ImportHourType { code: "REG".into(), name: "Regular".into() }],
        ).await.unwrap();

        let count = repo.import_entries(&[ExportEntry {
            wbs_number: "WBS-A".into(),
            hour_type_code: "REG".into(),
            telework: true,
            date: "2026-05-21".into(),
            start_time: "08:00".into(),
            end_time: Some("16:00".into()),
        }]).await.unwrap();

        assert_eq!(count, 1);
        let entries = repo.list_timecard_entries("2026-05-21", "2026-05-21").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].telework);
        assert_eq!(entries[0].decimal_hours, Some(8.0));
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_entries_fails_missing_wbs(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.import_lookup_data(
            &[ImportLaborCode { wbs_number: "WBS-A".into(), name: "Alpha".into() }],
            &[ImportHourType { code: "REG".into(), name: "Regular".into() }],
        ).await.unwrap();

        let result = repo.import_entries(&[ExportEntry {
            wbs_number: "WBS-MISSING".into(),
            hour_type_code: "REG".into(),
            telework: false,
            date: "2026-05-21".into(),
            start_time: "08:00".into(),
            end_time: None,
        }]).await;

        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Labor code not found"), "error should mention missing labor code: {}", msg);
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_entries_fails_missing_hour_type(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.import_lookup_data(
            &[ImportLaborCode { wbs_number: "WBS-A".into(), name: "Alpha".into() }],
            &[ImportHourType { code: "REG".into(), name: "Regular".into() }],
        ).await.unwrap();

        let result = repo.import_entries(
            &[ExportEntry {
                wbs_number: "WBS-A".into(),
                hour_type_code: "MISSING".into(),
                telework: false,
                date: "2026-05-21".into(),
                start_time: "08:00".into(),
                end_time: None,
            }]).await;

        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Hour type not found"), "error should mention missing hour type: {}", msg);
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_entries_rolls_back_on_error(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.import_lookup_data(
            &[ImportLaborCode { wbs_number: "WBS-A".into(), name: "Alpha".into() }],
            &[ImportHourType { code: "REG".into(), name: "Regular".into() }],
        ).await.unwrap();

        // First entry is valid, second references missing WBS
        let result = repo.import_entries(&[
                ExportEntry {
                    wbs_number: "WBS-A".into(),
                    hour_type_code: "REG".into(),
                    telework: false,
                    date: "2026-05-21".into(),
                    start_time: "08:00".into(),
                    end_time: None,
                },
                ExportEntry {
                    wbs_number: "WBS-MISSING".into(),
                    hour_type_code: "REG".into(),
                    telework: false,
                    date: "2026-05-21".into(),
                    start_time: "09:00".into(),
                    end_time: None,
                },
            ]).await;

        assert!(result.is_err());
        // The first entry must NOT have been persisted (transaction rolled back)
        let entries = repo.list_timecard_entries("2026-05-21", "2026-05-21").await.unwrap();
        assert!(entries.is_empty(), "transaction should have rolled back, but entries exist");
    }

    #[sqlx::test(migrator = "crate::db::MIGRATOR")]
    async fn import_entries_round_trip_preserves_data(pool: sqlx::SqlitePool) {
        let repo = Repository::new(&pool);
        repo.import_lookup_data(
            &[ImportLaborCode { wbs_number: "WBS-A".into(), name: "Alpha".into() }],
            &[ImportHourType { code: "REG".into(), name: "Regular".into() }],
        ).await.unwrap();

        // Create an entry via normal API
        repo.create_timecard_entry(&CreateTimecardEntry {
            labor_code_id: repo.list_labor_codes().await.unwrap()[0].id,
            hour_type_id:  repo.list_hour_types().await.unwrap()[0].id,
            telework: true,
            date: "2026-05-21".into(),
            start_time: "08:00".into(),
            end_time: Some("16:00".into()),
        }).await.unwrap();

        // Export it
        let exported = repo.export_entries().await.unwrap();
        assert_eq!(exported.entries.len(), 1);

        // Clear the entries table (keep lookup data)
        sqlx::query!("DELETE FROM timecard_entries").execute(&pool).await.unwrap();
        let empty = repo.list_timecard_entries("2026-05-21", "2026-05-21").await.unwrap();
        assert!(empty.is_empty());

        // Re-import
        let count = repo.import_entries(&exported.entries).await.unwrap();
        assert_eq!(count, 1);

        // Verify the re-imported entry matches
        let entries = repo.list_timecard_entries("2026-05-21", "2026-05-21").await.unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.decimal_hours, Some(8.0));
        assert!(e.telework);
    }
}
