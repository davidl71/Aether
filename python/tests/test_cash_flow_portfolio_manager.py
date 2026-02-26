"""
test_cash_flow_portfolio_manager.py - Tests for cash flow-aware portfolio allocation.
"""

import pytest
from datetime import datetime, timedelta

from python.integration.cash_flow_portfolio_manager import (
    CashFlowPortfolioManager,
    CashManagementSnapshot,
    CashFlowAlert,
    AllocationRecommendation,
    CashReservePolicy,
    AlertSeverity,
    AllocationAction,
)


@pytest.fixture
def manager():
    return CashFlowPortfolioManager()


@pytest.fixture
def manager_strict():
    policy = CashReservePolicy(
        min_months_coverage=6,
        min_reserve_usd=50000.0,
        reserve_multiplier=1.5,
    )
    return CashFlowPortfolioManager(policy=policy)


@pytest.fixture
def position_with_maturity():
    maturity = (datetime.now() + timedelta(days=60)).isoformat()
    return {
        "name": "Box-SPX-90d",
        "maturity_date": maturity,
        "cash_flow": 50000.0,
        "instrument_type": "box_spread",
        "candle": {"close": 49.80},
    }


@pytest.fixture
def loan_position():
    maturity = (datetime.now() + timedelta(days=180)).isoformat()
    return {
        "name": "Loan-Bank-A",
        "maturity_date": maturity,
        "cash_flow": 100000.0,
        "instrument_type": "bank_loan",
        "rate": 0.06,
        "candle": {"close": 100000.0},
    }


@pytest.fixture
def bank_account():
    return {
        "account_name": "Margin Account",
        "balance": 200000.0,
        "debit_rate": 0.04,
    }


class TestCashReservePolicy:
    def test_defaults(self):
        policy = CashReservePolicy()
        assert policy.min_months_coverage == 3
        assert policy.min_reserve_usd == 10000.0
        assert policy.reserve_multiplier == 1.2
        assert policy.alert_threshold_pct == 0.80
        assert policy.critical_threshold_pct == 0.50

    def test_custom(self):
        policy = CashReservePolicy(min_months_coverage=6, min_reserve_usd=50000.0)
        assert policy.min_months_coverage == 6
        assert policy.min_reserve_usd == 50000.0


class TestCashFlowAlert:
    def test_to_dict(self):
        alert = CashFlowAlert(
            severity=AlertSeverity.WARNING,
            message="Test alert",
            trigger_date="2026-03-01",
            amount=-5000.0,
            position_name="Loan-A",
            action_recommended=AllocationAction.INCREASE_RESERVES,
        )
        d = alert.to_dict()
        assert d["severity"] == "warning"
        assert d["message"] == "Test alert"
        assert d["trigger_date"] == "2026-03-01"
        assert d["action_recommended"] == "increase_reserves"


class TestAllocationRecommendation:
    def test_to_dict(self):
        rec = AllocationRecommendation(
            action=AllocationAction.DEPLOY_EXCESS,
            reason="Excess cash available",
            amount_usd=25000.0,
            target_instrument="box_spread_or_treasury",
            urgency_days=30,
            confidence=0.7,
        )
        d = rec.to_dict()
        assert d["action"] == "deploy_excess"
        assert d["amount_usd"] == 25000.0
        assert d["target_instrument"] == "box_spread_or_treasury"


class TestCashManagementSnapshot:
    def test_to_dict(self):
        snapshot = CashManagementSnapshot(
            available_cash=100000.0,
            required_reserves=30000.0,
            reserve_coverage_ratio=3.33,
            upcoming_outflows_30d=5000.0,
            upcoming_outflows_90d=15000.0,
            upcoming_inflows_30d=0.0,
            upcoming_inflows_90d=50000.0,
            net_position_30d=-5000.0,
            net_position_90d=35000.0,
            excess_cash=70000.0,
            deployable_amount=56000.0,
        )
        d = snapshot.to_dict()
        assert d["available_cash"] == 100000.0
        assert d["required_reserves"] == 30000.0
        assert d["excess_cash"] == 70000.0
        assert isinstance(d["alerts"], list)
        assert isinstance(d["recommendations"], list)


class TestCashFlowPortfolioManager:
    def test_analyze_no_positions(self, manager):
        snapshot = manager.analyze([], [], available_cash=50000.0)
        assert snapshot.available_cash == 50000.0
        assert snapshot.upcoming_outflows_30d == 0.0
        assert snapshot.upcoming_outflows_90d == 0.0

    def test_analyze_with_maturity(self, manager, position_with_maturity):
        snapshot = manager.analyze(
            positions=[position_with_maturity],
            bank_accounts=[],
            available_cash=100000.0,
        )
        assert snapshot.available_cash == 100000.0
        assert snapshot.upcoming_inflows_90d >= 50000.0

    def test_analyze_with_loan(self, manager, loan_position):
        snapshot = manager.analyze(
            positions=[loan_position],
            bank_accounts=[],
            available_cash=50000.0,
        )
        assert snapshot.upcoming_outflows_90d > 0

    def test_analyze_with_bank_account(self, manager, bank_account):
        snapshot = manager.analyze(
            positions=[],
            bank_accounts=[bank_account],
            available_cash=50000.0,
        )
        assert snapshot.upcoming_outflows_90d > 0

    def test_low_cash_generates_warning(self, manager, loan_position):
        snapshot = manager.analyze(
            positions=[loan_position],
            bank_accounts=[],
            available_cash=500.0,
        )
        warnings = [a for a in snapshot.alerts if a.severity in (AlertSeverity.WARNING, AlertSeverity.CRITICAL)]
        assert len(warnings) > 0

    def test_adequate_cash_no_critical_alerts(self, manager, position_with_maturity):
        snapshot = manager.analyze(
            positions=[position_with_maturity],
            bank_accounts=[],
            available_cash=500000.0,
        )
        critical = [a for a in snapshot.alerts if a.severity == AlertSeverity.CRITICAL]
        assert len(critical) == 0

    def test_excess_cash_deploy_recommendation(self, manager):
        snapshot = manager.analyze(
            positions=[],
            bank_accounts=[],
            available_cash=100000.0,
        )
        deploy_recs = [r for r in snapshot.recommendations if r.action == AllocationAction.DEPLOY_EXCESS]
        assert len(deploy_recs) > 0
        assert deploy_recs[0].amount_usd > 0

    def test_reserve_coverage_ratio(self, manager, loan_position):
        snapshot = manager.analyze(
            positions=[loan_position],
            bank_accounts=[],
            available_cash=100000.0,
        )
        assert snapshot.reserve_coverage_ratio > 0

    def test_strict_policy(self, manager_strict, loan_position):
        snapshot = manager_strict.analyze(
            positions=[loan_position],
            bank_accounts=[],
            available_cash=20000.0,
        )
        assert snapshot.required_reserves >= 50000.0
        critical_or_warn = [a for a in snapshot.alerts
                           if a.severity in (AlertSeverity.WARNING, AlertSeverity.CRITICAL)]
        assert len(critical_or_warn) > 0

    def test_net_position_calculations(self, manager, position_with_maturity, loan_position):
        snapshot = manager.analyze(
            positions=[position_with_maturity, loan_position],
            bank_accounts=[],
            available_cash=100000.0,
        )
        assert snapshot.net_position_90d == snapshot.upcoming_inflows_90d - snapshot.upcoming_outflows_90d

    def test_to_dict_complete(self, manager, position_with_maturity, loan_position):
        snapshot = manager.analyze(
            positions=[position_with_maturity, loan_position],
            bank_accounts=[],
            available_cash=100000.0,
        )
        d = snapshot.to_dict()
        required_keys = [
            "available_cash", "required_reserves", "reserve_coverage_ratio",
            "upcoming_outflows_30d", "upcoming_outflows_90d",
            "upcoming_inflows_30d", "upcoming_inflows_90d",
            "net_position_30d", "net_position_90d",
            "alerts", "recommendations",
            "excess_cash", "deployable_amount", "generated_time",
        ]
        for key in required_keys:
            assert key in d, f"Missing key: {key}"


class TestRebalanceTriggers:
    def test_get_rebalance_triggers_empty(self, manager):
        triggers = manager.get_rebalance_triggers([], [], available_cash=50000.0)
        assert isinstance(triggers, list)

    def test_get_rebalance_triggers_with_large_inflow(self, manager):
        maturity = (datetime.now() + timedelta(days=30)).isoformat()
        position = {
            "name": "Bond-Maturity",
            "maturity_date": maturity,
            "cash_flow": 50000.0,
            "instrument_type": "bond",
            "candle": {"close": 50000.0},
        }
        triggers = manager.get_rebalance_triggers(
            positions=[position],
            bank_accounts=[],
            available_cash=100000.0,
        )
        reinvest = [t for t in triggers if t.get("type") == "reinvestment_opportunity"]
        assert len(reinvest) > 0

    def test_triggers_sorted_by_date(self, manager):
        now = datetime.now()
        positions = [
            {
                "name": f"Bond-{i}",
                "maturity_date": (now + timedelta(days=30 * i)).isoformat(),
                "cash_flow": 20000.0,
                "instrument_type": "bond",
                "candle": {"close": 20000.0},
            }
            for i in range(1, 4)
        ]
        triggers = manager.get_rebalance_triggers(positions, [], available_cash=100000.0)
        dates = [t["date"] for t in triggers]
        assert dates == sorted(dates)
