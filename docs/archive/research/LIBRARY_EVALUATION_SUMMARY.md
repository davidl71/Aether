# Rust Library Evaluation Summary

**Last updated:** 2026-03-19
**Platform:** Box spread synthetic financing — IBKR connectivity, ratatui TUI, NATS message bus, 21+ accounts

---

## Evaluation Methodology

Each crate was assessed against five criteria:
1. **API surface match** — does it cover what we actually need?
2. **Box spread support** — native or composable?
3. **Dependency weight** — does it bloat the binary?
4. **Maturity / production evidence** — version, stars, activity, real usage?
5. **Integration fit** — alongside `ib_adapter`, `crates/quant`, `tui_service`?

---

## Broker / TWS API

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| **yatws** (drpngx/yatws) | **Adopt (planned Step 5)** | T-1773940123381922000 | Production-tested (9-figure volume), native `OptionsStrategyBuilder` with box spread, auto conId via `DataRefManager`, rate limiting, session replay via SQLite, ~3ms/order. Replaces `ibapi` in services via `yatws_adapter`. Cloned at `/Users/davidl/Projects/Trading/yatws`. |
| ibapi (current) | Keep (via ib_adapter) | — | Direct TWS socket protocol; `IbApiEngine` implements `broker_engine` traits. Stays until `yatws_adapter` (Step 5) is complete. |

---

## Options Pricing & Quant

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| **optionstratlib** v0.15.2 | **Selective augmentation** | T-1773941258705443000 | 25+ strategies, Plotly visualization, higher-order Greeks (Vanna, Charm, Vomma, Color), 14 exotic options. No native box spread, no parity/APR. Add to dev-deps first for Greeks cross-validation. ~37 deps. |
| **RustQuant** v0.3.1 | **Selective augmentation** | T-1773940296400695000 | Hull-White/Vasicek/CIR short-rate models, bond pricing/duration/convexity, Nelson-Siegel yield curves, Heston/SABR, Peter Jaeckel IV (high precision). No box spread. Use for APR vs term structure validation. Heavy deps (polars — feature-gate). 1.7k stars. |

**Validation chain:**
```
T-1773950798624429000  optionstratlib Greeks cross-validation  [dev-dep, <0.1% divergence threshold]
    └── T-1773951161888509000  RustQuant Greeks + yield curve integration
```

---

## Symbol & Market Infrastructure

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| **financial_symbols** | **Adopt** | T-1773944860145518000 | OSI format parsing (`SPXW231127C03850000` → ticker/type/expiry/strike) at 10.5ns (M1). Zero deps, Copy types, `chrono` + `rust_decimal`. Use for box spread leg parsing, strike extraction, symbol normalization. Follow-up: T-1773952155149900000. |
| **trading-calendar** | **Consider** | T-1773944869350343000 | NYSE/NASDAQ/LSE/TSE/TSX market hours + holidays 2020–2030. `is_open_now()`, `next_open()`, `is_trading_day()`, early-close detection. Useful for TUI market-status display and box spread roll scheduling. Follow-up: T-1773952155198669000. |
| **pricelevel** | **Consider** | T-1773944855328552000 | Lock-free price level for limit order books, ~100k ops/sec, SHA-256 snapshots. Useful if TUI needs live order book depth for re-pricing. Simple bid/ask tracking suffices for current needs. |

---

## Market Data

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| databento/dbn | **Skip** | T-1773940128897305000 | High-performance normalized market data. IBKR native feeds sufficient for box spreads. Adds vendor cost + complexity without clear ROI. |

---

## Order Execution / Simulation

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| matchcore | **Skip** | T-1773940125139676000 | Lock-free order book matching engine. Orders go to IBKR, not a local engine. No utility for our workflow. |

---

## Portfolio / Risk

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| nt-portfolio | **Skip** | T-1773944864497462000 | Zero docs, no accessible GitHub repo. Our `margin_calculator.rs`, `risk_calculator.rs`, `greeks_calculator.rs` are superior for box spread constraints. |

---

## Technical Indicators

| Crate | Verdict | Task | Notes |
|-------|---------|------|-------|
| m4rs | **Skip** | T-1773944873332205000 | 27+ indicators (SMA, EMA, RSI, MACD, VWAP). Box spreads are not trend-following — irrelevant for T-bill proxy strategy. |

---

## TUI Reference Implementations

| Project | Location | Key Patterns | Research Task |
|---------|----------|-------------|---------------|
| **longbridge-terminal** | `/longbridge-terminal/` | Dirty flags (40–60% render reduction), `ScrollableTableState`, input handler separation, Toast notifications, conditional row styling | T-1773941611822304000 |
| **rust-trade** | `/rust-trade/` | `Strategy` trait + `BacktestEngine`, `TieredCache` (L1 RwLock / L2 Redis / L3 Postgres), `PaperTradingProcessor` | T-1773942174043460000–82000 |

### Longbridge Terminal Follow-up Tasks

| Task | Description | Priority |
|------|-------------|----------|
| T-1773952110044212000 | Dirty flags render optimization | medium |
| T-1773952110096607000 | Extract input handlers to `src/input.rs` | low |
| T-1773952110143329000 | `ScrollableTableState` widget | low |
| T-1773952110189533000 | Right-align numerics + conditional row styling + scrollbars | low |
| T-1773952110237059000 | Toast notification pattern | low |

### rust-trade Follow-up Tasks

| Task | Description | Priority |
|------|-------------|----------|
| T-1773952155244511000 | `BoxSpreadStrategy` implementing `Strategy` trait for backtesting | low |
| T-1773952155290442000 | `MultiLegOrderManager` for atomic 4-leg paper trading | low |
| T-1773952155336082000 | Extend `TieredCache` trait for Greeks/IV surface | low |

---

## Quick Reference: Adopt / Consider / Skip

```
ADOPT
  financial_symbols    — OSI symbol parsing, zero deps
  yatws               — IBKR replacement (planned broker_engine Step 5)

CONSIDER
  trading-calendar    — TUI market hours + roll scheduling
  pricelevel          — TUI order book depth (if needed)

DEV-DEP FIRST, THEN SELECTIVE
  optionstratlib      — Greeks visualization + cross-validation
  RustQuant           — Yield curve modeling + Peter Jaeckel IV

SKIP
  databento/dbn       — IBKR feeds sufficient
  matchcore           — no local matching needed
  nt-portfolio        — undocumented
  m4rs                — not applicable to box spread
```
