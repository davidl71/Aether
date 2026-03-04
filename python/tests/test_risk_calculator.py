"""Tests for risk_calculator.py - Python port of C++ risk calculator."""

import math

import pytest

from python.integration.risk_calculator import (
    RiskAlertLevel,
    RiskCalculator,
    RiskConfig,
    RiskMonitor,
    SimpleBoxSpreadLeg,
    SimplePosition,
    annualize_return,
    annualize_volatility,
    calculate_beta,
    calculate_correlation,
    calculate_mean,
    calculate_percentile,
    calculate_standard_deviation,
)


# ---------------------------------------------------------------------------
# Statistical helpers
# ---------------------------------------------------------------------------


class TestStatisticalHelpers:
    def test_mean_empty(self):
        assert calculate_mean([]) == 0.0

    def test_mean(self):
        assert calculate_mean([1.0, 2.0, 3.0]) == pytest.approx(2.0)

    def test_std_empty(self):
        assert calculate_standard_deviation([]) == 0.0

    def test_std_constant(self):
        assert calculate_standard_deviation([5.0, 5.0, 5.0]) == pytest.approx(0.0)

    def test_std_known(self):
        vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
        std = calculate_standard_deviation(vals)
        assert std > 0

    def test_percentile_empty(self):
        assert calculate_percentile([], 0.5) == 0.0

    def test_percentile_median(self):
        assert calculate_percentile([1, 2, 3, 4, 5], 0.5) == 3

    def test_correlation_identical(self):
        x = [1.0, 2.0, 3.0, 4.0, 5.0]
        assert calculate_correlation(x, x) == pytest.approx(1.0)

    def test_correlation_inverse(self):
        x = [1.0, 2.0, 3.0, 4.0, 5.0]
        y = [5.0, 4.0, 3.0, 2.0, 1.0]
        assert calculate_correlation(x, y) == pytest.approx(-1.0)

    def test_correlation_empty(self):
        assert calculate_correlation([], []) == 0.0

    def test_correlation_mismatched_length(self):
        assert calculate_correlation([1, 2], [1]) == 0.0

    def test_beta(self):
        asset = [0.01, 0.02, -0.01, 0.03, -0.02]
        market = [0.005, 0.015, -0.005, 0.02, -0.01]
        b = calculate_beta(asset, market)
        assert isinstance(b, float)

    def test_annualize_return(self):
        assert annualize_return(0.01, 12) == pytest.approx(0.12)

    def test_annualize_volatility(self):
        daily_vol = 0.01
        annual = annualize_volatility(daily_vol, 252)
        assert annual == pytest.approx(daily_vol * math.sqrt(252))


# ---------------------------------------------------------------------------
# SimplePosition
# ---------------------------------------------------------------------------


class TestSimplePosition:
    def test_market_value_stock(self):
        p = SimplePosition(symbol="AAPL", quantity=100, current_price=150.0)
        assert p.get_market_value() == pytest.approx(15_000.0)

    def test_market_value_option(self):
        p = SimplePosition(
            symbol="AAPL", quantity=1, current_price=5.0, is_option=True
        )
        assert p.get_market_value() == pytest.approx(500.0)

    def test_is_long(self):
        assert SimplePosition(quantity=10).is_long()
        assert not SimplePosition(quantity=-10).is_long()

    def test_cost_basis_from_field(self):
        p = SimplePosition(cost_basis=1000.0)
        assert p.get_cost_basis() == pytest.approx(1000.0)

    def test_cost_basis_from_avg_price(self):
        p = SimplePosition(avg_price=50.0, quantity=10)
        assert p.get_cost_basis() == pytest.approx(500.0)


# ---------------------------------------------------------------------------
# SimpleBoxSpreadLeg
# ---------------------------------------------------------------------------


class TestSimpleBoxSpreadLeg:
    def test_strike_width(self):
        s = SimpleBoxSpreadLeg(strike_low=4500, strike_high=4600)
        assert s.get_strike_width() == pytest.approx(100.0)


# ---------------------------------------------------------------------------
# RiskCalculator - Box Spread Risk
# ---------------------------------------------------------------------------


class TestBoxSpreadRisk:
    def test_defined_risk(self):
        rc = RiskCalculator()
        spread = SimpleBoxSpreadLeg(net_debit=98.0, strike_low=4500, strike_high=4600)
        risk = rc.calculate_box_spread_risk(spread)
        assert risk.max_loss == pytest.approx(9800.0)
        assert risk.max_gain == pytest.approx(200.0)
        assert risk.delta == 0.0
        assert risk.probability_of_profit == 1.0

    def test_risk_reward_ratio(self):
        rc = RiskCalculator()
        spread = SimpleBoxSpreadLeg(net_debit=98.0, strike_low=4500, strike_high=4600)
        risk = rc.calculate_box_spread_risk(spread)
        assert risk.risk_reward_ratio == pytest.approx(200.0 / 9800.0)


# ---------------------------------------------------------------------------
# RiskCalculator - Position Risk
# ---------------------------------------------------------------------------


class TestPositionRisk:
    def test_basic_position_risk(self):
        rc = RiskCalculator()
        pos = SimplePosition(symbol="SPY", quantity=100, current_price=450.0)
        risk = rc.calculate_position_risk(pos)
        assert risk.position_size == pytest.approx(45_000.0)

    def test_max_loss_long_option(self):
        rc = RiskCalculator()
        pos = SimplePosition(
            quantity=1, avg_price=5.0, is_option=True, strike=100.0
        )
        ml = rc.calculate_max_loss(pos)
        assert ml == pytest.approx(pos.get_cost_basis())

    def test_max_loss_short_option(self):
        rc = RiskCalculator()
        pos = SimplePosition(quantity=-1, strike=100.0, is_option=True)
        ml = rc.calculate_max_loss(pos)
        assert ml == pytest.approx(100.0 * 100.0 * 1)


# ---------------------------------------------------------------------------
# RiskCalculator - Portfolio Risk
# ---------------------------------------------------------------------------


class TestPortfolioRisk:
    def test_total_exposure(self):
        rc = RiskCalculator()
        positions = [
            SimplePosition(symbol="A", quantity=100, current_price=10.0),
            SimplePosition(symbol="B", quantity=50, current_price=20.0),
        ]
        pr = rc.calculate_portfolio_risk(positions)
        assert pr.total_exposure == pytest.approx(2000.0)

    def test_var_estimates(self):
        rc = RiskCalculator()
        positions = [SimplePosition(symbol="A", quantity=100, current_price=100.0)]
        pr = rc.calculate_portfolio_risk(positions)
        assert pr.var_95 == pytest.approx(10_000.0 * 0.05)
        assert pr.var_99 == pytest.approx(10_000.0 * 0.10)


# ---------------------------------------------------------------------------
# RiskCalculator - Limits
# ---------------------------------------------------------------------------


class TestLimits:
    def test_within_limits(self):
        rc = RiskCalculator(RiskConfig(max_total_exposure=50_000))
        existing = [SimplePosition(symbol="A", quantity=100, current_price=100.0)]
        new_pos = SimplePosition(symbol="B", quantity=100, current_price=100.0)
        assert rc.is_within_limits(new_pos, existing)

    def test_exceeds_limits(self):
        rc = RiskCalculator(RiskConfig(max_total_exposure=10_000))
        existing = [SimplePosition(symbol="A", quantity=100, current_price=100.0)]
        new_pos = SimplePosition(symbol="B", quantity=100, current_price=100.0)
        assert not rc.is_within_limits(new_pos, existing)

    def test_box_spread_within_limits(self):
        rc = RiskCalculator(RiskConfig(max_total_exposure=50_000, max_positions=5))
        spread = SimpleBoxSpreadLeg(net_debit=98.0, strike_low=4500, strike_high=4600)
        existing = [SimplePosition(symbol="A", quantity=10, current_price=10.0)]
        assert rc.is_box_spread_within_limits(spread, existing)

    def test_remaining_capacity(self):
        rc = RiskCalculator(
            RiskConfig(max_total_exposure=50_000, position_size_percent=0.5)
        )
        positions = [SimplePosition(symbol="A", quantity=100, current_price=100.0)]
        cap = rc.calculate_remaining_capacity(positions, 50_000)
        assert cap == pytest.approx(25_000.0 - 10_000.0)

    def test_would_exceed_limits(self):
        rc = RiskCalculator(RiskConfig(max_total_exposure=5_000))
        new_pos = SimplePosition(symbol="A", quantity=100, current_price=100.0)
        assert rc.would_exceed_limits(new_pos, [])


# ---------------------------------------------------------------------------
# RiskCalculator - Position Sizing
# ---------------------------------------------------------------------------


class TestPositionSizing:
    def test_optimal_size(self):
        rc = RiskCalculator()
        spread = SimpleBoxSpreadLeg(net_debit=98.0, strike_low=4500, strike_high=4600)
        size = rc.calculate_optimal_position_size(spread, 100_000, 0.10)
        assert size >= 1

    def test_kelly_basic(self):
        rc = RiskCalculator()
        size = rc.calculate_kelly_position_size(0.55, 100.0, 100.0, 100_000)
        assert size >= 0

    def test_kelly_zero_loss(self):
        rc = RiskCalculator()
        assert rc.calculate_kelly_position_size(0.5, 100.0, 0.0, 100_000) == 0

    def test_kelly_high_probability(self):
        rc = RiskCalculator()
        size = rc.calculate_kelly_position_size(0.99, 100.0, 100.0, 100_000)
        assert size > 0

    def test_fixed_fractional(self):
        rc = RiskCalculator()
        size = rc.calculate_fixed_fractional_size(1000.0, 100_000, 0.02)
        assert size == 2


# ---------------------------------------------------------------------------
# RiskCalculator - VaR
# ---------------------------------------------------------------------------


class TestVaR:
    def test_historical_var_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_var_historical([]) == 0.0

    def test_historical_var(self):
        rc = RiskCalculator()
        returns = [-0.05, -0.03, -0.01, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07]
        var = rc.calculate_var_historical(returns, 0.95)
        assert var > 0

    def test_parametric_var_95(self):
        rc = RiskCalculator()
        var = rc.calculate_var_parametric(0.0, 0.20, 100_000, 0.95, 1)
        expected = 100_000 * 1.645 * 0.20 * math.sqrt(1.0 / 252.0)
        assert var == pytest.approx(expected)

    def test_parametric_var_99(self):
        rc = RiskCalculator()
        var = rc.calculate_var_parametric(0.0, 0.20, 100_000, 0.99, 1)
        expected = 100_000 * 2.326 * 0.20 * math.sqrt(1.0 / 252.0)
        assert var == pytest.approx(expected)

    def test_expected_shortfall_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_expected_shortfall([]) == 0.0

    def test_expected_shortfall(self):
        rc = RiskCalculator()
        returns = [-0.10, -0.05, -0.03, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07]
        es = rc.calculate_expected_shortfall(returns, 0.95)
        assert es > 0


# ---------------------------------------------------------------------------
# RiskCalculator - Risk-Adjusted Returns
# ---------------------------------------------------------------------------


class TestRiskAdjustedReturns:
    def test_sharpe_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_sharpe_ratio([]) == 0.0

    def test_sharpe_positive(self):
        rc = RiskCalculator()
        returns = [0.01, 0.02, 0.015, 0.008, 0.012]
        sharpe = rc.calculate_sharpe_ratio(returns, 0.001)
        assert sharpe > 0

    def test_sharpe_zero_std(self):
        rc = RiskCalculator()
        assert rc.calculate_sharpe_ratio([0.01, 0.01, 0.01]) == 0.0

    def test_sortino_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_sortino_ratio([]) == 0.0

    def test_sortino_no_downside(self):
        rc = RiskCalculator()
        ratio = rc.calculate_sortino_ratio([0.01, 0.02, 0.03])
        assert ratio == float("inf")

    def test_sortino_with_downside(self):
        rc = RiskCalculator()
        returns = [0.01, -0.02, 0.03, -0.01, 0.02]
        ratio = rc.calculate_sortino_ratio(returns)
        assert isinstance(ratio, float)

    def test_calmar_zero_drawdown(self):
        rc = RiskCalculator()
        assert rc.calculate_calmar_ratio(0.10, 0.0) == 0.0

    def test_calmar_positive(self):
        rc = RiskCalculator()
        assert rc.calculate_calmar_ratio(0.15, 0.05) == pytest.approx(3.0)

    def test_information_ratio_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_information_ratio([], []) == 0.0

    def test_information_ratio_mismatched(self):
        rc = RiskCalculator()
        assert rc.calculate_information_ratio([0.01], [0.01, 0.02]) == 0.0

    def test_information_ratio(self):
        rc = RiskCalculator()
        returns = [0.02, 0.03, 0.01, 0.04, 0.02]
        benchmark = [0.01, 0.02, 0.01, 0.02, 0.01]
        ir = rc.calculate_information_ratio(returns, benchmark)
        assert ir > 0


# ---------------------------------------------------------------------------
# RiskCalculator - Drawdown
# ---------------------------------------------------------------------------


class TestDrawdown:
    def test_max_drawdown_empty(self):
        rc = RiskCalculator()
        assert rc.calculate_max_drawdown([]) == 0.0

    def test_max_drawdown_no_drawdown(self):
        rc = RiskCalculator()
        assert rc.calculate_max_drawdown([1, 2, 3, 4, 5]) == 0.0

    def test_max_drawdown(self):
        rc = RiskCalculator()
        curve = [100, 110, 90, 95, 80, 120]
        dd = rc.calculate_max_drawdown(curve)
        expected = (110 - 80) / 110
        assert dd == pytest.approx(expected)

    def test_current_drawdown(self):
        rc = RiskCalculator()
        curve = [100, 110, 105]
        dd = rc.calculate_current_drawdown(curve)
        assert dd == pytest.approx((110 - 105) / 110)


# ---------------------------------------------------------------------------
# RiskCalculator - Correlation Risk
# ---------------------------------------------------------------------------


class TestCorrelationRisk:
    def test_single_position(self):
        rc = RiskCalculator()
        assert rc.calculate_correlation_risk([SimplePosition()]) == 0.0

    def test_same_symbol(self):
        rc = RiskCalculator()
        positions = [
            SimplePosition(symbol="SPX", quantity=10, current_price=100),
            SimplePosition(symbol="SPX", quantity=5, current_price=100),
        ]
        risk = rc.calculate_correlation_risk(positions)
        assert risk == pytest.approx(1.0)

    def test_different_symbols(self):
        rc = RiskCalculator()
        positions = [
            SimplePosition(symbol="SPX", quantity=10, current_price=100, avg_price=95),
            SimplePosition(symbol="QQQ", quantity=10, current_price=100, avg_price=90),
        ]
        risk = rc.calculate_correlation_risk(positions)
        assert 0.0 < risk < 1.0


# ---------------------------------------------------------------------------
# RiskCalculator - Greeks
# ---------------------------------------------------------------------------


class TestGreeks:
    def test_box_spread_greeks_neutral(self):
        rc = RiskCalculator()
        spread = SimpleBoxSpreadLeg(net_debit=98.0, strike_low=4500, strike_high=4600)
        m = rc.calculate_box_spread_greeks(spread)
        assert m.delta == 0.0
        assert m.gamma == 0.0
        assert m.theta == 0.0
        assert m.vega == 0.0
        assert m.probability_of_profit == 1.0


# ---------------------------------------------------------------------------
# RiskCalculator - Scenario Analysis
# ---------------------------------------------------------------------------


class TestScenarioAnalysis:
    def test_scenarios(self):
        rc = RiskCalculator()
        pos = SimplePosition(symbol="SPY", quantity=100, current_price=450)
        results = rc.run_scenario_analysis(pos, 450.0, [400.0, 450.0, 500.0])
        assert len(results) == 3
        assert results[0].price_change_percent < 0
        assert results[1].price_change_percent == pytest.approx(0.0)
        assert results[2].price_change_percent > 0


# ---------------------------------------------------------------------------
# RiskCalculator - Config
# ---------------------------------------------------------------------------


class TestConfig:
    def test_update_config(self):
        rc = RiskCalculator()
        new_cfg = RiskConfig(max_total_exposure=200_000)
        rc.update_config(new_cfg)
        assert rc.get_config().max_total_exposure == pytest.approx(200_000)


# ---------------------------------------------------------------------------
# RiskMonitor
# ---------------------------------------------------------------------------


class TestRiskMonitor:
    def test_no_alert_below_threshold(self):
        mon = RiskMonitor(RiskConfig(max_total_exposure=100_000))
        positions = [SimplePosition(symbol="A", quantity=10, current_price=10)]
        alerts = mon.check_risks(positions)
        assert len(alerts) == 0

    def test_alert_near_limit(self):
        mon = RiskMonitor(RiskConfig(max_total_exposure=1_000))
        positions = [SimplePosition(symbol="A", quantity=100, current_price=10)]
        alerts = mon.check_risks(positions)
        assert len(alerts) == 1
        assert alerts[0].level == RiskAlertLevel.Warning
        assert alerts[0].category == "EXPOSURE"
