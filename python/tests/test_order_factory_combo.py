"""Tests for combo/BAG order creation in order_factory.py."""

import pytest
from unittest.mock import MagicMock, patch
from datetime import datetime

with patch.dict("sys.modules", {
    "nautilus_trader": MagicMock(),
    "nautilus_trader.core": MagicMock(),
    "nautilus_trader.core.nautilus_pyo3": MagicMock(),
    "nautilus_trader.model": MagicMock(),
    "nautilus_trader.model.orders": MagicMock(),
    "nautilus_trader.model.identifiers": MagicMock(),
}):
    from python.integration.order_factory import OrderFactory


class FakeTimeInForce:
    DAY = "DAY"
    GTC = "GTC"
    IOC = "IOC"


@pytest.fixture
def factory():
    return OrderFactory()


@pytest.fixture
def box_spread_legs():
    return [
        {"symbol": "SPX", "expiry": "20260320", "strike": 5000.0, "right": "C", "action": "BUY"},
        {"symbol": "SPX", "expiry": "20260320", "strike": 5050.0, "right": "C", "action": "SELL"},
        {"symbol": "SPX", "expiry": "20260320", "strike": 5050.0, "right": "P", "action": "BUY"},
        {"symbol": "SPX", "expiry": "20260320", "strike": 5000.0, "right": "P", "action": "SELL"},
    ]


class TestComboOrderCreation:
    def test_creates_combo_order(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs, net_price=49.50)
        assert result is not None
        assert result["sec_type"] == "BAG"
        assert result["order_type"] == "LMT"
        assert result["net_price"] == 49.50
        assert len(result["legs"]) == 4

    def test_market_order_when_no_net_price(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs)
        assert result["order_type"] == "MKT"
        assert result["net_price"] is None

    def test_leg_structure(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs)
        leg = result["legs"][0]
        assert leg["symbol"] == "SPX"
        assert leg["strike"] == 5000.0
        assert leg["right"] == "C"
        assert leg["action"] == "BUY"
        assert leg["ratio"] == 1
        assert leg["exchange"] == "SMART"

    def test_rejects_too_few_legs(self, factory):
        result = factory.create_combo_order(legs=[{"action": "BUY"}])
        assert result is None

    def test_rejects_invalid_action(self, factory):
        legs = [
            {"symbol": "SPX", "action": "INVALID"},
            {"symbol": "SPX", "action": "BUY"},
        ]
        result = factory.create_combo_order(legs=legs)
        assert result is None

    def test_dry_run_flag(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs, dry_run=True)
        assert result is not None
        assert result["dry_run"] is True

    def test_order_id_generated(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs)
        assert "order_id" in result
        assert result["order_id"]  # non-empty string

    def test_quantity_parameter(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs, quantity=10)
        assert result["quantity"] == 10

    def test_con_id_passthrough(self, factory):
        legs = [
            {"symbol": "SPX", "action": "BUY", "con_id": 12345},
            {"symbol": "SPX", "action": "SELL", "con_id": 67890},
        ]
        result = factory.create_combo_order(legs=legs)
        assert result["legs"][0]["con_id"] == 12345
        assert result["legs"][1]["con_id"] == 67890

    def test_custom_exchange(self, factory):
        legs = [
            {"symbol": "SPX", "action": "BUY", "exchange": "CBOE"},
            {"symbol": "SPX", "action": "SELL", "exchange": "CBOE"},
        ]
        result = factory.create_combo_order(legs=legs)
        assert result["legs"][0]["exchange"] == "CBOE"

    def test_created_at_field(self, factory, box_spread_legs):
        result = factory.create_combo_order(legs=box_spread_legs)
        assert "created_at" in result
        datetime.fromisoformat(result["created_at"])
