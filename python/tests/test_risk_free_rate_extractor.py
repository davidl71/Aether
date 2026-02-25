"""
test_risk_free_rate_extractor.py - Tests for risk-free rate extraction from box spreads.
"""

import pytest
from datetime import datetime

from python.integration.risk_free_rate_extractor import (
    RiskFreeRateExtractor,
    RiskFreeRatePoint,
    RiskFreeRateCurve,
)


@pytest.fixture
def extractor():
    return RiskFreeRateExtractor(min_liquidity_score=50.0)


@pytest.fixture
def sample_point():
    return RiskFreeRatePoint(
        symbol="SPX",
        expiry="20260320",
        days_to_expiry=30,
        strike_width=50.0,
        buy_implied_rate=4.50,
        sell_implied_rate=4.30,
        mid_rate=4.40,
        net_debit=49.80,
        net_credit=49.70,
        liquidity_score=80.0,
        timestamp=datetime.now(),
        spread_id="SPX-4500-4550-20260320",
    )


class TestRiskFreeRatePoint:
    def test_valid_point(self, sample_point):
        assert sample_point.is_valid()

    def test_invalid_zero_dte(self):
        point = RiskFreeRatePoint(
            symbol="SPX", expiry="20260320", days_to_expiry=0,
            strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
            mid_rate=4.40, net_debit=49.80, net_credit=49.70,
            liquidity_score=80.0, timestamp=datetime.now(),
        )
        assert not point.is_valid()

    def test_invalid_zero_strike_width(self):
        point = RiskFreeRatePoint(
            symbol="SPX", expiry="20260320", days_to_expiry=30,
            strike_width=0.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
            mid_rate=4.40, net_debit=49.80, net_credit=49.70,
            liquidity_score=80.0, timestamp=datetime.now(),
        )
        assert not point.is_valid()

    def test_invalid_zero_rates(self):
        point = RiskFreeRatePoint(
            symbol="SPX", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=0.0, sell_implied_rate=0.0,
            mid_rate=0.0, net_debit=49.80, net_credit=49.70,
            liquidity_score=80.0, timestamp=datetime.now(),
        )
        assert not point.is_valid()

    def test_valid_with_only_buy_rate(self):
        point = RiskFreeRatePoint(
            symbol="SPX", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=0.0,
            mid_rate=4.50, net_debit=49.80, net_credit=0.0,
            liquidity_score=80.0, timestamp=datetime.now(),
        )
        assert point.is_valid()


class TestRiskFreeRateCurve:
    def test_get_rates_by_dte(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260620", days_to_expiry=120,
                strike_width=50.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=49.70, net_credit=49.60,
                liquidity_score=75.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())

        rates = curve.get_rates_by_dte()
        assert 30 in rates
        assert 120 in rates
        assert abs(rates[30] - 4.40) < 0.01
        assert abs(rates[120] - 4.50) < 0.01

    def test_get_rate_at_dte_exact(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=90,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())

        rate = curve.get_rate_at_dte(90)
        assert rate is not None
        assert abs(rate - 4.40) < 0.01

    def test_get_rate_at_dte_within_tolerance(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=88,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())

        rate = curve.get_rate_at_dte(90, tolerance=5)
        assert rate is not None

    def test_get_rate_at_dte_not_found(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())

        rate = curve.get_rate_at_dte(365, tolerance=5)
        assert rate is None

    def test_sort_by_dte(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260920", days_to_expiry=180,
                strike_width=50.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=49.70, net_credit=49.60,
                liquidity_score=75.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())
        curve.sort_by_dte()

        assert curve.points[0].days_to_expiry == 30
        assert curve.points[1].days_to_expiry == 180

    def test_filter_by_liquidity(self):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260620", days_to_expiry=120,
                strike_width=50.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=49.70, net_credit=49.60,
                liquidity_score=30.0, timestamp=datetime.now(),
            ),
        ]
        curve = RiskFreeRateCurve(symbol="SPX", points=points, timestamp=datetime.now())

        filtered = curve.filter_by_liquidity(min_liquidity=50.0)
        assert len(filtered.points) == 1
        assert filtered.points[0].days_to_expiry == 30


class TestRiskFreeRateExtractor:
    def test_extract_from_box_spread(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="SPX",
            expiry="20260320",
            days_to_expiry=30,
            strike_width=50.0,
            buy_implied_rate=4.50,
            sell_implied_rate=4.30,
            net_debit=49.80,
            net_credit=49.70,
            liquidity_score=80.0,
            spread_id="test-spread",
        )

        assert point is not None
        assert point.symbol == "SPX"
        assert abs(point.mid_rate - 4.40) < 0.01
        assert point.spread_id == "test-spread"

    def test_extract_mid_rate_both_rates(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="XSP", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=4.60, sell_implied_rate=4.20,
            net_debit=49.80, net_credit=49.70, liquidity_score=80.0,
        )
        assert point is not None
        assert abs(point.mid_rate - 4.40) < 0.01

    def test_extract_mid_rate_only_buy(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="XSP", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=0.0,
            net_debit=49.80, net_credit=0.0, liquidity_score=80.0,
        )
        assert point is not None
        assert abs(point.mid_rate - 4.50) < 0.01

    def test_extract_mid_rate_only_sell(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="XSP", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=0.0, sell_implied_rate=4.30,
            net_debit=0.0, net_credit=49.70, liquidity_score=80.0,
        )
        assert point is not None
        assert abs(point.mid_rate - 4.30) < 0.01

    def test_extract_returns_none_no_rates(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="XSP", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=0.0, sell_implied_rate=0.0,
            net_debit=0.0, net_credit=0.0, liquidity_score=80.0,
        )
        assert point is None

    def test_extract_returns_none_low_liquidity(self, extractor):
        point = extractor.extract_from_box_spread(
            symbol="XSP", expiry="20260320", days_to_expiry=30,
            strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
            net_debit=49.80, net_credit=49.70, liquidity_score=20.0,
        )
        assert point is None

    def test_extract_from_dict(self, extractor):
        data = {
            "symbol": "SPX",
            "expiry": "20260320",
            "days_to_expiry": 30,
            "strike_width": 50.0,
            "buy_implied_rate": 4.50,
            "sell_implied_rate": 4.30,
            "buy_net_debit": 49.80,
            "sell_net_credit": 49.70,
            "liquidity_score": 80.0,
            "spread_id": "dict-test",
        }
        point = extractor.extract_from_box_spread_dict(data)
        assert point is not None
        assert point.symbol == "SPX"
        assert point.spread_id == "dict-test"

    def test_extract_from_dict_empty(self, extractor):
        point = extractor.extract_from_box_spread_dict({})
        assert point is None

    def test_aggregate_rates_single_point(self, extractor):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
        ]
        curve = extractor.aggregate_rates(points, "SPX")
        assert len(curve.points) == 1
        assert curve.symbol == "SPX"

    def test_aggregate_rates_weighted_average(self, extractor):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=100.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=99.60, net_credit=99.50,
                liquidity_score=20.0, timestamp=datetime.now(),
            ),
        ]
        curve = extractor.aggregate_rates(points, "SPX", aggregation_method="weighted_average")

        assert len(curve.points) == 1
        # Weighted average: (4.40*80 + 4.50*20) / 100 = 4.42
        assert abs(curve.points[0].mid_rate - 4.42) < 0.01

    def test_aggregate_rates_best_liquidity(self, extractor):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=100.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=99.60, net_credit=99.50,
                liquidity_score=20.0, timestamp=datetime.now(),
            ),
        ]
        curve = extractor.aggregate_rates(points, "SPX", aggregation_method="best_liquidity")

        assert len(curve.points) == 1
        assert abs(curve.points[0].mid_rate - 4.40) < 0.01

    def test_aggregate_rates_simple_average(self, extractor):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=100.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=99.60, net_credit=99.50,
                liquidity_score=20.0, timestamp=datetime.now(),
            ),
        ]
        curve = extractor.aggregate_rates(points, "SPX", aggregation_method="average")

        assert len(curve.points) == 1
        assert abs(curve.points[0].mid_rate - 4.45) < 0.01

    def test_aggregate_rates_multiple_dtes(self, extractor):
        points = [
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260320", days_to_expiry=30,
                strike_width=50.0, buy_implied_rate=4.50, sell_implied_rate=4.30,
                mid_rate=4.40, net_debit=49.80, net_credit=49.70,
                liquidity_score=80.0, timestamp=datetime.now(),
            ),
            RiskFreeRatePoint(
                symbol="SPX", expiry="20260620", days_to_expiry=120,
                strike_width=50.0, buy_implied_rate=4.60, sell_implied_rate=4.40,
                mid_rate=4.50, net_debit=49.70, net_credit=49.60,
                liquidity_score=75.0, timestamp=datetime.now(),
            ),
        ]
        curve = extractor.aggregate_rates(points, "SPX")

        assert len(curve.points) == 2
        assert curve.points[0].days_to_expiry == 30
        assert curve.points[1].days_to_expiry == 120

    def test_build_curve_from_opportunities(self, extractor):
        opportunities = [
            {
                "spread": {
                    "symbol": "SPX",
                    "expiry": "20260320",
                    "days_to_expiry": 30,
                    "strike_width": 50.0,
                    "buy_implied_rate": 4.50,
                    "sell_implied_rate": 4.30,
                    "buy_net_debit": 49.80,
                    "sell_net_credit": 49.70,
                    "liquidity_score": 80.0,
                }
            },
            {
                "spread": {
                    "symbol": "SPX",
                    "expiry": "20260620",
                    "days_to_expiry": 120,
                    "strike_width": 50.0,
                    "buy_implied_rate": 4.60,
                    "sell_implied_rate": 4.40,
                    "buy_net_debit": 49.70,
                    "sell_net_credit": 49.60,
                    "liquidity_score": 75.0,
                }
            },
        ]

        curve = extractor.build_curve_from_opportunities(opportunities, "SPX")
        assert curve.symbol == "SPX"
        assert len(curve.points) == 2

    def test_build_curve_skips_invalid(self, extractor):
        opportunities = [
            {
                "spread": {
                    "symbol": "SPX",
                    "expiry": "20260320",
                    "days_to_expiry": 30,
                    "strike_width": 50.0,
                    "buy_implied_rate": 4.50,
                    "sell_implied_rate": 4.30,
                    "buy_net_debit": 49.80,
                    "sell_net_credit": 49.70,
                    "liquidity_score": 80.0,
                }
            },
            {
                "spread": {
                    "symbol": "SPX",
                    "expiry": "20260620",
                    "days_to_expiry": 120,
                    "strike_width": 50.0,
                    "buy_implied_rate": 0.0,
                    "sell_implied_rate": 0.0,
                    "buy_net_debit": 0.0,
                    "sell_net_credit": 0.0,
                    "liquidity_score": 10.0,
                }
            },
        ]

        curve = extractor.build_curve_from_opportunities(opportunities, "SPX")
        assert len(curve.points) == 1
