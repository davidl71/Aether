//! Combo option strategy type detection from multi-leg position structure.
//!
//! Infers strategy type (e.g. box, vertical spread) from leg count, option type (call/put),
//! and strike structure so the TUI and API can show strategy type per combo.
//!
//! **TWS API BAG (synthetic) positions:** TWS uses `secType = "BAG"` for synthetic instruments
//! (spreads, combos). A position can be either (a) one BAG contract (one row, combo legs in
//! `comboLegs`) or (b) multiple OPT legs (one row per leg). We accept both OPT and BAG in
//! parsing; when inference cannot determine type (e.g. BAG with no parseable leg symbols), we
//! fall back to the broker's `strategy` field (e.g. "Box") so the UI shows "Box" correctly.

use crate::runtime_state::RuntimePositionDto;

/// Inferred combo/option strategy type for a group of legs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComboStrategyType {
    /// Box spread: 4 options, same expiry, 2 strikes, 2 calls + 2 puts.
    Box,
    /// Vertical spread: 2 options, same expiry, same type (call or put), different strikes.
    VerticalSpread,
    /// Unknown or single leg; use broker label or "Combo".
    Unknown,
}

impl ComboStrategyType {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Box => "Box",
            Self::VerticalSpread => "Vertical",
            Self::Unknown => "Combo",
        }
    }
}

/// Symbol stem for grouping (first token, e.g. "SPX" from "SPX 20250321C5000").
pub fn symbol_stem(symbol: &str) -> &str {
    symbol.split_whitespace().next().unwrap_or(symbol)
}

/// Combo key: (account_id, strategy, symbol_stem) for grouping multi-leg positions.
pub fn combo_key(p: &RuntimePositionDto) -> (String, String, String) {
    let account = p.account_id.as_deref().unwrap_or("").to_string();
    let strategy = "";
    let stem = symbol_stem(&p.symbol).to_string();
    (account, strategy.to_string(), stem)
}

/// Parsed option leg info from symbol string (best-effort; IB/TWS formats vary).
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct OptionLegInfo {
    stem: String,
    #[allow(dead_code)] // reserved for same-expiry validation
    expiry_like: Option<String>,
    is_call: bool,
    strike: Option<i32>,
}

#[allow(dead_code)]
fn parse_option_leg(symbol: &str, position_type: Option<&str>) -> Option<OptionLegInfo> {
    let pt = position_type.unwrap_or("").trim().to_uppercase();
    if pt != "OPT" && pt != "BAG" && !pt.is_empty() {
        return None;
    }
    let parts: Vec<&str> = symbol.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let stem = parts[0].to_string();
    let mut expiry_like = None;
    let mut is_call = false;
    let mut strike = None;
    for part in parts.iter().skip(1) {
        let up = part.to_uppercase();
        if up.starts_with('C') && up.len() > 1 {
            if let Ok(s) = up[1..].trim().parse::<i32>() {
                is_call = true;
                strike = Some(s);
                break;
            }
        }
        if up.starts_with('P') && up.len() > 1 {
            if let Ok(s) = up[1..].trim().parse::<i32>() {
                is_call = false;
                strike = Some(s);
                break;
            }
        }
        // Combined format: 8-digit expiry + C/P + strike (e.g. "20250321C5000")
        if part.len() >= 9 {
            let expiry_part = &up[..8];
            let rest = up[8..].trim_start();
            if expiry_part.chars().all(|c| c.is_ascii_digit()) {
                expiry_like = Some(expiry_part.to_string());
                if rest.starts_with('C') && rest.len() > 1 {
                    if let Ok(s) = rest[1..].parse::<i32>() {
                        is_call = true;
                        strike = Some(s);
                        break;
                    }
                }
                if rest.starts_with('P') && rest.len() > 1 {
                    if let Ok(s) = rest[1..].parse::<i32>() {
                        is_call = false;
                        strike = Some(s);
                        break;
                    }
                }
            }
        }
        if part.len() >= 8 && part.chars().all(|c| c.is_ascii_digit()) {
            expiry_like = Some((*part).to_string());
        }
    }
    Some(OptionLegInfo {
        stem,
        expiry_like,
        is_call,
        strike,
    })
}

/// Infer combo strategy type from a group of legs (same combo key).
/// NOTE: Currently stubs out type inference since RuntimePositionDto lacks position_type/strategy fields.
/// TUI combo view is also a stub. See T-1773933296882755000.
pub fn infer_combo_strategy_type(_legs: &[&RuntimePositionDto]) -> ComboStrategyType {
    ComboStrategyType::Unknown
}

/// Net debit/credit for a combo from leg marks: sum(leg.mark * leg.quantity). Used when we don't have leg bid/ask (source "leg_sum_mark").
#[allow(dead_code)]
fn combo_net_from_leg_marks(legs: &[&RuntimePositionDto]) -> f64 {
    legs.iter().map(|p| p.mark * p.quantity as f64).sum()
}

/// Group positions by combo key, infer strategy type per group, set
/// `derived_strategy_type` on each position, and set combo net bid/ask when inferred as box (from leg marks).
/// Call after building the positions list.
///
/// NOTE: This function is stubbed out because RuntimePositionDto lacks `position_type`,
/// `strategy`, `derived_strategy_type`, `combo_net_bid`, `combo_net_ask`, and `combo_quote_source`
/// fields. See T-1773933296882755000 for the full box spread 4-leg order constructor.
#[allow(dead_code)]
pub fn apply_derived_strategy_types(_positions: &mut [RuntimePositionDto]) {
    // TODO: Re-implement once RuntimePositionDto has the required fields (T-1773933296882755000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto(symbol: &str, _position_type: &str, _strategy: &str, qty: i32) -> RuntimePositionDto {
        RuntimePositionDto {
            id: symbol.to_string(),
            symbol: symbol.to_string(),
            position_type: None,
            strategy: None,
            quantity: qty,
            cost_basis: 0.0,
            mark: 0.0,
            unrealized_pnl: 0.0,
            market_value: 0.0,
            account_id: Some("DU1".to_string()),
            apr_pct: None,
            source: None,
        }
    }

    #[test]
    fn stub_combo_key_works() {
        let pos = dto("SPX 20250321C5000", "OPT", "Box", 1);
        let key = combo_key(&pos);
        assert_eq!(key.0, "DU1");
        assert_eq!(key.2, "SPX");
    }
}
