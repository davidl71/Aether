# Code Audit — 2026-03-12

**Scope:** `native/src/` and `native/include/` (C++ core engine)
**Auditor:** Claude Code automated audit
**Result:** No critical issues. Two medium-priority findings; stubs and one potential unused interface header.

---

## Summary

| Severity | Category | Count |
|----------|----------|-------|
| Medium | Magic numbers in trading/risk logic | 29 instances across 6 files |
| Medium | Unused `#include` | 2 (`option_chain.h`, `<iterator>` in `risk_calculator.cpp`) |
| Low | Clearly-marked stubs | 5 |
| Low | Interface-only headers (no `src/` references) | ~10 `*_agg.h` + `broker_interface.h` |
| ✅ Clean | Dead branches, `#if 0`, commented-out code | 0 |
| ✅ Clean | TODO / FIXME / HACK comments | 0 |
| ✅ Clean | Unreachable code | 0 |

---

## Medium: Magic numbers in trading/risk logic

Hardcoded numeric literals appear in safety-critical code paths. These should be named
constants so intent is documented and updates are centralised.

### `native/src/risk_calculator.cpp`

| Line(s) | Literal | Meaning |
|---------|---------|---------|
| 66, 67, 68, 122, 131 | `100.0` | Options contract multiplier |
| 157 | `0.05` | VaR confidence scaling factor |
| 201 | `0.045` | Default risk-free rate |
| 213 | `252.0` | Trading days per year |
| 219 | `0.05`, `1.5` | Implied-volatility clamp bounds |
| 284, 286, 290 | `0.7`, `0.3`, `0.5` | Pairwise correlation estimates |
| 487, 494 | *(stub returns)* | See §Stubs |

### `native/src/margin_calculator.cpp`

| Line(s) | Literal | Meaning |
|---------|---------|---------|
| 85, 125, 166, 237 | `0.75` | Maintenance-to-initial margin ratio |
| 256, 267, 275, 306 | `100.0` | Percentage scaling |
| 311 | `0.20`, `0.10` | IV default and strike-width default |

### `native/src/greeks_calculator.cpp`

| Line(s) | Literal | Meaning |
|---------|---------|---------|
| 49 | `0.30` | Initial IV Newton-Raphson seed |
| 62, 382 | `1e-10` | Vega / std-dev near-zero threshold |

### Other files

| File | Line(s) | Literal | Meaning |
|------|---------|---------|---------|
| `strategies/box_spread/box_spread_strategy.cpp` | 29 | `0.7` | Execution probability threshold |
| `strategies/box_spread/box_spread_strategy.cpp` | 443, 503 | `100.0` | Percentage |
| `order_manager.cpp` | 68 | `0.05` | Order efficiency threshold |

**Recommendation:** Extract into a shared `native/include/constants.h` (or per-domain
headers: `risk_constants.h`, `margin_constants.h`). Suggested names:

```cpp
inline constexpr double kOptionsContractMultiplier = 100.0;
inline constexpr double kTradingDaysPerYear        = 252.0;
inline constexpr double kDefaultRiskFreeRate       = 0.045;
inline constexpr double kMaintenanceMarginRatio    = 0.75;
inline constexpr double kIvNewtonSeed              = 0.30;
inline constexpr double kVegaEpsilon               = 1e-10;
```

---

## Medium: Unused `#include` in `risk_calculator.cpp`

| Line | Include | Issue |
|------|---------|-------|
| 6 | `#include "option_chain.h"` | No `option_chain` symbols used in this TU |
| 10 | `#include <iterator>` | `std::accumulate` doesn't require it; already pulled in transitively |

Safe to remove both. Low blast radius — will tighten build times slightly.

---

## Low: Clearly-marked stubs

All five stubs are explicitly annotated `// Stub` and represent deferred
implementations, not oversights. Listed for completeness:

| File | Line | Stub return |
|------|------|-------------|
| `order_manager.cpp` | 468 | `return 0.5;` (order fill probability) |
| `risk_calculator_var.cpp` | 74 | `return 0.0;` (VaR) |
| `strategies/box_spread/box_spread_strategy.cpp` | 634 | `return current_positions < 10;` (position limit) |
| `risk_calculator_stats.cpp` | 169 | `return 0;` (stat helper) |
| `risk_calculator.cpp` | 487, 494 | Empty returns |

The VaR stub (`risk_calculator_var.cpp:74`) and the position-limit stub
(`box_spread_strategy.cpp:634`) are the highest priority to implement before
enabling live trading at scale.

---

## Low: Interface-only / aggregator headers

The following headers in `native/include/` have zero references in `native/src/`
(excluding `native/tests/`). They appear to be aggregator includes or broker-adapter
interfaces consumed only by test targets or external consumers:

- `broker_interface.h`
- `agg_types_agg.h`, `aggregator_agg.h`, `circuit_breaker_agg.h` (and similar `*_agg.h` family)

No action required unless a full include-what-you-use pass is planned.

---

## Also audited (clean)

- **Rust agents** (`agents/backend/`): No `todo!()`, `unimplemented!()`, or
  `panic!("not implemented")` in non-test code.
- **CMakeLists.txt / CMakePresets.json**: No stale pybind11, Cython, or
  `PYTHON_BINDINGS` references.
- **config/**: No references to archived Python TUI or Lean.

---

## Prior stale-reference cleanup (same session)

See commit `0a2c71e` — purged all references to deleted `box_spread_pybind.cpp`,
archived `python/tui/` TUI, and removed the non-existent `ENABLE_PYTHON_BINDINGS`
CMake option from docs/AGENTS.md.

Files updated: `AGENTS.md`, `.claude/agents/exploration.md`,
`docs/design/LOGIC_WE_COULD_UNIFY.md`,
`docs/design/ABSTRACTIONS_AND_FRAMEWORKS_BEYOND_PROTOBUF.md`,
`DATA_SOURCE_CONFIG.md`.
