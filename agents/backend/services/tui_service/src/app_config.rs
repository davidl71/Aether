use std::collections::HashMap;

use crate::config::{PositionsSortMode, TuiConfig};

/// Returns `Some(true)` when NYSE is currently open, `Some(false)` when closed, `None` on error.
/// Uses NYSE as proxy for options market hours (CBOE follows NYSE schedule).
pub(crate) fn nyse_is_open() -> Option<bool> {
    use trading_calendar::{Market, TradingCalendar};
    let cal = TradingCalendar::new(Market::NYSE).ok()?;
    cal.is_open_now().ok()
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
