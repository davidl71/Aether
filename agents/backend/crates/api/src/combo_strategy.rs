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
use std::collections::{BTreeMap, BTreeSet, HashSet};

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
    let strategy = p.strategy.as_deref().unwrap_or("").to_string();
    let stem = symbol_stem(&p.symbol).to_string();
    (account, strategy, stem)
}

/// Parsed option leg info from symbol string (best-effort; IB/TWS formats vary).
#[derive(Clone, Debug)]
struct OptionLegInfo {
    stem: String,
    #[allow(dead_code)] // reserved for same-expiry validation
    expiry_like: Option<String>,
    is_call: bool,
    strike: Option<i32>,
}

fn parse_option_leg(symbol: &str, position_type: Option<&str>) -> Option<OptionLegInfo> {
    let pt = position_type.unwrap_or("").trim().to_uppercase();
    if pt != "OPT" && pt != "BAG" && pt != "" {
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
pub fn infer_combo_strategy_type(legs: &[&RuntimePositionDto]) -> ComboStrategyType {
    if legs.len() == 1 {
        return ComboStrategyType::Unknown;
    }
    let opt_legs: Vec<OptionLegInfo> = legs
        .iter()
        .filter_map(|p| parse_option_leg(&p.symbol, p.position_type.as_deref()))
        .collect();
    if opt_legs.len() != legs.len() {
        return ComboStrategyType::Unknown;
    }
    let stems: HashSet<&str> = opt_legs.iter().map(|l| l.stem.as_str()).collect();
    if stems.len() != 1 {
        return ComboStrategyType::Unknown;
    }
    let strikes: BTreeSet<i32> = opt_legs.iter().filter_map(|l| l.strike).collect();
    let calls = opt_legs.iter().filter(|l| l.is_call).count();
    let puts = opt_legs.len() - calls;

    if legs.len() == 4 && strikes.len() == 2 && calls == 2 && puts == 2 {
        return ComboStrategyType::Box;
    }
    if legs.len() == 2 && strikes.len() == 2 && (calls == 2 || puts == 2) {
        return ComboStrategyType::VerticalSpread;
    }
    ComboStrategyType::Unknown
}

/// Net debit/credit for a combo from leg marks: sum(leg.mark * leg.quantity). Used when we don't have leg bid/ask (source "leg_sum_mark").
fn combo_net_from_leg_marks(legs: &[&RuntimePositionDto]) -> f64 {
    legs.iter().map(|p| p.mark * p.quantity as f64).sum()
}

/// Group positions by combo key, infer strategy type per group, set
/// `derived_strategy_type` on each position, and set combo net bid/ask when inferred as box (from leg marks).
/// Call after building the positions list.
pub fn apply_derived_strategy_types(positions: &mut [RuntimePositionDto]) {
    if positions.is_empty() {
        return;
    }
    let mut groups: BTreeMap<(String, String, String), Vec<usize>> = BTreeMap::new();
    for (i, p) in positions.iter().enumerate() {
        groups.entry(combo_key(p)).or_default().push(i);
    }
    for (_, indices) in groups {
        if indices.len() == 1 {
            // Single position: still set Box when broker says so (e.g. one BAG contract for a box spread).
            let p = &positions[indices[0]];
            if p.position_type
                .as_deref()
                .map_or(false, |t| t.eq_ignore_ascii_case("BAG"))
                && p.strategy
                    .as_deref()
                    .map_or(false, |s| s.to_lowercase().contains("box"))
            {
                positions[indices[0]].derived_strategy_type = Some("Box".to_string());
            }
            continue;
        }
        let legs: Vec<&RuntimePositionDto> = indices.iter().map(|&i| &positions[i]).collect();
        let mut combo_type = infer_combo_strategy_type(&legs);
        // Fallback: if inference said Unknown but broker (or user) says it's a box spread, treat as Box.
        if matches!(combo_type, ComboStrategyType::Unknown)
            && legs.len() == 4
            && legs.iter().any(|p| {
                p.strategy
                    .as_deref()
                    .map_or(false, |s| s.to_lowercase().contains("box"))
            })
        {
            combo_type = ComboStrategyType::Box;
        }
        let label = combo_type.as_label();
        if label == "Combo" {
            continue;
        }
        let net = combo_net_from_leg_marks(&legs);
        let source = "leg_sum_mark".to_string();
        for &idx in &indices {
            positions[idx].derived_strategy_type = Some(label.to_string());
            if label == "Box" {
                positions[idx].combo_net_bid = Some(net);
                positions[idx].combo_net_ask = Some(net);
                positions[idx].combo_quote_source = Some(source.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto(symbol: &str, position_type: &str, strategy: &str, qty: i32) -> RuntimePositionDto {
        RuntimePositionDto {
            id: symbol.to_string(),
            symbol: symbol.to_string(),
            quantity: qty,
            cost_basis: 0.0,
            mark: 0.0,
            unrealized_pnl: 0.0,
            market_value: 0.0,
            account_id: Some("DU1".to_string()),
            source: Some("IB".to_string()),
            position_type: Some(position_type.to_string()),
            strategy: Some(strategy.to_string()),
            apr_pct: None,
            derived_strategy_type: None,
            combo_net_bid: None,
            combo_net_ask: None,
            combo_quote_source: None,
        }
    }

    #[test]
    fn box_four_legs_two_strikes_two_c_two_p() {
        let legs = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321C6000", "OPT", "Box", -1),
            dto("SPX 20250321P5000", "OPT", "Box", -1),
            dto("SPX 20250321P6000", "OPT", "Box", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Box);
    }

    #[test]
    fn vertical_two_legs_two_strikes_same_type() {
        let legs = vec![
            dto("SPX 20250321C5000", "OPT", "Vertical", 1),
            dto("SPX 20250321C6000", "OPT", "Vertical", -1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(
            infer_combo_strategy_type(&refs),
            ComboStrategyType::VerticalSpread
        );
    }

    #[test]
    fn single_leg_unknown() {
        let legs = vec![dto("SPX 20250321C5000", "OPT", "Single", 1)];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Unknown);
    }

    #[test]
    fn apply_derived_sets_box_on_four_legs() {
        let mut positions = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321C6000", "OPT", "Box", -1),
            dto("SPX 20250321P5000", "OPT", "Box", -1),
            dto("SPX 20250321P6000", "OPT", "Box", 1),
        ];
        apply_derived_strategy_types(&mut positions);
        assert_eq!(positions[0].derived_strategy_type.as_deref(), Some("Box"));
        assert_eq!(positions[1].derived_strategy_type.as_deref(), Some("Box"));
        assert_eq!(positions[2].derived_strategy_type.as_deref(), Some("Box"));
        assert_eq!(positions[3].derived_strategy_type.as_deref(), Some("Box"));
    }

    #[test]
    fn apply_derived_fallback_box_from_broker_strategy() {
        // Inference returns Unknown (e.g. symbol format not parsed), but broker sends strategy "Box" → still show Box.
        let mut positions = vec![
            dto("SPX 20250321", "BAG", "Box", 1),
            dto("SPX 20250321", "BAG", "Box", -1),
            dto("SPX 20250321", "BAG", "Box", -1),
            dto("SPX 20250321", "BAG", "Box", 1),
        ];
        apply_derived_strategy_types(&mut positions);
        assert_eq!(positions[0].derived_strategy_type.as_deref(), Some("Box"));
        assert_eq!(positions[3].derived_strategy_type.as_deref(), Some("Box"));
    }

    #[test]
    fn apply_derived_single_bag_box() {
        // Single BAG position (one row from TWS) with strategy "Box" → derived_strategy_type set so UI shows "Box".
        let mut positions = vec![dto("SPX", "BAG", "Box spread", 1)];
        apply_derived_strategy_types(&mut positions);
        assert_eq!(positions[0].derived_strategy_type.as_deref(), Some("Box"));
    }

    #[test]
    fn apply_derived_sets_combo_net_for_box() {
        // Box group: combo_net_bid/ask = sum(leg.mark * leg.quantity), source = leg_sum_mark.
        let mut positions = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321C6000", "OPT", "Box", -1),
            dto("SPX 20250321P5000", "OPT", "Box", -1),
            dto("SPX 20250321P6000", "OPT", "Box", 1),
        ];
        positions[0].mark = 10.0;
        positions[1].mark = 5.0;
        positions[2].mark = 2.0;
        positions[3].mark = 7.0;
        apply_derived_strategy_types(&mut positions);
        let net = 10.0 * 1.0 + 5.0 * (-1.0) + 2.0 * (-1.0) + 7.0 * 1.0; // 10 - 5 - 2 + 7 = 10
        for p in &positions {
            assert_eq!(p.combo_net_bid, Some(net));
            assert_eq!(p.combo_net_ask, Some(net));
            assert_eq!(p.combo_quote_source.as_deref(), Some("leg_sum_mark"));
        }
    }
}
