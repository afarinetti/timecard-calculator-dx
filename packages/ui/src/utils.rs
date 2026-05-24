use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};
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

/// Convert a UTC `DateTime` to "HH:MM" in the system local timezone.
pub fn utc_to_central_hhmm(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M").to_string()
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
    minutes.div_ceil(15) * 15
}

pub fn start_now_hhmm() -> String {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let total = now.hour() * 60 + now.minute();
    let rounded = round_to_nearest_15(total).min(1439);
    format!("{:02}:{:02}", rounded / 60, rounded % 60)
}

pub fn end_now_hhmm() -> String {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let total = now.hour() * 60 + now.minute();
    let rounded = ceil_to_next_15(total).min(1439);
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
        let dt = DateTime::parse_from_rfc3339("2026-05-21T14:00:00Z").unwrap().with_timezone(&Utc);
        let expected = dt.with_timezone(&Local).format("%H:%M").to_string();
        assert_eq!(utc_to_central_hhmm(dt), expected);
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

    // ── Overlap detection ──

    fn make_entry(id: i64, start: &str, end: Option<&str>) -> TimecardEntryView {
        let dt = |s: &str| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc);
        TimecardEntryView {
            id,
            labor_code_id: 1, hour_type_id: 1, telework: false,
            date: NaiveDate::from_ymd_opt(2026, 5, 23).unwrap(),
            start_time: dt(start),
            end_time: end.map(dt),
            decimal_hours: None,
            wbs_number: String::new(),
            labor_code_name: String::new(),
            hour_type_code: String::new(),
            hour_type_name: String::new(),
            hour_type_badge_class: String::new(),
        }
    }

    #[test]
    fn no_overlap_returns_empty() {
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", Some("2026-05-23T12:00:00Z")),
            make_entry(2, "2026-05-23T13:00:00Z", Some("2026-05-23T17:00:00Z")),
        ];
        assert!(overlapping_ids(&entries).is_empty());
    }

    #[test]
    fn touching_boundaries_do_not_overlap() {
        // end of A == start of B → not an overlap
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", Some("2026-05-23T12:00:00Z")),
            make_entry(2, "2026-05-23T12:00:00Z", Some("2026-05-23T16:00:00Z")),
        ];
        assert!(overlapping_ids(&entries).is_empty());
    }

    #[test]
    fn overlapping_entries_both_flagged() {
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", Some("2026-05-23T12:30:00Z")),
            make_entry(2, "2026-05-23T12:00:00Z", Some("2026-05-23T16:00:00Z")),
        ];
        let ids = overlapping_ids(&entries);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn midnight_spanning_overlap_detected() {
        // The exact 5/23 case from the DB:
        // A: 17:00Z→01:15Z(next day)  B: 01:00Z(next day)→01:30Z(next day)
        // Overlap: 01:00–01:15 UTC
        let entries = vec![
            make_entry(1, "2026-05-23T17:00:00+00:00", Some("2026-05-24T01:15:00+00:00")),
            make_entry(2, "2026-05-24T01:00:00+00:00", Some("2026-05-24T01:30:00+00:00")),
        ];
        let ids = overlapping_ids(&entries);
        assert!(ids.contains(&1), "entry 1 should be flagged");
        assert!(ids.contains(&2), "entry 2 should be flagged");
    }

    // ── start_now_hhmm / end_now_hhmm ──

    #[test]
    fn start_now_rounds_to_nearest_15() {
        // At 14:00 (840 min) should stay 14:00
        let total = 14 * 60;
        let rounded = round_to_nearest_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "14:00");
    }

    #[test]
    fn start_now_rounds_down_at_7m() {
        // At 14:07 (847 min) should round down to 14:00
        let total = 14 * 60 + 7;
        let rounded = round_to_nearest_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "14:00");
    }

    #[test]
    fn start_now_rounds_up_at_8m() {
        // At 14:08 (848 min) should round up to 14:15
        let total = 14 * 60 + 8;
        let rounded = round_to_nearest_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "14:15");
    }

    #[test]
    fn start_now_clamps_at_2359() {
        // At 23:59 (1439 min) should clamp to 23:59
        let total = 23 * 60 + 59;
        let rounded = round_to_nearest_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "23:59");
    }

    #[test]
    fn end_now_ceil_to_next_15() {
        // At 14:01 (841 min) should ceil to 14:15
        let total = 14 * 60 + 1;
        let rounded = ceil_to_next_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "14:15");
    }

    #[test]
    fn end_now_exact_boundary_stays() {
        // At 14:15 (855 min) should stay 14:15
        let total = 14 * 60 + 15;
        let rounded = ceil_to_next_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "14:15");
    }

    #[test]
    fn end_now_clamps_at_2359() {
        // At 23:59 (1439 min) should clamp to 23:59
        let total = 23 * 60 + 59;
        let rounded = ceil_to_next_15(total).min(1439);
        let result = format!("{:02}:{:02}", rounded / 60, rounded % 60);
        assert_eq!(result, "23:59");
    }

    // ── date_range ──

    #[test]
    fn date_range_single_day() {
        assert_eq!(date_range("2026-05-21", "2026-05-21"), vec!["2026-05-21"]);
    }

    #[test]
    fn date_range_multiple_days() {
        assert_eq!(
            date_range("2026-05-20", "2026-05-22"),
            vec!["2026-05-20", "2026-05-21", "2026-05-22"]
        );
    }

    #[test]
    fn date_range_empty_when_start_after_end() {
        assert!(date_range("2026-05-22", "2026-05-20").is_empty());
    }

    #[test]
    fn date_range_invalid_dates_return_empty() {
        assert!(date_range("not-a-date", "2026-05-21").is_empty());
        assert!(date_range("2026-05-21", "not-a-date").is_empty());
    }

    // ── format helpers ──

    #[test]
    fn format_day_col_returns_weekday_and_date() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 21).unwrap();
        assert_eq!(format_day_col(date), ("Thu".into(), "5/21".into()));
    }

    #[test]
    fn format_day_label_returns_abbreviated_date() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 21).unwrap();
        assert_eq!(format_day_label(date), "Thu 05/21");
    }


    // ── overlapping_ids edge cases ──

    #[test]
    fn open_ended_overlaps_with_closed() {
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", None),
            make_entry(2, "2026-05-23T09:00:00Z", Some("2026-05-23T17:00:00Z")),
        ];
        let ids = overlapping_ids(&entries);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn open_ended_does_not_overlap_before_start() {
        let entries = vec![
            make_entry(1, "2026-05-23T12:00:00Z", None),
            make_entry(2, "2026-05-23T08:00:00Z", Some("2026-05-23T10:00:00Z")),
        ];
        assert!(overlapping_ids(&entries).is_empty());
    }

    #[test]
    fn open_ended_entries_always_overlap() {
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", None),
            make_entry(2, "2026-05-23T12:00:00Z", None),
        ];
        let ids = overlapping_ids(&entries);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn three_way_overlap_all_flagged() {
        let entries = vec![
            make_entry(1, "2026-05-23T08:00:00Z", Some("2026-05-23T12:00:00Z")),
            make_entry(2, "2026-05-23T10:00:00Z", Some("2026-05-23T14:00:00Z")),
            make_entry(3, "2026-05-23T11:00:00Z", Some("2026-05-23T15:00:00Z")),
        ];
        let ids = overlapping_ids(&entries);
        assert_eq!(ids.len(), 3);
    }
}