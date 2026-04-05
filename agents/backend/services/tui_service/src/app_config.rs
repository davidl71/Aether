use std::collections::HashMap;

use chrono::Datelike;

use crate::config::{PositionsSortMode, TuiConfig, TuiTheme};

/// Returns `Some(true)` when NYSE is currently open, `Some(false)` when closed.
///
/// Notes:
/// - This is an approximation intended for TUI status display only.
/// - It ignores US market holidays and any special trading sessions.
/// - It uses US DST rules to approximate the NY timezone offset.
pub(crate) fn nyse_is_open() -> Option<bool> {
    use chrono::{Datelike, Timelike, Utc, Weekday};

    // NYSE regular session: 09:30–16:00 America/New_York, Mon–Fri.
    let now_utc = Utc::now();
    let ny_offset_hours = ny_utc_offset_hours(now_utc.date_naive())?;
    let now_ny = now_utc + chrono::Duration::hours(ny_offset_hours.into());

    match now_ny.weekday() {
        Weekday::Sat | Weekday::Sun => return Some(false),
        _ => {}
    }

    let minutes = (now_ny.hour() * 60 + now_ny.minute()) as i32;
    let open = 9 * 60 + 30;
    let close = 16 * 60;
    Some(minutes >= open && minutes < close)
}

fn ny_utc_offset_hours(date: chrono::NaiveDate) -> Option<i32> {
    // DST for America/New_York: second Sunday in March → first Sunday in November.
    // Within that range offset is -4; otherwise -5.
    let year = date.year();
    let dst_start = nth_weekday_of_month(year, 3, chrono::Weekday::Sun, 2)?;
    let dst_end = nth_weekday_of_month(year, 11, chrono::Weekday::Sun, 1)?;
    if date >= dst_start && date < dst_end {
        Some(-4)
    } else {
        Some(-5)
    }
}

fn nth_weekday_of_month(
    year: i32,
    month: u32,
    weekday: chrono::Weekday,
    n: u32,
) -> Option<chrono::NaiveDate> {
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let first_wd = first.weekday();

    let first_offset =
        (7 + weekday.num_days_from_monday() as i32 - first_wd.num_days_from_monday() as i32) % 7;
    let day = 1 + first_offset as u32 + (n - 1) * 7;
    chrono::NaiveDate::from_ymd_opt(year, month, day)
}

/// Merge in-TUI config overrides on top of base (file/env) config.
pub(crate) fn merge_config_overrides(
    base: TuiConfig,
    overrides: &HashMap<String, String>,
) -> TuiConfig {
    let mut c = base;
    if let Some(v) = overrides.get("NATS_URL") {
        c.nats_url = v.trim().to_string();
    }
    if let Some(v) = overrides.get("BACKEND_ID") {
        let v = v.trim().to_lowercase();
        if !v.is_empty() {
            c.backend_id = v;
        }
    }
    if let Some(v) = overrides.get("TICK_MS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.tick_ms = n.max(1);
        }
    }
    if let Some(v) = overrides.get("REST_URL") {
        let value = v.trim().to_string();
        if !value.is_empty() {
            c.rest_url = value;
        }
    }
    if let Some(v) = overrides.get("REST_POLL_MS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.rest_poll_ms = n.max(1);
        }
    }
    if let Some(v) = overrides.get("REST_FALLBACK") {
        let v = v.trim().to_lowercase();
        c.rest_fallback = v == "1" || v == "true" || v == "yes";
    }
    if let Some(v) = overrides.get("SNAPSHOT_TTL_SECS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.snapshot_ttl_secs = n.max(1);
        }
    }
    if let Some(v) = overrides.get("SPLIT_PANE") {
        let v = v.trim().to_lowercase();
        c.split_pane = v == "1" || v == "true" || v == "yes";
    }
    if let Some(v) = overrides.get("BENCHMARKS_REFRESH_SECS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.benchmarks_refresh_secs = n.max(60);
        }
    }
    if let Some(v) = overrides.get("NATS_KV_BUCKET") {
        let value = v.trim().to_string();
        if !value.is_empty() {
            c.yield_kv_bucket = value;
        }
    }
    if let Some(v) = overrides.get("TUI_POSITIONS_SORT") {
        if let Some(mode) = PositionsSortMode::parse_env(v) {
            c.positions_sort = mode;
        }
    }
    if let Some(v) = overrides.get("TUI_THEME") {
        if let Some(theme) = TuiTheme::parse_env(v) {
            c.theme = theme;
        }
    }
    c
}

/// Config keys visible from the active Settings surface.
/// Legacy REST compatibility knobs are intentionally omitted here so the
/// operator-facing TUI reflects the current NATS-first runtime.
pub(crate) fn config_key_value_at(config: &TuiConfig, index: usize) -> Option<(String, String)> {
    let (key, value) = match index {
        0 => ("NATS_URL", config.nats_url.clone()),
        1 => ("BACKEND_ID", config.backend_id.clone()),
        2 => ("TICK_MS", config.tick_ms.to_string()),
        3 => ("SNAPSHOT_TTL_SECS", config.snapshot_ttl_secs.to_string()),
        4 => ("SPLIT_PANE", config.split_pane.to_string()),
        5 => (
            "BENCHMARKS_REFRESH_SECS",
            config.benchmarks_refresh_secs.to_string(),
        ),
        6 => ("NATS_KV_BUCKET", config.yield_kv_bucket.clone()),
        7 => (
            "TUI_POSITIONS_SORT",
            config.positions_sort.as_setting_value().to_string(),
        ),
        8 => ("TUI_THEME", config.theme.as_setting_value().to_string()),
        _ => return None,
    };
    Some((key.to_string(), value))
}

/// Returns a short validation hint if config is missing required fields.
pub(crate) fn validate_config_hint(config: &TuiConfig) -> Option<String> {
    let mut issues = Vec::new();
    if config.nats_url.trim().is_empty() {
        issues.push("NATS_URL empty");
    }
    if config.backend_id.trim().is_empty() {
        issues.push("BACKEND_ID empty");
    }
    if issues.is_empty() {
        None
    } else {
        Some(issues.join("; "))
    }
}
