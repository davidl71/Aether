# C++ to Rust Datapath Contract

**Last updated**: 2026-03-11

## Objective

Define one canonical native-producer to Rust-backend datapath.

```text
C++ native producers
  -> NATS subjects
  -> protobuf NatsEnvelope
  -> Rust collector/state owner
  -> NATS KV LIVE_STATE + QuestDB
  -> Rust client-facing APIs
```

## Required wire format

All platform events on this datapath must use:

- concrete NATS subjects for publish
- `ib.platform.v1.NatsEnvelope`
- protobuf inner payloads from [messages.proto](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/proto/messages.proto)

Envelope fields:

- `id`
- `timestamp`
- `source`
- `message_type`
- `payload`

## Canonical subjects

Publish subjects:

- `market-data.tick.<symbol>` -> `MarketDataEvent`
- `strategy.signal.<symbol>` -> `StrategySignal`
- `strategy.decision.<symbol>` -> `StrategyDecision`
- `system.health` -> `BackendHealth` or `NatsEnvelope(BackendHealth)`

Subscribe subjects:

- `market-data.tick.>`
- `strategy.signal.>`
- `strategy.decision.>`
- `system.health`

Rule:

- wildcard subjects are subscriber-only
- publishers must publish to concrete symbol-scoped subjects

## Ownership

- C++ owns native event production while the native engine remains in place
- Rust owns collection, `LIVE_STATE`, QuestDB fanout, and client-facing state/API ownership
- clients should consume Rust-owned state and APIs, not raw native payload conventions

## LIVE_STATE contract

Bucket:

- `LIVE_STATE`

Key format:

- `messageType.symbol`
- examples:
  - `MarketDataEvent.SPY`
  - `StrategySignal.XSP`
  - `StrategyDecision.XSP`

Value format:

- full serialized `NatsEnvelope`

## Minimum event set

The active unified datapath currently requires:

- `MarketDataEvent`
- `StrategySignal`
- `StrategyDecision`
- `BackendHealth`

## References

- [nats_client.cpp](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/native/src/nats_client.cpp)
- [topics.rs](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/agents/backend/crates/nats_adapter/src/topics.rs)
- [collection_aggregation.rs](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/agents/backend/services/backend_service/src/collection_aggregation.rs)
- [nats_integration.rs](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/agents/backend/services/backend_service/src/nats_integration.rs)
- [messages.proto](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/proto/messages.proto)
