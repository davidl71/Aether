# Platform Improvement Plan

**Last updated**: 2026-03-07

## Completed Tasks

| Task | Notes | Exarp Task ID |
|------|-------|---------------|
| Remove dead gRPC and backend proto | No gRPC (tonic/grpc) ever existed in agents/backend; prost-only protobuf via nats_adapter. Dead code removed in cleanup commits (lean_integration, ml, nautilus, dsl, wrapper). | T-1772609703288099000 ✅ |
| Add box spread and yield curve messages to proto | BoxSpreadLeg, BoxSpreadScenario, BoxSpreadExecution, YieldCurvePoint, YieldCurve, BoxSpreadOpportunity, StrategyParams all present in `proto/messages.proto`. | T-1772609676030467000 ✅ |
| Structured logging in Go agents (slog) | All five Go agents use `log/slog` with JSON handler. No `log.Printf` calls remain. | T-1772887222034956306 ✅ |
| WebSocket delta compression (P2-A) | Server sends full snapshot once on connect then only changed sections every 2s; client merges deltas. WS mounted on same server at `/ws`. | T-1772887222103963807 ✅ |
**Priority lens**: System responsiveness + data persistence + thin collection daemons.
Trading volume is low; the focus is on correctness, fast reads, durable writes, and
lean background processes — not throughput optimization.

See `docs/platform/DATAFLOW_ARCHITECTURE.md` for the full issue analysis.

---

## Design Principles (session-derived)

1. **Persistence first**: every market event, order, and position change must land in durable
   storage before the UI updates. Don't trade correctness for speed.
2. **Thin daemons**: background collectors and bridges should be single-binary, low-memory
   Go or C++ processes. No Python daemons doing collection — Python is for analysis/TUI only.
3. **Responsive reads**: clients should receive delta updates, not full snapshots. Store once,
   serve many via NATS KV or Arrow Flight.
4. **Single source of truth**: one writer per data store. Eliminate dual-write patterns.

---

<!-- task-discovery: scan this file for [ ] items and exarp task IDs -->

## Priority 1 — Fix Data Integrity (do first, blocks everything)

### P1-A: Fix dual SQLite writers (CRITICAL) <!-- exarp: T-1772887221775761020 -->
**Issue**: Rust ledger and Python both write to `ledger.db` — concurrent writes corrupt WAL.
**Fix**:
- Enable WAL mode (`PRAGMA journal_mode=WAL`) as immediate mitigation.
- Longer term: Python reads from Rust ledger via REST API (`GET /api/ledger/...`).
  Python never writes directly to SQLite — Rust ledger is the single writer.
**Files**: `agents/backend/crates/ledger/src/lib.rs`, `python/integration/` ledger write paths.

### P1-B: Unify TUI and Web data backends <!-- exarp: T-1772887221914991889 -->
**Issue**: TUI reads Python :8000-8006; Web reads Rust :8080. Two pipelines, potential divergence.
**Fix**:
- Route TUI's `RestProvider` through the Go `api-gateway` (:8090), which already proxies
  both Python services and Rust backend. One entry point, same data.
- Or: expose a unified `/api/snapshot` from the Go gateway that aggregates Python + Rust.
**Files**: `python/tui/providers.py`, `agents/go/cmd/api-gateway/main.go`.

---

## Priority 2 — Responsiveness (thin, event-driven)

### P2-A: WebSocket delta compression <!-- exarp: T-1772887222103963807 --> ✅
**Done**: Server sends full snapshot once on connect, then only changed sections every 2s;
client merges deltas; WebSocket mounted on same server at `/ws`. ~90% bandwidth reduction when state is stable.
**Files**: `agents/backend/crates/api/src/websocket.rs`, `agents/backend/crates/api/src/rest.rs`.

### P2-B: Decode NatsEnvelope in Go agents <!-- exarp: T-1772887221969976131 -->
**Issue**: `nats-questdb-bridge` and `heartbeat-aggregator` parse raw bytes as strings.
**Fix**: Deserialize `NatsEnvelope` using `agents/go/proto/v1/messages.pb.go` (already
generated). Dispatch on `message_type` field. Write QuestDB columns from proto field names.
**Files**: `agents/go/cmd/nats-questdb-bridge/main.go`.
**Benefit**: Type-safe; field names match proto schema; survives format changes.

### P2-C: NATS KV as primary live-state store
**Issue**: Clients poll REST every 1-2s to get current state.
**Fix**: C++ engine and Python services write current state to NATS KV buckets
(`kv.positions`, `kv.rates`, `kv.risk`). Clients subscribe to KV watch — zero-latency
updates with no polling overhead. NATS KV is persistent (backed by JetStream).
**Files**: `native/src/nats_client.cpp`, `python/tui/providers.py` (NatsProvider).

---

## Priority 3 — Financial Math Correctness

### P3-A: Replace hardcoded ETF duration table with QuantLib <!-- exarp: T-1772887222158664215 -->
**Issue**: `greeks_calculator.cpp` uses a static lookup table for ETF duration/convexity.
New ETFs or changed compositions produce wrong values.
**Fix**: Use `QuantLib::BondFunctions::duration()` and `BondFunctions::convexity()` with
the ETF's known underlying bond parameters. Fall back to lookup for non-bond ETFs.
**Files**: `native/src/greeks_calculator.cpp`, `native/include/greeks_calculator.h`.
**Test**: Add Catch2 test comparing computed vs known duration for AGG, TLT, SHY.

### P3-B: Add Newton-Raphson IV solver <!-- exarp: T-1772887222213114929 -->
**Issue**: `BlackCalculator` is used correctly but IV must be supplied externally.
**Fix**: Implement `calculate_implied_vol(market_price, S, K, r, T, option_type)` using
Newton-Raphson bisection on `BlackCalculator::value()`. QuantLib provides
`blackImpliedVolAdaptiveStrike()` as an alternative.
**Files**: `native/src/greeks_calculator.cpp`.
**Test**: Catch2 round-trip: compute price → compute IV → recompute price, assert < 1bp error.

### P3-C: Nelson-Siegel/Svensson yield curve fitting <!-- exarp: T-1772887222348905245 -->
**Issue**: `yield_curve_comparison.py` uses simple interpolation for benchmark rates.
**Fix**: Fit Nelson-Siegel (3-param) or Svensson (4-param) model to Treasury/SOFR term
structure. Enables:
- Smooth par rate / discount factor curve at any maturity
- Quantitative comparison of box spread implied rate vs model rate
- QuantLib has `NelsonSiegelFitting` and `SvenssonFitting` in `ql/termstructures/yield/`.
**Files**: `python/integration/yield_curve_comparison.py`, possibly a new C++ `yield_curve.cpp`.

### P3-D: Standard amortization schedule generation <!-- exarp: T-1772887222449509427 -->
**Issue**: `cash_flow_calculator.py` handles Israeli SHIR/CPI loans but not standard PMT.
**Fix**: Add `generate_amortization_schedule(principal, rate, periods, type)` returning
a DataFrame of (period, payment, interest, principal, balance). Support:
- Fixed-rate (standard PMT formula)
- SHIR-linked (existing, refactor into same interface)
- CPI-linked (existing)
- Balloon / interest-only
**Files**: `python/integration/cash_flow_calculator.py`.

---

## Priority 4 — Schema & Tooling

### P4-A: Migrate to `buf` for proto schema management <!-- exarp: T-1772887222270264987 -->
**Issue**: `proto/generate.sh` is a shell script — no lint, no breaking-change detection.
**Fix**: Add `buf.yaml` + `buf.gen.yaml`. Replace `proto/generate.sh` with `buf generate`.
Add `buf breaking --against .git#tag=v1.0` to CI.
**Files**: `proto/buf.yaml`, `proto/buf.gen.yaml` (new), `proto/generate.sh` (retire).

### P4-B: Structured logging in Go agents <!-- exarp: T-1772887222034956306 --> ✅ DONE
**Issue**: `log.Printf` — no levels, no JSON, no correlation IDs.
**Fix**: All Go agents already use `log/slog` with `slog.SetDefault(slog.New(slog.NewJSONHandler(...)))`.
All `log.Printf` calls replaced with structured `slog.Info` / `slog.Error` calls.
**Files**: `agents/go/cmd/*/main.go` — complete.

### P4-C: Rust REST: derive serde on prost types
**Issue**: Rust REST responses use hand-written `serde` structs that mirror proto types.
**Fix**: Add `#[derive(serde::Serialize, serde::Deserialize)]` to prost-generated types
via `prost-build` with `serde` feature. Eliminates duplicate type definitions.
**Files**: `agents/backend/crates/nats_adapter/build.rs`, `Cargo.toml`.

---

## Priority 5 — Long-term Epics

### E1: ConnectRPC — replace REST polling with streaming
Replace TUI's 1s REST polling and Web's 2s snapshot polling with ConnectRPC streaming RPCs.
ConnectRPC serves gRPC, gRPC-Web, and plain JSON HTTP/1.1 from the same handler.
- Rust: `connectrpc` crate
- Python TUI: `grpclib` (async) or betterproto stubs
- React web: `@connectrpc/connect-web`
**Impact**: Latency drops from 1-2s to ~10ms for state changes. Eliminates polling overhead.

### E2: Apache Arrow Flight for bulk/historical data
Replace QuestDB HTTP polling with Arrow Flight SQL for columnar bulk reads.
QuestDB natively supports Arrow Flight SQL. Python analytics get zero-copy columnar data.
The Go `nats-questdb-bridge` becomes an Arrow Flight writer.
**Impact**: 10-100x faster for bulk position/tick queries. Enables notebook-level analysis.

### E3: Asset Relationship Graph (Phase 2 of SYNTHETIC_FINANCING_ARCHITECTURE)
Implement `AssetRelationshipGraph` + `CollateralValuator` + `FinancingInstrumentRegistry`
as designed in `docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md`.
Prerequisite for multi-instrument optimization.

### E4: Financing Optimizer (Phase 4)
NLopt-based `FinancingOptimizer` that minimizes effective cost across box spreads, T-bills,
bank loans, pension loans, and FX swaps simultaneously.
Prerequisite: E3 (asset relationship graph).

### E5: Unified thin collection daemon
Replace Python data-collection polling loops with a single Go daemon that:
- Subscribes to NATS for C++ events
- Polls broker REST APIs on configurable intervals
- Writes to NATS KV (live state) and QuestDB (history)
- Exposes a single `/metrics` endpoint (Prometheus format)
Python services become read-only analytics — they query NATS KV or QuestDB.

---

## Quick-reference Priority Matrix

<!-- task-discovery: exarp task IDs in this table link directly to tracked work items -->

| Task | Effort | Impact | Priority | Exarp Task ID |
|------|--------|--------|----------|---------------|
| P1-A: Fix dual SQLite writers | S | Critical | Do first | T-1772887221775761020 |
| P1-B: Unify TUI/Web backends via api-gateway | S | High | Do first | T-1772887221914991889 |
| P2-B: NatsEnvelope decode in Go agents | S | High | Low hanging fruit | T-1772887221969976131 |
| ~~P4-B: slog in Go agents~~ ✅ DONE | XS | Medium | Low hanging fruit | T-1772887222034956306 |
| P2-A: WS delta compression | M | High | Next sprint | T-1772887222103963807 |
| P3-A: QuantLib ETF duration | M | High | Next sprint | T-1772887222158664215 |
| P3-B: IV solver | M | High | Next sprint | T-1772887222213114929 |
| P4-A: buf schema management | S | Medium | Next sprint | T-1772887222270264987 |
| P3-C: Nelson-Siegel | M | Medium | Financial math sprint | T-1772887222348905245 |
| P3-D: Amortization schedule | S | Medium | Financial math sprint | T-1772887222449509427 |
| P2-C: NATS KV live state | L | High | Architecture sprint | (not yet created) |
| P4-C: Rust serde on prost | S | Medium | Cleanup | (not yet created) |
| E1: ConnectRPC | XL | High | Epic | T-1772887222509770969 |
| E2: Arrow Flight | XL | High | Epic | T-1772887222569465548 |
| E3: Asset Relationship Graph | XL | High | Epic | T-1772887222624798220 |
| E4: Financing Optimizer | XL | Very High | Epic | T-1772887222913841962 |
| E5: Unified Go collection daemon | XL | High | Epic | T-1772887222970694620 |

Size: XS < 2h, S < 1d, M 1-3d, L 3-7d, XL > 1w
