# NATS decode hotspots (Aether) — JSON/protobuf eager decode audit

**Context:** This document supports `T-1774965480423783000` (“avoid eager protobuf/JSON deserialization in NATS paths”).

## Goal

Find places where we:

- **Decode then immediately re-encode** (wasted CPU/allocs).
- **Decode when not needed** (e.g., only need 1 field, or can treat payload as bytes).
- **Decode as `serde_json::Value`** when we can decode directly to a small struct (or skip decode entirely).
- **Decode protobuf eagerly** in hot paths where a bytes-forwarding approach is viable.

## Hotspot list (current codebase)

### Backend service — NATS API handlers (`agents/backend/services/backend_service/src/handlers/*`)

- **`handlers/strategy.rs`**
  - Uses `serde_json::from_slice(&bytes)` for `ScenarioDto` in request handling.
  - Re-encodes replies with `serde_json::to_vec(...)` in multiple places.
  - Candidate: if callers rarely pass bodies or only pass a tiny schema, consider a lighter request type or “empty request” convention.

- **`handlers/loans.rs`**
  - Repeated `body.as_deref().and_then(|b| serde_json::from_slice(b).ok())` patterns.
  - Many handler branches return `serde_json::to_vec(&r)` even when `r` is already an error DTO.
  - Candidate: parse a minimal request struct once and pass typed data; avoid repeated `Value` parsing.

- **`handlers/calculate.rs`**
  - Multiple request handlers parse request bodies via `serde_json::from_slice`.
  - Candidate: unify parsing/validation path and only parse when request requires it.

- **`handlers/ledger.rs`**
  - Parses `limit` out of JSON as `serde_json::Value` then returns a JSON-encoded DTO.
  - Candidate: define a tiny request struct `{"limit": u64}` and decode directly (or accept “no body = default limit”).

### Backend service — yield curve writer (`agents/backend/services/backend_service/src/yield_curve_writer.rs`)

- Multiple `serde_json::to_vec(...)` calls to build payloads that are published to NATS / KV.
- Candidate: if downstream just forwards or stores, keep payload bytes as bytes and avoid intermediate serde types where possible.

### Backend service — health aggregation and DLQ consumer

- **`health_aggregation.rs` / `dlq_consumer.rs` / `collection_aggregation.rs`**
  - These are long-lived subscription loops. The decode hotspot here is usually “decode everything on receipt” rather than selective decode.
  - Candidate: ensure we only decode payload formats we actually need for the operator surfaces; consider deferring decode to the UI boundary.

### Dev-only benchmark binary (`services/backend_service/src/rlt_nats_api_benchmarks.rs`)

- Uses `request_json_with_timeout` and then `serde_json::to_vec(&response)` to count bytes.
- Candidate: measure “raw bytes round-trip” cost separately (no JSON parse), if `request_json_with_timeout` supports it.

## Recommended next refactor slices (small + safe)

1. **Replace `serde_json::Value` parsing** with tiny request structs for “single-field” requests:
   - `handlers/ledger.rs` limit parsing
   - any `{"limit":...}` / `{"symbol":...}` style payloads

2. **Centralize request parsing** for handlers with multiple endpoints:
   - reduce repeated `from_slice` blocks in `handlers/loans.rs` / `handlers/calculate.rs`

3. **Bytes-forwarding where possible**:
   - if a handler just forwards a payload (or stores it as-is), accept and forward bytes without decoding.

## Internet Research (2026)

🔗 **[NATS Docs: Request-Reply Semantics](https://docs.nats.io/using-nats/developer/sending/request_reply)**
- **Key insight**: request/reply is payload-agnostic; serialization format is application-owned. That makes “bytes-forwarding” a valid optimization when the service doesn’t need to interpret payloads.

🔗 **[NATS by Example (Rust): Request-Reply](https://natsbyexample.com/examples/messaging/request-reply/rust)**
- **Key insight**: canonical Rust request/reply handler patterns; useful for keeping handler loops simple when refactoring parsing.

🔗 **[`serde_json::from_slice` docs](https://docs.rs/serde_json/latest/serde_json/de/fn.from_slice.html)**
- **Key insight**: `from_slice` is full parse+deserialize each call; repeated parsing shows up quickly in hot request loops.

## Notes / assumptions

- This doc is an **audit index**; it does not claim a specific site is the top perf bottleneck without profiling.
- For trading safety, prefer changes that do not alter request semantics; treat “decode less” as an internal implementation detail.

