/// Position sizing — Rust port of `native/src/risk_calculator_sizing.cpp`.
///
/// Implements:
///   - `calculate_optimal_position_size` — exposure-based max contracts
///   - `calculate_kelly_position_size`   — half-Kelly criterion
///   - `calculate_fixed_fractional_size` — fixed-fraction of account
///
/// All inputs and outputs are primitive scalars.
/// Exposure-based position size: how many contracts fit within the risk
/// budget defined by `risk_tolerance` (fraction of account).
///
/// C++ equivalent: `RiskCalculator::calculate_optimal_position_size`
///
/// Returns at least 1.
pub fn calculate_optimal_position_size(
    net_debit: f64,
    account_value: f64,
    risk_tolerance: f64,
) -> u32 {
    let position_cost = net_debit * 100.0;
    if position_cost <= 0.0 {
        return 1;
    }
    let max_exposure = account_value * risk_tolerance;
    let contracts = (max_exposure / position_cost) as u32;
    contracts.max(1)
}

/// Half-Kelly criterion position size in contracts.
///
/// Formula: `f = (b*p - q) / b` where `b = win/loss`, `p = win prob`,
/// `q = 1 - p`. Result is halved (fractional Kelly), clamped to [0, 25%]
/// of account, then converted to contracts at $100/contract.
///
/// Returns 0 when `loss_amount == 0` or Kelly fraction is negative.
///
/// C++ equivalent: `RiskCalculator::calculate_kelly_position_size`
pub fn calculate_kelly_position_size(
    win_probability: f64,
    win_amount: f64,
    loss_amount: f64,
    account_value: f64,
) -> u32 {
    if loss_amount == 0.0 {
        return 0;
    }

    let b = win_amount / loss_amount;
    let p = win_probability;
    let q = 1.0 - p;

    let kelly_fraction = (b * p - q) / b;
    // Half Kelly
    let kelly_fraction = kelly_fraction * 0.5;
    // Clamp to [0, 25%]
    let kelly_fraction = kelly_fraction.clamp(0.0, 0.25);

    let position_size = account_value * kelly_fraction;
    (position_size / 100.0) as u32
}

/// Fixed-fractional position size.
///
/// `risk_percent` fraction of `account_value` divided by `position_cost`
/// gives the number of contracts. Returns at least 1.
///
/// C++ equivalent: `RiskCalculator::calculate_fixed_fractional_size`
pub fn calculate_fixed_fractional_size(
    position_cost: f64,
    account_value: f64,
    risk_percent: f64,
) -> u32 {
    if position_cost <= 0.0 {
        return 1;
    }
    let risk_amount = account_value * risk_percent;
    let contracts = (risk_amount / position_cost) as u32;
    contracts.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- calculate_optimal_position_size ---

    #[test]
    fn optimal_size_basic() {
        // net_debit=10 → cost=1000; budget=100_000*0.1=10_000 → 10 contracts
        assert_eq!(calculate_optimal_position_size(10.0, 100_000.0, 0.10), 10);
    }

    #[test]
    fn optimal_size_minimum_one() {
        // risk_tolerance so small result would be 0 → clamp to 1
        assert_eq!(calculate_optimal_position_size(500.0, 1_000.0, 0.001), 1);
    }

    #[test]
    fn optimal_size_zero_cost_returns_one() {
        assert_eq!(calculate_optimal_position_size(0.0, 100_000.0, 0.1), 1);
    }

    // --- calculate_kelly_position_size ---

    #[test]
    fn kelly_zero_loss_returns_zero() {
        assert_eq!(calculate_kelly_position_size(0.6, 100.0, 0.0, 50_000.0), 0);
    }

    #[test]
    fn kelly_negative_edge_returns_zero() {
        // win_prob=0.2, win=1, loss=1 → kelly=(1*0.2-0.8)/1=-0.6 → clamped to 0
        assert_eq!(calculate_kelly_position_size(0.2, 1.0, 1.0, 50_000.0), 0);
    }

    #[test]
    fn kelly_clamped_at_25_percent() {
        // Extreme edge: huge b, p≈1 → raw kelly close to 1, half-kelly=0.5 → clamped 0.25
        // account=100_000; 0.25*100_000/100 = 250 contracts
        let contracts = calculate_kelly_position_size(0.99, 1000.0, 1.0, 100_000.0);
        assert_eq!(contracts, 250);
    }

    #[test]
    fn kelly_symmetric_coin_flip() {
        // b=1, p=0.5, q=0.5 → kelly=(0.5-0.5)/1=0 → 0 contracts
        assert_eq!(calculate_kelly_position_size(0.5, 1.0, 1.0, 100_000.0), 0);
    }

    #[test]
    fn kelly_positive_edge() {
        // b=2 (win 2, lose 1), p=0.6 → kelly=(2*0.6-0.4)/2 → half-kelly → ~0.2
        // Float truncation (matching C++ static_cast<int>): result is 19 not 20.
        let contracts = calculate_kelly_position_size(0.6, 2.0, 1.0, 10_000.0);
        assert_eq!(contracts, 19);
    }

    // --- calculate_fixed_fractional_size ---

    #[test]
    fn fixed_fractional_basic() {
        // risk_amount = 50_000*0.02 = 1000; cost=500 → 2 contracts
        assert_eq!(calculate_fixed_fractional_size(500.0, 50_000.0, 0.02), 2);
    }

    #[test]
    fn fixed_fractional_minimum_one() {
        // risk_amount very small → clamp to 1
        assert_eq!(calculate_fixed_fractional_size(10_000.0, 100.0, 0.001), 1);
    }

    #[test]
    fn fixed_fractional_zero_cost_returns_one() {
        assert_eq!(calculate_fixed_fractional_size(0.0, 50_000.0, 0.02), 1);
    }
}
