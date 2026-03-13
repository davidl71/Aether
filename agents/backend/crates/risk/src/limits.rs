/// Risk limits checker — Rust port of the C++ RiskCalculator limits surface.
///
/// Mirrors the logic in `native/src/risk_calculator.cpp`:
///   - `is_within_limits`
///   - `is_box_spread_within_limits`
///   - `calculate_remaining_capacity`
///   - `would_exceed_limits`
///   - `check_exposure_alert` (from RiskMonitor::check_risks)
///
/// All inputs are primitive scalars — no Eigen, no QuantLib.
/// StrategyParams / RiskConfig equivalents are captured in [`RiskLimits`].
use serde::{Deserialize, Serialize};

/// Configuration mirror of C++ `config::RiskConfig` limits fields.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RiskLimits {
    /// Maximum total capital deployed across all positions (abs market value sum).
    pub max_total_exposure: f64,
    /// Maximum number of concurrent positions.
    pub max_positions: usize,
    /// Position size as a fraction of account value (e.g. 0.10 = 10%).
    pub position_size_percent: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_total_exposure: 50_000.0,
            max_positions: 10,
            position_size_percent: 0.10,
        }
    }
}

/// Stateless limits checker. Construct once per config update; call freely.
#[derive(Clone, Debug)]
pub struct LimitsChecker {
    limits: RiskLimits,
}

impl LimitsChecker {
    pub fn new(limits: RiskLimits) -> Self {
        Self { limits }
    }

    /// True if adding a position with the given market value keeps total
    /// exposure within `max_total_exposure`.
    ///
    /// C++ equivalent: `RiskCalculator::is_within_limits`
    pub fn is_within_limits(&self, position_market_value: f64, existing_exposure: f64) -> bool {
        let total = existing_exposure + position_market_value.abs();
        total <= self.limits.max_total_exposure
    }

    /// True if a box spread with the given net debit fits within both the
    /// exposure cap and the position count cap.
    ///
    /// C++ equivalent: `RiskCalculator::is_box_spread_within_limits`
    ///
    /// `net_debit` is the per-share debit; multiply by 100 (one contract) internally.
    pub fn is_box_spread_within_limits(
        &self,
        net_debit: f64,
        existing_exposure: f64,
        existing_position_count: usize,
    ) -> bool {
        let position_cost = net_debit * 100.0;
        let total = existing_exposure + position_cost;
        total <= self.limits.max_total_exposure
            && existing_position_count < self.limits.max_positions
    }

    /// Remaining capacity = min(max_total_exposure, account_value * position_size_percent)
    /// minus current exposure. Negative means already over-allocated.
    ///
    /// C++ equivalent: `RiskCalculator::calculate_remaining_capacity`
    pub fn calculate_remaining_capacity(&self, existing_exposure: f64, account_value: f64) -> f64 {
        let max_allowed = self
            .limits
            .max_total_exposure
            .min(account_value * self.limits.position_size_percent);
        max_allowed - existing_exposure
    }

    /// True if adding the position would exceed limits (inverse of `is_within_limits`).
    ///
    /// C++ equivalent: `RiskCalculator::would_exceed_limits`
    pub fn would_exceed_limits(&self, position_market_value: f64, existing_exposure: f64) -> bool {
        !self.is_within_limits(position_market_value, existing_exposure)
    }

    /// Returns a warning string if total exposure is ≥ 90% of the cap.
    ///
    /// C++ equivalent: `RiskMonitor::check_risks` exposure branch.
    pub fn exposure_alert(&self, existing_exposure: f64) -> Option<String> {
        if existing_exposure >= self.limits.max_total_exposure * 0.9 {
            Some(format!(
                "Approaching maximum exposure limit ({:.0}/{:.0})",
                existing_exposure, self.limits.max_total_exposure
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker() -> LimitsChecker {
        LimitsChecker::new(RiskLimits {
            max_total_exposure: 10_000.0,
            max_positions: 5,
            position_size_percent: 0.10,
        })
    }

    // --- is_within_limits ---

    #[test]
    fn within_limits_when_under_cap() {
        let c = checker();
        assert!(c.is_within_limits(4_000.0, 5_000.0)); // 9_000 < 10_000
    }

    #[test]
    fn within_limits_at_exactly_cap() {
        let c = checker();
        assert!(c.is_within_limits(5_000.0, 5_000.0)); // 10_000 == 10_000
    }

    #[test]
    fn exceeds_limits_above_cap() {
        let c = checker();
        assert!(!c.is_within_limits(5_001.0, 5_000.0)); // 10_001 > 10_000
    }

    #[test]
    fn within_limits_uses_abs_market_value() {
        let c = checker();
        // Short position has negative market value; still counts as exposure
        assert!(!c.is_within_limits(-6_000.0, 5_000.0));
    }

    // --- is_box_spread_within_limits ---

    #[test]
    fn box_spread_within_both_caps() {
        let c = checker();
        // net_debit=20 → cost=2_000; existing=7_000; total=9_000 < 10_000; positions=3 < 5
        assert!(c.is_box_spread_within_limits(20.0, 7_000.0, 3));
    }

    #[test]
    fn box_spread_blocked_by_exposure_cap() {
        let c = checker();
        // net_debit=20 → cost=2_000; existing=9_000; total=11_000 > 10_000
        assert!(!c.is_box_spread_within_limits(20.0, 9_000.0, 3));
    }

    #[test]
    fn box_spread_blocked_by_position_count() {
        let c = checker();
        // exposure fine but already at max_positions
        assert!(!c.is_box_spread_within_limits(10.0, 1_000.0, 5));
    }

    #[test]
    fn box_spread_multiplies_net_debit_by_100() {
        let c = checker();
        // net_debit=100 → cost=10_000; existing=1 → over cap
        assert!(!c.is_box_spread_within_limits(100.0, 1.0, 0));
    }

    // --- calculate_remaining_capacity ---

    #[test]
    fn remaining_capacity_limited_by_exposure_cap() {
        // account_value * 0.10 = 200_000; cap = 10_000 → max_allowed = 10_000
        let c = checker();
        let cap = c.calculate_remaining_capacity(3_000.0, 2_000_000.0);
        assert!((cap - 7_000.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_capacity_limited_by_position_size_percent() {
        // account_value=50_000; 0.10 * 50_000 = 5_000 < max_total_exposure 10_000
        let c = checker();
        let cap = c.calculate_remaining_capacity(1_000.0, 50_000.0);
        assert!((cap - 4_000.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_capacity_negative_when_over_allocated() {
        let c = checker();
        let cap = c.calculate_remaining_capacity(11_000.0, 2_000_000.0);
        assert!(cap < 0.0);
    }

    // --- would_exceed_limits ---

    #[test]
    fn would_exceed_is_inverse_of_within() {
        let c = checker();
        assert!(!c.would_exceed_limits(1_000.0, 8_000.0));
        assert!(c.would_exceed_limits(1_001.0, 9_999.0));
    }

    // --- exposure_alert ---

    #[test]
    fn no_alert_below_90_percent() {
        let c = checker();
        assert!(c.exposure_alert(8_999.0).is_none());
    }

    #[test]
    fn alert_at_exactly_90_percent() {
        let c = checker();
        assert!(c.exposure_alert(9_000.0).is_some());
    }

    #[test]
    fn alert_above_90_percent() {
        let c = checker();
        let msg = c.exposure_alert(9_500.0).unwrap();
        assert!(msg.contains("9500") || msg.contains("9_500") || msg.contains("Approaching"));
    }
}
