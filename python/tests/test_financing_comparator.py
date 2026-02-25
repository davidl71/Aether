"""
test_financing_comparator.py - Tests for the FinancingComparator decision matrix.
"""

import pytest
from python.integration.financing_comparator import (
    FinancingComparator,
    FinancingDirection,
    TaxConfig,
    InstrumentMetrics,
    ComparisonRow,
    DecisionMatrix,
)


@pytest.fixture
def tax_config():
    return TaxConfig(
        federal_rate=0.37,
        state_rate=0.05,
        ltcg_rate=0.20,
        stcg_rate=0.37,
        state_exempt_treasuries=True,
    )


@pytest.fixture
def comparator(tax_config):
    return FinancingComparator(
        tax_config=tax_config,
        per_contract_fee=0.65,
        treasury_client=None,
        sofr_client=None,
    )


class TestTaxConfig:
    def test_section_1256_blended_rate(self, tax_config):
        expected = 0.60 * 0.20 + 0.40 * 0.37  # = 0.268
        assert abs(tax_config.section_1256_blended_rate - expected) < 0.001

    def test_treasury_tax_rate_state_exempt(self, tax_config):
        assert tax_config.treasury_tax_rate == 0.37

    def test_treasury_tax_rate_not_exempt(self):
        cfg = TaxConfig(federal_rate=0.37, state_rate=0.05, state_exempt_treasuries=False)
        assert abs(cfg.treasury_tax_rate - 0.42) < 0.001


class TestBoxSpreadAfterTaxRate:
    def test_qualified_index(self, comparator):
        gross = 4.50
        blended = 0.60 * 0.20 + 0.40 * 0.37  # 0.268
        expected = gross * (1.0 - blended)
        result = comparator.calculate_box_spread_after_tax_rate(gross, is_qualified_index=True)
        assert abs(result - expected) < 0.01

    def test_non_qualified(self, comparator):
        gross = 4.50
        expected = gross * (1.0 - (0.37 + 0.05))
        result = comparator.calculate_box_spread_after_tax_rate(gross, is_qualified_index=False)
        assert abs(result - expected) < 0.01


class TestTreasuryAfterTaxRate:
    def test_state_exempt(self, comparator):
        gross = 4.50
        expected = gross * (1.0 - 0.37)
        result = comparator.calculate_treasury_after_tax_rate(gross)
        assert abs(result - expected) < 0.01


class TestComparisonRow:
    def test_winner_box_spread(self):
        bs = InstrumentMetrics(
            instrument_type="box_spread",
            tenor_days=30,
            gross_rate_pct=4.50,
            after_tax_rate_pct=3.30,
            tax_rate_applied=0.268,
            available=True,
        )
        tr = InstrumentMetrics(
            instrument_type="treasury",
            tenor_days=30,
            gross_rate_pct=4.50,
            after_tax_rate_pct=2.84,
            tax_rate_applied=0.37,
            available=True,
        )
        row = ComparisonRow(tenor_days=30, box_spread=bs, treasury=tr)
        assert row.winner == "box_spread"
        assert row.spread_bps_aftertax is not None
        assert row.spread_bps_aftertax > 0

    def test_winner_treasury(self):
        bs = InstrumentMetrics(
            instrument_type="box_spread",
            tenor_days=30,
            gross_rate_pct=3.00,
            after_tax_rate_pct=2.10,
            tax_rate_applied=0.268,
            available=True,
        )
        tr = InstrumentMetrics(
            instrument_type="treasury",
            tenor_days=30,
            gross_rate_pct=5.00,
            after_tax_rate_pct=3.15,
            tax_rate_applied=0.37,
            available=True,
        )
        row = ComparisonRow(tenor_days=30, box_spread=bs, treasury=tr)
        assert row.winner == "treasury"

    def test_winner_tie(self):
        bs = InstrumentMetrics(
            instrument_type="box_spread",
            tenor_days=30,
            gross_rate_pct=4.50,
            after_tax_rate_pct=3.00,
            tax_rate_applied=0.268,
            available=True,
        )
        tr = InstrumentMetrics(
            instrument_type="treasury",
            tenor_days=30,
            gross_rate_pct=4.76,
            after_tax_rate_pct=3.00,
            tax_rate_applied=0.37,
            available=True,
        )
        row = ComparisonRow(tenor_days=30, box_spread=bs, treasury=tr)
        assert row.winner == "tie"


class TestDecisionMatrix:
    def test_build_matrix_with_manual_rates(self, comparator):
        box_rates = {30: 4.25, 90: 4.50, 180: 4.75}
        treasury_rates = {30: 4.50, 90: 4.75, 180: 5.00}

        matrix = comparator.build_decision_matrix(
            box_spread_rates=box_rates,
            treasury_rates=treasury_rates,
            principal=100000.0,
            direction=FinancingDirection.LENDING,
        )

        assert len(matrix.rows) == 3
        assert matrix.rows[0].tenor_days == 30
        assert matrix.rows[1].tenor_days == 90
        assert matrix.rows[2].tenor_days == 180

        for row in matrix.rows:
            assert row.box_spread is not None
            assert row.treasury is not None
            assert row.spread_bps_pretax is not None
            assert row.spread_bps_aftertax is not None

    def test_summary(self, comparator):
        box_rates = {30: 4.25, 90: 4.50}
        treasury_rates = {30: 4.50, 90: 4.75}

        matrix = comparator.build_decision_matrix(
            box_spread_rates=box_rates,
            treasury_rates=treasury_rates,
        )

        s = matrix.summary
        assert "direction" in s
        assert "tenors_compared" in s
        assert s["tenors_compared"] == 2

    def test_to_dict(self, comparator):
        box_rates = {30: 4.25}
        treasury_rates = {30: 4.50}

        matrix = comparator.build_decision_matrix(
            box_spread_rates=box_rates,
            treasury_rates=treasury_rates,
        )

        d = matrix.to_dict()
        assert "rows" in d
        assert len(d["rows"]) == 1
        assert "box_spread" in d["rows"][0]
        assert "treasury" in d["rows"][0]

    def test_text_report(self, comparator):
        box_rates = {30: 4.25, 90: 4.50, 180: 4.75}
        treasury_rates = {30: 4.50, 90: 4.75, 180: 5.00}

        matrix = comparator.build_decision_matrix(
            box_spread_rates=box_rates,
            treasury_rates=treasury_rates,
        )

        report = comparator.format_text_report(matrix)
        assert "Box Spread" in report
        assert "Treasury" in report
        assert "30d" in report
        assert "90d" in report


class TestCompareFromYieldCurve:
    def test_from_curve_data(self, comparator):
        curve_rates = [
            {"days_to_expiry": 30, "mid_rate": 4.25, "strike_width": 50.0},
            {"days_to_expiry": 90, "implied_rate": 4.50, "strike_width": 50.0},
        ]

        matrix = comparator.compare_from_yield_curve(curve_rates, principal=50000.0)
        assert len(matrix.rows) == 2
