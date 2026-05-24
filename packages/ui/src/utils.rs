use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use chrono_tz::America::Chicago;
use dioxus::prelude::*;
use std::collections::HashSet;
use api::TimecardEntryView;

/// Newtype wrappers so that `Signal<String>` for the current date and the
/// current week-start have distinct `TypeId`s in the Dioxus context store.
/// Without these, both would collide under `TypeId::of::<Signal<String>>()`.
#[derive(Clone, Copy)]
pub struct CurrentDateSig(pub Signal<String>);

#[derive(Clone, Copy)]
pub struct CurrentWeekSig(pub Signal<String>);

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

/// Return the Sunday of the week containing `date` (Sun–Sat week).
pub fn week_start_for(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| {
            let days_from_sun = d.weekday().num_days_from_sunday() as i64;
            (d - Duration::days(days_from_sun)).format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| date.to_string())
}

/// Today's date as YYYY-MM-DD in local system time.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Convert a UTC `DateTime` to "HH:MM" in Central time.
pub fn utc_to_central_hhmm(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Chicago).format("%H:%M").to_string()
}

/// Elapsed decimal hours from `start` to now, rounded to nearest 15 minutes.
pub fn live_elapsed_hours(start: DateTime<Utc>) -> f64 {
    let mins = (Utc::now() - start).num_minutes().max(0) as f64;
    (mins / 15.0).round() * 15.0 / 60.0
}

/// Return a Vec of YYYY-MM-DD strings from `start` to `end` inclusive.
pub fn date_range(start: &str, end: &str) -> Vec<String> {
    let Ok(s) = NaiveDate::parse_from_str(start, "%Y-%m-%d") else { return vec![]; };
    let Ok(e) = NaiveDate::parse_from_str(end,   "%Y-%m-%d") else { return vec![]; };
    let mut dates = Vec::new();
    let mut cur = s;
    while cur <= e {
        dates.push(cur.format("%Y-%m-%d").to_string());
        cur = match cur.succ_opt() { Some(d) => d, None => break };
    }
    dates
}

/// Format a `NaiveDate` as ("Mon", "5/20") for two-line pivot column headers.
pub fn format_day_col(date: NaiveDate) -> (String, String) {
    (date.format("%a").to_string(), format!("{}/{}", date.month(), date.day()))
}

/// Format a `NaiveDate` as "Mon 05/20" (weekday abbreviation + month/day).
pub fn format_day_label(date: NaiveDate) -> String {
    date.format("%a %m/%d").to_string()
}


fn round_to_nearest_15(minutes: u32) -> u32 {
    ((minutes + 7) / 15) * 15
}

fn ceil_to_next_15(minutes: u32) -> u32 {
    ((minutes + 14) / 15) * 15
}

pub fn start_now_hhmm() -> String {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let total = now.hour() * 60 + now.minute();
    let adjusted = total.saturating_sub(15);
    let rounded = round_to_nearest_15(adjusted).min(1439);
    format!("{:02}:{:02}", rounded / 60, rounded % 60)
}

pub fn end_now_hhmm() -> String {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let total = now.hour() * 60 + now.minute();
    let adjusted = total + 15;
    let rounded = ceil_to_next_15(adjusted).min(1439);
    format!("{:02}:{:02}", rounded / 60, rounded % 60)
}

/// Returns the IDs of entries that overlap in time with at least one other entry.
///
/// Two entries overlap when their time intervals intersect: `a.start < b.end && b.start < a.end`.
/// An open-ended entry (no `end_time`) is treated as extending to infinity.
pub fn overlapping_ids(entries: &[TimecardEntryView]) -> HashSet<i64> {
    let times: Vec<(i64, DateTime<Utc>, Option<DateTime<Utc>>)> = entries
        .iter()
        .map(|e| (e.id, e.start_time, e.end_time))
        .collect();

    let mut result = HashSet::new();
    for i in 0..times.len() {
        for j in (i + 1)..times.len() {
            let (id_a, start_a, end_a) = times[i];
            let (id_b, start_b, end_b) = times[j];
            let overlaps = match (end_a, end_b) {
                (Some(ea), Some(eb)) => start_a < eb && start_b < ea,
                (None,     Some(eb)) => start_a < eb,
                (Some(ea), None)     => start_b < ea,
                (None,     None)     => true,
            };
            if overlaps {
                result.insert(id_a);
                result.insert(id_b);
            }
        }
    }
    result
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
        // Wednesday 2026-05-20 → Sunday 2026-05-17
        assert_eq!(week_start_for("2026-05-20"), "2026-05-17");
    }

    #[test]
    fn week_start_for_sunday_is_itself() {
        // Sunday 2026-05-17 → 2026-05-17
        assert_eq!(week_start_for("2026-05-17"), "2026-05-17");
    }

    #[test]
    fn week_start_for_monday() {
        // Monday 2026-05-18 → Sunday 2026-05-17
        assert_eq!(week_start_for("2026-05-18"), "2026-05-17");
    }

    #[test]
    fn utc_to_central_hhmm_converts_correctly() {
        // 14:00 UTC = 09:00 CDT (UTC-5 in May; Central Daylight Time)
        let dt = DateTime::parse_from_rfc3339("2026-05-21T14:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(utc_to_central_hhmm(dt), "09:00");
    }

    // ── Now-button rounding ──

    #[test]
    fn round_start_mid_interval() {
        // 13:52 (832 min) is 7 from 13:45 (825) and 8 from 14:00 (840) → rounds to 825
        assert_eq!(round_to_nearest_15(832), 825);
    }

    #[test]
    fn round_start_equidistant_rounds_up() {
        // 15:07 = 907 min → nearest: 900 (7 away) or 915 (8 away) → 900
        assert_eq!(round_to_nearest_15(907), 900);
    }

    #[test]
    fn round_start_on_boundary() {
        // 14:00 (840) is already on a boundary → stays 840
        assert_eq!(round_to_nearest_15(840), 840);
    }

    #[test]
    fn round_end_mid_interval() {
        // 14:22 (862 min) → ceil to next 15 → 870 (14:30)
        assert_eq!(ceil_to_next_15(862), 870);
    }

    #[test]
    fn round_end_on_boundary() {
        // 14:30 (870 min) → already on boundary → stays 870
        assert_eq!(ceil_to_next_15(870), 870);
    }

    #[test]
    fn round_end_just_past_boundary() {
        // 14:31 (871 min) → next boundary is 14:45 (885)
        assert_eq!(ceil_to_next_15(871), 885);
    }

    #[test]
    fn start_now_hhmm_format() {
        let s = start_now_hhmm();
        assert_eq!(s.len(), 5);
        assert_eq!(&s[2..3], ":");
    }

    #[test]
    fn end_now_hhmm_format() {
        let s = end_now_hhmm();
        assert_eq!(s.len(), 5);
        assert_eq!(&s[2..3], ":");
    }
}
