# LEAN + pybind11 Integration Analysis

**Date**: 2025-11-18
**Status**: Feasibility Analysis
**Purpose**: Evaluate using LEAN with pybind11 to bridge C++ and LEAN

---

## Overview

This document analyzes the feasibility of using **LEAN (QuantConnect LEAN)** with **pybind11** to maintain C++ integration while leveraging LEAN's multi-broker capabilities.

**Key Question**: Can we use pybind11 to bridge our existing C++ code with LEAN's Python bindings?

---

## Architecture Proposal

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    C++ Core (Existing)                      │
│  - Box spread calculations                                  │
│  - Risk calculator                                          │
│  - Option chain scanner                                     │
│  - Order validation                                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ pybind11 (C++ → Python)
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  Python Strategy Layer                      │
│  - Strategy logic                                           │
│  - Data transformation                                      │
│  - Orchestration                                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ LEAN Python API
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                    LEAN Engine (C#)                         │
│  - Multi-broker support (IBKR, Alpaca, etc.)                │
│  - Market data handling                                     │
│  - Order execution                                          │
│  - Position management                                      │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **C++ Calculations** → pybind11 → **Python**
2. **Python Strategy** → LEAN Python API → **LEAN (C#)**
3. **LEAN** → Broker APIs → **Market/Orders**
4. **Market Data** → LEAN → Python → pybind11 → **C++**

---

## Feasibility Analysis

### ✅ **Feasible: Yes**

**Why it works:**

- ✅ LEAN has Python bindings (Python.NET or similar)
- ✅ pybind11 creates Python modules from C++
- ✅ Python can call both pybind11 modules and LEAN
- ✅ Data can flow: C++ ↔ Python ↔ LEAN

**Architecture:**

```
C++ (pybind11) ←→ Python ←→ LEAN (Python bindings)
```

---

## Technical Implementation

### 1. pybind11 Module Creation

**Replace Cython with pybind11:**

```cpp
// native/src/pybind11_bindings.cpp

#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include "box_spread_strategy.h"
#include "risk_calculator.h"
#include "option_chain.h"

namespace py = pybind11;

PYBIND11_MODULE(box_spread_cpp, m) {
    m.doc() = "C++ box spread calculations via pybind11";

    // Expose box spread calculations
    m.def("calculate_box_spread", &calculate_box_spread);
    m.def("calculate_risk", &calculate_risk);
    m.def("scan_option_chain", &scan_option_chain);

    // Expose types
    py::class_<BoxSpreadLeg>(m, "BoxSpreadLeg")
        .def_readwrite("net_debit", &BoxSpreadLeg::net_debit)
        .def_readwrite("arbitrage_profit", &BoxSpreadLeg::arbitrage_profit);
}
```

**Build with pybind11:**

```cmake
find_package(pybind11 REQUIRED)
pybind11_add_module(box_spread_cpp src/pybind11_bindings.cpp)
```

### 2. Python Strategy Layer

**Bridge between C++ and LEAN:**

```python

# python/strategy/lean_box_spread_strategy.py

import box_spread_cpp  # pybind11 module
from QuantConnect import *

class LeanBoxSpreadStrategy(QCAlgorithm):
    def Initialize(self):
        # LEAN initialization
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)

        # Subscribe to options
        option = self.AddOption("SPY")
        option.SetFilter(self.OptionFilter)

        # Initialize C++ calculator
        self.cpp_calculator = box_spread_cpp.BoxSpreadCalculator()

    def OnData(self, data):
        # Get market data from LEAN
        option_chain = data.OptionChains.get("SPY", None)
        if option_chain is None:
            return

        # Convert LEAN data to C++ format
        cpp_chain = self._convert_to_cpp_format(option_chain)

        # Use C++ for calculations
        opportunities = self.cpp_calculator.scan_option_chain(cpp_chain)

        # Evaluate opportunities
        for opp in opportunities:
            if self.cpp_calculator.calculate_risk(opp) < self.max_risk:
                # Use LEAN to place orders
                self._place_box_spread_order(opp)

    def _place_box_spread_order(self, opportunity):
        # Use LEAN's order API
        combo_order = self.ComboMarketOrder(
            [opportunity.long_call, opportunity.short_call,
             opportunity.long_put, opportunity.short_put],
            [1, -1, 1, -1]  # Quantities
        )
```

### 3. Data Conversion Layer

**Convert between LEAN and C++ formats:**

```python
def _convert_to_cpp_format(self, lean_chain):
    """Convert LEAN OptionChain to C++ format"""
    cpp_chain = []
    for contract in lean_chain:
        cpp_contract = box_spread_cpp.OptionContract(
            symbol=contract.Symbol.Value,
            expiry=contract.Expiry.strftime("%Y%m%d"),
            strike=float(contract.Strike),
            option_type="C" if contract.Right == OptionRight.Call else "P",
            bid=float(contract.BidPrice),
            ask=float(contract.AskPrice)
        )
        cpp_chain.append(cpp_contract)
    return cpp_chain
```

---

## Comparison: pybind11 vs Cython

### Current Approach: Cython

**Pros:**

- ✅ Already implemented
- ✅ Good performance
- ✅ Works with existing code
- ✅ Mature and stable

**Cons:**

- ⚠️ Requires .pyx files (separate syntax)
- ⚠️ Compilation can be complex
- ⚠️ Less Pythonic API

### Proposed Approach: pybind11

**Pros:**

- ✅ Pure C++ (no .pyx files)
- ✅ More Pythonic API
- ✅ Better error messages
- ✅ Easier to maintain
- ✅ Header-only library
- ✅ Better type conversion

**Cons:**

- ⚠️ Migration effort (rewrite bindings)
- ⚠️ Different build system
- ⚠️ New dependency

**Verdict**: pybind11 is a good alternative to Cython, but migration may not be necessary if Cython works well.

---

## LEAN + pybind11 Integration

### Architecture Benefits

1. **Keep C++ Core**: Maintain existing C++ calculations
2. **Use LEAN Brokers**: Leverage LEAN's multi-broker support
3. **Python Bridge**: pybind11 connects C++ to Python, Python connects to LEAN
4. **Best of Both**: C++ performance + LEAN broker support

### Integration Points

**1. Market Data Flow:**

```
LEAN → Python → pybind11 → C++ (calculations)
```

**2. Order Execution Flow:**

```
C++ (opportunity) → Python → LEAN → Broker
```

**3. Risk Management:**

```
C++ (risk calc) → Python → LEAN (position limits)
```

---

## Implementation Plan

### Phase 1: pybind11 Migration (Optional)

**If migrating from Cython to pybind11:**

1. **Create pybind11 Module** (1 week)
   - Replace Cython bindings with pybind11
   - Expose same C++ functions
   - Test with existing Python code

2. **Update Build System** (3 days)
   - Add pybind11 to CMake
   - Update Python setup.py
   - Test compilation

3. **Migration Testing** (1 week)
   - Test all C++ functions
   - Verify performance
   - Compare with Cython

**Total**: 2-3 weeks (optional, only if migrating from Cython)

### Phase 2: LEAN Integration

1. **Set Up LEAN** (1 week)
   - Install LEAN locally
   - Set up Python environment
   - Test basic LEAN functionality

2. **Create Bridge Layer** (2 weeks)
   - Data conversion functions
   - Strategy wrapper
   - Order management bridge

3. **Integrate C++ Calculations** (1 week)
   - Connect pybind11 module to LEAN
   - Test data flow
   - Verify calculations

4. **Broker Integration** (2 weeks)
   - Configure IBKR in LEAN
   - Configure Alpaca in LEAN
   - Test order placement

5. **Testing & Validation** (1 week)
   - Paper trading tests
   - Box spread execution
   - Performance validation

**Total**: 7-8 weeks

---

## Performance Considerations

### Latency Analysis

**Current (NautilusTrader + Cython):**

- C++ calculation: < 1ms
- Cython call overhead: ~0.1ms
- Python → NautilusTrader: ~0.5ms
- **Total**: ~1.6ms

**Proposed (LEAN + pybind11):**

- C++ calculation: < 1ms
- pybind11 call overhead: ~0.1ms (similar to Cython)
- Python → LEAN: ~1-2ms (C# interop)
- LEAN → Broker: ~5-10ms
- **Total**: ~7-13ms

**Verdict**: LEAN adds latency due to C# interop, but still acceptable for box spread trading.

### Throughput

- **C++ calculations**: Same (no change)
- **Python layer**: Minimal overhead
- **LEAN**: Good throughput (C# performance)
- **Overall**: Acceptable for box spread strategies

---

## Migration Effort Comparison

### Option A: Enhance NautilusTrader (Current Recommendation)

**Effort**: 2-4 weeks

- Add broker adapters
- Integrate with unified interface
- Test and validate

**Pros:**

- ✅ Already integrated
- ✅ Best performance
- ✅ Lowest effort

### Option B: Migrate to LEAN + pybind11

**Effort**: 7-10 weeks

- Migrate Cython → pybind11 (optional, 2-3 weeks)
- Integrate LEAN (5-7 weeks)
- Test and validate

**Pros:**

- ✅ Excellent multi-broker support
- ✅ Large community
- ✅ Keep C++ code
- ✅ Better documentation

**Cons:**

- ❌ Higher migration effort
- ❌ C# runtime dependency
- ❌ More latency (C# interop)
- ❌ Lose existing NautilusTrader integration

### Option C: LEAN + Keep Cython

**Effort**: 5-7 weeks

- Keep existing Cython bindings
- Integrate LEAN
- Bridge Cython → LEAN

**Pros:**

- ✅ No pybind11 migration needed
- ✅ Keep existing bindings
- ✅ Faster integration

**Cons:**

- ⚠️ Still need LEAN integration
- ⚠️ C# runtime dependency

---

## Recommendation

### Primary: Enhance NautilusTrader (Keep Current Approach)

**Rationale:**

- Already integrated and working
- Best performance (Rust core)
- Lowest migration effort (2-4 weeks)
- No C# dependency

### Alternative: LEAN + pybind11 (If Migration Acceptable)

**If you choose LEAN:**

1. **Keep Cython** (don't migrate to pybind11 unless needed)
   - Cython works fine
   - Migration adds 2-3 weeks
   - No significant benefit

2. **Use LEAN with Cython** (not pybind11)
   - Cython → Python → LEAN
   - Simpler architecture
   - Faster integration

3. **Consider pybind11 only if:**
   - You want more Pythonic API
   - You're starting fresh
   - You prefer header-only libraries

**Architecture:**

```
C++ (Cython) → Python → LEAN (Python bindings)
```

**Not:**

```
C++ (pybind11) → Python → LEAN (Python bindings)
```

(Unless you specifically want to migrate from Cython)

---

## pybind11 vs Cython for LEAN Integration

### Both Work Equally Well

**Cython (Current):**

- ✅ Already implemented
- ✅ Works with LEAN
- ✅ Good performance
- ✅ No migration needed

**pybind11 (Alternative):**

- ✅ More Pythonic
- ✅ Pure C++ (no .pyx)
- ✅ Better error messages
- ⚠️ Requires migration

**Verdict**: Use Cython (already working) unless you have specific reasons to migrate to pybind11.

---

## Final Recommendation

### For LEAN Integration

**Use: Cython + LEAN** (not pybind11 + LEAN)

**Why:**

1. Cython already works
2. No migration needed
3. Same performance
4. Faster integration

**Architecture:**

```
C++ Core → Cython → Python → LEAN → Brokers
```

### For pybind11 Migration

**Only migrate if:**

- You want more Pythonic API
- You're refactoring bindings anyway
- You prefer header-only libraries
- You want better error messages

**Otherwise**: Keep Cython, it works fine.

---

## Conclusion

**LEAN + pybind11 is feasible**, but:

1. **pybind11 migration is optional** - Cython works fine
2. **LEAN integration is the main effort** - 5-7 weeks
3. **NautilusTrader is still better** - Already integrated, better performance

**If choosing LEAN:**

- Use **Cython + LEAN** (not pybind11 + LEAN)
- Architecture: `C++ (Cython) → Python → LEAN`
- Effort: 5-7 weeks
- Benefit: Excellent multi-broker support

**If staying with NautilusTrader:**

- Keep **Cython** (no need for pybind11)
- Architecture: `C++ (Cython) → Python → NautilusTrader`
- Effort: 2-4 weeks (add adapters)
- Benefit: Best performance, already integrated

---

## References

- [LEAN GitHub](https://github.com/QuantConnect/Lean)
- [LEAN Python Documentation](https://www.lean.io/docs)
- [pybind11 Documentation](https://pybind11.readthedocs.io/)
- [Cython Documentation](https://cython.readthedocs.io/)
- [QuantConnect Forum - pybind11 Discussion](https://www.quantconnect.com/forum/discussion/18551/using-pybind11/)
