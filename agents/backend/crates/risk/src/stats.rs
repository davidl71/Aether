/// Risk-adjusted return metrics and statistical helpers.
/// Rust port of `native/src/risk_calculator_stats.cpp`.
///
/// Implements:
///   - `mean`, `std_dev`, `percentile`, `correlation`, `beta`
///   - `annualize_return`, `annualize_volatility`
///   - `sharpe_ratio`, `sortino_ratio`, `calmar_ratio`, `information_ratio`
///   - `max_drawdown`, `current_drawdown`
// ============================================================================
// Statistical helpers
// ============================================================================
///     Arithmetic mean. Returns 0.0 for empty slice.
///
/// C++ equivalent: `calculate_mean`
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Population standard deviation. Returns 0.0 for empty slice.
///
/// C++ equivalent: `calculate_standard_deviation`
pub fn std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

/// Percentile (0.0–1.0) using the same floor-index method as the C++ version.
/// Returns 0.0 for empty slice.
///
/// C++ equivalent: `calculate_percentile`
pub fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p * sorted.len() as f64) as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Pearson correlation coefficient. Returns 0.0 if inputs differ in length,
/// are empty, or denominator is zero.
///
/// C++ equivalent: `calculate_correlation`
pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    let mx = mean(x);
    let my = mean(y);
    let (mut sum_xy, mut sum_x2, mut sum_y2) = (0.0, 0.0, 0.0);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mx;
        let dy = yi - my;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }
    let denom = (sum_x2 * sum_y2).sqrt();
    if denom == 0.0 {
        0.0
    } else {
        sum_xy / denom
    }
}

/// Beta of asset returns relative to market returns.
///
/// C++ equivalent: `calculate_beta`
pub fn beta(asset_returns: &[f64], market_returns: &[f64]) -> f64 {
    let market_std = std_dev(market_returns);
    if market_std == 0.0 {
        return 0.0;
    }
    let corr = correlation(asset_returns, market_returns);
    corr * (std_dev(asset_returns) / market_std)
}

/// Scale a period return to annual.
///
/// C++ equivalent: `annualize_return`
pub fn annualize_return(period_return: f64, periods_per_year: u32) -> f64 {
    period_return * periods_per_year as f64
}

/// Scale a period volatility to annual using square-root-of-time rule.
///
/// C++ equivalent: `annualize_volatility`
pub fn annualize_volatility(period_volatility: f64, periods_per_year: u32) -> f64 {
    period_volatility * (periods_per_year as f64).sqrt()
}

// ============================================================================
// Risk-adjusted return ratios
// ============================================================================

/// Sharpe ratio: `(mean_return - risk_free_rate) / std_dev(returns)`.
/// Returns 0.0 if returns is empty or std_dev is zero.
///
/// C++ equivalent: `RiskCalculator::calculate_sharpe_ratio`
pub fn sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let sd = std_dev(returns);
    if sd == 0.0 {
        return 0.0;
    }
    (mean(returns) - risk_free_rate) / sd
}

/// Sortino ratio: uses only downside (negative) returns for the denominator.
/// Returns `f64::INFINITY` if there are no negative returns.
/// Returns 0.0 if returns is empty or downside std_dev is zero.
///
/// C++ equivalent: `RiskCalculator::calculate_sortino_ratio`
pub fn sortino_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let m = mean(returns);
    let downside: Vec<f64> = returns.iter().copied().filter(|&r| r < 0.0).collect();
    if downside.is_empty() {
        return f64::INFINITY;
    }
    let dd = std_dev(&downside);
    if dd == 0.0 {
        return 0.0;
    }
    (m - risk_free_rate) / dd
}

/// Calmar ratio: `annualized_return / max_drawdown`.
/// Returns 0.0 if `max_drawdown` is zero.
///
/// C++ equivalent: `RiskCalculator::calculate_calmar_ratio`
pub fn calmar_ratio(annualized_return: f64, max_drawdown: f64) -> f64 {
    if max_drawdown == 0.0 {
        return 0.0;
    }
    annualized_return / max_drawdown
}

/// Information ratio: `mean(excess) / std_dev(excess)` where
/// `excess[i] = returns[i] - benchmark[i]`.
/// Returns 0.0 if slices differ in length, are empty, or tracking error is zero.
///
/// C++ equivalent: `RiskCalculator::calculate_information_ratio`
pub fn information_ratio(returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    if returns.len() != benchmark_returns.len() || returns.is_empty() {
        return 0.0;
    }
    let excess: Vec<f64> = returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(r, b)| r - b)
        .collect();
    let te = std_dev(&excess);
    if te == 0.0 {
        return 0.0;
    }
    mean(&excess) / te
}

// ============================================================================
// Drawdown
// ============================================================================

/// Maximum peak-to-trough drawdown as a fraction (0.0–1.0).
/// Returns 0.0 for empty curve.
///
/// C++ equivalent: `RiskCalculator::calculate_max_drawdown`
pub fn max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }
    let mut peak = equity_curve[0];
    let mut max_dd = 0.0;
    for &value in equity_curve {
        if value > peak {
            peak = value;
        }
        if peak > 0.0 {
            let dd = (peak - value) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

/// Current drawdown from the all-time high as a fraction (0.0–1.0).
/// Returns 0.0 for empty curve or zero peak.
///
/// C++ equivalent: `RiskCalculator::calculate_current_drawdown`
pub fn current_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }
    let peak = equity_curve
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    if peak == 0.0 {
        return 0.0;
    }
    let current = *equity_curve.last().unwrap();
    (peak - current) / peak
}

// ============================================================================
// Correlation Matrix
// ============================================================================

pub type CorrelationMatrix = Vec<Vec<f64>>;

#[allow(clippy::needless_range_loop)]
pub fn correlation_matrix(returns: &[Vec<f64>]) -> CorrelationMatrix {
    if returns.is_empty() {
        return vec![];
    }

    let n = returns.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            let corr = correlation(returns[i].as_slice(), returns[j].as_slice());
            matrix[i][j] = corr;
            matrix[j][i] = corr;
        }
    }

    matrix
}

pub fn calculate_correlation_risk(
    symbols: &[String],
    current_prices: &[f64],
    avg_prices: &[f64],
    historical_returns: &[Vec<f64>],
) -> f64 {
    let n = symbols.len();
    if n < 2 {
        return 0.0;
    }

    let mut matrix = vec![vec![0.5; n]; n];
    for (i, row) in matrix.iter_mut().enumerate().take(n) {
        row[i] = 1.0;
    }

    for i in 0..n {
        for j in (i + 1)..n {
            let corr = if historical_returns.get(i).is_none_or(|r| r.len() < 2)
                || historical_returns.get(j).is_none_or(|r| r.len() < 2)
            {
                if current_prices[i] > 0.0
                    && avg_prices[i] > 0.0
                    && current_prices[j] > 0.0
                    && avg_prices[j] > 0.0
                {
                    let ret1 = (current_prices[i] - avg_prices[i]) / avg_prices[i];
                    let ret2 = (current_prices[j] - avg_prices[j]) / avg_prices[j];
                    if (ret1 > 0.0 && ret2 > 0.0) || (ret1 < 0.0 && ret2 < 0.0) {
                        0.7
                    } else {
                        0.3
                    }
                } else {
                    0.5
                }
            } else if let (Some(ri), Some(rj)) =
                (historical_returns.get(i), historical_returns.get(j))
            {
                if ri.len() == rj.len() && ri.len() >= 2 {
                    correlation(ri, rj)
                } else {
                    0.5
                }
            } else {
                0.5
            };

            matrix[i][j] = corr;
            matrix[j][i] = corr;
        }
    }

    let total_value: f64 = current_prices.iter().sum();
    if total_value == 0.0 {
        return 0.0;
    }

    let weights: Vec<f64> = current_prices
        .iter()
        .map(|p| p.abs() / total_value)
        .collect();

    let mut variance = 0.0;
    for i in 0..n {
        for j in 0..n {
            variance += weights[i] * weights[j] * matrix[i][j];
        }
    }

    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- helpers ---

    #[test]
    fn mean_empty() {
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn mean_basic() {
        assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < EPS);
    }

    #[test]
    fn std_dev_empty() {
        assert_eq!(std_dev(&[]), 0.0);
    }

    #[test]
    fn std_dev_constant() {
        assert_eq!(std_dev(&[5.0, 5.0, 5.0]), 0.0);
    }

    #[test]
    fn std_dev_known_values() {
        // population std dev of [2, 4, 4, 4, 5, 5, 7, 9] = 2.0
        let v = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        assert!((std_dev(&v) - 2.0).abs() < EPS);
    }

    #[test]
    fn percentile_median() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        // floor(0.5*5)=2 → v[2]=3
        assert_eq!(percentile(&v, 0.5), 3.0);
    }

    #[test]
    fn correlation_perfect_positive() {
        let x = [1.0, 2.0, 3.0];
        let y = [2.0, 4.0, 6.0];
        assert!((correlation(&x, &y) - 1.0).abs() < EPS);
    }

    #[test]
    fn correlation_perfect_negative() {
        let x = [1.0, 2.0, 3.0];
        let y = [3.0, 2.0, 1.0];
        assert!((correlation(&x, &y) + 1.0).abs() < EPS);
    }

    #[test]
    fn correlation_unequal_lengths() {
        assert_eq!(correlation(&[1.0, 2.0], &[1.0]), 0.0);
    }

    #[test]
    fn beta_with_market() {
        // asset = 2 * market → beta = 2
        let market = [0.01, 0.02, -0.01, 0.03];
        let asset: Vec<f64> = market.iter().map(|r| r * 2.0).collect();
        assert!((beta(&asset, &market) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn annualize_return_daily() {
        assert!((annualize_return(0.001, 252) - 0.252).abs() < EPS);
    }

    #[test]
    fn annualize_volatility_daily() {
        // 1% daily vol → ~15.87% annual (sqrt(252)*0.01)
        let annual = annualize_volatility(0.01, 252);
        assert!((annual - 0.01 * 252f64.sqrt()).abs() < EPS);
    }

    // --- ratios ---

    #[test]
    fn sharpe_empty() {
        assert_eq!(sharpe_ratio(&[], 0.02), 0.0);
    }

    #[test]
    fn sharpe_zero_std() {
        // Use exact-zero values so population std_dev is exactly 0.0 in IEEE 754.
        assert_eq!(sharpe_ratio(&[0.0, 0.0, 0.0], 0.0), 0.0);
    }

    #[test]
    fn sharpe_positive_edge() {
        let returns = [0.10, 0.08, 0.12, 0.09];
        let rf = 0.02;
        let expected = (mean(&returns) - rf) / std_dev(&returns);
        assert!((sharpe_ratio(&returns, rf) - expected).abs() < EPS);
    }

    #[test]
    fn sortino_empty() {
        assert_eq!(sortino_ratio(&[], 0.0), 0.0);
    }

    #[test]
    fn sortino_no_negative_returns() {
        assert_eq!(sortino_ratio(&[0.05, 0.08, 0.03], 0.0), f64::INFINITY);
    }

    #[test]
    fn sortino_with_negatives() {
        let returns = [0.05, -0.03, 0.02, -0.01];
        let rf = 0.0;
        let s = sortino_ratio(&returns, rf);
        assert!(s.is_finite() && s > 0.0);
    }

    #[test]
    fn calmar_zero_drawdown() {
        assert_eq!(calmar_ratio(0.15, 0.0), 0.0);
    }

    #[test]
    fn calmar_basic() {
        assert!((calmar_ratio(0.30, 0.10) - 3.0).abs() < EPS);
    }

    #[test]
    fn information_ratio_zero_tracking_error() {
        // identical returns → zero excess → zero te
        assert_eq!(information_ratio(&[0.05, 0.05], &[0.05, 0.05]), 0.0);
    }

    #[test]
    fn information_ratio_mismatched_lengths() {
        assert_eq!(information_ratio(&[0.1, 0.2], &[0.1]), 0.0);
    }

    // --- drawdown ---

    #[test]
    fn max_drawdown_empty() {
        assert_eq!(max_drawdown(&[]), 0.0);
    }

    #[test]
    fn max_drawdown_monotonic_up() {
        assert_eq!(max_drawdown(&[1.0, 2.0, 3.0]), 0.0);
    }

    #[test]
    fn max_drawdown_known() {
        // peak=100, trough=80 → dd=0.20
        let curve = [100.0, 90.0, 80.0, 95.0];
        assert!((max_drawdown(&curve) - 0.20).abs() < EPS);
    }

    #[test]
    fn current_drawdown_at_peak() {
        let curve = [100.0, 110.0, 120.0];
        assert_eq!(current_drawdown(&curve), 0.0);
    }

    #[test]
    fn current_drawdown_below_peak() {
        let curve = [100.0, 120.0, 90.0];
        // peak=120, current=90 → dd = 30/120 = 0.25
        assert!((current_drawdown(&curve) - 0.25).abs() < EPS);
    }
}
