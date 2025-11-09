// risk_calculator.h - Risk management and calculation
#pragma once

#include "types.h"
#include "config_manager.h"
#include <vector>
#include <optional>

namespace risk {

// ============================================================================
// Portfolio Risk Metrics
// ============================================================================

struct PortfolioRisk {
    double total_exposure;          // Total capital at risk
    double total_delta;             // Net delta exposure
    double total_gamma;             // Net gamma exposure
    double total_theta;             // Net theta exposure
    double total_vega;              // Net vega exposure

    double var_95;                  // Value at Risk (95% confidence)
    double var_99;                  // Value at Risk (99% confidence)
    double expected_shortfall;      // Conditional VaR

    double concentration_risk;      // Risk from concentrated positions
    double liquidity_risk;          // Risk from illiquid positions

    double sharpe_ratio;            // Risk-adjusted return
    double sortino_ratio;           // Downside risk-adjusted return
    double max_drawdown;            // Maximum drawdown
};

// ============================================================================
// Position Risk Metrics
// ============================================================================

struct PositionRisk {
    double position_size;           // Size of position
    double max_loss;                // Maximum possible loss
    double max_gain;                // Maximum possible gain
    double expected_value;          // Expected P&L
    double probability_of_profit;   // Probability of profit

    // Greeks
    double delta;
    double gamma;
    double theta;
    double vega;
    double rho;

    // Additional metrics
    double leverage;                // Leverage ratio
    double risk_reward_ratio;       // Risk/reward ratio
    double kelly_criterion;         // Optimal position size
};

// ============================================================================
// Risk Calculator Class
// ============================================================================

class RiskCalculator {
public:
    explicit RiskCalculator(const config::RiskConfig& config);
    ~RiskCalculator();

    // ========================================================================
    // Position Risk Analysis
    // ========================================================================

    // Calculate risk for box spread
    PositionRisk calculate_box_spread_risk(
        const types::BoxSpreadLeg& spread,
        double underlying_price,
        double implied_volatility
    ) const;

    // Calculate risk for single position
    PositionRisk calculate_position_risk(
        const types::Position& position,
        double underlying_price,
        double implied_volatility
    ) const;

    // Calculate maximum loss for position
    double calculate_max_loss(const types::Position& position) const;

    // Calculate maximum gain for position
    double calculate_max_gain(const types::Position& position) const;

    // ========================================================================
    // Portfolio Risk Analysis
    // ========================================================================

    // Calculate portfolio-level risk
    PortfolioRisk calculate_portfolio_risk(
        const std::vector<types::Position>& positions,
        const types::AccountInfo& account
    ) const;

    // Calculate aggregate Greeks
    types::RiskMetrics calculate_aggregate_greeks(
        const std::vector<types::Position>& positions
    ) const;

    // Calculate correlation risk
    double calculate_correlation_risk(
        const std::vector<types::Position>& positions
    ) const;

    // ========================================================================
    // Risk Limits and Validation
    // ========================================================================

    // Check if position is within limits
    bool is_within_limits(
        const types::Position& position,
        const std::vector<types::Position>& existing_positions
    ) const;

    // Check if box spread is within limits
    bool is_box_spread_within_limits(
        const types::BoxSpreadLeg& spread,
        const std::vector<types::Position>& existing_positions
    ) const;

    // Calculate remaining capacity
    double calculate_remaining_capacity(
        const std::vector<types::Position>& positions,
        double account_value
    ) const;

    // Check if adding position exceeds limits
    bool would_exceed_limits(
        const types::Position& new_position,
        const std::vector<types::Position>& existing_positions,
        double account_value
    ) const;

    // ========================================================================
    // Position Sizing
    // ========================================================================

    // Calculate optimal position size
    int calculate_optimal_position_size(
        const types::BoxSpreadLeg& spread,
        double account_value,
        double risk_tolerance
    ) const;

    // Calculate position size using Kelly Criterion
    int calculate_kelly_position_size(
        double win_probability,
        double win_amount,
        double loss_amount,
        double account_value
    ) const;

    // Calculate fixed-fractional position size
    int calculate_fixed_fractional_size(
        double position_cost,
        double account_value,
        double risk_percent
    ) const;

    // ========================================================================
    // Value at Risk (VaR) Calculations
    // ========================================================================

    // Calculate VaR using historical simulation
    double calculate_var_historical(
        const std::vector<double>& returns,
        double confidence_level
    ) const;

    // Calculate VaR using parametric method
    double calculate_var_parametric(
        double expected_return,
        double volatility,
        double position_value,
        double confidence_level,
        int time_horizon_days = 1
    ) const;

    // Calculate VaR using Monte Carlo simulation
    double calculate_var_monte_carlo(
        const types::Position& position,
        double underlying_price,
        double volatility,
        int simulations,
        double confidence_level
    ) const;

    // Calculate Expected Shortfall (CVaR)
    double calculate_expected_shortfall(
        const std::vector<double>& returns,
        double confidence_level
    ) const;

    // ========================================================================
    // Scenario Analysis
    // ========================================================================

    struct ScenarioResult {
        std::string scenario_name;
        double price_change_percent;
        double position_pnl;
        double portfolio_pnl;
    };

    // Run scenario analysis on position
    std::vector<ScenarioResult> run_scenario_analysis(
        const types::Position& position,
        double current_price,
        const std::vector<double>& price_scenarios
    ) const;

    // Stress test portfolio
    std::vector<ScenarioResult> stress_test_portfolio(
        const std::vector<types::Position>& positions,
        const std::vector<double>& market_scenarios
    ) const;

    // ========================================================================
    // Greeks Calculations
    // ========================================================================

    // Calculate delta for box spread
    double calculate_box_spread_delta(
        const types::BoxSpreadLeg& spread
    ) const;

    // Calculate all Greeks for box spread
    types::RiskMetrics calculate_box_spread_greeks(
        const types::BoxSpreadLeg& spread,
        double underlying_price,
        double volatility
    ) const;

    // ========================================================================
    // Risk-Adjusted Returns
    // ========================================================================

    // Calculate Sharpe ratio
    double calculate_sharpe_ratio(
        const std::vector<double>& returns,
        double risk_free_rate
    ) const;

    // Calculate Sortino ratio
    double calculate_sortino_ratio(
        const std::vector<double>& returns,
        double risk_free_rate
    ) const;

    // Calculate Calmar ratio
    double calculate_calmar_ratio(
        double annualized_return,
        double max_drawdown
    ) const;

    // Calculate information ratio
    double calculate_information_ratio(
        const std::vector<double>& returns,
        const std::vector<double>& benchmark_returns
    ) const;

    // ========================================================================
    // Drawdown Analysis
    // ========================================================================

    // Calculate maximum drawdown
    double calculate_max_drawdown(
        const std::vector<double>& equity_curve
    ) const;

    // Calculate current drawdown
    double calculate_current_drawdown(
        const std::vector<double>& equity_curve
    ) const;

    // Calculate drawdown duration
    int calculate_drawdown_duration(
        const std::vector<double>& equity_curve
    ) const;

    // ========================================================================
    // Configuration
    // ========================================================================

    // Update risk configuration
    void update_config(const config::RiskConfig& config);

    // Get current configuration
    config::RiskConfig get_config() const;

private:
    class Impl;
    std::unique_ptr<Impl> pimpl_;
};

// ============================================================================
// Risk Alerts
// ============================================================================

enum class RiskAlertLevel {
    Info,
    Warning,
    Critical
};

struct RiskAlert {
    RiskAlertLevel level;
    std::string message;
    std::string category;  // e.g., "EXPOSURE", "CONCENTRATION", "LIQUIDITY"
    std::chrono::system_clock::time_point timestamp;
};

class RiskMonitor {
public:
    explicit RiskMonitor(const config::RiskConfig& config);

    // Check for risk alerts
    std::vector<RiskAlert> check_risks(
        const std::vector<types::Position>& positions,
        const types::AccountInfo& account
    ) const;

    // Check position-level risks
    std::vector<RiskAlert> check_position_risks(
        const types::Position& position
    ) const;

    // Check portfolio-level risks
    std::vector<RiskAlert> check_portfolio_risks(
        const PortfolioRisk& portfolio_risk,
        const types::AccountInfo& account
    ) const;

private:
    config::RiskConfig config_;
};

// ============================================================================
// Helper Functions
// ============================================================================

// Calculate standard deviation
double calculate_standard_deviation(const std::vector<double>& values);

// Calculate mean
double calculate_mean(const std::vector<double>& values);

// Calculate percentile
double calculate_percentile(
    const std::vector<double>& values,
    double percentile
);

// Calculate correlation coefficient
double calculate_correlation(
    const std::vector<double>& x,
    const std::vector<double>& y
);

// Calculate beta
double calculate_beta(
    const std::vector<double>& asset_returns,
    const std::vector<double>& market_returns
);

// Annualize return
double annualize_return(double period_return, int periods_per_year);

// Annualize volatility
double annualize_volatility(double period_volatility, int periods_per_year);

} // namespace risk
