//! Parse option expiry strings in YYYYMMDD form.
//!
//! Shared by quant, risk, and ib_adapter to avoid duplicated parsing logic.
//!
//! **Alternatives in existing dependencies:** The workspace already has crates that can parse
//! YYYYMMDD if we added a dependency here:
//!
//! - **time** (used by quant and risk): with features `parsing` and `macros`,
//!   `Date::parse(s, &format_description!("[year][month][day]"))` parses YYYYMMDD and validates
//!   calendar dates (e.g. rejects Feb 30). See <https://docs.rs/time/latest/time/struct.Date.html#method.parse>.
//! - **chrono** (used by api, ledger, tui_service, etc.): `NaiveDate::parse_from_str(s, "%Y%m%d")`
//!   parses YYYYMMDD. See <https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html>.
//!
//! This module uses a zero-dependency implementation so `common` stays minimal and `ib_adapter`
//! (which only needs `(year, month, day)` for ibapi) does not need to pull in `time` or `chrono`.

/// Parses an expiry string in YYYYMMDD form into (year, month, day).
///
/// Returns an error string if the input is not exactly 8 ASCII digits or
/// if month/day are out of range (month 1–12, day 1–31).
pub fn parse_expiry_yyyy_mm_dd(expiry: &str) -> Result<(u16, u8, u8), String> {
    let s = expiry.trim();
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("Option expiry must be YYYYMMDD, got {:?}", expiry));
    }
    let y: u16 = s[0..4]
        .parse()
        .map_err(|_| format!("invalid year in expiry {}", expiry))?;
    let m: u8 = s[4..6]
        .parse()
        .map_err(|_| format!("invalid month in expiry {}", expiry))?;
    let d: u8 = s[6..8]
        .parse()
        .map_err(|_| format!("invalid day in expiry {}", expiry))?;
    if m == 0 || m > 12 || d == 0 || d > 31 {
        return Err(format!("invalid date in expiry {}", expiry));
    }
    Ok((y, m, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_yyyy_mm_dd() {
        assert_eq!(parse_expiry_yyyy_mm_dd("20251219"), Ok((2025, 12, 19)));
        assert_eq!(parse_expiry_yyyy_mm_dd("  20260101  "), Ok((2026, 1, 1)));
    }

    #[test]
    fn rejects_invalid_length() {
        assert!(parse_expiry_yyyy_mm_dd("2025121").is_err());
        assert!(parse_expiry_yyyy_mm_dd("202512199").is_err());
    }

    #[test]
    fn rejects_invalid_month_or_day() {
        assert!(parse_expiry_yyyy_mm_dd("20250015").is_err());
        assert!(parse_expiry_yyyy_mm_dd("20251301").is_err());
        assert!(parse_expiry_yyyy_mm_dd("20251200").is_err());
        assert!(parse_expiry_yyyy_mm_dd("20251232").is_err());
    }
}
