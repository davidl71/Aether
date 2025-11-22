# Box Spread Strategy

**Status**: Active Strategy Component
**Allocation**: 7-10% of portfolio (spare cash)
**Purpose**: Synthetic financing via options arbitrage

---

## Overview

Box spreads are one strategy component of the **Synthetic Financing Platform**, used for spare cash allocation to achieve T-bill-equivalent yields. This strategy module provides automated box spread identification, analysis, and execution.

### Strategy Context

- **Platform Role**: Spare cash allocation strategy (Tier 2, 7-10% of portfolio)
- **Integration Point**: Cash management module
- **Use Case**: Enhanced yield on spare cash comparable to T-bills or SOFR
- **Target Duration**: 1-3 month box spreads (aligns with T-bill ladder)

---

## Documentation

### Core Guides

- **[Comprehensive Box Spread Trading Guide](BOX_SPREAD_COMPREHENSIVE_GUIDE.md)** - Complete reference for box spread mechanics, risks, and implementation
- **[Box Spread BAG Implementation](BOX_SPREAD_BAG_IMPLEMENTATION.md)** - IBKR BAG (Bundle All-or-Nothing) order implementation details
- **[Data Feeds for Box Spreads](DATA_FEEDS_BOX_SPREADS.md)** - Market data sources and quoting mechanisms
- **[Box Spread Resources Index](BOX_SPREAD_RESOURCES_INDEX.md)** - Comprehensive index of educational resources and tools

### Quick Reference

- **Strategy Type**: Synthetic financing (risk-free borrowing/lending)
- **Position Structure**: 4-leg options strategy (long call + short call + long put + short put)
- **Risk Profile**: Market-neutral (theoretically risk-free)
- **Execution**: IBKR BAG orders for atomic 4-leg execution

---

## Integration with Platform

### Cash Management Integration

Box spreads are integrated into the platform's **Tier 2: Spare Cash (7-10% of portfolio)** allocation:

```text
Cash Management Strategy:
├── Tier 1: Immediate Cash (3-5%)
│   └── Liquidity buffer + loan payment reserve
└── Tier 2: Spare Cash (7-10%)
    └── Box spreads ← This strategy
```

### Opportunity Simulation

Box spreads can be simulated in "what-if" scenarios:
- **Scenario**: "I have a loan at 4% APR. Can I use it as margin for box spreads?"
- **Simulation**: Use loan as collateral, execute box spread, calculate effective financing rate

### Multi-Instrument Relationships

Box spreads can be part of financing chains:
```
Bank Loan (5% APR)
  ↓ (use as collateral)
Box Spread Margin (4% implied rate)
  ↓ (use proceeds)
Investment Fund (6% return)
  ↓ (net benefit: 2% spread)
```

### Risk Calculator Integration

- Uses platform `RiskCalculator` for position sizing
- Validates against portfolio risk limits
- Calculates Value at Risk (VaR) for box spread positions

---

## Implementation Status

### ✅ Completed

- Core box spread calculation engine (`native/src/box_spread_strategy.cpp`)
- Box spread validation and opportunity detection
- Risk-based position sizing integration
- IBKR TWS API integration framework
- Cython bindings for Python integration
- WebAssembly (WASM) module for cross-platform use

### ⏳ In Progress

- Full IBKR BAG order implementation
- Real-time market data integration
- Multi-leg order execution and tracking

### 📋 Planned

- Advanced opportunity filtering
- Historical performance analysis
- Automated execution workflows

---

## Code Structure

### Core Files

- **Strategy Engine**: `native/src/box_spread_strategy.cpp`
- **Strategy Header**: `native/include/box_spread_strategy.h`
- **Python DSL**: `python/dsl/box_spread_dsl.py`
- **Notebooks**: `notebooks/box_spread_analysis.ipynb`

### Integration Points

- **Platform Core**: Cash flow modeler, opportunity simulator
- **Risk Management**: `native/src/risk_calculator.cpp`
- **Order Management**: `native/src/order_manager.cpp`
- **Broker Integration**: `native/src/brokers/tws_adapter.cpp`

---

## Usage Examples

### Python DSL

```python
from box_spread_dsl import BoxSpreadStrategy

# Create box spread opportunity
strategy = BoxSpreadStrategy(
    symbol="SPX",
    lower_strike=5000,
    upper_strike=5050,
    expiration="2025-03-21"
)

# Calculate implied rate
rate = strategy.calculate_implied_rate()
print(f"Implied financing rate: {rate:.2f}% APR")
```

### C++ Usage

```cpp
#include "box_spread_strategy.h"

BoxSpreadStrategy strategy(config);
auto opportunities = strategy.scan_for_opportunities();
for (const auto& opp : opportunities) {
    if (opp.profit > min_threshold) {
        // Execute box spread
        order_manager.place_box_spread(opp);
    }
}
```

---

## Configuration

Box spread strategy configuration in `config/config.json`:

```json
{
  "strategy": {
    "box_spread": {
      "symbols": ["SPY", "QQQ", "IWM"],
      "min_arbitrage_profit": 0.1,
      "min_roi_percent": 0.5,
      "target_rate": "t_bill_rate_plus_50bps",
      "allocation_percentage": 0.07
    }
  }
}
```

---

## See Also

- **[Platform Overview](../../platform/README.md)** - Synthetic Financing Platform architecture
- **[Investment Strategy Framework](../../platform/INVESTMENT_STRATEGY_FRAMEWORK.md)** - Portfolio allocation framework
- **[Cash Flow Forecasting](../../platform/CASH_FLOW_FORECASTING_SYSTEM.md)** - Cash flow modeling system
- **[Primary Goals](../../platform/PRIMARY_GOALS_AND_REQUIREMENTS.md)** - Platform primary objectives

---

**Last Updated**: 2025-01-27
**Maintained By**: Synthetic Financing Platform Team
