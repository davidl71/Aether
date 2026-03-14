# NATS Consumer Audit

**Last updated**: 2026-03-12
**Purpose**: Verify all active NATS consumers use protobuf-only transport.

## Findings

### Active Rust Consumers (protobuf-only)

| Consumer | Subscribes To | Wire Format | Location |
|----------|---------------|-------------|----------|
| **tui_service** | `snapshot.{backend_id}` | Protobuf `SystemSnapshot` in envelope | `agents/backend/services/tui_service/src/nats.rs` |
| **collection_aggregation** | `market-data.>`, `strategy.>`, `system.>` | Protobuf `NatsEnvelope` → inner payload | `agents/backend/services/backend_service/src/collection_aggregation.rs` |
| **health_aggregation** | `system.health` | Protobuf `BackendHealth` in envelope | `agents/backend/services/backend_service/src/health_aggregation.rs` |
| **snapshot_publisher** | `snapshot.{backend_id}` | Protobuf `SystemSnapshot` as envelope | `agents/backend/services/backend_service/src/snapshot_publisher.rs` |
| **REST live state** | NATS KV (`LIVE_STATE`) | Reads envelope metadata for REST exposure | `agents/backend/crates/api/src/rest.rs` |

### Retired / Inactive

| Component | Status | Notes |
|-----------|--------|-------|
| **Python NATS consumers** | Retired | `python/tui/providers/_nats.py` was part of deleted Python TUI |
| **Python nats_client** | Inactive | Still exists in `python/integration/nats_client.py` but not used by active runtime |

## Conclusion

**All active NATS consumers use protobuf-only transport via the `nats_adapter` crate.** No JSON or raw payload consumers remain in the active codebase.

The canonical encoding/decoding utilities live in `agents/backend/crates/nats_adapter/src/serde.rs`:

- `encode_envelope()` — wraps proto in `NatsEnvelope`
- `decode_envelope()` — extracts proto from `NatsEnvelope`
- `extract_proto_payload()` — extracts inner proto from envelope for subscribers

## Related Documentation

- `docs/platform/DATAFLOW_ARCHITECTURE.md` — NATS contract and data flow
- `docs/NATS_TOPICS_REGISTRY.md` — NATS subject names and payload semantics
- `agents/backend/crates/nats_adapter/` — NATS adapter library
