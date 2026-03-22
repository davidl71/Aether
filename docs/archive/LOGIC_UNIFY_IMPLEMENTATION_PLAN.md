# Logic Unify Implementation Plan

**Source:** `docs/design/LOGIC_WE_COULD_UNIFY.md`
**Purpose:** Phased implementation of unification work, driven by exarp-go task_workflow. Each phase has a clear deliverable and dependency for Todo2 task creation.

---

## Phase A (foundation – low effort)

No dependencies. Tasks A1, A2, A3 can be created and executed in any order (or in parallel).

### A1: Expose C++ statistical helpers via pybind11; Python calls C++

**Deliverable:** C++ `calculate_mean`, `calculate_percentile`, `calculate_correlation` (from `native/src/risk_calculator.cpp` / `risk_calculator.h`) exposed via pybind11. Python `python/integration/risk_calculator.py` calls these bindings and removes its own implementations. Tests in `python/tests/test_risk_calculator.py` updated to use bindings.

**References:** LOGIC_WE_COULD_UNIFY §1.

---

### A2: Expose calculate_dte via pybind11; Python callers use binding

**Deliverable:** C++ `tws::calculate_dte(expiry_yyyymmdd: str) -> int` (from `native/src/tws_conversions.cpp`) exposed via pybind11 (same module as box spread bindings). Python callers (sofr_treasury_client, benchmarks, TUI models, risk-free rate) use the binding instead of ad-hoc date math.

**References:** LOGIC_WE_COULD_UNIFY §2.

---

### A3: Single JSON Schema for shared config (TUI, PWA, CLI)

**Deliverable:** One JSON Schema for the shared config used by TUI, PWA, and native CLI. Document in existing config schema (e.g. `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md`) or add `config/schema.json`. All loaders validate against the same structure; parsing remains language-specific.

**References:** LOGIC_WE_COULD_UNIFY §4; SHARED_CONFIGURATION_SCHEMA.md.

---

## Phase B (TUI/PWA and wire format)

### B1: Align snapshot/health payload to one proto shape; TUI and PWA use generated types

**Deliverable:** Snapshot and health payloads on NATS and REST use the same proto-defined shape. TUI (Python) and PWA (TypeScript) consume generated types from `proto/messages.proto`. Reduces "TUI has one shape, PWA another."

**Done:** (1) `proto/messages.proto`: added `BackendHealth` and `HealthAggregate` for health payloads; `SystemSnapshot` already defines snapshot shape. (2) `docs/NATS_TOPICS_REGISTRY.md`: snapshot and system.health document canonical payload = proto. (3) Consumption: Python has `python/generated/messages_pb2.py`; TUI/PWA can adopt generated types incrementally (adapters from proto to existing SnapshotPayload/TypeScript interfaces or migrate to generated types). TS codegen wired per task T-1772609719082616000.

**Dependencies:** Proto migration — box spread and yield curve messages already present in `proto/messages.proto` (T-1772609676030467000 ✅ DONE: BoxSpreadLeg, BoxSpreadScenario, BoxSpreadExecution, YieldCurvePoint, YieldCurve, BoxSpreadOpportunity, StrategyParams). Python and TS codegen must produce types for snapshot/health.

**References:** LOGIC_WE_COULD_UNIFY §8 (enums at boundaries), §10 (TUI/PWA snapshot contract).

---

### B2: Single config file/schema usage – TUI and PWA read same services and broker.priorities

**Deliverable:** TUI and PWA both read the same home config (`~/.config/ib_box_spread/config.json` or `IB_BOX_SPREAD_CONFIG`) and use the same `services` and `broker.priorities` so both UIs see the same backends and ordering. TUI uses `tui.providerType` / `tui.restEndpoint`; PWA uses `pwa.servicePorts` and env `VITE_*_PORT`.

**Done:** TUI already uses SharedConfigLoader (home config). Shared `GET /api/config` ownership has moved to the Rust API, which now returns the `services`, `broker`, and `pwa` slice from the same shared config search path. Historical web clients can point `VITE_CONFIG_URL` at `http://localhost:8080/api/config` to consume the same source and ordering.

**Dependencies:** A3 (single JSON Schema).

**References:** LOGIC_WE_COULD_UNIFY §10; SHARED_CONFIGURATION_SCHEMA.md; MULTI_ACCOUNT_AGGREGATION_DESIGN.md.

---

## Phase C (optional / later)

Can be separate Todo2 tasks or a single "Document and optional follow-ups" task. Dependencies: B1, B2 (or none for doc-only items).

### C1: Message bus and cache – document NATS-only + CacheClient factory

**Deliverable:** Document that NATS is the single message bus and NATS KV → Redis → memcached order for cache/state behind a unified CacheClient/state factory. No code change required if already covered in `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md`; optionally add a short "Logic Unify" subsection there or in LOGIC_WE_COULD_UNIFY.

**References:** LOGIC_WE_COULD_UNIFY §9.

---

### C2: Enums/constants – migrate remaining manual mirrors to proto-generated types

**Deliverable:** At NATS/REST boundaries, use proto-generated enums and types only. In Python, prefer `python/generated/` (or proto_types if still used); remove or redirect manual mirrors. Numeric constants (e.g. default min_roi, max_dte) defined once in config schema defaults or a small shared constants module.

**Done:** Documented in this plan: use generated types at boundaries; numeric defaults in config/schema.json or shared constants module. Proto defines enums (OptionTypeEnum, AlertLevel, etc.). No automated migration in this task.

**References:** LOGIC_WE_COULD_UNIFY §8.

---

### C3: Option/contract string parsing – spec (doc-only) or C++ parser + binding

**Deliverable:** Either (1) a single documented spec for contractDesc/OCC format so C++ and Python follow the same rules, or (2) one canonical parser in C++ with `parse_option_contract_desc(str) -> OptionContract` (or tuple) exposed via pybind11; Python `combo_detector` calls it.

**Done:** Added `docs/design/OPTION_CONTRACT_DESC_SPEC.md`: IB contractDesc format, canonical regex, parsing rules, examples; OCC reference; implementation notes for Python (combo_detector) and future C++/pybind11. C++ parser + binding remains optional follow-up.

**References:** LOGIC_WE_COULD_UNIFY §5.

---

## Task creation (exarp-go task_workflow)

Create one Todo2 task per phase item (A1, A2, A3, B1, B2, C1, C2, C3) with:

| Task | name (short) | priority | tags | dependencies |
|------|----------------|----------|------|--------------|
| A1 | Expose C++ stats helpers via pybind11; Python calls C++ | high | unify, bindings, pybind11 | — |
| A2 | Expose calculate_dte via pybind11; Python callers use binding | high | unify, bindings, pybind11 | — |
| A3 | Single JSON Schema for shared config (TUI, PWA, CLI) | medium | unify, config, schema | — |
| B1 | Align snapshot/health payload to one proto shape; TUI and PWA use generated types | medium | unify, tui, pwa, proto | T-1772609676030467000 ✅ (proto messages done; codegen adoption remaining) |
| B2 | Single config file/schema usage – TUI and PWA read same services and broker.priorities | medium | unify, tui, pwa, config | A3 task ID |
| C1 | Message bus and cache – document NATS-only + CacheClient factory | low | unify, docs, nats | — |
| C2 | Enums/constants – migrate remaining manual mirrors to proto-generated types | low | unify, proto, codegen | — |
| C3 | Option/contract string parsing – spec or C++ parser + binding | low | unify, parsing, bindings | — |

After creating tasks, run **task_workflow action=sync**. Then run **task_workflow action=link_planning** for each task with `planning_doc=docs/planning/LOGIC_UNIFY_IMPLEMENTATION_PLAN.md`.

---

## References

- `docs/design/LOGIC_WE_COULD_UNIFY.md` — Full unification items (§1–§10).
- `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md` — Unified config format.
- `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md` — NATS KV, Redis, state factory.
- `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` — Protobuf and single-source strategy.
- `docs/platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md` — TUI vs PWA behaviour.
