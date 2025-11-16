# Trading Frameworks Index

<!--
@index: trading-frameworks
@category: trading-frameworks
@tags: trading-framework, c++, low-latency, infrastructure, nautilus
@last-updated: 2025-01-27
-->

**Purpose**: Focused index of trading frameworks and infrastructure for algorithmic trading.

**Full Documentation**: See `API_DOCUMENTATION_INDEX.md` for complete details.

---

## Quick Comparison

| Framework | Language | Focus | Best For |
|-----------|----------|-------|----------|
| **FLOX** | C++ | Modular trading framework | Modular, extensible trading systems |
| **SmartQuant C++** | C++ | Ultra-low latency | Institutional HFT systems |
| **Nautilus Trader** | Python | Event-driven trading | Python-based strategies, backtesting |

---

## Decision Tree

### Which Trading Framework?

```
Need C++ framework?
  → Yes → FLOX (modular) or SmartQuant (ultra-low latency)
  → No → Continue...

Need Python framework?
  → Yes → Nautilus Trader
  → No → Continue...

Need ultra-low latency?
  → Yes → SmartQuant C++ (institutional HFT)
  → No → FLOX (modular) or Nautilus (Python)

Need modular/extensible?
  → Yes → FLOX
  → No → SmartQuant (specialized) or Nautilus (Python)
```

---

## Framework Details

### FLOX (Modular Trading Framework)

- **GitHub**: <https://github.com/FLOX-Foundation/flox>
- **Language**: Modern C++ (C++20)
- **License**: MIT License
- **Focus**: Modular, extensible trading framework
- **Key Features**:
  - Modular architecture
  - Extensible components
  - Modern C++ design
- **Best For**: Modular trading systems requiring flexibility
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#flox`

### SmartQuant C++ Ultra-Low Latency Framework

- **Website**: <https://www.smartquant.com/cpp.html>
- **Language**: C++
- **Focus**: Ultra-low latency, institutional HFT
- **Key Features**:
  - Ultra-low latency design
  - Institutional-grade
  - High-frequency trading optimized
- **Best For**: Institutional HFT systems requiring maximum performance
- **Note**: Commercial license required, significant refactoring needed for integration
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#smartquant-cpp`

### Nautilus Trader

- **Documentation**: <https://docs.nautilustrader.io/>
- **Language**: Python (with C++ core)
- **Focus**: Event-driven trading, backtesting
- **Key Features**:
  - Event-driven architecture
  - Comprehensive backtesting
  - Strategy lifecycle management
- **Best For**: Python-based strategies, backtesting, event-driven trading
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#nautilus-trader`
- **Integration**: `../NAUTILUS_LEARNINGS.md`

---

## Use Cases

### Building Modular Trading System
- **Framework**: FLOX
- **Benefits**: Modular architecture, extensible components
- **Best For**: Systems requiring flexibility and modularity

### Ultra-Low Latency Trading
- **Framework**: SmartQuant C++
- **Benefits**: Maximum performance, institutional-grade
- **Best For**: HFT systems, institutional trading

### Python-Based Strategies
- **Framework**: Nautilus Trader
- **Benefits**: Event-driven, comprehensive backtesting
- **Best For**: Python strategies, backtesting, rapid prototyping

---

## Integration Considerations

### FLOX
- **Integration**: Modular components can be integrated incrementally
- **Effort**: Medium (modular design allows gradual integration)
- **Best For**: New systems or major refactoring

### SmartQuant C++
- **Integration**: Significant refactoring required
- **Effort**: High (institutional framework, commercial license)
- **Best For**: New institutional systems from scratch

### Nautilus Trader
- **Integration**: Python bindings, event-driven architecture
- **Effort**: Low-Medium (Python integration, event-driven patterns)
- **Best For**: Python-based strategies, backtesting

---

## See Also

- **Full Documentation**: `../API_DOCUMENTATION_INDEX.md#trading-frameworks-infrastructure`
- **Summary**: `../API_DOCUMENTATION_SUMMARY.md`
- **Nautilus Learnings**: `../NAUTILUS_LEARNINGS.md`
- **SmartQuant Research**: `../SMARTQUANT_CPP_FRAMEWORK_RESEARCH.md`
