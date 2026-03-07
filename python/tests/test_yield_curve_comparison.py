"""
test_yield_curve_comparison.py - Tests for yield curve overlay comparison.
"""

import pytest
from datetime import datetime

from python.integration.yield_curve_comparison import (
    CurvePoint,
    NelsonSiegelFitter,
    SpreadPoint,
    YieldCurveComparison,
    YieldCurveComparer,
    _tenor_label,
)


@pytest.fixture
def comparer():
    return YieldCurveComparer(treasury_client=None, sofr_client=None)


@pytest.fixture
def box_rates():
    return {
        "SPX": {30: 4.25, 90: 4.50, 180: 4.75, 365: 5.00},
        "XSP": {30: 4.20, 90: 4.45, 180: 4.70},
    }


@pytest.fixture
def treasury_rates():
    return {30: 4.40, 90: 4.65, 180: 4.90, 365: 5.10}


@pytest.fixture
def sofr_rates():
    return {1: 4.30, 30: 4.32, 90: 4.35}


class TestCurvePoint:
    def test_fields(self):
        pt = CurvePoint(
            days_to_expiry=90, rate_pct=4.50,
            source="Treasury 3-Month", tenor_label="3M",
        )
        assert pt.days_to_expiry == 90
        assert pt.rate_pct == 4.50
        assert pt.source == "Treasury 3-Month"

    def test_default_liquidity(self):
        pt = CurvePoint(days_to_expiry=30, rate_pct=4.25, source="test")
        assert pt.liquidity_score == 0.0


class TestSpreadPoint:
    def test_box_wins(self):
        sp = SpreadPoint(
            days_to_expiry=90, tenor_label="3M",
            box_rate_pct=4.80, benchmark_rate_pct=4.50,
            benchmark_source="Treasury", spread_bps=30.0,
        )
        assert sp.box_wins is True
        assert sp.benchmark_wins is False

    def test_benchmark_wins(self):
        sp = SpreadPoint(
            days_to_expiry=90, tenor_label="3M",
            box_rate_pct=4.20, benchmark_rate_pct=4.50,
            benchmark_source="Treasury", spread_bps=-30.0,
        )
        assert sp.box_wins is False
        assert sp.benchmark_wins is True

    def test_tie(self):
        sp = SpreadPoint(
            days_to_expiry=90, tenor_label="3M",
            box_rate_pct=4.50, benchmark_rate_pct=4.50,
            benchmark_source="Treasury", spread_bps=0.0,
        )
        assert sp.box_wins is False
        assert sp.benchmark_wins is False


class TestTenorLabel:
    def test_exact_match(self):
        assert _tenor_label(30) == "1M"
        assert _tenor_label(90) == "3M"
        assert _tenor_label(365) == "1Y"

    def test_near_match(self):
        assert _tenor_label(88) == "3M"
        assert _tenor_label(92) == "3M"
        assert _tenor_label(362) == "1Y"

    def test_short_days(self):
        assert _tenor_label(7) == "7d"
        assert _tenor_label(14) == "14d"

    def test_month_approximation(self):
        label = _tenor_label(45)
        assert "M" in label or "d" in label

    def test_overnight(self):
        assert _tenor_label(1) == "O/N"


class TestYieldCurveComparison:
    def test_all_tenors(self):
        comp = YieldCurveComparison()
        comp.box_curves = {
            "SPX": [CurvePoint(30, 4.25, "test"), CurvePoint(90, 4.50, "test")],
        }
        comp.treasury_curve = [CurvePoint(180, 4.90, "test")]

        tenors = comp.all_tenors
        assert tenors == [30, 90, 180]

    def test_symbols(self):
        comp = YieldCurveComparison()
        comp.box_curves = {
            "XSP": [CurvePoint(30, 4.20, "test")],
            "SPX": [CurvePoint(30, 4.25, "test")],
        }
        assert comp.symbols == ["SPX", "XSP"]

    def test_get_benchmark_at_dte_exact(self):
        comp = YieldCurveComparison()
        comp.treasury_curve = [
            CurvePoint(90, 4.65, "Treasury"),
            CurvePoint(180, 4.90, "Treasury"),
        ]
        bench = comp.get_benchmark_at_dte(90)
        assert bench is not None
        assert bench.rate_pct == 4.65

    def test_get_benchmark_at_dte_tolerance(self):
        comp = YieldCurveComparison()
        comp.treasury_curve = [CurvePoint(88, 4.65, "Treasury")]

        bench = comp.get_benchmark_at_dte(90, tolerance=5)
        assert bench is not None
        assert bench.rate_pct == 4.65

    def test_get_benchmark_at_dte_prefers_closest(self):
        comp = YieldCurveComparison()
        comp.treasury_curve = [
            CurvePoint(85, 4.60, "Treasury"),
            CurvePoint(91, 4.65, "Treasury"),
        ]
        bench = comp.get_benchmark_at_dte(90, tolerance=10)
        assert bench is not None
        assert bench.rate_pct == 4.65  # 91 is closer to 90 than 85

    def test_get_benchmark_at_dte_not_found(self):
        comp = YieldCurveComparison()
        comp.treasury_curve = [CurvePoint(365, 5.10, "Treasury")]

        bench = comp.get_benchmark_at_dte(30, tolerance=10)
        assert bench is None

    def test_get_benchmark_includes_sofr(self):
        comp = YieldCurveComparison()
        comp.sofr_curve = [CurvePoint(1, 4.30, "SOFR")]

        bench = comp.get_benchmark_at_dte(1)
        assert bench is not None
        assert bench.rate_pct == 4.30

    def test_summary(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)
        s = comp.summary()

        assert "symbols" in s
        assert set(s["symbols"]) == {"SPX", "XSP"}
        assert s["treasury_points"] == 4
        assert s["spread_points"] > 0
        assert s["box_spread_wins"] + s["benchmark_wins"] + s["ties"] == s["spread_points"]

    def test_to_dict(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)
        d = comp.to_dict()

        assert "box_curves" in d
        assert "treasury_curve" in d
        assert "sofr_curve" in d
        assert "spreads" in d
        assert "SPX" in d["box_curves"]
        assert len(d["treasury_curve"]) == 4


class TestYieldCurveComparer:
    def test_compare_manual_rates(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)

        assert len(comp.box_curves) == 2
        assert "SPX" in comp.box_curves
        assert "XSP" in comp.box_curves
        assert len(comp.box_curves["SPX"]) == 4
        assert len(comp.box_curves["XSP"]) == 3
        assert len(comp.treasury_curve) == 4

    def test_spreads_computed(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)

        assert len(comp.spreads) > 0
        for sp in comp.spreads:
            expected_bps = (sp.box_rate_pct - sp.benchmark_rate_pct) * 100.0
            assert abs(sp.spread_bps - expected_bps) < 0.01

    def test_spread_direction(self, comparer):
        """Box at 4.25% vs Treasury at 4.40% => negative spread (benchmark wins)."""
        comp = comparer.compare(
            box_spread_rates={"SPX": {90: 4.25}},
            treasury_rates={90: 4.40},
        )
        assert len(comp.spreads) == 1
        assert comp.spreads[0].spread_bps < 0
        assert comp.spreads[0].benchmark_wins

    def test_spread_direction_positive(self, comparer):
        """Box at 5.00% vs Treasury at 4.40% => positive spread (box wins)."""
        comp = comparer.compare(
            box_spread_rates={"SPX": {90: 5.00}},
            treasury_rates={90: 4.40},
        )
        assert len(comp.spreads) == 1
        assert comp.spreads[0].spread_bps > 0
        assert comp.spreads[0].box_wins

    def test_compare_with_sofr(self, comparer, box_rates, treasury_rates, sofr_rates):
        comp = comparer.compare(
            box_rates,
            treasury_rates=treasury_rates,
            sofr_rates=sofr_rates,
        )
        assert len(comp.sofr_curve) == 3
        assert comp.sofr_curve[0].days_to_expiry == 1

    def test_compare_with_liquidity_scores(self, comparer, treasury_rates):
        comp = comparer.compare(
            box_spread_rates={"SPX": {90: 4.50}},
            treasury_rates=treasury_rates,
            liquidity_scores={"SPX": {90: 85.0}},
        )
        assert comp.box_curves["SPX"][0].liquidity_score == 85.0

    def test_compare_no_benchmarks(self, comparer, box_rates):
        comp = comparer.compare(box_rates)

        assert len(comp.treasury_curve) == 0
        assert len(comp.sofr_curve) == 0
        assert len(comp.spreads) == 0

    def test_spreads_sorted(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)

        for i in range(1, len(comp.spreads)):
            prev = comp.spreads[i - 1]
            curr = comp.spreads[i]
            if prev.box_symbol == curr.box_symbol:
                assert prev.days_to_expiry <= curr.days_to_expiry

    def test_format_text(self, comparer, box_rates, treasury_rates):
        comp = comparer.compare(box_rates, treasury_rates=treasury_rates)
        report = comparer.format_text(comp)

        assert "YIELD CURVE COMPARISON" in report
        assert "TREASURY YIELD CURVE" in report
        assert "BOX SPREAD: SPX" in report
        assert "BOX SPREAD: XSP" in report
        assert "SPREAD ANALYSIS" in report
        assert "Box spread wins" in report

    def test_format_text_no_treasury(self, comparer, box_rates):
        comp = comparer.compare(box_rates)
        report = comparer.format_text(comp)

        assert "YIELD CURVE COMPARISON" in report
        assert "BOX SPREAD: SPX" in report
        assert "TREASURY YIELD CURVE" not in report

    def test_multi_symbol_spread_attribution(self, comparer, treasury_rates):
        comp = comparer.compare(
            box_spread_rates={"SPX": {90: 4.80}, "XSP": {90: 4.20}},
            treasury_rates=treasury_rates,
        )
        spx_spreads = [s for s in comp.spreads if s.box_symbol == "SPX"]
        xsp_spreads = [s for s in comp.spreads if s.box_symbol == "XSP"]
        assert len(spx_spreads) == 1
        assert len(xsp_spreads) == 1
        assert spx_spreads[0].spread_bps > xsp_spreads[0].spread_bps

    def test_tolerance_matching(self, comparer):
        """Box at 92d should match Treasury at 90d within tolerance."""
        comp = comparer.compare(
            box_spread_rates={"SPX": {92: 4.50}},
            treasury_rates={90: 4.65},
        )
        assert len(comp.spreads) == 1
        assert comp.spreads[0].benchmark_rate_pct == 4.65

    def test_no_match_outside_tolerance(self, comparer):
        """Box at 45d should NOT match Treasury at 90d (tolerance=20)."""
        comp = comparer.compare(
            box_spread_rates={"SPX": {45: 4.50}},
            treasury_rates={90: 4.65},
        )
        assert len(comp.spreads) == 0


class TestCompareFromExtractorCurves:
    def test_from_extractor_curves(self, comparer):
        """Test integration with RiskFreeRateCurve if available."""
        try:
            from python.integration.risk_free_rate_extractor import (
                RiskFreeRateCurve,
                RiskFreeRatePoint,
            )
        except ImportError:
            pytest.skip("risk_free_rate_extractor not available")

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

        comp = comparer.compare_from_extractor_curves(
            curves={"SPX": curve},
            treasury_rates={30: 4.50, 90: 4.65, 180: 4.90},
        )

        assert "SPX" in comp.box_curves
        assert len(comp.box_curves["SPX"]) == 2
        assert comp.box_curves["SPX"][0].liquidity_score == 80.0
        assert len(comp.spreads) > 0


class TestNelsonSiegelFitter:
    def _make_points(self):
        # Synthetic normal upward-sloping curve
        tenors = [(30, 5.3), (90, 5.25), (180, 5.1), (365, 4.9),
                  (730, 4.6), (1095, 4.4), (1825, 4.3), (3650, 4.2)]
        return [CurvePoint(dte, rate, "test", f"{dte}d") for dte, rate in tenors]

    def test_fit_returns_true_with_enough_points(self):
        fitter = NelsonSiegelFitter()
        assert fitter.fit(self._make_points()) is True

    def test_rate_at_interpolates_within_range(self):
        fitter = NelsonSiegelFitter()
        fitter.fit(self._make_points())
        r = fitter.rate_at(180)
        assert r is not None
        assert 4.0 <= r <= 6.0

    def test_rate_at_extrapolates_beyond_tenors(self):
        fitter = NelsonSiegelFitter()
        fitter.fit(self._make_points())
        r = fitter.rate_at(7300)  # 20yr, beyond the curve
        assert r is not None
        assert 3.5 <= r <= 6.0

    def test_fit_returns_false_with_too_few_points(self):
        fitter = NelsonSiegelFitter()
        assert fitter.fit([CurvePoint(90, 5.0, "test", "3m")]) is False

    def test_params_are_set_after_fit(self):
        fitter = NelsonSiegelFitter()
        fitter.fit(self._make_points())
        assert fitter.params is not None
        assert len(fitter.params) == 4
