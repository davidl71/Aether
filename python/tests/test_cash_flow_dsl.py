"""Tests for python/dsl/cash_flow_dsl.py"""

from decimal import Decimal
from datetime import datetime, timedelta

import pytest

from python.dsl.cash_flow_dsl import (
    CashFlowModel,
    CashFlowResult,
    Position,
    box_spread_lending,
    bank_loan,
    pension_loan,
    _payment_multiplier,
    _parse_maturity,
)


class TestPositionHelpers:
    def test_box_spread_lending(self):
        pos = box_spread_lending(100000, 0.045, "2026-06-30")
        assert pos.type == "box_spread_lending"
        assert pos.amount == Decimal("100000")
        assert pos.rate == Decimal("0.045")
        assert pos.maturity == "2026-06-30"
        assert pos.currency == "USD"

    def test_bank_loan(self):
        pos = bank_loan(50000, 0.065, "monthly")
        assert pos.type == "bank_loan"
        assert pos.amount == Decimal("50000")
        assert pos.payments == "monthly"

    def test_pension_loan(self):
        pos = pension_loan(20000, 0.04, "pension_fund_A")
        assert pos.type == "pension_loan"
        assert pos.payments == "monthly"


class TestPaymentMultiplier:
    def test_monthly(self):
        assert _payment_multiplier("monthly") == 12

    def test_quarterly(self):
        assert _payment_multiplier("quarterly") == 4

    def test_annually(self):
        assert _payment_multiplier("annually") == 1

    def test_none_defaults_monthly(self):
        assert _payment_multiplier(None) == 12


class TestParseMaturity:
    def test_iso_date(self):
        dt = _parse_maturity("2026-06-30")
        assert dt == datetime(2026, 6, 30)

    def test_compact_date(self):
        dt = _parse_maturity("20260630")
        assert dt == datetime(2026, 6, 30)

    def test_empty_string(self):
        assert _parse_maturity("") is None

    def test_invalid_string(self):
        assert _parse_maturity("not-a-date") is None


class TestCashFlowModelValidation:
    def test_empty_model_fails(self):
        model = CashFlowModel()
        errors = model.validate()
        assert "At least one position is required" in errors

    def test_valid_model(self):
        model = CashFlowModel()
        model.add_position(bank_loan(10000, 0.05))
        errors = model.validate()
        assert errors == []

    def test_project_rejects_zero(self):
        model = CashFlowModel()
        with pytest.raises(ValueError):
            model.project(0)

    def test_optimize_rejects_invalid(self):
        model = CashFlowModel()
        with pytest.raises(ValueError):
            model.optimize("invalid")

    def test_optimize_accepts_valid(self):
        model = CashFlowModel()
        model.optimize("net_cash_flow")
        assert model.optimization == "net_cash_flow"


class TestCashFlowModelChaining:
    def test_fluent_api(self):
        result = (
            CashFlowModel()
            .add_position(bank_loan(10000, 0.05))
            .project(6)
            .optimize("net_cash_flow")
        )
        assert isinstance(result, CashFlowModel)
        assert result.projection_months == 6
        assert len(result.positions) == 1


class TestSimulateValidationErrors:
    def test_returns_errors_when_invalid(self):
        result = CashFlowModel().simulate()
        assert result.errors is not None
        assert len(result.errors) > 0
        assert result.monthly_cash_flows == []
        assert result.total_net_cash_flow == Decimal("0")


class TestSimulateBankLoan:
    def test_loan_produces_negative_cash_flows(self):
        model = CashFlowModel()
        model.add_position(bank_loan(120000, 0.06))
        model.project(6)
        result = model.simulate()

        assert result.errors is None
        assert len(result.monthly_cash_flows) > 0
        for entry in result.monthly_cash_flows:
            assert entry["net_cash_flow"] < 0
        assert result.total_net_cash_flow < 0

    def test_loan_payment_amount(self):
        principal = 120000
        rate = 0.06
        expected_monthly = round(principal * rate / 12, 2)

        model = CashFlowModel()
        model.add_position(bank_loan(principal, rate))
        model.project(1)
        result = model.simulate()

        assert len(result.monthly_cash_flows) == 1
        assert result.monthly_cash_flows[0]["net_cash_flow"] == Decimal(str(-expected_monthly))


class TestSimulateBoxSpreadLending:
    def test_lending_produces_positive_flows(self):
        maturity = (datetime.now() + timedelta(days=200)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(100000, 0.048, maturity))
        model.project(3)
        result = model.simulate()

        assert result.errors is None
        assert len(result.monthly_cash_flows) > 0
        assert result.total_net_cash_flow > 0

    def test_maturity_principal_returned(self):
        maturity = (datetime.now() + timedelta(days=45)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(50000, 0.05, maturity))
        model.project(3)
        result = model.simulate()

        total = result.total_net_cash_flow
        assert total > Decimal("50000")


class TestSimulateMixedPortfolio:
    def test_loan_and_lending_net_out(self):
        maturity = (datetime.now() + timedelta(days=400)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(100000, 0.05, maturity))
        model.add_position(bank_loan(100000, 0.05))
        model.project(6)
        result = model.simulate()

        assert result.errors is None
        assert abs(result.total_net_cash_flow) < Decimal("1")

    def test_multiple_positions(self):
        maturity = (datetime.now() + timedelta(days=400)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(200000, 0.045, maturity))
        model.add_position(bank_loan(80000, 0.065))
        model.add_position(pension_loan(30000, 0.04, "fund"))
        model.project(12)
        result = model.simulate()

        assert result.errors is None
        assert len(result.monthly_cash_flows) > 0


class TestSimulateOptimization:
    def test_minimize_cost_sorts_ascending(self):
        maturity = (datetime.now() + timedelta(days=400)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(100000, 0.05, maturity))
        model.add_position(bank_loan(50000, 0.06))
        model.project(6)
        model.optimize("minimize_cost")
        result = model.simulate()

        values = [e["net_cash_flow"] for e in result.monthly_cash_flows]
        assert values == sorted(values)

    def test_maximize_return_sorts_descending(self):
        maturity = (datetime.now() + timedelta(days=400)).strftime("%Y-%m-%d")
        model = CashFlowModel()
        model.add_position(box_spread_lending(100000, 0.05, maturity))
        model.add_position(bank_loan(50000, 0.06))
        model.project(6)
        model.optimize("maximize_return")
        result = model.simulate()

        values = [e["net_cash_flow"] for e in result.monthly_cash_flows]
        assert values == sorted(values, reverse=True)


class TestCashFlowModelStr:
    def test_str_representation(self):
        model = CashFlowModel()
        model.add_position(bank_loan(10000, 0.05))
        assert "positions=1" in str(model)
        assert "months=12" in str(model)
