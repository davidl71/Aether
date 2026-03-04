"""Tests for order_manager.py - Python port of C++ order manager."""

import pytest

from python.integration.order_manager import (
    BoxSpreadLegSimple,
    MultiLegOrder,
    Order,
    OrderAction,
    OrderContract,
    OrderManager,
)


# ---------------------------------------------------------------------------
# OrderContract
# ---------------------------------------------------------------------------


class TestOrderContract:
    def test_is_valid(self):
        assert OrderContract(symbol="SPX").is_valid()
        assert not OrderContract().is_valid()

    def test_to_string(self):
        c = OrderContract(symbol="SPX", expiry="20260401", strike=4500.0, option_type="C")
        s = c.to_string()
        assert "SPX" in s
        assert "4500" in s


# ---------------------------------------------------------------------------
# MultiLegOrder
# ---------------------------------------------------------------------------


class TestMultiLegOrder:
    def test_is_complete(self):
        ml = MultiLegOrder(legs=[Order(), Order()], legs_filled=2)
        assert ml.is_complete()

    def test_not_complete(self):
        ml = MultiLegOrder(legs=[Order(), Order()], legs_filled=1)
        assert not ml.is_complete()

    def test_partially_filled(self):
        ml = MultiLegOrder(legs=[Order(), Order(), Order()], legs_filled=1)
        assert ml.is_partially_filled()

    def test_not_partially_filled(self):
        ml = MultiLegOrder(legs=[Order(), Order()], legs_filled=0)
        assert not ml.is_partially_filled()


# ---------------------------------------------------------------------------
# OrderManager - Dry Run
# ---------------------------------------------------------------------------


class TestOrderManagerDryRun:
    def test_default_is_dry_run(self):
        om = OrderManager()
        assert om.is_dry_run()

    def test_place_order_dry_run(self):
        om = OrderManager(dry_run=True)
        contract = OrderContract(symbol="SPX", expiry="20260401", strike=4500.0)
        result = om.place_order(contract, OrderAction.Buy, 1, 10.0)
        assert result.success
        assert len(result.order_ids) == 1
        assert om.get_stats().total_orders_placed == 1

    def test_cancel_order_dry_run(self):
        om = OrderManager(dry_run=True)
        assert om.cancel_order(999)

    def test_cancel_all_dry_run(self):
        om = OrderManager(dry_run=True)
        om.cancel_all_orders()  # Should not raise


# ---------------------------------------------------------------------------
# OrderManager - Validation
# ---------------------------------------------------------------------------


class TestOrderValidation:
    def test_invalid_contract(self):
        om = OrderManager()
        result = om.place_order(OrderContract(), OrderAction.Buy, 1)
        assert not result.success
        assert "Invalid contract" in result.error_message

    def test_zero_quantity(self):
        om = OrderManager()
        result = om.place_order(OrderContract(symbol="SPX"), OrderAction.Buy, 0)
        assert not result.success
        assert "Quantity" in result.error_message

    def test_negative_quantity(self):
        om = OrderManager()
        result = om.place_order(OrderContract(symbol="SPX"), OrderAction.Buy, -1)
        assert not result.success

    def test_exceeds_size_limit(self):
        om = OrderManager(max_order_size=10)
        result = om.place_order(OrderContract(symbol="SPX"), OrderAction.Buy, 20)
        assert not result.success
        assert "exceeds limits" in result.error_message

    def test_negative_limit_price(self):
        om = OrderManager()
        result = om.place_order(OrderContract(symbol="SPX"), OrderAction.Buy, 1, -5.0)
        assert not result.success
        assert "negative" in result.error_message

    def test_validate_order_method(self):
        om = OrderManager()
        ok, err = om.validate_order(OrderContract(symbol="SPX"), OrderAction.Buy, 5, 10.0)
        assert ok
        assert err == ""

    def test_exceeds_limits(self):
        om = OrderManager(max_order_size=50)
        assert not om.exceeds_limits(50)
        assert om.exceeds_limits(51)


# ---------------------------------------------------------------------------
# OrderManager - Box Spread
# ---------------------------------------------------------------------------


class TestBoxSpreadOrder:
    def test_place_box_spread_dry_run(self):
        om = OrderManager(dry_run=True)
        spread = BoxSpreadLegSimple(
            long_call=OrderContract("SPX", "20260401", 4500, "C"),
            short_call=OrderContract("SPX", "20260401", 4600, "C"),
            long_put=OrderContract("SPX", "20260401", 4600, "P"),
            short_put=OrderContract("SPX", "20260401", 4500, "P"),
            net_debit=98.0,
        )
        result = om.place_box_spread(spread)
        assert result.success
        assert len(result.order_ids) == 4
        assert om.get_stats().total_orders_placed == 4

    def test_close_box_spread_dry_run(self):
        om = OrderManager(dry_run=True)
        result = om.close_box_spread("strat-1")
        assert result.success

    def test_get_multi_leg_order_missing(self):
        om = OrderManager()
        assert om.get_multi_leg_order("missing") is None

    def test_are_all_legs_filled_missing(self):
        om = OrderManager()
        assert not om.are_all_legs_filled("missing")

    def test_no_client_fails(self):
        om = OrderManager(dry_run=False, client=None)
        result = om.place_order(OrderContract(symbol="SPX"), OrderAction.Buy, 1)
        assert not result.success
        assert "client" in result.error_message.lower()


# ---------------------------------------------------------------------------
# OrderManager - Specialty Orders
# ---------------------------------------------------------------------------


class TestSpecialtyOrders:
    def test_ioc_order(self):
        om = OrderManager(dry_run=True)
        result = om.execute_ioc(OrderContract(symbol="SPX"), OrderAction.Buy, 1, 10.0)
        assert result.success

    def test_fok_order(self):
        om = OrderManager(dry_run=True)
        result = om.execute_fok(OrderContract(symbol="SPX"), OrderAction.Sell, 1, 10.0)
        assert result.success


# ---------------------------------------------------------------------------
# OrderManager - Config
# ---------------------------------------------------------------------------


class TestOrderManagerConfig:
    def test_set_max_order_size(self):
        om = OrderManager()
        om.set_max_order_size(200)
        assert not om.exceeds_limits(200)
        assert om.exceeds_limits(201)

    def test_set_dry_run(self):
        om = OrderManager(dry_run=True)
        om.set_dry_run(False)
        assert not om.is_dry_run()


# ---------------------------------------------------------------------------
# OrderManager - Stats / Tracking
# ---------------------------------------------------------------------------


class TestOrderStats:
    def test_initial_stats(self):
        om = OrderManager()
        stats = om.get_stats()
        assert stats.total_orders_placed == 0
        assert stats.efficiency_ratio == 0.0

    def test_track_fill(self):
        om = OrderManager()
        om.track_order_fill(100)
        stats = om.get_stats()
        assert stats.total_orders_filled == 1
        assert stats.executed_trades == 1

    def test_track_duplicate_fill(self):
        om = OrderManager()
        om.track_order_fill(100)
        om.track_order_fill(100)
        assert om.get_stats().total_orders_filled == 1

    def test_efficiency_ratio(self):
        om = OrderManager(dry_run=True)
        om.place_order(OrderContract(symbol="A"), OrderAction.Buy, 1)
        om.place_order(OrderContract(symbol="B"), OrderAction.Buy, 1)
        om.track_order_fill(1)
        stats = om.get_stats()
        assert stats.efficiency_ratio == pytest.approx(0.5)


# ---------------------------------------------------------------------------
# OrderManager - Get Order Status
# ---------------------------------------------------------------------------


class TestGetOrderStatus:
    def test_no_client(self):
        om = OrderManager(client=None)
        assert om.get_order_status(123) is None
