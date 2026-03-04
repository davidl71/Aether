"""
test_cash_flow_timeline.py - Tests for the simplified cash flow timeline calculator.

Used by TUI and PWA frontend components.
"""

import pytest
from datetime import datetime, timedelta

from python.integration.cash_flow_timeline import (
    calculate_cash_flow_timeline,
    CashFlowEvent,
)


@pytest.fixture
def now():
    return datetime.now()


class TestCashFlowEvent:
    def test_event_fields(self):
        event = CashFlowEvent(
            date="2026-03-15",
            amount=5000.0,
            description="Bond maturity",
            position_name="US5Y",
            type="maturity",
        )
        assert event.date == "2026-03-15"
        assert event.amount == 5000.0
        assert event.type == "maturity"


class TestPositionCashFlows:
    def test_maturity_within_horizon(self, now):
        maturity = now + timedelta(days=90)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "cash_flow": 10000.0,
                "instrument_type": "bond",
                "name": "US5Y Bond",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 1
        assert maturity_events[0].amount == 10000.0
        assert maturity_events[0].position_name == "US5Y Bond"

    def test_maturity_beyond_horizon(self, now):
        maturity = now + timedelta(days=500)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "cash_flow": 10000.0,
                "instrument_type": "bond",
                "name": "US10Y Bond",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 0

    def test_maturity_uses_candle_close(self, now):
        maturity = now + timedelta(days=60)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "candle": {"close": 5500.0},
                "instrument_type": "option",
                "name": "SPX Box",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 1
        assert maturity_events[0].amount == 5500.0

    def test_loan_monthly_payments(self, now):
        maturity = now + timedelta(days=180)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "cash_flow": 500000.0,
                "instrument_type": "bank_loan",
                "rate": 0.06,
                "name": "SHIR Loan",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        loan_events = [e for e in result.events if e.type == "loan_payment"]
        assert len(loan_events) > 0
        assert all(e.amount < 0 for e in loan_events)  # outflows

        # monthly_payment = (500000 * 0.06) / 12 = 2500
        assert abs(abs(loan_events[0].amount) - 2500.0) < 0.01

    def test_pension_loan_payments(self, now):
        maturity = now + timedelta(days=180)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "cash_flow": 300000.0,
                "instrument_type": "pension_loan",
                "rate": 0.04,
                "name": "Pension Loan",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        loan_events = [e for e in result.events if e.type == "loan_payment"]
        assert len(loan_events) > 0
        # monthly = (300000 * 0.04) / 12 = 1000
        assert abs(abs(loan_events[0].amount) - 1000.0) < 0.01

    def test_current_cash_flow(self, now):
        positions = [
            {
                "cash_flow": -1500.0,
                "instrument_type": "option",
                "name": "Short Put",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        other_events = [e for e in result.events if e.type == "other"]
        assert len(other_events) == 1
        assert other_events[0].amount == -1500.0

    def test_zero_cash_flow_skipped(self, now):
        positions = [
            {
                "cash_flow": 0,
                "instrument_type": "stock",
                "name": "AAPL",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        other_events = [e for e in result.events if e.type == "other"]
        assert len(other_events) == 0

    def test_no_maturity_date(self, now):
        positions = [
            {
                "instrument_type": "stock",
                "name": "AAPL",
                "cash_flow": 500.0,
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)

        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 0


class TestBankAccountCashFlows:
    def test_debit_interest_payments(self, now):
        bank_accounts = [
            {
                "debit_rate": 0.08,
                "balance": 100000.0,
                "account_name": "Bank Leumi",
            }
        ]
        result = calculate_cash_flow_timeline([], bank_accounts, projection_months=6)

        loan_events = [e for e in result.events if e.type == "loan_payment"]
        assert len(loan_events) == 6  # 6 months of payments
        # monthly = (100000 * 0.08) / 12 = 666.67
        assert abs(abs(loan_events[0].amount) - 666.67) < 0.01

    def test_zero_debit_rate(self, now):
        bank_accounts = [
            {
                "debit_rate": 0.0,
                "balance": 50000.0,
                "account_name": "No Interest Account",
            }
        ]
        result = calculate_cash_flow_timeline([], bank_accounts, projection_months=6)
        assert len(result.events) == 0

    def test_no_debit_rate(self, now):
        bank_accounts = [
            {
                "balance": 50000.0,
                "account_name": "Savings Account",
            }
        ]
        result = calculate_cash_flow_timeline([], bank_accounts, projection_months=6)
        assert len(result.events) == 0


class TestMonthlyAggregation:
    def test_monthly_flows_grouped(self, now):
        maturity1 = now + timedelta(days=15)
        maturity2 = now + timedelta(days=45)
        positions = [
            {
                "maturity_date": maturity1.isoformat(),
                "cash_flow": 5000.0,
                "instrument_type": "bond",
                "name": "Bond A",
            },
            {
                "maturity_date": maturity2.isoformat(),
                "cash_flow": 3000.0,
                "instrument_type": "bond",
                "name": "Bond B",
            },
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=3)

        assert len(result.monthly_flows) > 0
        for month, flow in result.monthly_flows.items():
            assert flow.month == month
            assert flow.net == flow.inflows - flow.outflows

    def test_totals_correct(self, now):
        maturity = now + timedelta(days=30)
        positions = [
            {
                "maturity_date": maturity.isoformat(),
                "cash_flow": 10000.0,
                "instrument_type": "bond",
                "name": "Bond",
            },
        ]
        bank_accounts = [
            {
                "debit_rate": 0.06,
                "balance": 100000.0,
                "account_name": "Bank",
            },
        ]
        result = calculate_cash_flow_timeline(positions, bank_accounts, projection_months=3)

        assert result.total_inflows > 0
        assert result.total_outflows > 0
        assert abs(result.net_cash_flow - (result.total_inflows - result.total_outflows)) < 0.01


class TestEdgeCases:
    def test_empty_inputs(self):
        result = calculate_cash_flow_timeline([], [], projection_months=12)
        assert len(result.events) == 0
        assert result.total_inflows == 0.0
        assert result.total_outflows == 0.0
        assert result.net_cash_flow == 0.0
        assert len(result.monthly_flows) == 0

    def test_invalid_maturity_date(self):
        positions = [
            {
                "maturity_date": "not-a-date",
                "cash_flow": 5000.0,
                "instrument_type": "bond",
                "name": "Bad Bond",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)
        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 0

    def test_past_maturity_excluded(self, now):
        past = now - timedelta(days=30)
        positions = [
            {
                "maturity_date": past.isoformat(),
                "cash_flow": 5000.0,
                "instrument_type": "bond",
                "name": "Expired Bond",
            }
        ]
        result = calculate_cash_flow_timeline(positions, [], projection_months=12)
        maturity_events = [e for e in result.events if e.type == "maturity"]
        assert len(maturity_events) == 0
