# Protobuf Dedup: Concrete Plan and Exarp Tasks

**Status:** Ready for execution  
**Parent context:** [PROTOBUF_DEDUP_OPPORTUNITIES.md](PROTOBUF_DEDUP_OPPORTUNITIES.md), [CROSS_LANGUAGE_DEDUP_PLAN.md](CROSS_LANGUAGE_DEDUP_PLAN.md)

---

## Phases Overview

| Phase | Name | Depends on | Deliverable |
|-------|------|------------|-------------|
| **P1** | Extend messages.proto with missing DTOs | — | Single schema for box spread, risk, discount bank |
| **P2** | Python codegen alignment and boundary migration | P1 | python/generated used at NATS/REST; proto_types deprecated |
| **P3** | TypeScript codegen from messages.proto | P1 | web uses generated proto; types/proto.ts removed |
| **P4** | NATS publish protobuf (C++ → Python/Rust) | P1 | C++ publish_raw; consumers deserialize proto |
| **P5** | Remove dead gRPC/proto and document | — | Single proto story documented |

---

## Phase 1: Extend proto/messages.proto

**Goal:** Add all DTOs referenced by `python/generated/__init__.py` and the cross-language plan so one schema serves C++, Python, Rust, TypeScript.

**Tasks:**

1. **Add box spread domain messages** to `proto/messages.proto`:
   - `OptionContract` (symbol, expiry, strike, option_type)
   - `OptionTypeEnum` (CALL, PUT)
   - `BoxSpreadLeg` (long_call, short_call, long_put, short_put as OptionContract; net_debit, theoretical_value, arbitrage_profit, roi_percent, bid/ask spreads, etc.)
   - `BoxSpreadOpportunity` (spread, confidence_score, expected_profit, risk_adjusted_return, liquidity_score, execution_probability, discovered_time)
   - `StrategyParams` (min/max days_to_expiry, min_arbitrage_profit, min_roi_percent, max_positions, max_total_exposure, max_bid_ask_spread)
   - `YieldCurvePoint` (days_to_expiry, implied_rate, effective_rate, net_debit, spread_bps)
   - `YieldCurve` (symbol, strike_width, benchmark_rate, repeated YieldCurvePoint)

2. **Add risk reporting messages** (or extend existing):
   - `RiskDecision` (allowed, reason)
   - `PositionRisk` (fields matching C++/Python reporting)
   - `PortfolioRisk` (aggregate fields)

3. **Add discount bank messages**:
   - `DiscountBankBalance` (account, balance, currency, balance_date, credit_rate, debit_rate, branch_number, section_number, account_number)
   - `DiscountBankTransaction` (value_date, amount, is_debit, reference)
   - `BankAccount` (account_path, account_name, bank_name, account_number, balance, currency, balance_date, credit_rate, debit_rate)

4. **Regenerate all languages:** Run `./proto/generate.sh` (or `just proto-gen`); verify C++ `native/generated/`, Python output, Rust `nats_adapter` build, and ts-proto (if wired). Fix any codegen breakage.

**Acceptance:** `proto/messages.proto` contains the new messages; Python, Rust, and C++ codegen succeed; no new manual type mirrors.

---

## Phase 2: Python codegen and retire mirrors

**Goal:** Python uses generated types at NATS/REST boundaries; `proto_types.py` and boundary DTOs in `box_spread_models.py` deprecated or removed.

**Tasks:**

1. **Align Python codegen and `python/generated/`:** Ensure `proto/generate.sh` produces Python that matches how the codebase consumes it (standard `_pb2` vs betterproto). Update `python/generated/__init__.py` to export from the actual generated module(s).

2. **Migrate NATS/REST boundary code** to use `python/generated`: Identify all call sites that use `proto_types` or `box_spread_models` for on-the-wire DTOs (NATS, REST handlers). Replace with generated types and thin conversion where needed.

3. **Keep internal Python-only types only where they add behavior** (e.g. `MarketData.get_mid_price()` in `box_spread_models`); at boundaries use generated types only.

4. **Deprecate `proto_types.py`:** Add deprecation notice; remove or redirect imports once all boundary consumers use generated code. Optionally keep a small compatibility shim that re-exports from generated.

**Acceptance:** No boundary code imports from `proto_types.py`; `box_spread_models` uses generated types for DTOs at boundaries; `./proto/generate.sh` is the single source for Python proto types.

---

## Phase 3: TypeScript codegen from messages.proto

**Goal:** Web app consumes generated types from `proto/messages.proto`; remove hand-written `web/src/types/proto.ts`.

**Tasks:**

1. **Wire ts-proto for platform proto:** Ensure `proto/generate.sh` (or Justfile) generates from `proto/messages.proto` into a single output dir (e.g. `web/src/generated/proto/` for platform messages). Resolve conflict with TWS API–generated files (e.g. different subdir or prefix).

2. **Replace `web/src/types/proto.ts`:** Change all imports to use generated code from `web/src/generated/proto/` (or equivalent). Delete or archive `web/src/types/proto.ts`.

3. **CI/build:** Add step to run proto codegen so generated TS is up to date; document in `docs/planning/PROTOBUF_DEDUP_OPPORTUNITIES.md` and `.cursor/agents/protobuf-justfile.md`.

**Acceptance:** Web build uses only generated proto types for `ib.platform.v1`; no hand-maintained duplicate interfaces.

---

## Phase 4: NATS publish protobuf

**Goal:** C++ publishes protobuf-serialized messages; Python and Rust subscribers deserialize protobuf instead of JSON.

**Tasks:**

1. **C++:** For each current JSON `publish_*` (e.g. market data, strategy signal, strategy decision), build the corresponding protobuf message, call `SerializeToString()`, publish via `publish_raw()`. Use `NatsEnvelope` + payload where applicable (see [NATS_PROTOBUF_MIGRATION.md](NATS_PROTOBUF_MIGRATION.md)).

2. **Python:** Update NATS subscribers to deserialize protobuf from payload; remove JSON parsing for those message types.

3. **Rust:** Ensure `nats_adapter` (or relevant consumers) deserialize protobuf; remove JSON handling for migrated topics.

4. **Config/docs:** Document that NATS payloads for platform topics are protobuf; update `docs/message_schemas/README.md` and NATS_PROTOBUF_MIGRATION.md status.

**Acceptance:** C++ uses only protobuf for platform NATS publish; Python and Rust consume protobuf for those topics; no JSON for migrated message types.

---

## Phase 5: Remove dead gRPC/proto and document

**Goal:** Single clear proto story: `proto/messages.proto` (+ TWS API vendor protos); no dead gRPC or backend protos.

**Tasks:**

1. **Remove or archive:** `agents/backend/crates/api/src/grpc.rs`, `agents/backend/proto/ib_backend.proto`, `agents/backend-market-data/` (proto and service). Per [REFACTORING_AND_DEAD_CODE_AUDIT.md](../analysis/REFACTORING_AND_DEAD_CODE_AUDIT.md).

2. **Update build:** Remove tonic_build / references to removed protos from Cargo and CMake if any.

3. **Document:** In `docs/message_schemas/README.md` state that the canonical cross-language contract is `proto/messages.proto`; TWS API protos are vendor-only. Link to this plan and PROTOBUF_DEDUP_OPPORTUNITIES.md.

**Acceptance:** No gRPC server or backend proto in use; docs describe single proto story.

---

## Exarp Task Definitions

Use the following blocks to create Todo2 tasks in exarp-go (e.g. in Cursor chat: "Create Todo2 task" with the title and description, or use task discovery on this file with `create_tasks=true`). Dependencies are noted so you can set task dependencies in exarp.

---

### Task P1-1: Add box spread and yield curve messages to proto/messages.proto

**Title:** Add box spread and yield curve messages to proto/messages.proto

**Description:** Extend `proto/messages.proto` with OptionContract, OptionTypeEnum, BoxSpreadLeg, BoxSpreadOpportunity, StrategyParams, YieldCurvePoint, and YieldCurve so all languages share one schema. Match fields to C++/Python usage (see python/generated/__init__.py and CROSS_LANGUAGE_DEDUP_PLAN). Run proto codegen and fix any breakage in C++, Python, Rust.

**Acceptance criteria:** New messages exist in messages.proto; ./proto/generate.sh succeeds for Python/C++/Go; Rust nats_adapter builds.

**Tags:** protobuf, codegen, cross-language  
**Depends on:** (none)

---

### Task P1-2: Add risk and discount bank messages to proto/messages.proto

**Title:** Add risk and discount bank messages to proto/messages.proto

**Description:** Add RiskDecision, PositionRisk, PortfolioRisk, DiscountBankBalance, DiscountBankTransaction, and BankAccount to proto/messages.proto. Align with C++/Python/Rust usage at API and NATS boundaries. Regenerate all languages and verify builds.

**Acceptance criteria:** Messages added; codegen succeeds; no new hand-written mirrors.

**Tags:** protobuf, codegen, cross-language  
**Depends on:** P1-1 (optional; can be same PR)

---

### Task P2-1: Align Python proto codegen and python/generated/__init__.py

**Title:** Align Python proto codegen and python/generated/__init__.py

**Description:** Ensure proto/generate.sh produces Python that python/generated/__init__.py can export (standard _pb2 or betterproto). Update __init__.py to match actual generator output so all P1 messages are importable from python/generated.

**Acceptance criteria:** python/generated exposes all proto types; "from python.generated import BoxSpreadLeg" (or equivalent) works after codegen.

**Tags:** protobuf, python  
**Depends on:** P1-1, P1-2

---

### Task P2-2: Migrate Python NATS/REST boundaries to generated proto types

**Title:** Migrate Python NATS/REST boundaries to generated proto types

**Description:** Replace imports from proto_types.py and box_spread_models (for DTOs only) with python/generated at all NATS and REST boundary call sites. Add thin conversion where needed. Keep internal dataclasses only where they add behavior (e.g. get_mid_price).

**Acceptance criteria:** No boundary code imports proto_types or box_spread_models for wire DTOs; tests pass.

**Tags:** protobuf, python, nats  
**Depends on:** P2-1

---

### Task P2-3: Deprecate proto_types.py

**Title:** Deprecate proto_types.py

**Description:** Add deprecation notice to python/proto_types.py; remove or redirect remaining imports to python/generated. Optionally keep a small compatibility shim that re-exports from generated. Update docs.

**Acceptance criteria:** proto_types.py deprecated or removed; no remaining callers for boundary types.

**Tags:** protobuf, python  
**Depends on:** P2-2

---

### Task P3-1: Wire ts-proto for proto/messages.proto into web

**Title:** Wire ts-proto for proto/messages.proto into web

**Description:** Ensure proto/generate.sh (or Justfile) generates TypeScript from proto/messages.proto into web/src/generated/proto/ (or agreed dir). Resolve any conflict with TWS API-generated files. Document in proto/generate.sh and protobuf-justfile.md.

**Acceptance criteria:** just proto-gen or ./proto/generate.sh produces TS for messages.proto; web build can import generated types.

**Tags:** protobuf, typescript, codegen  
**Depends on:** P1-1

---

### Task P3-2: Replace web/src/types/proto.ts with generated imports

**Title:** Replace web/src/types/proto.ts with generated imports

**Description:** Change all imports from web/src/types/proto.ts to use generated code from web/src/generated/proto/. Delete or archive types/proto.ts. Fix any type or path breakage.

**Acceptance criteria:** Web app uses only generated proto types for ib.platform.v1; types/proto.ts removed or archived.

**Tags:** protobuf, typescript  
**Depends on:** P3-1

---

### Task P4-1: C++ NATS publish protobuf instead of JSON

**Title:** C++ NATS publish protobuf instead of JSON

**Description:** For each platform NATS publish (market data, strategy signal, strategy decision), build the corresponding protobuf message, serialize with SerializeToString(), and use publish_raw(). Use NatsEnvelope + payload where applicable. See NATS_PROTOBUF_MIGRATION.md.

**Acceptance criteria:** C++ publishes only protobuf for those topics; no JSON construction for migrated messages.

**Tags:** protobuf, nats, c++  
**Depends on:** P1-1, P1-2

---

### Task P4-2: Python and Rust NATS subscribers deserialize protobuf

**Title:** Python and Rust NATS subscribers deserialize protobuf

**Description:** Update Python and Rust NATS subscribers for platform topics to deserialize protobuf payloads instead of JSON. Remove JSON parsing for migrated message types. Verify end-to-end with C++ publisher.

**Acceptance criteria:** Python and Rust consume protobuf for platform NATS topics; no JSON path for those messages.

**Tags:** protobuf, nats, python, rust  
**Depends on:** P4-1

---

### Task P5-1: Remove dead gRPC and backend proto

**Title:** Remove dead gRPC and backend proto

**Description:** Remove or archive agents/backend/crates/api/src/grpc.rs, agents/backend/proto/ib_backend.proto, and agents/backend-market-data/ proto and service. Remove tonic_build and references. See REFACTORING_AND_DEAD_CODE_AUDIT.md.

**Acceptance criteria:** No gRPC server or backend proto in use; build succeeds without removed crates/protos.

**Tags:** cleanup, protobuf, rust  
**Depends on:** (none)

---

### Task P5-2: Document single proto story in message_schemas

**Title:** Document single proto story in message_schemas

**Description:** Update docs/message_schemas/README.md to state that canonical cross-language contract is proto/messages.proto; TWS API protos are vendor-only. Link to PROTOBUF_DEDUP_OPPORTUNITIES.md and this plan.

**Acceptance criteria:** README clearly describes single proto story and points to planning docs.

**Tags:** documentation, protobuf  
**Depends on:** P5-1 (optional)

---

## How to create these tasks in exarp-go

1. **In Cursor:** Ensure exarp-go MCP is enabled and `workingDirectory` is this repo root. In chat you can say:  
   *"Create Todo2 tasks from docs/planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md. Use the Exarp Task Definitions section: one task per ### Task block, with the given title, description, acceptance criteria, and tags. Set dependencies where Depends on is listed."*

2. **Batch create (paste this into Cursor with exarp-go):**  
   *"Create the following Todo2 tasks. workingDirectory: <this project root>. For each, use the title as the task title and the description as the long description. Add tags: protobuf, and the other tags listed. Set dependencies when indicated."*  
   Then paste the table below.

3. **Or use task discovery:**  
   *"Discover tasks from docs/planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md with create_tasks=true"*  
   (exarp may create tasks from the phase list and task blocks; then add dependencies manually if needed.)

4. **Or create manually:** Copy each "### Task Px-y" block into exarp-go and create a task with the same title and description; set dependency to the task matching "Depends on" (e.g. P2-1 depends on P1-1 and P1-2).

5. **One-shot prompt:** In Cursor with exarp-go (workingDirectory = repo root), say: "Create 11 Todo2 tasks from docs/planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md. Use each ### Task Px-y heading as title and Description + Acceptance criteria as long description. Tags: protobuf plus task tags. Dependencies: P1-2 after P1-1; P2-1 after P1-1,P1-2; P2-2 after P2-1; P2-3 after P2-2; P3-1 after P1-1; P3-2 after P3-1; P4-1 after P1-1,P1-2; P4-2 after P4-1; P5-2 after P5-1. P1-1 and P5-1 have no deps."

### Compact task list (for batch create)

| ID | Title | Depends on |
|----|-------|------------|
| P1-1 | Add box spread and yield curve messages to proto/messages.proto | — |
| P1-2 | Add risk and discount bank messages to proto/messages.proto | P1-1 |
| P2-1 | Align Python proto codegen and python/generated/__init__.py | P1-1, P1-2 |
| P2-2 | Migrate Python NATS/REST boundaries to generated proto types | P2-1 |
| P2-3 | Deprecate proto_types.py | P2-2 |
| P3-1 | Wire ts-proto for proto/messages.proto into web | P1-1 |
| P3-2 | Replace web/src/types/proto.ts with generated imports | P3-1 |
| P4-1 | C++ NATS publish protobuf instead of JSON | P1-1, P1-2 |
| P4-2 | Python and Rust NATS subscribers deserialize protobuf | P4-1 |
| P5-1 | Remove dead gRPC and backend proto | — |
| P5-2 | Document single proto story in message_schemas | P5-1 |
