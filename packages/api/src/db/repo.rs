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
}
