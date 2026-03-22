# Protobuf conversion and NATS KV

**Status:** Improvement note (2026-03).  
**Related:** [IMPROVEMENT_PLAN.md](IMPROVEMENT_PLAN.md) P4-C, [NATS_API.md](NATS_API.md), [PROTOBUF_DEDUP_OPPORTUNITIES.md](../planning/PROTOBUF_DEDUP_OPPORTUNITIES.md).

## 1. Summary

We are **not** doing unnecessary double encode/decode (e.g. proto → JSON → proto on the same message). We do have **redundant conversion logic** and **two different domain shapes** for the same snapshot, which leads to duplicate code and two parallel conversion paths. NATS KV stores a **mix of formats** (envelope-wrapped proto, bare proto, and JSON) that should be documented and optionally unified.

---

## 2. Current data flows

### 2.1 Snapshot (backend → NATS → TUI)

| Step | Location | Transformation |
|------|----------|----------------|
| Backend in-memory | `api::state::SystemSnapshot` | Domain type (Rust structs with `chrono`, `serde`) |
| Publish | `snapshot_publisher.rs` | `snapshot_to_proto(&snap)` → `pb::SystemSnapshot` → `encode_envelope()` → NATS |
| Wire | `snapshot.{backend_id}` | `NatsEnvelope(SystemSnapshot)` (protobuf bytes) |
| TUI receive | `tui_service/src/nats.rs` | `extract_proto_payload::<pb::SystemSnapshot>` → **`proto_to_snapshot()`** → `RuntimeSnapshotDto` |

- **Backend:** One conversion (domain → proto) before send. Necessary for wire format.
- **TUI:** One decode then one hand-written conversion (proto → `RuntimeSnapshotDto`). No second encode.

So there is **one** domain→proto and **one** proto→DTO path. The redundancy is:

- **Duplicate domain↔proto logic:** `snapshot_publisher` implements `position_to_proto`, `symbol_to_proto`, etc., while `nats_adapter/conversions.rs` has `From`/`Into` for mirror types (`nats_adapter::PositionSnapshot` ↔ `pb::Position`). The publisher does **not** use the nats_adapter conversions.
- **Two domain shapes:** Backend uses `api::state::SystemSnapshot`; TUI uses `RuntimeSnapshotDto` (and `RuntimePositionDto`, etc.). So we maintain two conversion paths (backend domain→proto, TUI proto→DTO) instead of one shared path (e.g. proto ↔ `api::SystemSnapshot` everywhere).

### 2.2 REST snapshot

- **JSON:** `SystemSnapshot` → `RuntimeSnapshotDto::from(&snap)` → `serde_json::to_vec`. One domain→DTO→JSON.
- **Proto:** `SystemSnapshot` → `snapshot_to_proto()` → `proto.encode_to_vec()`. One domain→proto. No double encode.

### 2.3 Collection aggregation (NATS → KV / QuestDB)

- **Incoming:** Decode **one** `NatsEnvelope` from the NATS payload.
- **KV write:** Store **raw payload** (`payload.to_vec()`) — i.e. the **full NatsEnvelope bytes** as received. No re-encode.
- **QuestDB:** Decode inner `MarketDataEvent` from `envelope.payload` to build the ILP line. One decode of the inner message; no unnecessary transformation.

---

## 3. NATS KV (LIVE_STATE bucket)

The same bucket (`NATS_KV_BUCKET`, default `LIVE_STATE`) is used for:

- Event replay (collection_aggregation)
- Position persistence (snapshot_publisher)
- Yield curve (yield_curve_writer, tws_yield_curve_daemon)

**Value formats differ by key pattern.** Consumers must know the format to decode correctly.

| Key pattern | Writer | Value format | Consumer / notes |
|-------------|--------|--------------|------------------|
| `MarketDataEvent.{symbol}` | collection_aggregation | **Raw NatsEnvelope bytes** (full message as received from NATS) | Decode `NatsEnvelope`, then `MarketDataEvent::decode(envelope.payload)`. |
| `StrategySignal.{symbol}` | collection_aggregation | **Raw NatsEnvelope bytes** | Decode envelope, then inner `StrategySignal`. |
| `StrategyDecision.{symbol}` | collection_aggregation | **Raw NatsEnvelope bytes** | Decode envelope, then inner `StrategyDecision`. |
| `Positions.{backend_id}` | snapshot_publisher | **Bare proto** `PositionsSnapshot` (no envelope) | `PositionsSnapshot::decode(bytes)` directly. Key example: `Positions.ib`. |
| `yield_curve.{symbol}` | yield_curve_writer, tws_yield_curve_daemon | **Proto** `YieldCurve` (see §4.4) | Decode `YieldCurve::decode(bytes)`; reader tries proto first, then JSON array fallback. |

Implications:

- **Event keys:** Storing the full envelope preserves `source`, `message_type`, and `timestamp`; consumers decode once (envelope then inner). No extra transformation on write.
- **Positions:** Stored as bare proto. Any future consumer (e.g. REST fallback or late joiner reading from KV) would decode proto and, if it needs domain or JSON, would use the same conversion story as the TUI (proto → domain or DTO). Unifying conversion (see §4) would help.
- **Yield curve:** Stored as proto `YieldCurve` (symbol, strike_width, repeated YieldCurvePoint). Readers try proto decode first, then fall back to legacy JSON array for backward compatibility.

---

## 4. Recommendations

### 4.1 Unify domain ↔ proto conversion (snapshot)

- **Option A:** Have `snapshot_publisher` use the existing `From` impls in `nats_adapter/conversions.rs` (e.g. convert `api::PositionSnapshot` to `nats_adapter::PositionSnapshot` then `.into()` to `pb::Position`) so there is a single implementation of position/symbol/alert→proto. Requires that `api` and backend_service share the same conversion types or that `api::state` types can be converted to nats_adapter mirror types without duplication.
- **Option B:** Add a single module (e.g. in `api` or `nats_adapter`) that owns **both** directions for the full snapshot: `api::SystemSnapshot` ↔ `pb::SystemSnapshot`, and use it from both `snapshot_publisher` and the TUI. Then remove the hand-written `proto_to_snapshot` in the TUI and the duplicate `*_to_proto` helpers in `snapshot_publisher`.

Reduces duplicate logic and keeps one place to update when proto or domain changes.

### 4.2 TUI: one domain type from proto

- Prefer TUI consuming the **same** domain type as the backend (`api::state::SystemSnapshot`) when it receives a snapshot over NATS. Then proto→domain is done once (in one place, e.g. nats_adapter or api) and TUI uses that type; the existing `RuntimeSnapshotDto` can remain the view for REST JSON and for UI binding if needed (built from `SystemSnapshot` as today via `RuntimeSnapshotDto::from(&snap)`).
- Alternative: have TUI hold and render from `pb::SystemSnapshot` (prost types) to avoid a second type, at the cost of using proto types (e.g. `Option<Timestamp>`) in the UI layer.

### 4.3 Document KV value formats

- In [NATS_API.md](NATS_API.md) or a dedicated **LIVE_STATE / NATS KV** section, document:
  - Key patterns and which component writes them.
  - Value format per pattern: **NatsEnvelope bytes** vs **bare proto** vs **JSON**.
  - Decode steps for consumers (envelope then inner vs direct proto decode vs JSON parse).
- Reduces confusion and prevents wrong decode paths when adding new KV consumers.

### 4.4 Proto for yield curve in KV (implemented)

- **Proto:** `YieldCurve` and `YieldCurvePoint` in `proto/messages.proto`. **Writers:** yield_curve_writer and tws_yield_curve_daemon write proto to `yield_curve.{symbol}`; **reader:** `load_yield_curve_from_kv` tries proto then JSON fallback. Legacy: define a small proto message for “yield curve points” (or reuse existing yield-curve proto if present) and have yield_curve_writer and tws_yield_curve_daemon write **proto** to `yield_curve.{symbol}`.
- api_handlers (build_curve) would decode proto and, if needed, convert to the existing JSON response. Aligns KV with “proto on the wire and in KV” and keeps a single decode path; migration can be phased (e.g. accept both JSON and proto by key convention or content-type).

---

## 5. What we are not doing wrong

- **No double encode/decode:** We do not decode proto, transform to something else, and re-encode to proto for the same logical message.
- **KV write path:** collection_aggregation stores the bytes it received; no extra encode step. snapshot_publisher writes one proto (positions) or envelope (subject) per key; yield curve is one JSON write.
- **REST:** JSON path is domain→DTO→JSON; proto path is domain→proto→bytes. Single conversion each.

The improvements above are about **consistency**, **maintainability**, and **clear contracts** for NATS and KV, not about removing a redundant second transformation of the same protobuf.
