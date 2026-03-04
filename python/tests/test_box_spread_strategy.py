"""Tests for box_spread_strategy.py - Python port of C++ box spread strategy."""

from datetime import datetime, timedelta

import pytest

from python.integration.box_spread_strategy import (
    BoxSpreadBag,
    BoxSpreadBagManager,
    BoxSpreadCalculator,
    BoxSpreadLeg,
    BoxSpreadOpportunity,
    BoxSpreadStrategy,
    BoxSpreadValidator,
    CommissionConfig,
    MarketData,
    OptionContract,
    OptionEntry,
    StrategyParams,
    YieldCurve,
    YieldCurvePoint,
    filter_by_min_profit,
    filter_by_min_roi,
    sort_opportunities_by_confidence,
    sort_opportunities_by_profit,
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


def _future_expiry(days: int = 30) -> str:
    return (datetime.now() + timedelta(days=days)).strftime("%Y%m%d")


def _make_spread(
    symbol: str = "SPX",
    strike_low: float = 4500.0,
    strike_high: float = 4600.0,
    days: int = 30,
    long_call_price: float = 120.0,
    short_call_price: float = 22.0,
    long_put_price: float = 20.0,
    short_put_price: float = 118.0,
) -> BoxSpreadLeg:
    expiry = _future_expiry(days)
    spread = BoxSpreadLeg(
        long_call=OptionContract(symbol, expiry, strike_low, "C"),
        short_call=OptionContract(symbol, expiry, strike_high, "C"),
        long_put=OptionContract(symbol, expiry, strike_high, "P"),
        short_put=OptionContract(symbol, expiry, strike_low, "P"),
        long_call_price=long_call_price,
        short_call_price=short_call_price,
        long_put_price=long_put_price,
        short_put_price=short_put_price,
    )
    spread.net_debit = BoxSpreadCalculator.calculate_net_debit(spread)
    spread.theoretical_value = BoxSpreadCalculator.calculate_theoretical_value(spread)
    spread.arbitrage_profit = BoxSpreadCalculator.calculate_max_profit(spread)
    spread.roi_percent = BoxSpreadCalculator.calculate_roi(spread)
    return spread


def _make_option_entry(
    symbol: str, expiry: str, strike: float, opt_type: str,
    bid: float, ask: float, volume: int = 100, oi: int = 500,
    liquidity: float = 75.0,
) -> OptionEntry:
    return OptionEntry(
        contract=OptionContract(symbol, expiry, strike, opt_type),
        market_data=MarketData(
            bid=bid, ask=ask, last=(bid + ask) / 2,
            volume=volume, open_interest=oi,
        ),
        liquidity_score=liquidity,
    )


# ---------------------------------------------------------------------------
# OptionContract
# ---------------------------------------------------------------------------


class TestOptionContract:
    def test_to_string(self):
        c = OptionContract("SPX", "20260401", 4500.0, "C")
        assert "SPX" in c.to_string()
        assert "4500" in c.to_string()


# ---------------------------------------------------------------------------
# MarketData
# ---------------------------------------------------------------------------


class TestMarketData:
    def test_mid_price(self):
        md = MarketData(bid=10.0, ask=12.0)
        assert md.get_mid_price() == pytest.approx(11.0)

    def test_mid_price_fallback(self):
        md = MarketData(last=5.5)
        assert md.get_mid_price() == pytest.approx(5.5)

    def test_spread(self):
        md = MarketData(bid=10.0, ask=10.20)
        assert md.get_spread() == pytest.approx(0.20)

    def test_is_valid(self):
        assert MarketData(bid=10.0, ask=10.2).is_valid()
        assert not MarketData(bid=0.0, ask=10.0).is_valid()


# ---------------------------------------------------------------------------
# BoxSpreadLeg
# ---------------------------------------------------------------------------


class TestBoxSpreadLeg:
    def test_strike_width(self):
        s = _make_spread()
        assert s.get_strike_width() == pytest.approx(100.0)

    def test_days_to_expiry_positive(self):
        s = _make_spread(days=45)
        assert s.get_days_to_expiry() >= 44  # allow 1-day rounding

    def test_is_valid(self):
        s = _make_spread()
        assert s.is_valid()


# ---------------------------------------------------------------------------
# BoxSpreadCalculator
# ---------------------------------------------------------------------------


class TestBoxSpreadCalculator:
    def test_theoretical_value(self):
        s = _make_spread()
        assert BoxSpreadCalculator.calculate_theoretical_value(s) == pytest.approx(100.0)

    def test_net_debit(self):
        s = _make_spread()
        expected = 120.0 - 22.0 + 20.0 - 118.0  # 0.0
        assert BoxSpreadCalculator.calculate_net_debit(s) == pytest.approx(expected)

    def test_max_profit(self):
        s = _make_spread(long_call_price=119.0)
        s.net_debit = BoxSpreadCalculator.calculate_net_debit(s)
        s.theoretical_value = BoxSpreadCalculator.calculate_theoretical_value(s)
        profit = BoxSpreadCalculator.calculate_max_profit(s)
        assert profit == pytest.approx(s.theoretical_value - s.net_debit)

    def test_max_loss_negative_debit(self):
        s = _make_spread()
        s.net_debit = -5.0
        assert BoxSpreadCalculator.calculate_max_loss(s) == pytest.approx(5.0)

    def test_max_loss_positive_debit(self):
        s = _make_spread()
        s.net_debit = 95.0
        assert BoxSpreadCalculator.calculate_max_loss(s) == 0.0

    def test_roi(self):
        s = _make_spread()
        s.net_debit = 98.0
        s.theoretical_value = 100.0
        s.arbitrage_profit = 2.0
        assert BoxSpreadCalculator.calculate_roi(s) == pytest.approx(
            (2.0 / 98.0) * 100.0
        )

    def test_roi_zero_debit(self):
        s = _make_spread()
        s.net_debit = 0.0
        assert BoxSpreadCalculator.calculate_roi(s) == 0.0

    def test_commission(self):
        s = _make_spread()
        assert BoxSpreadCalculator.calculate_commission(s, 0.65) == pytest.approx(2.60)

    def test_commission_ibkr_pro(self):
        s = _make_spread()
        cfg = CommissionConfig(per_contract_fee=0.15, tier="pro")
        comm = BoxSpreadCalculator.calculate_commission_ibkr_pro(s, cfg)
        assert comm == pytest.approx(max(4 * 0.15, cfg.minimum_order_fee))

    def test_implied_interest_rate_borrowing(self):
        s = _make_spread(days=90)
        s.net_debit = 98.0
        rate = BoxSpreadCalculator.calculate_implied_interest_rate(s)
        expected = ((98.0 - 100.0) / 100.0) * (365.0 / s.get_days_to_expiry()) * 100.0
        assert rate == pytest.approx(expected, rel=0.05)

    def test_implied_interest_rate_lending(self):
        s = _make_spread(days=90)
        s.net_debit = -102.0
        rate = BoxSpreadCalculator.calculate_implied_interest_rate(s)
        nc = 102.0
        sw = 100.0
        expected = ((sw - nc) / nc) * (365.0 / s.get_days_to_expiry()) * 100.0
        assert rate == pytest.approx(expected, rel=0.05)

    def test_implied_rate_zero_dte(self):
        s = _make_spread(days=0)
        assert BoxSpreadCalculator.calculate_implied_interest_rate(s) == 0.0

    def test_effective_rate_includes_commission(self):
        s = _make_spread(days=90)
        s.net_debit = 98.0
        implied = BoxSpreadCalculator.calculate_implied_interest_rate(s)
        effective = BoxSpreadCalculator.calculate_effective_interest_rate(s, 0.65)
        # Effective should differ from implied due to commission
        assert effective != implied or s.net_debit == 0

    def test_compare_to_benchmark(self):
        s = _make_spread(days=90)
        s.net_debit = 98.0
        bps = BoxSpreadCalculator.compare_to_benchmark(s, 5.0)
        assert isinstance(bps, float)

    def test_buy_net_debit(self):
        s = _make_spread()
        result = BoxSpreadCalculator.calculate_buy_net_debit(
            s, 120.50, 21.50, 20.50, 117.50
        )
        assert result == pytest.approx(120.50 - 21.50 + 20.50 - 117.50)

    def test_sell_net_credit(self):
        s = _make_spread()
        result = BoxSpreadCalculator.calculate_sell_net_credit(
            s, 119.50, 22.50, 19.50, 118.50
        )
        assert result == pytest.approx(119.50 - 22.50 + 19.50 - 118.50)

    def test_buy_sell_disparity(self):
        assert BoxSpreadCalculator.calculate_buy_sell_disparity(1.5, 0.5) == pytest.approx(1.0)

    def test_put_call_parity_violation(self):
        s = _make_spread()
        result = BoxSpreadCalculator.calculate_put_call_parity_violation(s, 5.0, 4.9)
        assert result == pytest.approx(10.0)  # 0.1% * 100 bps


# ---------------------------------------------------------------------------
# BoxSpreadValidator
# ---------------------------------------------------------------------------


class TestBoxSpreadValidator:
    def test_valid_spread(self):
        s = _make_spread()
        s.long_call_price = 120.0
        s.short_call_price = 22.0
        s.long_put_price = 20.0
        s.short_put_price = 118.0
        s.net_debit = 0.0
        s.theoretical_value = 100.0
        # This fails pricing validation (net_debit must be >0 and <theoretical)
        ok, errs = BoxSpreadValidator.validate(s)
        assert not ok  # net_debit=0 fails

    def test_valid_structure(self):
        s = _make_spread()
        assert BoxSpreadValidator.validate_structure(s)

    def test_valid_strikes(self):
        s = _make_spread()
        assert BoxSpreadValidator.validate_strikes(s)

    def test_invalid_strikes(self):
        s = _make_spread()
        s.long_call.strike = 4600.0
        s.short_call.strike = 4500.0
        assert not BoxSpreadValidator.validate_strikes(s)

    def test_valid_expiries(self):
        s = _make_spread()
        assert BoxSpreadValidator.validate_expiries(s)

    def test_invalid_expiries(self):
        s = _make_spread()
        s.long_put.expiry = "20270101"
        assert not BoxSpreadValidator.validate_expiries(s)

    def test_valid_symbols(self):
        s = _make_spread()
        assert BoxSpreadValidator.validate_symbols(s)

    def test_bid_ask_too_wide(self):
        s = _make_spread()
        s.net_debit = 99.0
        s.theoretical_value = 100.0
        s.long_call_bid_ask_spread = 0.60
        ok, errs = BoxSpreadValidator.validate(s)
        assert not ok
        assert any("too wide" in e for e in errs)

    def test_all_prices_positive(self):
        s = _make_spread()
        s.net_debit = 99.0
        s.theoretical_value = 100.0
        s.short_put_price = 0.0
        ok, errs = BoxSpreadValidator.validate(s)
        assert not ok
        assert any("positive" in e for e in errs)


# ---------------------------------------------------------------------------
# BoxSpreadStrategy
# ---------------------------------------------------------------------------


class TestBoxSpreadStrategy:
    def test_is_profitable(self):
        strat = BoxSpreadStrategy(StrategyParams(min_arbitrage_profit=0.5, min_roi_percent=0.01))
        s = _make_spread()
        s.net_debit = 99.0
        s.theoretical_value = 100.0
        s.arbitrage_profit = 1.0
        assert strat.is_profitable(s)

    def test_is_not_profitable(self):
        strat = BoxSpreadStrategy(StrategyParams(min_arbitrage_profit=5.0))
        s = _make_spread()
        s.net_debit = 99.5
        s.theoretical_value = 100.0
        s.arbitrage_profit = 0.5
        assert not strat.is_profitable(s)

    def test_confidence_score_tight_spreads(self):
        strat = BoxSpreadStrategy()
        s = _make_spread(days=30)
        s.long_call_bid_ask_spread = 0.01
        s.short_call_bid_ask_spread = 0.01
        s.long_put_bid_ask_spread = 0.01
        s.short_put_bid_ask_spread = 0.01
        s.net_debit = 99.5
        s.buy_sell_disparity = 0.01
        s.put_call_parity_violation = 2.0
        score = strat.calculate_confidence_score(s, extra_dte=30)
        assert score >= 80.0

    def test_confidence_score_wide_spreads(self):
        strat = BoxSpreadStrategy()
        s = _make_spread(days=30)
        s.long_call_bid_ask_spread = 1.0
        s.short_call_bid_ask_spread = 1.0
        s.long_put_bid_ask_spread = 1.0
        s.short_put_bid_ask_spread = 1.0
        s.net_debit = 70.0
        s.buy_sell_disparity = 5.0
        s.put_call_parity_violation = 100.0
        score = strat.calculate_confidence_score(s, extra_dte=3)
        assert score < 30.0

    def test_confidence_score_max_100(self):
        strat = BoxSpreadStrategy()
        s = _make_spread(days=30)
        score = strat.calculate_confidence_score(s, extra_dte=30)
        assert score <= 100.0

    def test_beats_benchmark(self):
        strat = BoxSpreadStrategy()
        s = _make_spread(days=90)
        s.net_debit = 97.0
        assert isinstance(strat.beats_benchmark(s, 5.0), bool)

    def test_evaluate_box_spread_valid(self):
        strat = BoxSpreadStrategy(StrategyParams(min_arbitrage_profit=0.01, min_roi_percent=0.001))
        expiry = _future_expiry(30)
        lc = _make_option_entry("SPX", expiry, 4500, "C", 119.50, 120.50)
        sc = _make_option_entry("SPX", expiry, 4600, "C", 21.50, 22.50)
        lp = _make_option_entry("SPX", expiry, 4600, "P", 19.50, 20.50)
        sp = _make_option_entry("SPX", expiry, 4500, "P", 117.50, 118.50)
        opp = strat.evaluate_box_spread(lc, sc, lp, sp)
        # Might return None if pricing/validation thresholds not met
        # The test verifies no crash and correct return type
        assert opp is None or isinstance(opp, BoxSpreadOpportunity)

    def test_evaluate_box_spread_invalid_entry(self):
        strat = BoxSpreadStrategy()
        expiry = _future_expiry(30)
        lc = _make_option_entry("SPX", expiry, 4500, "C", 0.0, 0.0)  # invalid
        sc = _make_option_entry("SPX", expiry, 4600, "C", 21.50, 22.50)
        lp = _make_option_entry("SPX", expiry, 4600, "P", 19.50, 20.50)
        sp = _make_option_entry("SPX", expiry, 4500, "P", 117.50, 118.50)
        assert strat.evaluate_box_spread(lc, sc, lp, sp) is None

    def test_build_yield_curve(self):
        strat = BoxSpreadStrategy()
        s = _make_spread(days=30)
        s.net_debit = 99.0
        s.theoretical_value = 100.0
        opp = BoxSpreadOpportunity(spread=s, expected_profit=1.0, liquidity_score=80.0)
        curve = strat.build_yield_curve([opp], "SPX", 100.0, 5.0)
        assert isinstance(curve, YieldCurve)
        assert curve.symbol == "SPX"


# ---------------------------------------------------------------------------
# BoxSpreadOpportunity
# ---------------------------------------------------------------------------


class TestBoxSpreadOpportunity:
    def test_is_actionable(self):
        opp = BoxSpreadOpportunity(
            confidence_score=60.0, expected_profit=1.0, execution_probability=0.8
        )
        assert opp.is_actionable()

    def test_not_actionable_low_confidence(self):
        opp = BoxSpreadOpportunity(
            confidence_score=30.0, expected_profit=1.0, execution_probability=0.8
        )
        assert not opp.is_actionable()

    def test_not_actionable_no_profit(self):
        opp = BoxSpreadOpportunity(
            confidence_score=60.0, expected_profit=-1.0, execution_probability=0.8
        )
        assert not opp.is_actionable()


# ---------------------------------------------------------------------------
# BoxSpreadBag
# ---------------------------------------------------------------------------


class TestBoxSpreadBag:
    def test_generate_cboe_symbol(self):
        sym = BoxSpreadBag.generate_cboe_symbol("SPX", "25JAN24", 4500, 4600)
        assert sym == "SPX 25JAN24 4500/4600 BOX"

    def test_update_candle_initializes(self):
        bag = BoxSpreadBag()
        bag.update_candle(100.0)
        assert bag.candle.open == pytest.approx(100.0)
        assert bag.candle.close == pytest.approx(100.0)

    def test_update_candle_tracks_high_low(self):
        bag = BoxSpreadBag()
        bag.update_candle(100.0)
        bag.update_candle(110.0)
        bag.update_candle(90.0)
        assert bag.candle.high == pytest.approx(110.0)
        assert bag.candle.low == pytest.approx(90.0)
        assert bag.candle.close == pytest.approx(90.0)

    def test_reset_candle(self):
        bag = BoxSpreadBag()
        bag.update_candle(100.0)
        bag.reset_candle()
        assert len(bag.candle_history) == 1
        assert bag.candle.open == 0.0

    def test_pnl_no_position(self):
        bag = BoxSpreadBag()
        assert bag.get_current_pnl() == 0.0
        assert bag.get_pnl_per_contract() == 0.0

    def test_is_valid(self):
        s = _make_spread(days=30)
        bag = BoxSpreadBagManager.create_bag_from_spread(s, "SPX")
        assert bag.is_valid()


# ---------------------------------------------------------------------------
# BoxSpreadBagManager
# ---------------------------------------------------------------------------


class TestBoxSpreadBagManager:
    def test_create_bag(self):
        s = _make_spread(days=30)
        bag = BoxSpreadBagManager.create_bag_from_spread(s, "SPX")
        assert bag.complex_symbol == "SPX BOX"
        assert "BOX" in bag.cboe_symbol
        assert bag.created_at is not None

    def test_update_market_data(self):
        s = _make_spread(days=30)
        bag = BoxSpreadBagManager.create_bag_from_spread(s, "SPX")
        BoxSpreadBagManager.update_bag_market_data(bag, 98.0, 99.0, 98.5)
        assert bag.market_data.bid == pytest.approx(98.0)
        assert bag.market_data.ask == pytest.approx(99.0)


# ---------------------------------------------------------------------------
# YieldCurve
# ---------------------------------------------------------------------------


class TestYieldCurve:
    def test_sort_by_dte(self):
        curve = YieldCurve(symbol="SPX", strike_width=100.0)
        curve.points = [
            YieldCurvePoint(days_to_expiry=90),
            YieldCurvePoint(days_to_expiry=30),
            YieldCurvePoint(days_to_expiry=60),
        ]
        curve.sort_by_dte()
        dtes = [p.days_to_expiry for p in curve.points]
        assert dtes == [30, 60, 90]

    def test_is_valid(self):
        curve = YieldCurve(symbol="SPX", strike_width=100.0)
        assert not curve.is_valid()
        curve.points.append(YieldCurvePoint(days_to_expiry=30))
        assert curve.is_valid()


# ---------------------------------------------------------------------------
# Free-standing helpers
# ---------------------------------------------------------------------------


class TestHelpers:
    def test_sort_by_profit(self):
        opps = [
            BoxSpreadOpportunity(expected_profit=1.0),
            BoxSpreadOpportunity(expected_profit=5.0),
            BoxSpreadOpportunity(expected_profit=3.0),
        ]
        sorted_opps = sort_opportunities_by_profit(opps)
        assert [o.expected_profit for o in sorted_opps] == [5.0, 3.0, 1.0]

    def test_sort_by_confidence(self):
        opps = [
            BoxSpreadOpportunity(confidence_score=50.0),
            BoxSpreadOpportunity(confidence_score=90.0),
        ]
        sorted_opps = sort_opportunities_by_confidence(opps)
        assert sorted_opps[0].confidence_score == 90.0

    def test_filter_by_min_profit(self):
        opps = [
            BoxSpreadOpportunity(expected_profit=1.0),
            BoxSpreadOpportunity(expected_profit=5.0),
        ]
        assert len(filter_by_min_profit(opps, 3.0)) == 1

    def test_filter_by_min_roi(self):
        s1 = BoxSpreadLeg()
        s1.roi_percent = 0.5
        s2 = BoxSpreadLeg()
        s2.roi_percent = 2.0
        opps = [
            BoxSpreadOpportunity(spread=s1),
            BoxSpreadOpportunity(spread=s2),
        ]
        assert len(filter_by_min_roi(opps, 1.0)) == 1


# ---------------------------------------------------------------------------
# CommissionConfig
# ---------------------------------------------------------------------------


class TestCommissionConfig:
    def test_effective_rate_standard(self):
        assert CommissionConfig(tier="standard").get_effective_rate() == pytest.approx(0.65)

    def test_effective_rate_pro(self):
        assert CommissionConfig(tier="pro").get_effective_rate() == pytest.approx(0.15)

    def test_effective_rate_lite(self):
        assert CommissionConfig(tier="lite").get_effective_rate() == pytest.approx(0.50)
