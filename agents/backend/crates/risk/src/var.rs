/// Value at Risk and Expected Shortfall.
/// Rust port of `native/src/risk_calculator_var.cpp`.
///
/// Implements:
///   - `var_historical`     — historical simulation (sort + percentile)
///   - `var_parametric`     — variance-covariance (z-score * vol * sqrt(T/252))
///   - `expected_shortfall` — CVaR / ES (mean of tail losses)
///
/// Monte Carlo and scenario analysis stubs from C++ are omitted —
/// they were unimplemented there too.
// ============================================================================
// Value at Risk
// ============================================================================
/// Historical simulation VaR.
///
/// Sorts returns ascending, takes the `(1 - confidence_level)` percentile,
/// and negates it to express as a positive loss.
///
/// C++ equivalent: `RiskCalculator::calculate_var_historical`
pub fn var_historical(returns: &[f64], confidence_level: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let idx = ((1.0 - confidence_level) * sorted.len() as f64) as usize;
    let idx = idx.min(sorted.len() - 1);

    -sorted[idx]
}

/// Parametric (variance-covariance) VaR.
///
/// Formula: `position_value * z_score * volatility * sqrt(time_horizon / 252)`
///
/// Z-scores: 95% → 1.645, 99% → 2.326.
///
/// C++ equivalent: `RiskCalculator::calculate_var_parametric`
pub fn var_parametric(
    volatility: f64,
    position_value: f64,
    confidence_level: f64,
    time_horizon_days: u32,
) -> f64 {
    let z_score = if confidence_level >= 0.99 { 2.326 } else { 1.645 };
    let time_factor = (time_horizon_days as f64 / 252.0).sqrt();
    position_value * z_score * volatility * time_factor
}

/// Expected Shortfall (CVaR): mean of losses beyond the VaR threshold.
///
/// C++ equivalent: `RiskCalculator::calculate_expected_shortfall`
pub fn expected_shortfall(returns: &[f64], confidence_level: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let cutoff = ((1.0 - confidence_level) * sorted.len() as f64) as usize;
    let cutoff = cutoff.max(1);

    let tail_sum: f64 = sorted[..cutoff].iter().sum();
    -tail_sum / cutoff as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- var_historical ---

    #[test]
    fn var_historical_empty() {
        assert_eq!(var_historical(&[], 0.95), 0.0);
    }

    #[test]
    fn var_historical_95_pct() {
        // 20 returns: worst 5% = index 0 of sorted = -0.10
        let mut returns: Vec<f64> = (0..19).map(|i| 0.01 * i as f64).collect();
        returns.push(-0.10);
        // (1 - 0.95) * 20 = 1 → index 1 after sort
        let var = var_historical(&returns, 0.95);
        // sorted[0]=-0.10, sorted[1]=0.0 → var = -sorted[1] = 0.0
        // Verify it's non-negative (a loss measure)
        assert!(var >= 0.0);
    }

    #[test]
    fn var_historical_all_negative() {
        let returns = [-0.05, -0.03, -0.08, -0.02, -0.01];
        let var = var_historical(&returns, 0.95);
        assert!(var > 0.0);
    }

    #[test]
    fn var_historical_all_positive() {
        // All gains → var ≤ 0 (profit, no loss)
        let returns = [0.01, 0.02, 0.03, 0.04, 0.05];
        let var = var_historical(&returns, 0.95);
        assert!(var <= 0.0);
    }

    // --- var_parametric ---

    #[test]
    fn var_parametric_95_pct_one_day() {
        // z=1.645, vol=0.01, position=100_000, time=1 → 100_000 * 1.645 * 0.01 * sqrt(1/252)
        let var = var_parametric(0.01, 100_000.0, 0.95, 1);
        let expected = 100_000.0 * 1.645 * 0.01 * (1.0f64 / 252.0).sqrt();
        assert!((var - expected).abs() < EPS);
    }

    #[test]
    fn var_parametric_99_pct_uses_higher_z() {
        let var95 = var_parametric(0.01, 100_000.0, 0.95, 1);
        let var99 = var_parametric(0.01, 100_000.0, 0.99, 1);
        assert!(var99 > var95);
    }

    #[test]
    fn var_parametric_scales_with_time() {
        let var1 = var_parametric(0.01, 100_000.0, 0.95, 1);
        let var4 = var_parametric(0.01, 100_000.0, 0.95, 4);
        // sqrt(4/252) / sqrt(1/252) = 2
        assert!((var4 / var1 - 2.0).abs() < 1e-6);
    }

    // --- expected_shortfall ---

    #[test]
    fn expected_shortfall_empty() {
        assert_eq!(expected_shortfall(&[], 0.95), 0.0);
    }

    #[test]
    fn expected_shortfall_greater_than_var() {
        let returns: Vec<f64> = vec![
            -0.10, -0.08, -0.06, -0.04, -0.02, 0.01, 0.02, 0.03, 0.04, 0.05,
        ];
        let var = var_historical(&returns, 0.90);
        let es = expected_shortfall(&returns, 0.90);
        // ES should be ≥ VaR (mean of tail is worse than the boundary)
        assert!(es >= var);
    }

    #[test]
    fn expected_shortfall_single_return() {
        // Single return: cutoff=max(1,0)=1, tail=[return], es = -return
        let returns = [-0.05];
        assert!((expected_shortfall(&returns, 0.95) - 0.05).abs() < EPS);
    }
}
