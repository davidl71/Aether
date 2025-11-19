# LEAN Strategy Architecture Design

**Date**: 2025-11-18
**Status**: Design Complete
**Purpose**: Architecture design for integrating C++ box spread calculations with LEAN algorithmic trading engine

---

## Overview

This document outlines the architecture for implementing a box spread trading strategy in LEAN that leverages existing C++ calculations while using LEAN's multi-broker capabilities for execution.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    LEAN Engine (C#)                         │
│  - Market data feed (IBKR/Alpaca)                           │
│  - Order execution                                          │
│  - Position management                                      │
│  - Portfolio tracking                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Python API
                       │
┌──────────────────────▼──────────────────────────────────────┐
│              LEAN Algorithm (Python)                        │
│  class BoxSpreadAlgorithm(QCAlgorithm):                      │
│    - Initialize()                                           │
│    - OnData()                                               │
│    - OnOrderEvent()                                         │
│    - OnSecuritiesChanged()                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│ Data         │ │ Strategy  │ │ Order     │
│ Converter    │ │ Logic     │ │ Manager   │
└───────┬──────┘ └────┬──────┘ └────┬──────┘
        │              │              │
        │              │              │
┌───────▼──────────────▼──────────────▼──────┐
│         C++ Integration Layer              │
│  - Cython bindings (box_spread_bindings)   │
│  - Data conversion (LEAN ↔ C++)            │
│  - Type mappings                           │
└───────┬───────────────────────────────────┘
        │
        │ Cython
        │
┌───────▼───────────────────────────────────┐
│            C++ Core (Existing)             │
│  - Box spread scanner                      │
│  - Risk calculator                         │
│  - Opportunity evaluator                  │
│  - Box spread calculator                   │
└────────────────────────────────────────────┘
```

---

## Component Design

### 1. LEAN Algorithm Class

**File**: `python/lean_integration/box_spread_algorithm.py`

```python
from AlgorithmImports import *
from lean_integration.data_converter import DataConverter
from lean_integration.strategy_config import StrategyConfig
from bindings.box_spread_bindings import (
    PyBoxSpreadStrategy,
    PyBoxSpreadLeg,
    PyOptionContract
)

class BoxSpreadAlgorithm(QCAlgorithm):
    """
    LEAN algorithm for box spread arbitrage trading.
    Integrates C++ calculations with LEAN execution.
    """

    def Initialize(self):
        """Initialize algorithm state and subscriptions."""
        # Configuration
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)
        self.SetBenchmark("SPY")

        # Strategy configuration
        self.config = StrategyConfig(self.GetParameter("config_path"))
        self.symbols = self.config.get_symbols()
        self.min_roi = self.config.get_min_roi()
        self.max_position_size = self.config.get_max_position_size()

        # Initialize C++ strategy
        self.cpp_strategy = PyBoxSpreadStrategy(
            min_roi=self.min_roi,
            max_position_size=self.max_position_size
        )

        # Data converter
        self.data_converter = DataConverter()

        # Subscribe to options
        for symbol in self.symbols:
            option = self.AddOption(symbol)
            option.SetFilter(self.OptionFilter)

        # Track positions
        self.active_positions = {}
        self.pending_orders = {}

        # Statistics
        self.opportunities_found = 0
        self.trades_executed = 0
        self.total_profit = 0.0

    def OptionFilter(self, universe):
        """Filter options by expiration and strike."""
        return universe.Strikes(-10, +10).Expiration(0, 60)

    def OnData(self, slice):
        """Process market data and evaluate opportunities."""
        # Get option chains for subscribed symbols
        for symbol in self.symbols:
            option_chain = slice.OptionChains.get(symbol, None)
            if option_chain is None:
                continue

            # Convert LEAN option chain to C++ format
            cpp_chain = self.data_converter.lean_to_cpp_option_chain(
                option_chain, symbol
            )

            # Scan for box spread opportunities using C++
            opportunities = self.cpp_strategy.find_box_spreads(
                symbol, cpp_chain
            )

            # Evaluate opportunities
            for opp in opportunities:
                if self.should_execute(opp):
                    self.execute_box_spread(opp, option_chain)

    def should_execute(self, opportunity):
        """Determine if opportunity should be executed."""
        # Check profitability
        if opportunity.expected_profit < self.min_roi:
            return False

        # Check position limits
        if len(self.active_positions) >= self.max_position_size:
            return False

        # Check risk limits (from C++ risk calculator)
        risk_score = self.cpp_strategy.calculate_risk(opportunity)
        if risk_score > self.config.get_max_risk():
            return False

        return True

    def execute_box_spread(self, opportunity, option_chain):
        """Execute box spread order via LEAN."""
        spread = opportunity.spread

        # Convert C++ contracts to LEAN symbols
        long_call_symbol = self.data_converter.cpp_to_lean_symbol(
            spread.long_call, option_chain
        )
        short_call_symbol = self.data_converter.cpp_to_lean_symbol(
            spread.short_call, option_chain
        )
        long_put_symbol = self.data_converter.cpp_to_lean_symbol(
            spread.long_put, option_chain
        )
        short_put_symbol = self.data_converter.cpp_to_lean_symbol(
            spread.short_put, option_chain
        )

        # Create combo order (4 legs)
        combo_order = self.ComboMarketOrder(
            [long_call_symbol, short_call_symbol, long_put_symbol, short_put_symbol],
            [1, -1, 1, -1]  # Quantities: long, short, long, short
        )

        # Track order
        order_id = combo_order.Id
        self.pending_orders[order_id] = {
            "opportunity": opportunity,
            "spread": spread,
            "timestamp": self.Time
        }

        self.Log(f"Placing box spread order: {order_id}")
        self.trades_executed += 1

    def OnOrderEvent(self, orderEvent):
        """Handle order events (fill, partial fill, cancellation)."""
        order = orderEvent.Order
        order_id = order.Id

        if order_id in self.pending_orders:
            info = self.pending_orders[order_id]

            if orderEvent.Status == OrderStatus.Filled:
                # Order filled - track position
                self.active_positions[order_id] = {
                    "spread": info["spread"],
                    "entry_time": info["timestamp"],
                    "entry_price": orderEvent.FillPrice
                }
                del self.pending_orders[order_id]

                self.Log(f"Box spread filled: {order_id}")

            elif orderEvent.Status == OrderStatus.Canceled:
                # Order cancelled
                del self.pending_orders[order_id]
                self.Log(f"Box spread cancelled: {order_id}")

    def OnSecuritiesChanged(self, changes):
        """Handle security additions/removals."""
        for security in changes.AddedSecurities:
            self.Log(f"Added security: {security.Symbol}")

        for security in changes.RemovedSecurities:
            self.Log(f"Removed security: {security.Symbol}")
```

---

### 2. Data Conversion Layer

**File**: `python/lean_integration/data_converter.py`

```python
from typing import List, Optional
from AlgorithmImports import *
from bindings.box_spread_bindings import (
    PyOptionContract,
    PyOptionChain,
    PyMarketData
)

class DataConverter:
    """Convert between LEAN and C++ data formats."""

    def lean_to_cpp_option_chain(self, lean_chain, symbol: str) -> PyOptionChain:
        """Convert LEAN OptionChain to C++ format."""
        cpp_chain = PyOptionChain(symbol)

        # Group by expiration
        for contract in lean_chain:
            # Convert LEAN contract to C++ contract
            cpp_contract = self.lean_to_cpp_contract(contract)

            # Get market data
            cpp_market_data = self.lean_to_cpp_market_data(contract)

            # Add to chain
            cpp_chain.add_option(
                cpp_contract,
                cpp_market_data
            )

        return cpp_chain

    def lean_to_cpp_contract(self, lean_contract) -> PyOptionContract:
        """Convert LEAN OptionContract to C++ OptionContract."""
        return PyOptionContract(
            symbol=lean_contract.Symbol.Underlying.Value,
            expiry=self.format_expiry(lean_contract.Expiry),
            strike=float(lean_contract.Strike),
            option_type="C" if lean_contract.Right == OptionRight.Call else "P"
        )

    def lean_to_cpp_market_data(self, lean_contract) -> PyMarketData:
        """Convert LEAN market data to C++ format."""
        return PyMarketData(
            bid=float(lean_contract.BidPrice),
            ask=float(lean_contract.AskPrice),
            last=float(lean_contract.LastPrice),
            volume=int(lean_contract.Volume),
            open_interest=int(lean_contract.OpenInterest)
        )

    def cpp_to_lean_symbol(self, cpp_contract: PyOptionContract, option_chain) -> Symbol:
        """Convert C++ contract to LEAN Symbol."""
        # Find matching contract in option chain
        for contract in option_chain:
            if (contract.Symbol.Underlying.Value == cpp_contract.symbol and
                contract.Strike == cpp_contract.strike and
                contract.Right == (OptionRight.Call if cpp_contract.option_type == "C" else OptionRight.Put) and
                self.format_expiry(contract.Expiry) == cpp_contract.expiry):
                return contract.Symbol

        raise ValueError(f"Contract not found: {cpp_contract}")

    def format_expiry(self, expiry_date) -> str:
        """Format expiry date to YYYYMMDD."""
        return expiry_date.strftime("%Y%m%d")
```

---

### 3. Strategy Configuration

**File**: `python/lean_integration/strategy_config.py`

```python
import json
from typing import List

class StrategyConfig:
    """Strategy configuration manager."""

    def __init__(self, config_path: str):
        with open(config_path, 'r') as f:
            self.config = json.load(f)

    def get_symbols(self) -> List[str]:
        """Get list of symbols to trade."""
        return self.config.get("strategy", {}).get("symbols", ["SPY"])

    def get_min_roi(self) -> float:
        """Get minimum ROI threshold."""
        return self.config.get("strategy", {}).get("min_roi_percent", 0.5)

    def get_max_position_size(self) -> int:
        """Get maximum position size."""
        return self.config.get("strategy", {}).get("max_position_size", 5)

    def get_max_risk(self) -> float:
        """Get maximum risk score."""
        return self.config.get("strategy", {}).get("max_risk", 0.1)
```

---

## Data Flow

### 1. Market Data Flow

```
LEAN Market Data (OptionChain)
    ↓
DataConverter.lean_to_cpp_option_chain()
    ↓
C++ OptionChain (PyOptionChain)
    ↓
C++ Box Spread Scanner
    ↓
Box Spread Opportunities
```

### 2. Opportunity Evaluation Flow

```
Box Spread Opportunities
    ↓
C++ Risk Calculator
    ↓
Profitability Check (should_execute)
    ↓
Order Decision
```

### 3. Order Execution Flow

```
Order Decision
    ↓
DataConverter.cpp_to_lean_symbol()
    ↓
LEAN ComboMarketOrder
    ↓
LEAN Broker Adapter (IBKR/Alpaca)
    ↓
Broker Execution
```

### 4. Position Tracking Flow

```
Order Filled Event
    ↓
OnOrderEvent()
    ↓
Position Tracking (active_positions)
    ↓
P&L Calculation
```

---

## Integration Points

### C++ Functions Used

**From `box_spread_bindings.pyx`:**

1. **Box Spread Detection:**
   - `find_box_spreads(symbol, option_chain)` → List of opportunities
   - `evaluate_box_spread(contracts)` → Opportunity evaluation

2. **Risk Calculation:**
   - `calculate_risk(opportunity)` → Risk score
   - `validate_box_spread(spread)` → Validation result

3. **Profitability:**
   - `calculate_arbitrage_profit(spread)` → Profit amount
   - `calculate_roi(spread)` → ROI percentage

4. **Box Spread Calculator:**
   - `calculate_net_debit(spread)` → Net debit
   - `calculate_theoretical_value(spread)` → Theoretical value
   - `calculate_implied_rate(spread)` → Implied interest rate

### LEAN API Used

1. **Algorithm Setup:**
   - `Initialize()` → Algorithm initialization
   - `AddOption(symbol)` → Subscribe to options
   - `SetCash(amount)` → Set starting capital

2. **Market Data:**
   - `OnData(slice)` → Process market data
   - `slice.OptionChains` → Access option chains
   - `OptionChain` → Option chain data structure

3. **Order Execution:**
   - `ComboMarketOrder(symbols, quantities)` → Place combo order
   - `ComboLimitOrder(symbols, quantities, limit_price)` → Place limit combo order
   - `OnOrderEvent(orderEvent)` → Handle order events

4. **Position Management:**
   - `Portfolio[symbol]` → Access position
   - `Securities[symbol]` → Access security data
   - `OnSecuritiesChanged(changes)` → Handle security changes

---

## Error Handling

### Connection Errors

```python
def OnBrokerageMessage(self, message):
    """Handle brokerage messages and errors."""
    if message.Type == BrokerageMessageType.Error:
        self.Log(f"Brokerage error: {message.Message}")
        # Implement reconnection logic if needed
```

### Order Rejection

```python
def OnOrderEvent(self, orderEvent):
    """Handle order events including rejections."""
    if orderEvent.Status == OrderStatus.Invalid:
        self.Log(f"Order rejected: {orderEvent.Message}")
        # Log rejection reason and retry if appropriate
```

### Data Quality Issues

```python
def should_execute(self, opportunity):
    """Validate data quality before execution."""
    # Check bid/ask spreads
    if opportunity.spread.max_bid_ask_spread > threshold:
        return False

    # Check liquidity
    if opportunity.liquidity_score < min_liquidity:
        return False

    return True
```

---

## Performance Considerations

### Data Conversion Optimization

- **Cache conversions**: Store converted data to avoid repeated conversions
- **Batch processing**: Convert multiple contracts at once
- **Lazy evaluation**: Only convert data when needed

### C++ Call Optimization

- **Minimize calls**: Batch C++ function calls when possible
- **Reuse objects**: Reuse C++ objects instead of creating new ones
- **Async processing**: Consider async C++ calls for non-blocking operations

### LEAN Performance

- **Efficient subscriptions**: Only subscribe to needed options
- **Filter options**: Use OptionFilter to limit option universe
- **Position limits**: Enforce position limits to manage memory

---

## Testing Strategy

### Unit Tests

1. **Data Conversion:**
   - Test LEAN → C++ conversion
   - Test C++ → LEAN conversion
   - Test edge cases (missing data, invalid formats)

2. **Strategy Logic:**
   - Test opportunity detection
   - Test profitability checks
   - Test risk calculations

### Integration Tests

1. **End-to-End:**
   - Test complete flow from market data to order execution
   - Test with paper trading
   - Test error handling

2. **Broker Integration:**
   - Test IBKR connection
   - Test Alpaca connection
   - Test order placement

---

## Configuration

### LEAN config.json

```json
{
  "algorithm-type-name": "BoxSpreadAlgorithm",
  "algorithm-language": "Python",
  "algorithm-location": "Main/box_spread_algorithm.py",
  "brokerage": {
    "brokerage-type": "InteractiveBrokers",
    "interactive-brokers": {
      "host": "127.0.0.1",
      "port": 7497,
      "account": "DU123456",
      "trading-mode": "paper"
    }
  }
}
```

### Strategy config.json

```json
{
  "strategy": {
    "symbols": ["SPY", "SPX", "XSP"],
    "min_roi_percent": 0.5,
    "max_position_size": 5,
    "max_risk": 0.1,
    "min_days_to_expiry": 7,
    "max_days_to_expiry": 60
  }
}
```

---

## Next Steps

1. ✅ **Architecture Design Complete** (this document)
2. ⏳ **Implement Data Conversion Layer** (T-42)
3. ⏳ **Implement LEAN Box Spread Strategy** (T-43)
4. ⏳ **Configure IBKR Integration** (T-45)
5. ⏳ **Configure Alpaca Integration** (T-46)
6. ⏳ **End-to-End Testing** (T-47)

---

## References

- [LEAN Algorithm Structure](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/algorithm-structure)
- [LEAN Options Trading](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/options)
- [LEAN Combo Orders](https://www.quantconnect.com/docs/v2/lean-engine/trading-and-orders/order-types/combo-orders)
- [LEAN Python Examples](https://github.com/QuantConnect/Lean/tree/master/Algorithm.Python)

---

## Status

- ✅ Architecture designed
- ✅ Component structure defined
- ✅ Data flow documented
- ✅ Integration points identified
- ✅ Error handling planned
- ✅ Performance considerations addressed
