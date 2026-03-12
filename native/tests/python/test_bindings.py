"""
test_bindings.py - Tests for the box_spread_bindings Python extension module.
"""
import sys
from pathlib import Path

import pytest


def _add_candidate_binding_paths() -> None:
    root = Path(__file__).resolve().parents[3]
    native_root = root / "native"
    candidates = [
        native_root / "build" / "python" / "bindings",
        native_root / "build" / "lib",
        native_root / "build" / "Debug",
        native_root / "build" / "Release",
        root / "build" / "lib",
        root / "build" / "python" / "bindings",
        root / "build" / "Debug",
        root / "build" / "Release",
    ]

    for candidate in candidates:
        if candidate.exists():
            sys.path.insert(0, str(candidate))


_add_candidate_binding_paths()

try:
    from box_spread_bindings import (
        PyOptionContract,
        PyBoxSpreadLeg,
        PyMarketData,
        OptionType,
        calculate_arbitrage_profit,
        calculate_implied_interest_rate,
        calculate_roi,
        validate_box_spread,
    )
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    pytest.skip("pybind11 box_spread_bindings module not available", allow_module_level=True)


class TestOptionContract:
    """Tests for PyOptionContract."""

    def test_create_option_contract(self):
        """Test creating an option contract."""
        contract = PyOptionContract(
            symbol="SPY",
            expiry="20241220",
            strike=500.0,
            option_type=OptionType.Call,
            exchange="SMART",
        )
        
        assert contract.symbol == "SPY"
        assert contract.expiry == "20241220"
        assert contract.strike == 500.0
        assert contract.type == OptionType.Call
        assert contract.exchange == "SMART"

    def test_is_valid(self):
        """Test contract validation."""
        contract = PyOptionContract(
            symbol="SPY",
            expiry="20241220",
            strike=500.0,
            option_type=OptionType.Call,
        )
        # Note: is_valid() implementation depends on C++ code
        # This test verifies the method exists and can be called
        result = contract.is_valid()
        assert isinstance(result, bool)


class TestBoxSpreadLeg:
    """Tests for PyBoxSpreadLeg."""

    def test_create_box_spread_leg(self):
        """Test creating a box spread leg."""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)

        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)

        assert spread is not None
        assert spread.get_strike_width() == 10.0  # 510 - 500

    def test_set_net_debit(self):
        """Test setting net debit."""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)

        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        spread.net_debit = 2.75
        spread.theoretical_value = 10.0
        spread.arbitrage_profit = 7.25

        assert spread.net_debit == 2.75
        assert spread.theoretical_value == 10.0
        assert spread.arbitrage_profit == 7.25


class TestCalculationFunctions:
    """Tests for calculation functions."""

    def test_calculate_implied_interest_rate(self):
        """Test implied interest rate (annualized %)."""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)

        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        spread.net_debit = 2.75
        spread.theoretical_value = 10.0
        spread.arbitrage_profit = 7.25

        profit = calculate_arbitrage_profit(spread)
        assert profit == 7.25
        rate = calculate_implied_interest_rate(spread)
        assert isinstance(rate, (int, float))

    def test_calculate_roi(self):
        """Test ROI calculation."""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)

        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        spread.net_debit = 2.75
        spread.arbitrage_profit = 7.25

        roi = calculate_roi(spread)
        expected_roi = (7.25 / 2.75) * 100.0
        assert abs(roi - expected_roi) < 0.01  # Allow small floating point differences

    def test_validate_box_spread(self):
        """Test box spread validation."""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)

        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)

        # Validation should return a boolean
        result = validate_box_spread(spread)
        assert isinstance(result, bool)
