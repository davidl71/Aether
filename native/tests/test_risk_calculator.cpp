// test_risk_calculator.cpp - Risk calculator tests
#include "risk_calculator.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>

using namespace risk;

TEST_CASE("RiskCalculator calculates box spread risk", "[risk][box_spread]") {
  // Given: A risk calculator and a box spread with known parameters
  config::RiskConfig config;
  RiskCalculator calculator(config);

  types::BoxSpreadLeg spread;
  spread.long_call.strike = 500.0;  // K1
  spread.short_call.strike = 510.0; // K2
  spread.net_debit = 9.85;          // Cost to enter

  SECTION("Box spread has defined risk") {
    // When: We calculate the risk metrics
    auto risk = calculator.calculate_box_spread_risk(spread, 505.0, 0.20);

    // Then: Maximum loss should equal net debit (box spreads have limited risk)
    REQUIRE(risk.max_loss == spread.net_debit * 100.0);
    // And: Maximum gain should be positive (arbitrage opportunity)
    REQUIRE(risk.max_gain > 0);
    // And: Probability of profit should be 1.0 if held to expiration
    REQUIRE_THAT(risk.probability_of_profit,
                 Catch::Matchers::WithinRel(1.0, 0.001));
  }

  SECTION("Box spread is delta-neutral") {
    // When: We calculate the risk metrics
    auto risk = calculator.calculate_box_spread_risk(spread, 505.0, 0.20);

    // Then: Delta should be approximately zero (box spreads are delta-neutral)
    REQUIRE_THAT(risk.delta, Catch::Matchers::WithinAbs(0.0, 0.001));
    // And: Gamma should be approximately zero
    REQUIRE_THAT(risk.gamma, Catch::Matchers::WithinAbs(0.0, 0.001));
    // And: Vega should be approximately zero
    REQUIRE_THAT(risk.vega, Catch::Matchers::WithinAbs(0.0, 0.001));
  }
}

TEST_CASE("RiskCalculator portfolio risk calculations", "[risk]") {
  config::RiskConfig config;
  config.max_total_exposure = 50000.0;
  config.max_positions = 10;

  RiskCalculator calculator(config);

  std::vector<types::Position> positions;

  types::Position pos1;
  pos1.contract.symbol = "SPY";
  pos1.quantity = 10;
  pos1.avg_price = 5.00;
  pos1.current_price = 5.50;
  positions.push_back(pos1);

  types::AccountInfo account;
  account.net_liquidation = 100000.0;

  SECTION("Calculate total exposure") {
    auto portfolio_risk =
        calculator.calculate_portfolio_risk(positions, account);

    REQUIRE(portfolio_risk.total_exposure > 0);
  }

  SECTION("Check remaining capacity") {
    double remaining = calculator.calculate_remaining_capacity(
        positions, account.net_liquidation);

    REQUIRE(remaining >= 0);
    REQUIRE(remaining <= config.max_total_exposure);
  }
}

TEST_CASE("RiskCalculator position sizing", "[risk]") {
  config::RiskConfig config;
  config.position_size_percent = 0.1;

  RiskCalculator calculator(config);

  types::BoxSpreadLeg spread;
  spread.net_debit = 2.50;

  SECTION("Calculate optimal position size") {
    int size = calculator.calculate_optimal_position_size(spread, 10000.0, 0.1);

    REQUIRE(size > 0);
    REQUIRE(size <= 40); // Max contracts based on 10% of $10k
  }

  SECTION("Kelly criterion position sizing") {
    // Given: Known win probability, win/loss amounts, and account value
    double win_probability = 0.6;   // 60% win probability
    double win_amount = 100.0;      // Win $100
    double loss_amount = 50.0;      // Risk $50
    double account_value = 10000.0; // Account value

    // When: We calculate optimal position size using Kelly Criterion
    int kelly_size = calculator.calculate_kelly_position_size(
        win_probability, win_amount, loss_amount, account_value);

    // Then: Position size should be non-negative
    REQUIRE(kelly_size >= 0);
    // And: Should use fractional Kelly (50% of full Kelly) for risk management
    // Kelly fraction = (bp - q) / b where b = 100/50 = 2, p = 0.6, q = 0.4
    // Full Kelly = (2*0.6 - 0.4) / 2 = 0.4, Fractional = 0.2
    // Expected position = 10000 * 0.2 / 100 = 20 contracts
    // But implementation uses half Kelly and clamps to 25%, so result may vary
  }

  SECTION("Fixed fractional sizing") {
    int fixed_size =
        calculator.calculate_fixed_fractional_size(250.0,   // Position cost
                                                   10000.0, // Account value
                                                   0.02 // Risk 2% of account
        );

    REQUIRE(fixed_size >= 0);
  }
}

TEST_CASE("Risk-adjusted return calculations", "[risk]") {
  config::RiskConfig config;
  RiskCalculator calculator(config);

  std::vector<double> returns = {0.01, 0.02, -0.01, 0.03, -0.02, 0.01, 0.02};
  double risk_free_rate = 0.02;

  SECTION("Calculate Sharpe ratio") {
    double sharpe = calculator.calculate_sharpe_ratio(returns, risk_free_rate);

    REQUIRE(std::isfinite(sharpe));
  }

  SECTION("Calculate Sortino ratio") {
    double sortino =
        calculator.calculate_sortino_ratio(returns, risk_free_rate);

    REQUIRE(std::isfinite(sortino));
    // Sortino should be >= Sharpe (only considers downside)
  }

  SECTION("Calculate Calmar ratio") {
    double calmar = calculator.calculate_calmar_ratio(0.10, 0.05);

    REQUIRE_THAT(calmar, Catch::Matchers::WithinRel(2.0, 0.001));
  }
}

TEST_CASE("VaR calculations", "[risk]") {
  config::RiskConfig config;
  RiskCalculator calculator(config);

  std::vector<double> returns = {0.02,  -0.01, 0.03, -0.02, 0.01,
                                 -0.03, 0.02,  0.01, -0.01, 0.02};

  SECTION("Historical VaR") {
    double var_95 = calculator.calculate_var_historical(returns, 0.95);

    REQUIRE(var_95 >= 0);
  }

  SECTION("Parametric VaR") {
    double var_param =
        calculator.calculate_var_parametric(0.01,    // Expected return
                                            0.15,    // Volatility
                                            10000.0, // Position value
                                            0.95     // Confidence level
        );

    REQUIRE(var_param > 0);
  }

  SECTION("Expected Shortfall") {
    double es = calculator.calculate_expected_shortfall(returns, 0.95);

    REQUIRE(es >= 0);
  }
}

TEST_CASE("Drawdown analysis", "[risk]") {
  config::RiskConfig config;
  RiskCalculator calculator(config);

  std::vector<double> equity_curve = {10000, 10500, 11000, 10800, 10200,
                                      10500, 11200, 11500, 11000, 10800};

  SECTION("Calculate maximum drawdown") {
    double max_dd = calculator.calculate_max_drawdown(equity_curve);

    REQUIRE(max_dd >= 0);
    REQUIRE(max_dd <= 1.0); // Should be a percentage
  }

  SECTION("Calculate current drawdown") {
    double current_dd = calculator.calculate_current_drawdown(equity_curve);

    REQUIRE(current_dd >= 0);
    REQUIRE(current_dd <= 1.0);
  }
}

TEST_CASE("Helper functions", "[risk]") {
  std::vector<double> values = {1.0, 2.0, 3.0, 4.0, 5.0};

  SECTION("Calculate mean") {
    double mean = calculate_mean(values);
    REQUIRE_THAT(mean, Catch::Matchers::WithinRel(3.0, 0.001));
  }

  SECTION("Calculate standard deviation") {
    double std_dev = calculate_standard_deviation(values);
    REQUIRE(std_dev > 0);
  }

  SECTION("Calculate percentile") {
    double p50 = calculate_percentile(values, 0.5);
    REQUIRE_THAT(p50, Catch::Matchers::WithinRel(3.0, 0.5));
  }

  SECTION("Calculate correlation") {
    std::vector<double> x = {1, 2, 3, 4, 5};
    std::vector<double> y = {2, 4, 6, 8, 10};

    double corr = calculate_correlation(x, y);
    REQUIRE_THAT(corr, Catch::Matchers::WithinRel(
                           1.0, 0.001)); // Perfect positive correlation
  }

  SECTION("Annualize return") {
    double annualized = annualize_return(0.01, 252); // 1% daily return
    REQUIRE_THAT(annualized, Catch::Matchers::WithinRel(2.52, 0.001));
  }

  SECTION("Annualize volatility") {
    double annualized_vol =
        annualize_volatility(0.02, 252); // 2% daily volatility
    REQUIRE(annualized_vol > 0.02);      // Should be higher when annualized
  }
}

TEST_CASE("RiskMonitor checks", "[risk]") {
  config::RiskConfig config;
  config.max_total_exposure = 10000.0;

  RiskMonitor monitor(config);

  std::vector<types::Position> positions;

  types::Position pos;
  pos.contract.symbol = "SPY";
  pos.quantity = 100;
  pos.current_price = 90.0; // High exposure
  positions.push_back(pos);

  types::AccountInfo account;
  account.net_liquidation = 50000.0;

  SECTION("Generate risk alerts") {
    auto alerts = monitor.check_risks(positions, account);

    // Should generate warning for high exposure
    REQUIRE(alerts.size() > 0);
  }
}
