# Logic Abstraction / Deduplication: More Protobuf Opportunities

**Status:** Planning  
**Related:** [CROSS_LANGUAGE_DEDUP_PLAN.md](CROSS_LANGUAGE_DEDUP_PLAN.md), [NATS_PROTOBUF_MIGRATION.md](NATS_PROTOBUF_MIGRATION.md), `proto/messages.proto`

---

## 1. Current State (Post–Cross-Language Dedup)

- **`proto/messages.proto`** defines: `MarketDataEvent`, `CandleSnapshot`, `SymbolSnapshot`, `Position`, `HistoricPosition`, `Order`, `StrategyDecision`, `StrategySignal`, `RiskStatus`, `RiskLimit`, `Alert`, `Metrics`, `SystemSnapshot`, `NatsEnvelope`, `BoxSpreadScenario`, `BoxSpreadExecution`.
- **C++**: `native/generated/`, `proto_adapter` for boundary serialization; canonical logic stays in C++.
- **Python**: `python/generated/` is generated from `proto/messages.proto`. The proto **already defines** `OptionContract`, `BoxSpreadLeg`, `BoxSpreadOpportunity`, `StrategyParams`, `RiskDecision`, `PositionRisk`, `PortfolioRisk`, `DiscountBankBalance`, `DiscountBankTransaction`, `BankAccount`, `YieldCurvePoint`, `YieldCurve`. Remaining work is to ensure Python boundary code uses generated types and to retire mirrors. So Python may still use:
  - **`python/proto_types.py`** — hand-written dataclasses mirroring the proto (duplication).
  - **`python/integration/box_spread_models.py`** — its own `OptionContract`, `BoxSpreadLeg`, etc. (duplication).
- **TypeScript**: **`web/src/types/proto.ts`** — hand-written interfaces mirroring the proto (duplication). ts-proto in `proto/generate.sh` writes to `web/src/proto`; `web/src/generated/proto/` holds TWS API–generated types, not `messages.proto`.
- **Rust**: `nats_adapter` compiles `messages.proto` via prost; uses generated types at NATS/REST boundaries.
- **NATS**: C++ still publishes JSON; migration to protobuf payloads is documented but not done ([NATS_PROTOBUF_MIGRATION.md](NATS_PROTOBUF_MIGRATION.md)).

---

## 2. Protobuf Opportunities

### 2.1 Extend `proto/messages.proto` (single schema)

**Status:** The DTOs below are **already in** `proto/messages.proto`. Remaining work is wiring consumers to generated code.

- **Box spread domain:** `OptionContract`, `OptionTypeEnum`, `BoxSpreadLeg`, `BoxSpreadOpportunity`, `StrategyParams`, `YieldCurvePoint`, `YieldCurve`.
- **Risk reporting:** `RiskDecision`, `PositionRisk`, `PortfolioRisk` (and existing `RiskLimit`/`RiskStatus`).
- **Discount bank:** `DiscountBankBalance`, `DiscountBankTransaction`, `BankAccount`.

**Effect:** Python and Rust can use generated types at boundaries; retire manual mirrors once codegen is wired. See `docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md` for further opportunities and buf config options.

### 2.2 Wire Python codegen and retire mirrors

- Ensure **`proto/generate.sh`** produces Python that matches what `python/generated/__init__.py` expects (either standard `_pb2` or betterproto; align `__init__.py` with actual generator output).
- **Migrate** NATS/REST-facing code from `proto_types.py` and from `box_spread_models.py` (boundary DTOs only) to `python/generated/`. Keep internal Python-only dataclasses only where they add behavior (e.g. `MarketData.get_mid_price()`); at boundaries use generated types and thin conversion.
- **Deprecate** `proto_types.py` once all boundary consumers use generated code.

### 2.3 Wire TypeScript codegen from `messages.proto`

- **Single output dir:** Use one directory for ts-proto output from `proto/messages.proto` (e.g. `web/src/generated/proto/` for platform messages), and keep TWS API–generated types elsewhere if needed.
- **Replace** `web/src/types/proto.ts` with imports from generated code so the web app uses the same contract as C++/Python/Rust.
- **Backlog ref:** T-1772135684260804000 (Wire up protobuf codegen for TypeScript / ts-proto).

### 2.4 NATS: publish protobuf instead of JSON

- Implement [NATS_PROTOBUF_MIGRATION.md](NATS_PROTOBUF_MIGRATION.md): C++ builds protobuf messages, uses `publish_raw()` with `SerializeToString()`; Python/Rust consumers deserialize protobuf.
- Reduces parsing drift and keeps one on-the-wire format.

### 2.5 Remove dead gRPC/proto

- Per [REFACTORING_AND_DEAD_CODE_AUDIT.md](../analysis/REFACTORING_AND_DEAD_CODE_AUDIT.md): remove or archive `agents/backend/crates/api/src/grpc.rs`, `agents/backend/proto/ib_backend.proto`, and `agents/backend-market-data/` proto so the only proto story is `proto/messages.proto` + TWS API vendor protos.

---

## 3. Logic Abstraction (non-protobuf)

- **Stats helpers (mean, stddev, percentile, correlation, beta):** Duplicated in C++ and Python. Options: (1) C++ helper library + Python bindings, or (2) document C++ as single source and remove from Python when strategy/risk call C++. Already called out in CROSS_LANGUAGE_DEDUP_PLAN §4.5.
- **Box spread math:** Canonical in C++; Python should be a thin wrapper (Phase 4 done per plan; verify no remaining duplicate logic in `box_spread_models` for calculations).
- **Risk:** C++ canonical; Python stub or bindings (Phase 5 done per plan).
- **Discount bank:** Rust canonical; Python uses binary + generated types (Phase 6 done); ensure API responses use proto-generated types where consumed by multiple services.

---

## 4. Suggested Order

1. **Proto schema:** `messages.proto` already defines OptionContract, BoxSpreadLeg, BoxSpreadOpportunity, BankAccount, DiscountBankBalance, etc. Remaining work is wiring Python/TS to generated code and retiring mirrors; regenerate all languages after any schema change.
2. **Align Python** generated output and `python/generated/__init__.py`; migrate boundary code from `proto_types.py` and `box_spread_models.py` to generated types; deprecate manual mirrors.
3. **Wire ts-proto** for `messages.proto` into `web/src/generated/proto/` and switch `web/src/types/proto.ts` to generated imports.
4. **NATS:** Switch C++ publish path to protobuf; update Python/Rust subscribers.
5. **Cleanup:** Remove dead gRPC/proto in backend; document single proto story in `docs/message_schemas/README.md`.

---

## 5. References

- `docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md` — Where else to use proto, buf config options (WIRE_JSON, enum_zero_value_suffix), and cross-ref to this doc
- `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` — Phases 1–6 status and file ownership
- `docs/planning/NATS_PROTOBUF_MIGRATION.md` — NATS message mapping and steps
- `docs/analysis/REFACTORING_AND_DEAD_CODE_AUDIT.md` — Dead proto/gRPC, dedup estimates
- `docs/message_schemas/README.md` — Canonical contract (proto + NATS)
- `.cursor/agents/protobuf-justfile.md` — proto-gen / proto-check
