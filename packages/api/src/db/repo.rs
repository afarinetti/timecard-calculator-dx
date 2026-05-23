use chrono::NaiveDateTime;
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
}

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
    if minutes < 0.0 {
        return None;
    }
    let rounded = (minutes / 15.0).round() * 15.0;
    Some(rounded / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rounds_down_8h7m() {
        // 8h 7m = 487m → nearest 15-min boundary is 480m (7m below 495, 7m above 480 — ties go up, but 7 < 7.5 rounds down) → 8.0
        // NOTE: spec had a typo using 15:08 (488m); 488/15=32.53 rounds UP to 33*15=495 → 8.25,
        // contradicting the "rounds down" intent. Fixed to 15:07 (487m): 487/15=32.47 rounds DOWN.
        assert_eq!(
            compute_decimal_hours("2026-05-21T07:00:00Z", Some("2026-05-21T15:07:00Z")),
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

    #[test]
    fn end_before_start_returns_none() {
        assert_eq!(
            compute_decimal_hours("2026-05-21T08:00:00Z", Some("2026-05-21T07:00:00Z")),
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
}
