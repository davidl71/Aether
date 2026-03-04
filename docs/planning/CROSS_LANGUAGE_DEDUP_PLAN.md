# Cross-Language Deduplication Plan: Protobuf + Strategy

**Status:** Implemented (all phases complete)  
**Last updated:** 2026-02-27  
**Related:** Duplication analysis (C++ / Python / Rust), `proto/messages.proto`, `docs/message_schemas/README.md`

**Exarp (Todo2) tasks:** All completed. Parent **T-1772142847778941000** (Cross-language dedup: Protobuf + single-source strategy). Phase 1: **T-1772142854494578000**. Phase 2: **T-1772142857852296000**. Phase 3: **T-1772142915869231000**. Phase 4: **T-1772142856323561000**. Phase 5: **T-1772142918323013000**. Phase 6: **T-1772142920963275000**. All marked **Done** in exarp-go Todo2 as of 2026-02-27.

---

## 1. Summary

This plan reduces duplication across C++, Python, and Rust by:

1. **Using Protobuf where relevant** — Extend `proto/messages.proto` for shared **data shapes** (box spread domain types, risk payloads, discount bank responses). Generate C++, Python, and Rust from the same `.proto` files.
2. **Single source of truth for calculation logic** — Keep **C++ as the canonical implementation** for box spread math and risk calculations; Python and Rust consume via bindings or RPC, not re-implementations.
3. **Strategy for the rest** — Discount bank: Rust parser as canonical, Python calls binary or uses generated types only. Order/strategy orchestration: keep language-specific entry points; share only serialisable DTOs via protobuf.

### Implementation status (2026-02-27)

All six phases are implemented. Exarp-go Todo2 tasks for this plan are marked **Done**. Key deliverables: extended `proto/messages.proto` and C++ codegen in `native/generated/`; Python `proto_types.py` and boundaries use generated types; C++ `proto_adapter` in `native/`; box spread and risk Python modules documented as thin wrapper / stub; discount bank Rust binary invoked with `--json`, Python fallback deprecated. See also the full implementation plan in `.cursor/plans/` (Cross-Language Dedup Full Implementation).

---

## 2. Current Duplication (Recap)

| Area | C++ | Python | Rust | Duplication |
|------|-----|--------|------|-------------|
| Box spread calculator (14 methods) | Full | Full port | None | HIGH |
| Box spread / risk data models | types.h, risk_calculator.h | box_spread_models, risk_calculator | Minimal | HIGH |
| Risk calculator (VaR, Greeks, limits) | Full | Full port | Trait only | HIGH |
| Discount bank parser | — | Fallback parser | Full | MODERATE |
| Order / strategy DTOs | types.h | order_manager, proto_types | state.rs, strategy model | MODERATE |

**Existing canonical schema:** `proto/messages.proto` (package `ib.platform.v1`) already defines `Position`, `Order`, `StrategyDecision`, `StrategySignal`, `RiskStatus`, `RiskLimit`, `SystemSnapshot`, `BoxSpreadScenario`, `BoxSpreadExecution`. Rust uses it via prost in `nats_adapter`; Python has a manual mirror in `python/proto_types.py`; C++ does not yet use it for these types.

---

## 3. Protobuf Where Relevant

### 3.1 Extend `proto/messages.proto`

Add messages that represent **cross-boundary DTOs** only. Do not encode business logic in protobuf.

**Box spread domain (for NATS, REST, and future C++/Python/Rust sharing):**

- `OptionContract` — symbol, expiry, strike, option_type (call/put).
- `BoxSpreadLeg` — long_call, short_call, long_put, short_put (as OptionContract), plus net_debit, theoretical_value, arbitrage_profit, roi_percent, buy_net_debit, sell_net_credit, buy_sell_disparity, put_call_parity_violation, and leg bid/ask spreads.
- `BoxSpreadOpportunity` — spread (BoxSpreadLeg), confidence_score, expected_profit, risk_adjusted_return, liquidity_score, execution_probability, discovered_time.
- `StrategyParams` — min_days_to_expiry, max_days_to_expiry, min_arbitrage_profit, min_roi_percent, max_positions, max_total_exposure, max_bid_ask_spread.
- `YieldCurvePoint` / `YieldCurve` — symbol, strike_width, benchmark_rate, points (repeated YieldCurvePoint with days_to_expiry, implied_rate, effective_rate, net_debit, spread_bps).

**Risk (for API and engine contracts):**

- Extend or keep `RiskLimit` / `RiskStatus`; add `RiskDecision` (allowed, reason) if not already present.
- Optional: `PositionRisk`, `PortfolioRisk` for reporting (fields matching current C++/Python structs).

**Discount bank (for API responses only):**

- `DiscountBankBalance` — account, balance, currency, balance_date, credit_rate, debit_rate, branch_number, section_number, account_number.
- `DiscountBankTransaction` — value_date, amount, is_debit, reference.
- `BankAccount` — account_path, account_name, bank_name, account_number, balance, currency, balance_date, credit_rate, debit_rate.

**Codegen:**

- **Rust:** Already compiles `proto/messages.proto` in `nats_adapter/build.rs`. Add the new messages to the same file; no new crate required for DTOs consumed by api/state.
- **Python:** Run `./proto/generate.sh` (or `just proto-gen` when available) to generate into `python/generated/`; migrate `python/proto_types.py` and `python/integration/box_spread_models.py` to use generated types where they cross process/network boundaries (NATS, REST). Keep internal Python-only dataclasses for in-process use if needed.
- **C++:** Add `protoc --cpp_out=native/generated/` for `messages.proto`; introduce a thin adapter layer in `native/` that maps `types::BoxSpreadLeg` ↔ generated `ib::platform::v1::BoxSpreadLeg` for serialisation (e.g. NATS, logging, optional REST). Do not replace all C++ types with protobuf; use only at boundaries.

### 3.2 What Not to Put in Protobuf

- **Pure calculation functions** — e.g. `calculate_implied_interest_rate`, `calculate_roi`, `validate_structure`. These stay in C++ (or one chosen implementation) and are invoked via bindings or RPC, not re-implemented in each language.
- **Internal state machines** — e.g. strategy runner lifecycle, option chain cache. Keep them in their respective language.
- **TWS-specific types** — Continue using native/third_party TWS proto or existing C++ types; only platform-level DTOs go into `proto/messages.proto`.

---

## 4. Strategy for the Rest

### 4.1 Box Spread Calculator & Validator

- **Canonical implementation:** C++ (`BoxSpreadCalculator`, `BoxSpreadValidator`, `BoxSpreadStrategy` in `native/src/strategies/box_spread/`).
- **Python:** Prefer **Cython/pybind11 bindings** to the C++ library (project already has `python/bindings/box_spread_bindings.pxd` and C++ build). Call C++ for all 14 calculator/validator methods and confidence scoring; remove or thin the Python port in `python/integration/box_spread_strategy.py` to a thin wrapper that builds inputs from Python data, calls C++, and maps outputs to Python/proto types.
- **Rust:** No duplication today; Rust backend continues to delegate strategy logic to Python (which in turn will call C++). No new Rust implementation of box spread math.
- **Data at boundaries:** Use protobuf `BoxSpreadLeg`, `BoxSpreadOpportunity`, `YieldCurve`, etc., for NATS, REST, and any C++ ↔ Python ↔ Rust exchange.

### 4.2 Risk Calculator

- **Canonical implementation:** C++ (`native/src/risk_calculator.cpp`) for full portfolio risk, VaR, Greeks, position sizing, and limit checks.
- **Python:** Either (a) add C++ bindings and call C++ for all risk calculations, or (b) keep a simplified Python risk module that only does the subset used by the strategy runner (e.g. `can_place_order` with max_position_size / max_order_value) and document it as a “stub” that must match C++ behaviour when both are used. Prefer (a) long-term to avoid drift.
- **Rust:** Keep the current minimal `risk` crate (trait `RiskCheck`, `RiskEngine`, `RiskDecision`). Use protobuf `RiskLimit` / `RiskDecision` for API and NATS. No full re-implementation of VaR/Greeks in Rust.

### 4.3 Discount Bank Parser

- **Canonical implementation:** Rust (`agents/backend/crates/discount_bank_parser`). It is the most complete (encoding, decimals, async).
- **Python:** Keep **Rust binary as primary path** (existing `parse_file_via_rust` in `python/integration/discount_bank_parser.py`). Treat the Python fallback parser as deprecated; eventually remove it once the Rust binary is the only supported path, or keep it only for environments where the binary is not available. API response types (e.g. balance, transactions) should use protobuf-generated types where the same responses are consumed by multiple services.

### 4.4 Order Management & Strategy Orchestration

- **C++:** Remains source of truth for TWS order submission and execution (order_manager, tws_orders).
- **Python:** Keeps order_factory, execution_handler, and strategy_runner; these orchestrate and call C++ or broker APIs. Use protobuf for any order/position DTOs sent over NATS or REST (already partially covered by `Order`, `Position` in messages.proto).
- **Rust:** No order execution; only state and API models. Use generated protobuf types in `api` and `state` for consistency with NATS and REST.

### 4.5 Statistical / Helper Functions

- **Mean, stddev, percentile, correlation, beta:** Duplicated in C++ and Python. Options: (1) Move to a small C++ helper library and call from Python via bindings, or (2) Keep one implementation (e.g. C++) and remove from Python if Python risk/strategy code is refactored to call C++. Document the single source in this plan.

---

## 5. Implementation Phases

All phases have been implemented. Summary:

| Phase | Scope | Outcome | Status |
|-------|--------|--------|--------|
| **Phase 1** | Extend `proto/messages.proto` with box spread, risk, and discount bank DTOs; run codegen for Rust, Python, C++ | Single schema for cross-language DTOs | Done |
| **Phase 2** | Python: Migrate NATS/REST-facing code to use generated types from `python/generated/`; keep or thin `proto_types.py` and box_spread_models where they cross boundaries | Python consumes proto-generated types at boundaries | Done |
| **Phase 3** | C++: Add native/generated from protobuf; thin adapter BoxSpreadLeg ↔ proto for NATS/serialisation | C++ can emit/consume proto at boundaries | Done |
| **Phase 4** | Python box spread: Prefer C++ bindings for calculator/validator; reduce Python port to a thin wrapper | Single implementation of box spread math | Done |
| **Phase 5** | Python risk: Add C++ bindings for risk calculator or formally restrict Python to a documented subset | Single or clearly subordinate risk implementation | Done (documented stub) |
| **Phase 6** | Discount bank: Deprecate or remove Python fallback parser; document Rust binary as canonical | Single parser implementation | Done |

---

## 6. File and Ownership Summary

| Asset | Owner / Action |
|-------|----------------|
| `proto/messages.proto` | Extend with new messages (Phase 1) |
| `proto/generate.sh` | Ensure Python/C++/TS codegen include new messages |
| `agents/backend/crates/nats_adapter/build.rs` | Already compiles messages.proto; no change unless new proto path |
| `python/generated/` | Generated Python from protobuf |
| `python/proto_types.py` | Migrate to generated or mark as legacy (Phase 2) |
| `python/integration/box_spread_models.py` | Use generated types for boundary DTOs; optional thin wrapper (Phase 2, 4) |
| `python/integration/box_spread_strategy.py` | Thin wrapper calling C++ (Phase 4) |
| `python/integration/risk_calculator.py` | Call C++ or document as stub (Phase 5) |
| `python/integration/discount_bank_parser.py` | Prefer Rust binary; deprecate fallback (Phase 6) |
| `native/generated/` | C++ protobuf codegen output (Phase 1, 3) |
| `native/include/types.h` | Keep for internal C++; add conversion to/from proto only at boundaries |
| `native/src/strategies/box_spread/` | Canonical box spread logic (unchanged as source of truth) |
| `native/src/risk_calculator.cpp` | Canonical risk logic (unchanged as source of truth) |
| Rust `api` / `state` | Use prost-generated types where applicable (Phase 1, 2) |

---

## 7. Success Criteria

- One canonical schema for box spread, risk, and discount bank DTOs in `proto/messages.proto`.
- Rust, Python, and (where applicable) C++ consume these DTOs from generated code at process/network boundaries.
- Box spread calculation and validation have a single implementation (C++), used by Python via bindings.
- Risk calculations have a single full implementation (C++), with Python either calling C++ or using a documented subset.
- Discount bank parsing has a single canonical implementation (Rust); Python uses the Rust binary or generated types only.
- Planning document and exarp tasks are created and linked for tracking.

---

## 8. References

- `docs/message_schemas/README.md` — NATS message schemas and proto as canonical contract  
- `docs/NATS_DLQ_IMPLEMENTATION.md` — References proto/messages.proto  
- `docs/analysis/REFACTORING_AND_DEAD_CODE_AUDIT.md` — Dead proto files (ib_backend, market_data)  
- `.cursor/agents/protobuf-justfile.md` — Justfile recipes for proto-gen / proto-check  
- Cross-language duplication analysis (prior conversation)
