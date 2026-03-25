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

use crate::snapshot_view::RuntimePositionDto;

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
struct OptionLegInfo {
    stem: String,
    expiry_like: Option<String>,
    is_call: bool,
    strike: Option<i32>,
}

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
/// Uses symbol parsing to detect Box (4 legs: 2 calls + 2 puts, same expiry, 2 strikes)
/// vs VerticalSpread (2 legs: same expiry, same type, different strikes).
pub fn infer_combo_strategy_type(legs: &[&RuntimePositionDto]) -> ComboStrategyType {
    if legs.is_empty() {
        return ComboStrategyType::Unknown;
    }

    let parsed: Vec<Option<OptionLegInfo>> = legs
        .iter()
        .map(|p| parse_option_leg(&p.symbol, p.position_type.as_deref()))
        .collect();

    if parsed.iter().all(|p| p.is_some()) {
        let legs_info: Vec<&OptionLegInfo> = parsed.iter().filter_map(|p| p.as_ref()).collect();

        if legs_info.len() == 4 {
            return infer_box_spread(&legs_info);
        }
        if legs_info.len() == 2 {
            return infer_vertical_spread(&legs_info);
        }
    }

    ComboStrategyType::Unknown
}

fn infer_box_spread(legs: &[&OptionLegInfo]) -> ComboStrategyType {
    let stems: Vec<&str> = legs.iter().map(|l| l.stem.as_str()).collect();
    if !stems.iter().all(|s| *s == stems[0]) {
        return ComboStrategyType::Unknown;
    }

    let strikes: Vec<i32> = legs.iter().filter_map(|l| l.strike).collect();
    let strikes_unique: Vec<i32> = strikes
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    if strikes_unique.len() != 2 {
        return ComboStrategyType::Unknown;
    }

    let calls = legs.iter().filter(|l| l.is_call).count();
    let puts = legs.iter().filter(|l| !l.is_call).count();
    if calls != 2 || puts != 2 {
        return ComboStrategyType::Unknown;
    }

    let expiries: Vec<&str> = legs
        .iter()
        .filter_map(|l| l.expiry_like.as_deref())
        .collect();
    if !expiries.is_empty() && !expiries.iter().all(|e| *e == expiries[0]) {
        return ComboStrategyType::Unknown;
    }

    ComboStrategyType::Box
}

fn infer_vertical_spread(legs: &[&OptionLegInfo]) -> ComboStrategyType {
    if legs.len() != 2 {
        return ComboStrategyType::Unknown;
    }

    let stems: Vec<&str> = legs.iter().map(|l| l.stem.as_str()).collect();
    if !stems.iter().all(|s| *s == stems[0]) {
        return ComboStrategyType::Unknown;
    }

    let expiries: Vec<&str> = legs
        .iter()
        .filter_map(|l| l.expiry_like.as_deref())
        .collect();
    if !expiries.is_empty() && !expiries.iter().all(|e| *e == expiries[0]) {
        return ComboStrategyType::Unknown;
    }

    let strikes: Vec<i32> = legs.iter().filter_map(|l| l.strike).collect();
    if strikes.len() != 2 || strikes[0] == strikes[1] {
        return ComboStrategyType::Unknown;
    }

    ComboStrategyType::VerticalSpread
}

/// Net debit/credit for a combo from leg marks: sum(leg.mark * leg.quantity). Used when we don't have leg bid/ask (source "leg_sum_mark").
#[allow(dead_code)]
fn combo_net_from_leg_marks(legs: &[&RuntimePositionDto]) -> f64 {
    legs.iter().map(|p| p.mark * p.quantity as f64).sum()
}

/// Group positions by combo key (account_id, symbol_stem), infer strategy type per group,
/// and set `position_type` and `strategy` on each position. Call after building positions list.
pub fn apply_derived_strategy_types(positions: &mut [RuntimePositionDto]) {
    use std::collections::HashMap;

    let mut groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, pos) in positions.iter().enumerate() {
        let key = combo_key(pos);
        groups.entry((key.0, key.2)).or_default().push(i);
    }

    for ((_account, _stem), indices) in groups {
        let legs: Vec<&RuntimePositionDto> = indices.iter().map(|&i| &positions[i]).collect();
        let strategy = infer_combo_strategy_type(&legs);
        let label = strategy.as_label().to_string();

        for &i in &indices {
            positions[i].position_type = Some(label.clone());
            positions[i].strategy = Some(label.clone());
        }
    }
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

    #[test]
    fn infer_box_spread_detects_4_leg_box() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321P5000", "OPT", "Box", 1),
            dto("SPX 20250321C5010", "OPT", "Box", 1),
            dto("SPX 20250321P5010", "OPT", "Box", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Box);
    }

    #[test]
    fn infer_box_spread_rejects_wrong_leg_count() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321P5000", "OPT", "Box", 1),
            dto("SPX 20250321C5010", "OPT", "Box", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Unknown);
    }

    #[test]
    fn infer_box_spread_rejects_same_strike_all_calls() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321C5010", "OPT", "Box", 1),
            dto("SPX 20250321C5010", "OPT", "Box", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Unknown);
    }

    #[test]
    fn infer_vertical_spread_detects_call_spread() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Vertical", 1),
            dto("SPX 20250321C5010", "OPT", "Vertical", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(
            infer_combo_strategy_type(&refs),
            ComboStrategyType::VerticalSpread
        );
    }

    #[test]
    fn infer_vertical_spread_detects_put_spread() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321P5000", "OPT", "Vertical", 1),
            dto("SPX 20250321P5010", "OPT", "Vertical", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(
            infer_combo_strategy_type(&refs),
            ComboStrategyType::VerticalSpread
        );
    }

    #[test]
    fn infer_vertical_spread_rejects_same_strike() {
        let legs: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Vertical", 1),
            dto("SPX 20250321C5000", "OPT", "Vertical", 1),
        ];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Unknown);
    }

    #[test]
    fn infer_unknown_for_single_leg() {
        let legs: Vec<RuntimePositionDto> = vec![dto("SPX 20250321C5000", "OPT", "Box", 1)];
        let refs: Vec<&RuntimePositionDto> = legs.iter().collect();
        assert_eq!(infer_combo_strategy_type(&refs), ComboStrategyType::Unknown);
    }

    #[test]
    fn apply_derived_strategy_types_sets_box_strategy() {
        let mut positions: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Box", 1),
            dto("SPX 20250321P5000", "OPT", "Box", 1),
            dto("SPX 20250321C5010", "OPT", "Box", 1),
            dto("SPX 20250321P5010", "OPT", "Box", 1),
        ];
        apply_derived_strategy_types(&mut positions);
        for pos in &positions {
            assert_eq!(pos.position_type.as_deref(), Some("Box"));
            assert_eq!(pos.strategy.as_deref(), Some("Box"));
        }
    }

    #[test]
    fn apply_derived_strategy_types_sets_vertical_strategy() {
        let mut positions: Vec<RuntimePositionDto> = vec![
            dto("SPX 20250321C5000", "OPT", "Vertical", 1),
            dto("SPX 20250321C5010", "OPT", "Vertical", 1),
        ];
        apply_derived_strategy_types(&mut positions);
        for pos in &positions {
            assert_eq!(pos.position_type.as_deref(), Some("Vertical"));
            assert_eq!(pos.strategy.as_deref(), Some("Vertical"));
        }
    }

    #[test]
    fn apply_derived_strategy_types_unknown_remains_unknown() {
        let mut positions: Vec<RuntimePositionDto> =
            vec![dto("SPX 20250321C5000", "OPT", "Box", 1)];
        apply_derived_strategy_types(&mut positions);
        assert_eq!(positions[0].position_type.as_deref(), Some("Combo"));
        assert_eq!(positions[0].strategy.as_deref(), Some("Combo"));
    }
}
