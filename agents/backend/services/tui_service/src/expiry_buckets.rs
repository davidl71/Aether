//! Expiry bucket labels aligned with boxtrades.com (see docs/platform/BOXTRADES_REFERENCE.md).
//! Used by the Yield curve tab and scenario explorer to show human-friendly tenor labels.

/// Returns a short label for the given days-to-expiry (e.g. "5 days", "about 1 month", "2 months").
/// Aligned with boxtrades.com expiry buckets.
#[must_use]
pub fn bucket_label(days_to_expiry: i32) -> &'static str {
    match days_to_expiry {
        d if d <= 0 => "expired",
        d if d <= 7 => "5 days",
        d if d <= 25 => "about 1 month",
        d if d <= 45 => "2 months",
        d if d <= 75 => "3 months",
        d if d <= 105 => "4 months",
        d if d <= 135 => "5 months",
        d if d <= 165 => "6 months",
        d if d <= 200 => "7 months",
        d if d <= 235 => "8 months",
        d if d <= 270 => "9 months",
        d if d <= 320 => "10 months",
        d if d <= 350 => "11 months",
        d if d <= 380 => "about 1 year",
        d if d <= 730 => "over 1 year",
        d if d <= 1095 => "almost 2 years",
        d if d <= 1460 => "almost 3 years",
        d if d <= 1825 => "almost 4 years",
        d if d <= 2190 => "almost 5 years",
        _ => "almost 6 years",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_labels_match_boxtrades_ranges() {
        assert_eq!(bucket_label(0), "expired");
        assert_eq!(bucket_label(5), "5 days");
        assert_eq!(bucket_label(20), "about 1 month");
        assert_eq!(bucket_label(35), "2 months");
        assert_eq!(bucket_label(60), "3 months");
        assert_eq!(bucket_label(365), "about 1 year");
        assert_eq!(bucket_label(400), "over 1 year");
        assert_eq!(bucket_label(730), "over 1 year");
        assert_eq!(bucket_label(1095), "almost 2 years");
        assert_eq!(bucket_label(2500), "almost 6 years");
    }
}
