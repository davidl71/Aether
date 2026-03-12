# C++ to Rust Migration Plan

**Date**: 2026-03-12
**Status**: Active planning document

## Context

The platform now has two active languages: **C++** (native engine) and **Rust** (backend, TUI).
Python and Go have been removed. This document defines what stays in C++, what can migrate to
Rust, and in what order.

The canonical datapath is:

```
C++ (TWS events) → NATS → Rust (collection, state, APIs) → Rust TUI
```

---

## What MUST Stay in C++

These cannot be migrated — C++ is the only viable language here.

| Component | Reason |
|-----------|--------|
| `tws_client.cpp` + all `tws_*.cpp` | IBKR TWS API is C++-only; no Rust FFI binding exists |
| `nats_client.cpp` | C++→NATS bridge; could theoretically move but tightly coupled to TWS |
| `greeks_calculator.cpp` | Uses QuantLib; no production-quality Rust options pricing library |
| `convexity_calculator.cpp` | Uses QuantLib bond math |
| `financing_optimizer.cpp` | Uses NLopt (C optimization library); Rust alternatives (argmin) are immature for this use case |
| `ib_box_spread.cpp` | CLI entry point; orchestrates TWS + strategy + NATS |

**Rule**: C++ stays as a thin event producer. It publishes `MarketDataEvent`, `StrategySignal`,
`StrategyDecision` over NATS in `NatsEnvelope` format. Rust owns everything downstream.

---

## What CAN Migrate to Rust

These are pure business logic with no C++-only library dependency.

### Phase 1 — Pure Math (high value, low risk)

| Component | C++ Files | Notes |
|-----------|-----------|-------|
| Risk calculator | `risk_calculator.cpp`, `risk_calculator_sizing.cpp`, `risk_calculator_stats.cpp`, `risk_calculator_var.cpp` | `agents/backend/crates/risk` already exists — extend it |
| Margin calculator | `margin_calculator.cpp` | Pure arithmetic, no external deps |

The `risk` crate in `agents/backend/crates/risk/` is the natural home. Extend it rather than
creating new crates.

### Phase 2 — Business Logic (medium effort)

| Component | C++ Files | Notes |
|-----------|-----------|-------|
| Loan manager | `loan_manager.cpp`, `loan_position.cpp` | Rust ledger crate already owns storage |
| Collateral valuator | `collateral_valuator.cpp` | Depends on risk calc (Phase 1 first) |
| Hedge manager | `hedge_manager.cpp` | Depends on risk calc and order manager |
| Order manager | `order_manager.cpp` | Needs careful review — touches TWS order submission |

### Phase 3 — Config (low priority, low risk)

| Component | C++ Files | Notes |
|-----------|-----------|-------|
| Config manager | `config_manager.cpp` | Rust already owns TOML config in `agents/backend`; C++ config is for the CLI only — keep until CLI is replaced or wrapped |

---

## What to Delete (no migration needed)

Already done or clearly dead:

| Item | Status |
|------|--------|
| `box_spread_pybind.cpp` | Deleted ✅ |
| `agents/go/` | Deleted ✅ |
| `python/` | Deleted ✅ |
| Python bindings CMake block | Removed from CMakeLists.txt ✅ |

---

## Migration Approach

**Do not rewrite just to rewrite.** Migrate a module only when:

1. The C++ version has a known correctness issue, or
2. The Rust backend needs the logic natively (to serve API endpoints without round-tripping
   through C++ via NATS), or
3. A Rust crate for the domain already exists and extending it is low effort.

**Pattern for each migration:**

1. Port C++ logic to Rust with identical test coverage
2. Add Catch2 test to C++ verifying outputs match Rust (cross-language parity test)
3. Wire Rust version into the relevant API endpoint or crate
4. Delete C++ version and the parity test together

---

## Constraints

- **QuantLib**: No Rust equivalent. Greeks and convexity math stay in C++ until a credible
  Rust options pricing library matures (none as of 2026-03).
- **NLopt**: `argmin` crate is viable for simple optimization; evaluate when migrating
  `financing_optimizer`.
- **Intel Decimal Library**: Rust `decimal` crates (rust_decimal, bigdecimal) handle most
  use cases. Evaluate when migrating order-level pricing.
- **TWS API**: C++ forever unless IBKR publishes a REST/WebSocket API that replaces the
  native socket protocol (no indication this is planned).

---

## Recommended Next Step

**Extend `agents/backend/crates/risk`** with the risk calculator logic from
`native/src/risk_calculator*.cpp`. This delivers:
- Native risk calculations in the Rust API layer (no NATS round-trip)
- Verifiable parity with C++ via cross-language tests
- Foundation for Phase 2 (collateral, hedge manager)
