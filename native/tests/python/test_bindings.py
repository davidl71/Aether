"""
test_bindings.py - Tests for Cython bindings
"""
import pytest
import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python" / "bindings"))

try:
    from box_spread_bindings import (
        PyOptionContract,
        PyBoxSpreadLeg,
        PyMarketData,
        OptionType,
        calculate_implied_interest_rate,
        calculate_roi,
        validate_box_spread,
    )
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    pytest.skip("Cython bindings not available", allow_module_level=True)


class TestOptionContract:
    """Tests for PyOptionContract"""
    
    def test_create_option_contract(self):
        """Test creating an option contract"""
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
        """Test contract validation"""
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
    """Tests for PyBoxSpreadLeg"""
    
    def test_create_box_spread_leg(self):
        """Test creating a box spread leg"""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)
        
        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        
        assert spread is not None
        assert spread.get_strike_width() == 10.0  # 510 - 500
    
    def test_set_net_debit(self):
        """Test setting net debit"""
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
    """Tests for calculation functions"""
    
    def test_calculate_implied_interest_rate(self):
        """Test implied interest rate basis calculation"""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)
        
        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        spread.net_debit = 2.75
        spread.theoretical_value = 10.0
        spread.arbitrage_profit = 7.25
        
        profit = calculate_implied_interest_rate(spread)
        assert profit == 7.25
    
    def test_calculate_roi(self):
        """Test ROI calculation"""
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
        """Test box spread validation"""
        long_call = PyOptionContract("SPY", "20241220", 500.0, OptionType.Call)
        short_call = PyOptionContract("SPY", "20241220", 510.0, OptionType.Call)
        long_put = PyOptionContract("SPY", "20241220", 510.0, OptionType.Put)
        short_put = PyOptionContract("SPY", "20241220", 500.0, OptionType.Put)
        
        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        
        # Validation should return a boolean
        result = validate_box_spread(spread)
        assert isinstance(result, bool)



