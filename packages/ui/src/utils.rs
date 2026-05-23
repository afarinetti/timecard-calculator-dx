use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::America::Chicago;

/// Navigate a YYYY-MM-DD date string by `delta` days (positive = forward).
pub fn navigate_date(date: &str, delta: i64) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(delta)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| date.to_string())
}

/// Navigate a week start (Monday) by `delta` weeks.
pub fn navigate_week(week_start: &str, delta: i64) -> String {
    navigate_date(week_start, delta * 7)
}

/// Return the Monday of the week containing `date`.
pub fn week_start_for(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| {
            let days_from_mon = d.weekday().num_days_from_monday() as i64;
            (d - Duration::days(days_from_mon)).format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| date.to_string())
}

/// Today's date as YYYY-MM-DD in local system time.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Convert UTC ISO 8601 timestamp to "HH:MM" in Central time.
/// Returns "??:??" on parse failure.
pub fn utc_to_central_hhmm(utc_iso: &str) -> String {
    let parsed = chrono::DateTime::parse_from_rfc3339(utc_iso)
        .or_else(|_| chrono::DateTime::parse_from_str(utc_iso, "%Y-%m-%dT%H:%M:%SZ"));
    match parsed {
        Ok(dt) => dt.with_timezone(&Chicago).format("%H:%M").to_string(),
        Err(_) => "??:??".to_string(),
    }
}

/// Elapsed decimal hours from `utc_start` to now, rounded to nearest 15 minutes.
pub fn live_elapsed_hours(utc_start: &str) -> f64 {
    let start = chrono::DateTime::parse_from_rfc3339(utc_start)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));
    match start {
        Some(s) => {
            let mins = (chrono::Utc::now() - s).num_minutes().max(0) as f64;
            (mins / 15.0).round() * 15.0 / 60.0
        }
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_date_forward() {
        assert_eq!(navigate_date("2026-05-21", 1), "2026-05-22");
    }

    #[test]
    fn navigate_date_backward() {
        assert_eq!(navigate_date("2026-05-21", -1), "2026-05-20");
    }

    #[test]
    fn navigate_date_crosses_month() {
        assert_eq!(navigate_date("2026-05-31", 1), "2026-06-01");
    }

    #[test]
    fn navigate_week_forward() {
        assert_eq!(navigate_week("2026-05-18", 1), "2026-05-25");
    }

    #[test]
    fn navigate_week_backward() {
        assert_eq!(navigate_week("2026-05-25", -1), "2026-05-18");
    }

    #[test]
    fn week_start_for_wednesday() {
        // Wednesday 2026-05-20 → Monday 2026-05-18
        assert_eq!(week_start_for("2026-05-20"), "2026-05-18");
    }

    #[test]
    fn week_start_for_monday_is_itself() {
        assert_eq!(week_start_for("2026-05-18"), "2026-05-18");
    }

    #[test]
    fn utc_to_central_hhmm_converts_correctly() {
        // 14:00 UTC = 09:00 CDT (UTC-5 in May; Central Daylight Time)
        assert_eq!(utc_to_central_hhmm("2026-05-21T14:00:00Z"), "09:00");
    }

    #[test]
    fn utc_to_central_hhmm_invalid_returns_placeholder() {
        assert_eq!(utc_to_central_hhmm("not-a-date"), "??:??");
    }
}
