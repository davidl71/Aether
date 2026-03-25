//! Parse IBKR/OCC option symbols to extract strike, TTE, and call/put.
use chrono::{NaiveDate, Utc};

/// Parsed option fields from a symbol string.
pub struct ParsedOption {
    pub strike: f64,
    pub tte_years: f64,
    pub option_type: String, // "call" or "put"
}

/// Try to parse common IBKR option symbol formats:
/// - `ROOT YYYYMMDC/PSTRIKE` (e.g., `SPX 20250321C5000`)
/// - `ROOT  YYMMDD C/P STRIKE` (OCC padded)
///
/// Returns None if not parseable.
pub fn parse_option_symbol(symbol: &str) -> Option<ParsedOption> {
    // Find C or P separator for call/put
    // Strategy: look for a digit sequence (date), then C or P, then strike
    let s = symbol.trim();

    // Find the first 6+ digit run (date portion)
    // Try format: <root> <YYYYMMDD><C|P><strike>
    let after_root = s.trim_start_matches(|c: char| c.is_alphabetic() || c == ' ');
    let after_root = after_root.trim();

    // Find C or P after the date digits
    let cp_pos = after_root.find(['C', 'P'])?;
    if cp_pos < 6 {
        return None; // need at least YYMMDD
    }
    let date_str = &after_root[..cp_pos];
    let option_char = &after_root[cp_pos..cp_pos + 1];
    let strike_str = &after_root[cp_pos + 1..];

    // Parse date: YYYYMMDD or YYMMDD
    let date = if date_str.len() == 8 {
        NaiveDate::parse_from_str(date_str, "%Y%m%d").ok()?
    } else if date_str.len() == 6 {
        NaiveDate::parse_from_str(date_str, "%y%m%d").ok()?
    } else {
        return None;
    };

    let today = Utc::now().date_naive();
    let days = (date - today).num_days().max(0);
    let tte_years = days as f64 / 365.25;

    // Strike: may be integer like 5000 or OCC scaled like 05000000 (divide by 1000)
    let strike_raw: f64 = strike_str.trim().parse().ok()?;
    let strike = if strike_raw > 100_000.0 {
        strike_raw / 1000.0 // OCC 8-digit format
    } else {
        strike_raw
    };

    Some(ParsedOption {
        strike,
        tte_years,
        option_type: if option_char == "C" {
            "call".to_string()
        } else {
            "put".to_string()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spx_call() {
        // Future date so TTE > 0; use a fixed far-future date
        let sym = "SPX 20991231C5000";
        let parsed = parse_option_symbol(sym).expect("should parse");
        assert_eq!(parsed.option_type, "call");
        assert!((parsed.strike - 5000.0).abs() < 0.01);
        assert!(parsed.tte_years > 0.0);
    }

    #[test]
    fn test_parse_occ_put() {
        let sym = "SPX 20991231P05000000";
        let parsed = parse_option_symbol(sym).expect("should parse OCC format");
        assert_eq!(parsed.option_type, "put");
        assert!((parsed.strike - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_no_cp() {
        assert!(parse_option_symbol("SPX 20250321X5000").is_none());
    }

    #[test]
    fn test_parse_too_short_date() {
        assert!(parse_option_symbol("SPX 2503C5000").is_none());
    }
}
