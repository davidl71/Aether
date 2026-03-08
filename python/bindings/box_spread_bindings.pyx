# box_spread_bindings.pyx - Cython implementation exposing C++ to Python
# distutils: language = c++

from libcpp.string cimport string
from libcpp.vector cimport vector
from libcpp.optional cimport optional
from libcpp.memory cimport unique_ptr
cimport numpy as np
import numpy as np

# Import declarations
cimport box_spread_bindings as c_bindings

# Python imports for type conversions
from datetime import datetime
from typing import List, Optional, Dict
from enum import IntEnum


# ============================================================================
# Python Enums matching C++ enums (suffixed to avoid shadowing .pxd C++ enums)
# ============================================================================

class OptionTypePy(IntEnum):
    Call = 0
    Put = 1


class OrderActionPy(IntEnum):
    Buy = 0
    Sell = 1


class OrderStatusPy(IntEnum):
    Pending = 0
    Submitted = 1
    Filled = 2
    PartiallyFilled = 3
    Cancelled = 4
    Rejected = 5
    Error = 6


class TimeInForcePy(IntEnum):
    Day = 0
    GTC = 1
    IOC = 2
    FOK = 3


# Backward compatibility: tests and callers may use OptionType
OptionType = OptionTypePy


# ============================================================================
# Python Wrapper Classes
# ============================================================================

cdef class PyOptionContract:
    """Python wrapper for C++ OptionContract"""
    cdef c_bindings.OptionContract* _ptr
    
    def __cinit__(self, str symbol, str expiry, double strike, int option_type,
                  str exchange="SMART", str local_symbol=""):
        self._ptr = new c_bindings.OptionContract()
        self._ptr.symbol = symbol.encode('utf-8')
        self._ptr.expiry = expiry.encode('utf-8')
        self._ptr.strike = strike
        self._ptr.type = <c_bindings.OptionType>option_type
        self._ptr.exchange = exchange.encode('utf-8')
        self._ptr.local_symbol = local_symbol.encode('utf-8')
    
    def __dealloc__(self):
        if self._ptr:
            del self._ptr
    
    @property
    def symbol(self):
        return self._ptr.symbol.decode('utf-8')
    
    @property
    def expiry(self):
        return self._ptr.expiry.decode('utf-8')
    
    @property
    def strike(self):
        return self._ptr.strike
    
    @property
    def type(self):
        return <int>self._ptr.type
    
    @property
    def exchange(self):
        return self._ptr.exchange.decode('utf-8')
    
    def is_valid(self):
        return self._ptr.is_valid()
    
    def __repr__(self):
        return f"OptionContract(symbol={self.symbol}, expiry={self.expiry}, strike={self.strike}, type={self.type})"


cdef class PyBoxSpreadLeg:
    """Python wrapper for C++ BoxSpreadLeg"""
    cdef c_bindings.BoxSpreadLeg* _ptr
    cdef bint _owns_ptr
    
    def __cinit__(self, PyOptionContract long_call, PyOptionContract short_call,
                  PyOptionContract long_put, PyOptionContract short_put):
        self._ptr = new c_bindings.BoxSpreadLeg()
        self._owns_ptr = True
        # Copy contracts
        self._ptr.long_call = long_call._ptr[0]
        self._ptr.short_call = short_call._ptr[0]
        self._ptr.long_put = long_put._ptr[0]
        self._ptr.short_put = short_put._ptr[0]
    
    def __dealloc__(self):
        if self._ptr and self._owns_ptr:
            del self._ptr
    
    @property
    def net_debit(self):
        return self._ptr.net_debit
    
    @net_debit.setter
    def net_debit(self, double value):
        self._ptr.net_debit = value
    
    @property
    def theoretical_value(self):
        return self._ptr.theoretical_value
    
    @theoretical_value.setter
    def theoretical_value(self, double value):
        self._ptr.theoretical_value = value
    
    @property
    def arbitrage_profit(self):
        return self._ptr.arbitrage_profit
    
    @arbitrage_profit.setter
    def arbitrage_profit(self, double value):
        self._ptr.arbitrage_profit = value
    
    @property
    def roi_percent(self):
        return self._ptr.roi_percent
    
    @roi_percent.setter
    def roi_percent(self, double value):
        self._ptr.roi_percent = value
    
    def get_strike_width(self):
        return self._ptr.get_strike_width()
    
    def get_days_to_expiry(self):
        return self._ptr.get_days_to_expiry()
    
    def is_valid(self):
        return self._ptr.is_valid()
    
    def __repr__(self):
        return f"BoxSpreadLeg(net_debit={self.net_debit}, profit={self.arbitrage_profit}, roi={self.roi_percent}%)"


cdef class PyMarketData:
    """Python wrapper for C++ MarketData"""
    cdef c_bindings.MarketData* _ptr
    
    def __cinit__(self, str symbol):
        self._ptr = new c_bindings.MarketData()
        self._ptr.symbol = symbol.encode('utf-8')
    
    def __dealloc__(self):
        if self._ptr:
            del self._ptr
    
    @property
    def symbol(self):
        return self._ptr.symbol.decode('utf-8')
    
    @property
    def bid(self):
        return self._ptr.bid
    
    @bid.setter
    def bid(self, double value):
        self._ptr.bid = value
    
    @property
    def ask(self):
        return self._ptr.ask
    
    @ask.setter
    def ask(self, double value):
        self._ptr.ask = value
    
    @property
    def last(self):
        return self._ptr.last
    
    @last.setter
    def last(self, double value):
        self._ptr.last = value
    
    def get_mid_price(self):
        return self._ptr.get_mid_price()
    
    def get_spread(self):
        return self._ptr.get_spread()
    
    def get_spread_percent(self):
        return self._ptr.get_spread_percent()


# ============================================================================
# Box Spread Strategy Wrapper
# ============================================================================

cdef class PyBoxSpreadStrategy:
    """Python wrapper for C++ BoxSpreadStrategy"""
    cdef unique_ptr[c_bindings.BoxSpreadStrategy] _strategy
    
    def __cinit__(self, dict strategy_params):
        # Create StrategyParams from dict
        # Note: This is a simplified version - full implementation would
        # require proper config::StrategyParams construction
        # For now, we'll create a minimal wrapper
        # TODO: Full implementation requires passing actual C++ objects
        pass
    
    def evaluate_box_spread(self, PyBoxSpreadLeg spread):
        """Evaluate if a box spread is profitable"""
        if not self._strategy:
            raise RuntimeError("Strategy not initialized")
        return self._strategy.get().is_profitable(spread._ptr[0])
    
    def calculate_arbitrage_profit(self, PyBoxSpreadLeg spread):
        """Compute max profit for a spread (delegates to BoxSpreadCalculator)."""
        if not self._strategy:
            raise RuntimeError("Strategy not initialized")
        return BoxSpreadCalculator.calculate_max_profit(spread._ptr[0])
    
    def calculate_roi(self, PyBoxSpreadLeg spread):
        """Calculate ROI for a box spread"""
        if not self._strategy:
            raise RuntimeError("Strategy not initialized")
        return self._strategy.get().calculate_roi(spread._ptr[0])


# ============================================================================
# Standalone Functions
# ============================================================================

def calculate_arbitrage_profit(PyBoxSpreadLeg spread):
    """Standalone: compute max profit for a spread (BoxSpreadCalculator)."""
    return BoxSpreadCalculator.calculate_max_profit(spread._ptr[0])


def calculate_roi(PyBoxSpreadLeg spread):
    """Standalone function to calculate ROI"""
    if spread.net_debit > 0:
        return (spread.arbitrage_profit / spread.net_debit) * 100.0
    return 0.0


def validate_box_spread(PyBoxSpreadLeg spread):
    """Validate a box spread structure"""
    return spread.is_valid()



