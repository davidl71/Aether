"""
Tests for Israeli broker models module.

Tests PositionSource enum and Position dataclass.
"""
import unittest
from datetime import datetime

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from integration.israeli_broker_models import PositionSource, Position


class TestPositionSource(unittest.TestCase):
    """Tests for PositionSource enum."""

    def test_position_source_values(self):
        """Test PositionSource enum values."""
        assert PositionSource.IBKR.value == "ibkr"
        assert PositionSource.ISRAELI_BROKER_EXCEL.value == "israeli_excel"
        assert PositionSource.ISRAELI_BROKER_RTD.value == "israeli_rtd"
        assert PositionSource.ISRAELI_BROKER_DDE.value == "israeli_dde"
        assert PositionSource.ISRAELI_BROKER_WEB.value == "israeli_web"

    def test_position_source_from_string(self):
        """Test creating PositionSource from string."""
        assert PositionSource("ibkr") == PositionSource.IBKR
        assert PositionSource("israeli_excel") == PositionSource.ISRAELI_BROKER_EXCEL


class TestPosition(unittest.TestCase):
    """Tests for Position dataclass."""

    def test_position_minimal(self):
        """Test Position with minimal required fields."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR
        )
        assert position.symbol == "AAPL"
        assert position.quantity == 100.0
        assert position.cost_basis == 150.0
        assert position.current_price == 155.0
        assert position.currency == "USD"
        assert position.broker == "IBKR"
        assert position.source == PositionSource.IBKR

    def test_position_with_optional_fields(self):
        """Test Position with optional fields."""
        now = datetime.now()
        position = Position(
            symbol="SPY",
            quantity=50.0,
            cost_basis=450.0,
            current_price=455.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR,
            account_id="DU123456",
            last_updated=now,
            unrealized_pnl=250.0,
            exchange="NYSE",
            instrument_type="stock"
        )
        assert position.account_id == "DU123456"
        assert position.last_updated == now
        assert position.unrealized_pnl == 250.0
        assert position.exchange == "NYSE"
        assert position.instrument_type == "stock"

    def test_position_calculate_pnl_with_unrealized(self):
        """Test Position.calculate_pnl() when unrealized_pnl is set."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR,
            unrealized_pnl=500.0
        )
        assert position.calculate_pnl() == 500.0

    def test_position_calculate_pnl_without_unrealized(self):
        """Test Position.calculate_pnl() when unrealized_pnl is None."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR
        )
        expected_pnl = (155.0 - 150.0) * 100.0
        assert position.calculate_pnl() == expected_pnl

    def test_position_get_market_value_usd_usd_currency(self):
        """Test Position.get_market_value_usd() with USD currency."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR
        )
        market_value = position.get_market_value_usd(0.27)
        assert market_value == 155.0 * 100.0

    def test_position_get_market_value_usd_ils_currency(self):
        """Test Position.get_market_value_usd() with ILS currency."""
        position = Position(
            symbol="TA35",
            quantity=10.0,
            cost_basis=2000.0,
            current_price=2100.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL
        )
        fx_rate = 0.27  # ILS to USD
        market_value = position.get_market_value_usd(fx_rate)
        expected = 2100.0 * 10.0 * 0.27
        assert market_value == expected

    def test_position_is_tase_instrument_true(self):
        """Test Position.is_tase_instrument() with TASE exchange."""
        position = Position(
            symbol="TA35",
            quantity=10.0,
            cost_basis=2000.0,
            current_price=2100.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE"
        )
        assert position.is_tase_instrument() is True

    def test_position_is_tase_instrument_false(self):
        """Test Position.is_tase_instrument() with non-TASE exchange."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR,
            exchange="NYSE"
        )
        assert position.is_tase_instrument() is False

    def test_position_is_tase_instrument_no_exchange(self):
        """Test Position.is_tase_instrument() with no exchange."""
        position = Position(
            symbol="AAPL",
            quantity=100.0,
            cost_basis=150.0,
            current_price=155.0,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR
        )
        assert position.is_tase_instrument() is False

    def test_position_is_tase_derivative_true(self):
        """Test Position.is_tase_derivative() with TASE option."""
        position = Position(
            symbol="TA35-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option"
        )
        assert position.is_tase_derivative() is True

    def test_position_is_tase_derivative_false_stock(self):
        """Test Position.is_tase_derivative() with TASE stock (not derivative)."""
        position = Position(
            symbol="TA35",
            quantity=10.0,
            cost_basis=2000.0,
            current_price=2100.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="stock"
        )
        assert position.is_tase_derivative() is False

    def test_position_is_tase_derivative_false_non_tase(self):
        """Test Position.is_tase_derivative() with non-TASE option."""
        position = Position(
            symbol="SPY-C-450",
            quantity=10.0,
            cost_basis=5.0,
            current_price=5.5,
            currency="USD",
            broker="IBKR",
            source=PositionSource.IBKR,
            exchange="NYSE",
            instrument_type="option"
        )
        assert position.is_tase_derivative() is False

    def test_position_get_tase_index_type_ta35(self):
        """Test Position.get_tase_index_type() with TA-35."""
        position = Position(
            symbol="TA35-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="TA-35"
        )
        assert position.get_tase_index_type() == "TA-35"

    def test_position_get_tase_index_type_ta35_alt(self):
        """Test Position.get_tase_index_type() with TA35 (no dash)."""
        position = Position(
            symbol="TA35-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="TA35"
        )
        assert position.get_tase_index_type() == "TA-35"

    def test_position_get_tase_index_type_ta125(self):
        """Test Position.get_tase_index_type() with TA-125."""
        position = Position(
            symbol="TA125-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="TA-125"
        )
        assert position.get_tase_index_type() == "TA-125"

    def test_position_get_tase_index_type_ta90(self):
        """Test Position.get_tase_index_type() with TA-90."""
        position = Position(
            symbol="TA90-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="TA-90"
        )
        assert position.get_tase_index_type() == "TA-90"

    def test_position_get_tase_index_type_banks(self):
        """Test Position.get_tase_index_type() with Banks index."""
        position = Position(
            symbol="BANKS-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="BANKS"
        )
        assert position.get_tase_index_type() == "TA-Banks5"

    def test_position_get_tase_index_type_not_derivative(self):
        """Test Position.get_tase_index_type() with non-derivative."""
        position = Position(
            symbol="TA35",
            quantity=10.0,
            cost_basis=2000.0,
            current_price=2100.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="stock"
        )
        assert position.get_tase_index_type() is None

    def test_position_get_tase_index_type_unknown(self):
        """Test Position.get_tase_index_type() with unknown underlying."""
        position = Position(
            symbol="UNKNOWN-C-2000",
            quantity=10.0,
            cost_basis=50.0,
            current_price=55.0,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="option",
            underlying="UNKNOWN"
        )
        assert position.get_tase_index_type() is None

    def test_position_with_tase_future(self):
        """Test Position with TASE future."""
        position = Position(
            symbol="USDILS-F",
            quantity=1.0,
            cost_basis=3.7,
            current_price=3.75,
            currency="ILS",
            broker="Discount",
            source=PositionSource.ISRAELI_BROKER_EXCEL,
            exchange="TASE",
            instrument_type="future",
            underlying="USD/ILS",
            expiration_date=datetime(2025, 12, 31)
        )
        assert position.is_tase_derivative() is True
        assert position.get_tase_index_type() is None  # Not an index derivative


if __name__ == "__main__":
    unittest.main()
