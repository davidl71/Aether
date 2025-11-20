# Portfolio Optimization Framework for Box Spread Arbitrage

**Status:** Design Phase
**Created:** 2025-11-20
**Reference:** [Code Improvements Analysis](../analysis/code-improvements-mathematical-finance.md)

## Overview

This document designs a portfolio optimization framework for allocating capital across multiple box spread opportunities. The framework applies mathematical finance optimization techniques to maximize risk-adjusted returns while respecting capital and margin constraints.

## Objectives

1. **Maximize Risk-Adjusted Returns:** Allocate capital to maximize Sharpe ratio or minimize variance
2. **Respect Constraints:** Capital limits, margin requirements, position size limits
3. **Diversification:** Reduce portfolio risk through correlation-aware allocation
4. **Dynamic Rebalancing:** Adjust allocations as new opportunities arise

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│              Portfolio Optimizer (Main Interface)            │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                     │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│ Mean-Variance  │  │ Kelly Criterion │  │ Hierarchical   │
│ Optimizer      │  │ Optimizer        │  │ Risk Parity    │
└────────────────┘  └──────────────────┘  └────────────────┘
        │                   │                     │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                     │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│ Correlation    │  │ Risk Calculator │  │ Constraint     │
│ Analyzer       │  │ (VaR, CVaR)     │  │ Handler        │
└────────────────┘  └──────────────────┘  └────────────────┘
```

## Data Structures

### Portfolio Allocation

```cpp
struct PortfolioAllocation {
    // Opportunities and weights
    std::vector<BoxSpreadOpportunity> opportunities;
    std::vector<double> weights;              // Position weights (0.0 - 1.0)
    std::vector<int> contracts;                // Number of contracts per position

    // Portfolio metrics
    double expected_return;                    // Portfolio expected return (%)
    double portfolio_variance;                 // Portfolio variance
    double portfolio_std_dev;                  // Portfolio standard deviation
    double sharpe_ratio;                       // Risk-adjusted return
    double sortino_ratio;                      // Downside risk-adjusted return

    // Capital requirements
    double total_capital_required;             // Total capital needed
    double margin_required;                    // Total margin required
    double margin_utilization;                 // Margin utilization (%)
    double available_capital_remaining;        // Unallocated capital

    // Risk metrics
    double portfolio_var_95;                   // 95% VaR
    double portfolio_cvar_95;                  // 95% CVaR
    double max_drawdown;                       // Maximum drawdown

    // Diversification metrics
    double effective_number_positions;         // Diversification measure
    double average_correlation;                // Average correlation
    double concentration_index;               // Herfindahl index

    // Metadata
    std::chrono::system_clock::time_point generated_time;
    std::string optimization_method;           // "mean_variance", "kelly", "hrp"
    double optimization_objective_value;       // Objective function value
};
```

### Optimization Parameters

```cpp
struct PortfolioOptimizationParams {
    // Optimization objective
    enum class Objective {
        MAXIMIZE_SHARPE,      // Maximize Sharpe ratio
        MINIMIZE_VARIANCE,    // Minimum variance portfolio
        MAXIMIZE_RETURN,      // Maximize expected return
        MAXIMIZE_UTILITY       // Maximize utility (mean - λ * variance)
    } objective = Objective::MAXIMIZE_SHARPE;

    // Risk parameters
    double risk_aversion = 1.0;                // Risk aversion (lambda)
    double confidence_level = 0.95;           // VaR/CVaR confidence level

    // Capital constraints
    double available_capital = 100000.0;      // Available capital
    double max_margin_utilization = 0.8;      // Maximum margin (80%)
    double min_position_size = 0.01;          // Minimum position weight (1%)
    double max_position_size = 0.25;          // Maximum position weight (25%)
    double min_position_value = 1000.0;       // Minimum position value ($)

    // Diversification constraints
    size_t max_positions = 10;                 // Maximum number of positions
    double max_correlation = 0.7;              // Maximum correlation between positions
    double min_diversification = 0.5;          // Minimum diversification score

    // Transaction costs
    double commission_per_contract = 0.65;    // Commission per contract
    double slippage_bps = 5.0;                 // Slippage in basis points

    // Optimization algorithm parameters
    size_t max_iterations = 1000;             // Maximum iterations
    double convergence_tolerance = 1e-6;      // Convergence tolerance
    bool use_short_selling = false;            // Allow short positions
    bool rebalance_on_new_opportunities = true; // Auto-rebalance

    // Filtering criteria
    double min_implied_rate_bps = 50.0;       // Minimum implied rate (50 bps)
    double min_sharpe_ratio = 0.5;            // Minimum Sharpe ratio
    int min_days_to_expiry = 7;               // Minimum DTE
    int max_days_to_expiry = 180;             // Maximum DTE
};
```

## Optimization Methods

### 1. Mean-Variance Optimization

**Mathematical Formulation:**

```
Maximize: μ^T * w - λ * w^T * Σ * w
Subject to:
  - Σw_i = 1                    (fully invested)
  - w_i >= min_position_size    (minimum position)
  - w_i <= max_position_size    (maximum position)
  - Σw_i * cost_i <= available_capital
  - margin_requirement(w) <= max_margin
```

**Implementation:**
```cpp
class MeanVarianceOptimizer {
public:
    PortfolioAllocation optimize(
        const std::vector<BoxSpreadOpportunity>& opportunities,
        const PortfolioOptimizationParams& params);

private:
    // Calculate expected returns vector
    Eigen::VectorXd calculate_expected_returns(
        const std::vector<BoxSpreadOpportunity>& opportunities);

    // Calculate covariance matrix
    Eigen::MatrixXd calculate_covariance_matrix(
        const std::vector<BoxSpreadOpportunity>& opportunities);

    // Solve quadratic program
    Eigen::VectorXd solve_quadratic_program(
        const Eigen::MatrixXd& Q,              // Covariance matrix
        const Eigen::VectorXd& c,             // Expected returns
        const Eigen::MatrixXd& A,              // Constraint matrix
        const Eigen::VectorXd& b_lower,        // Lower bounds
        const Eigen::VectorXd& b_upper);       // Upper bounds
};
```

**Algorithm:**
1. Calculate expected returns vector `μ` from implied rates
2. Calculate covariance matrix `Σ` from historical returns or correlation
3. Set up quadratic programming problem
4. Solve using QP solver (OSQP, qpOASES, or Eigen)
5. Validate constraints (capital, margin, position sizes)
6. Calculate portfolio metrics

### 2. Kelly Criterion (Multi-Asset)

**Mathematical Formulation:**

```
f* = Σ^(-1) * μ
```

Where:
- `f*` = optimal position weights
- `Σ` = covariance matrix
- `μ` = expected returns vector

**Implementation:**
```cpp
class KellyOptimizer {
public:
    PortfolioAllocation optimize(
        const std::vector<BoxSpreadOpportunity>& opportunities,
        const PortfolioOptimizationParams& params);

private:
    // Calculate Kelly weights
    Eigen::VectorXd calculate_kelly_weights(
        const Eigen::MatrixXd& covariance_matrix,
        const Eigen::VectorXd& expected_returns);

    // Apply fractional Kelly (safety factor)
    Eigen::VectorXd apply_fractional_kelly(
        const Eigen::VectorXd& kelly_weights,
        double fraction = 0.5);
};
```

**Algorithm:**
1. Calculate covariance matrix `Σ`
2. Calculate expected returns vector `μ`
3. Compute Kelly weights: `f* = Σ^(-1) * μ`
4. Apply fractional Kelly (typically 0.5 = half Kelly)
5. Scale to available capital
6. Apply position size constraints

### 3. Hierarchical Risk Parity (HRP)

**Advantage:** Works well with highly correlated assets (box spreads on same underlying).

**Algorithm:**
1. Calculate correlation matrix
2. Build hierarchical clustering tree (linkage matrix)
3. Calculate HRP weights using inverse variance allocation
4. Apply constraints

**Implementation:**
```cpp
class HierarchicalRiskParityOptimizer {
public:
    PortfolioAllocation optimize(
        const std::vector<BoxSpreadOpportunity>& opportunities,
        const PortfolioOptimizationParams& params);

private:
    // Build hierarchical clustering tree
    Eigen::MatrixXd build_linkage_matrix(
        const Eigen::MatrixXd& correlation_matrix);

    // Calculate HRP weights
    Eigen::VectorXd calculate_hrp_weights(
        const Eigen::MatrixXd& covariance_matrix,
        const Eigen::MatrixXd& linkage_matrix);

    // Quasi-diagonalization
    Eigen::MatrixXd quasi_diagonalize(
        const Eigen::MatrixXd& covariance_matrix,
        const Eigen::MatrixXd& linkage_matrix);
};
```

## Correlation Analysis

### Correlation Factors

1. **Same Underlying:** High correlation (0.8-0.95)
2. **Different Underlyings:** Low correlation (0.1-0.3)
3. **Same Expiration:** Higher correlation than different expirations
4. **Market Regime:** Correlation increases in high volatility

### Implementation

```cpp
class CorrelationAnalyzer {
public:
    // Calculate correlation between two opportunities
    double calculate_correlation(
        const BoxSpreadOpportunity& opp1,
        const BoxSpreadOpportunity& opp2);

    // Calculate full correlation matrix
    Eigen::MatrixXd calculate_correlation_matrix(
        const std::vector<BoxSpreadOpportunity>& opportunities);

    // Find diversification pairs (low correlation)
    std::vector<std::pair<size_t, size_t>> find_diversification_pairs(
        const Eigen::MatrixXd& correlation_matrix,
        double max_correlation = 0.5);

    // Calculate portfolio diversification score
    double calculate_diversification_score(
        const Eigen::MatrixXd& correlation_matrix,
        const Eigen::VectorXd& weights);

private:
    // Historical correlation (if data available)
    double calculate_historical_correlation(
        const std::vector<double>& returns1,
        const std::vector<double>& returns2);

    // Factor-based correlation
    double calculate_factor_correlation(
        const BoxSpreadOpportunity& opp1,
        const BoxSpreadOpportunity& opp2);
};
```

## Constraint Handling

### Capital Constraints

```cpp
class ConstraintHandler {
public:
    // Validate capital constraints
    bool validate_capital_constraints(
        const PortfolioAllocation& allocation,
        const PortfolioOptimizationParams& params);

    // Validate margin constraints
    bool validate_margin_constraints(
        const PortfolioAllocation& allocation,
        const PortfolioOptimizationParams& params);

    // Apply constraints to weights
    Eigen::VectorXd apply_constraints(
        const Eigen::VectorXd& weights,
        const std::vector<BoxSpreadOpportunity>& opportunities,
        const PortfolioOptimizationParams& params);

    // Calculate margin requirement
    double calculate_margin_requirement(
        const PortfolioAllocation& allocation);
};
```

### Margin Calculation

For box spreads, margin requirements depend on:
- **Reg-T Margin:** Typically strike width difference
- **Portfolio Margin:** Reduced margin due to offsetting positions
- **SPAN Margin:** Exchange-specific margin calculation

```cpp
double calculate_box_spread_margin(
    const types::BoxSpreadLeg& spread,
    bool use_portfolio_margin = true) {

    if (use_portfolio_margin) {
        // Portfolio margin: typically 15-25% of strike width
        return spread.get_strike_width() * 0.20;
    } else {
        // Reg-T margin: typically strike width difference
        return spread.get_strike_width();
    }
}
```

## Risk Metrics

### Portfolio VaR

```cpp
double calculate_portfolio_var(
    const PortfolioAllocation& allocation,
    const Eigen::MatrixXd& covariance_matrix,
    double confidence_level = 0.95) {

    // Portfolio variance: σ²_p = w^T * Σ * w
    Eigen::VectorXd weights = Eigen::Map<const Eigen::VectorXd>(
        allocation.weights.data(), allocation.weights.size());

    double portfolio_variance = (weights.transpose() * covariance_matrix * weights)(0, 0);
    double portfolio_std = std::sqrt(portfolio_variance);

    // Z-score for confidence level
    double z_score = (confidence_level == 0.95) ? 1.645 : 2.326;

    // Portfolio VaR
    double portfolio_value = weights.sum() * allocation.total_capital_required;
    double var = portfolio_value * z_score * portfolio_std;

    return var;
}
```

### Portfolio CVaR

```cpp
double calculate_portfolio_cvar(
    const PortfolioAllocation& allocation,
    const std::vector<std::vector<double>>& historical_returns,
    double confidence_level = 0.95) {

    // Calculate portfolio returns for each historical period
    std::vector<double> portfolio_returns;
    for (const auto& period_returns : historical_returns) {
        double portfolio_return = 0.0;
        for (size_t i = 0; i < allocation.weights.size(); ++i) {
            portfolio_return += allocation.weights[i] * period_returns[i];
        }
        portfolio_returns.push_back(portfolio_return);
    }

    // Calculate CVaR
    std::sort(portfolio_returns.begin(), portfolio_returns.end());
    size_t var_index = static_cast<size_t>((1.0 - confidence_level) * portfolio_returns.size());

    double cvar = 0.0;
    for (size_t i = 0; i <= var_index && i < portfolio_returns.size(); ++i) {
        cvar += portfolio_returns[i];
    }

    return -cvar / (var_index + 1);  // Negative because losses are negative returns
}
```

## Dynamic Rebalancing

### Rebalancing Triggers

1. **New Opportunity:** Better risk-adjusted return than current positions
2. **Expiring Position:** Roll to new expiration or close
3. **Correlation Change:** Portfolio correlation exceeds threshold
4. **Margin Utilization:** Approaches limit
5. **Time-Based:** Periodic rebalancing (daily, weekly)

### Implementation

```cpp
class PortfolioRebalancer {
public:
    // Check if rebalancing is needed
    bool should_rebalance(
        const PortfolioAllocation& current_allocation,
        const std::vector<BoxSpreadOpportunity>& new_opportunities,
        const PortfolioOptimizationParams& params);

    // Rebalance portfolio
    PortfolioAllocation rebalance(
        const PortfolioAllocation& current_allocation,
        const std::vector<BoxSpreadOpportunity>& all_opportunities,
        const PortfolioOptimizationParams& params);

    // Calculate rebalancing cost
    double calculate_rebalancing_cost(
        const PortfolioAllocation& old_allocation,
        const PortfolioAllocation& new_allocation);

private:
    // Transaction cost model
    double calculate_transaction_cost(
        int contracts,
        double price_per_contract);
};
```

## Integration with Existing Code

### BoxSpreadStrategy Integration

```cpp
class BoxSpreadStrategy {
    // ... existing code ...

    // NEW: Portfolio optimization methods
    PortfolioAllocation optimize_portfolio(
        const std::vector<BoxSpreadOpportunity>& opportunities,
        const PortfolioOptimizationParams& params);

    // NEW: Multi-position execution
    bool execute_portfolio_allocation(
        const PortfolioAllocation& allocation);

    // NEW: Portfolio monitoring
    void monitor_portfolio(
        const PortfolioAllocation& allocation);
};
```

## Testing Strategy

### Unit Tests

1. **Optimization Algorithms:**
   - Mean-variance optimization with known solution
   - Kelly criterion with 2-asset case
   - HRP with correlated assets

2. **Constraint Handling:**
   - Capital constraints
   - Margin constraints
   - Position size limits

3. **Risk Metrics:**
   - Portfolio VaR calculation
   - Portfolio CVaR calculation
   - Correlation matrix properties

### Integration Tests

1. **End-to-End Optimization:**
   - Find opportunities → Optimize → Execute
   - Rebalancing workflow
   - Multi-symbol optimization

2. **Performance Tests:**
   - Optimization speed (target: < 1 second for 100 opportunities)
   - Memory usage
   - Numerical stability

## Dependencies

### Required Libraries

1. **Eigen3** - Matrix operations
   ```cmake
   find_package(Eigen3 REQUIRED)
   ```

2. **OSQP** (Optional) - Quadratic programming solver
   ```cmake
   find_package(osqp REQUIRED)
   ```

3. **Intel Decimal Library** - Already included for exact calculations

### CMake Configuration

```cmake
# Portfolio optimization library
add_library(portfolio_optimizer
    src/portfolio_optimizer.cpp
    src/mean_variance_optimizer.cpp
    src/kelly_optimizer.cpp
    src/hierarchical_risk_parity.cpp
    src/correlation_analyzer.cpp
    src/constraint_handler.cpp
)

target_link_libraries(portfolio_optimizer
    PRIVATE
    Eigen3::Eigen
    box_spread_strategy
    ${INTEL_DECIMAL_LIB}
)

# Optional: OSQP for QP solving
if(ENABLE_OSQP)
    target_link_libraries(portfolio_optimizer PRIVATE osqp::osqp)
endif()
```

## Performance Considerations

### Optimization

1. **Covariance Matrix:** Cache and update incrementally
2. **Correlation Matrix:** Pre-compute for common symbol pairs
3. **QP Solver:** Use warm starts for rebalancing
4. **Parallelization:** Calculate correlations in parallel

### Memory Management

1. **Matrix Storage:** Use sparse matrices for large portfolios
2. **Opportunity Filtering:** Filter before optimization to reduce problem size
3. **Result Caching:** Cache optimization results for similar inputs

## Future Enhancements

1. **Machine Learning:** Use ML to predict correlations and returns
2. **Real-Time Optimization:** Continuous optimization as market data updates
3. **Multi-Objective Optimization:** Pareto frontier for return vs risk
4. **Robust Optimization:** Handle uncertainty in expected returns
5. **Transaction Cost Optimization:** Minimize turnover while maintaining target allocation

## References

- [Code Improvements Analysis](../analysis/code-improvements-mathematical-finance.md)
- Markowitz, H. (1952). "Portfolio Selection"
- Kelly, J. L. (1956). "A New Interpretation of Information Rate"
- Lopez de Prado, M. (2016). "Building Diversified Portfolios that Outperform Out of Sample"
