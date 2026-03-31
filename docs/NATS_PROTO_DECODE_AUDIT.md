# NATS protobuf/envelope decode audit

This doc supports `T-1774968959479487000` (“audit envelope decode sites for avoidable decode→encode”).

## Scope

Review decode sites that are on high-frequency paths and decide whether each decode is:

- **Keep**: required to compute a derived view / side effect.
- **Refactor**: we can avoid decode, decode less, or reuse decoded values instead of re-decoding.
- **Profile**: unclear; measure first.

## Findings (current codebase)

### `services/backend_service/src/collection_aggregation.rs`

- **Envelope decode**: `NatsEnvelope::decode(payload)` in `handle_message`.
  - **Decision**: **Keep**
  - **Why**: We need `message_type` (for KV key) and `payload` (for ILP conversion). This is a single decode per message.

- **MarketDataEvent decode**: `MarketDataEvent::decode(envelope.payload.as_slice())` inside `market_data_ilp_line`.
  - **Decision**: **Keep**, with a small “decode only when needed” guard already present.
  - **Why**: Decode is conditional on `envelope.message_type == "MarketDataEvent"`. This is the correct place to gate decoding.
  - **Possible micro-refactor** (optional): pass `message_type` as `&str` to avoid cloning it earlier (perf is likely negligible vs decode).

- **KV write**: `kv.put(... payload.to_vec().into())`
  - **Decision**: **Profile**
  - **Why**: This clones bytes for KV; if KV is enabled and traffic is high, this can dominate. It’s not decode-related, but it’s a likely hotspot.

### `crates/api/src/health.rs`

- **Dual decode fallback**: `backend_health_from_message` tries `NatsEnvelope::decode` then `BackendHealth::decode(payload)`, else tries `BackendHealth::decode(data)`.
  - **Decision**: **Keep** (correct for backwards compatibility).
  - **Why**: Supports both enveloped and legacy direct-proto payloads. This is not expected to be a high-frequency hot path relative to market data.

### `crates/nats_adapter/src/serde.rs`

- `decode_envelope` does `NatsEnvelope::decode` and then `T::decode(envelope.payload.as_slice())`.
  - **Decision**: **Keep**
  - **Why**: This is the correct “single place” decode for envelope+payload and avoids ad-hoc decoding in each consumer.

## Recommendation

No obvious decode→encode loops were found in the audited paths. The current patterns already:

- Decode the envelope once per message.
- Gate payload decoding by `message_type` where appropriate.
- Support legacy wire formats safely.

If we chase performance next, the most likely wins are **bytes copies** (KV persistence / buffering), not proto decoding itself.

