# NATS Protobuf Migration

**Status:** Infrastructure ready; migration incremental  
**Related:** `native/include/nats_client.h`, `native/src/nats_client.cpp`, `proto/messages.proto`

## Current State

- C++ NATS client uses JSON for `publish_market_data`, `publish_strategy_signal`, `publish_strategy_decision`
- `publish_raw(topic, payload)` added for binary payloads (enables protobuf)
- `just proto-gen` generates C++ from `proto/messages.proto` to `native/generated/proto/`

## Migration Steps

1. Run `just proto-gen` to ensure `native/generated/proto/` has `messages.pb.h` / `messages.pb.cc`
2. Add generated proto to CMake (include dir, link libprotobuf)
3. For each `publish_*` method: build the corresponding protobuf message, call `SerializeToString()`, use `publish_raw()`
4. Update Python/Rust consumers to deserialize protobuf instead of JSON
5. Remove JSON construction from C++ `nats_client.cpp`

## Message Mapping

| Current JSON Type | Protobuf Message |
|------------------|------------------|
| MarketDataTick   | MarketDataEvent (+ NatsEnvelope) |
| StrategySignal   | StrategySignal (+ NatsEnvelope) |
| StrategyDecision| StrategyDecision (+ NatsEnvelope) |
