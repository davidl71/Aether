# NATS Integration - Parallel Implementation Plan

**Date**: 2025-11-22
**Status**: Active Implementation
**Goal**: Implement NATS integration across all 4 languages in parallel

---

## Parallel Work Strategy

### ✅ Foundation Complete (Phase 1)
- NATS server deployed and running
- Rust backend integration complete (reference implementation)
- Topic registry and validation layer
- Message schemas defined
- Health monitoring

### 🚀 Phase 2: Multi-Language Integration (Parallel)

All 4 language integrations are **independent** and can be worked on in parallel:

1. **C++ TWS Client** - Publish market data
2. **Python Strategy Runner** - Subscribe to market data, publish signals/decisions
3. **TypeScript Frontend** - Subscribe to real-time updates
4. **Swift iPad App** - Subscribe to real-time updates

---

## Implementation Tasks

### Task 1: C++ TWS Client Integration
**Task ID**: T-20251122115544
**Status**: Ready to start
**Dependencies**: None
**Complexity**: Medium

**Work Items**:
- [ ] Add nats.c library to CMake
- [ ] Create NATS connection manager class
- [ ] Add NATS publishing to `tickPrice()` callback
- [ ] Add NATS publishing to `tickSize()` callback
- [ ] Implement message serialization (JSON)
- [ ] Add error handling and reconnection logic
- [ ] Test with running NATS server

**Files to Modify**:
- `native/src/tws_client.cpp` - Add NATS publishing in callbacks
- `native/src/tws_client.h` - Add NATS connection member
- `native/CMakeLists.txt` - Add nats.c dependency
- `CMakeLists.txt` - Find NATS library

---

### Task 2: Python Strategy Runner Integration
**Task ID**: T-20251122115545
**Status**: Ready to start
**Dependencies**: None
**Complexity**: Low-Medium

**Work Items**:
- [ ] Add `nats-py` to requirements.txt
- [ ] Create NATS client wrapper class
- [ ] Subscribe to `market-data.tick.>` topics
- [ ] Publish to `strategy.signal.>` topics
- [ ] Publish to `strategy.decision.>` topics
- [ ] Add async message handling
- [ ] Test with mock data

**Files to Create/Modify**:
- `python/integration/nats_client.py` - NATS client wrapper
- `python/integration/strategy_runner.py` - Add NATS integration
- `python/requirements.txt` - Add nats-py

---

### Task 3: TypeScript Frontend Integration
**Task ID**: T-20251122115546
**Status**: Ready to start
**Dependencies**: None
**Complexity**: Medium

**Work Items**:
- [ ] Add `nats.ws` or `nats` package
- [ ] Create NATS connection service
- [ ] Subscribe to market data topics
- [ ] Subscribe to strategy topics
- [ ] Update UI with real-time data
- [ ] Handle reconnection
- [ ] Test with WebSocket connection

**Files to Create/Modify**:
- `web/src/services/nats.ts` - NATS connection service
- `web/src/hooks/useNATS.ts` - React hook for NATS
- `web/package.json` - Add nats.ws dependency

---

### Task 4: Swift iPad App Integration
**Task ID**: T-20251122115547
**Status**: Ready to start
**Dependencies**: None
**Complexity**: Medium-High

**Work Items**:
- [ ] Add SwiftNATS package dependency
- [ ] Create NATS manager class
- [ ] Subscribe to market data topics
- [ ] Subscribe to strategy topics
- [ ] Update UI with real-time data
- [ ] Handle background/foreground transitions
- [ ] Test on iPad simulator

**Files to Create/Modify**:
- `app/NATSManager.swift` - NATS connection manager
- `app/ContentView.swift` - Add NATS subscriptions
- `app/Package.swift` - Add SwiftNATS dependency

---

## Parallel Execution Plan

### Phase A: Setup & Dependencies (Can do in parallel)
1. **C++**: Add nats.c to CMake, verify build
2. **Python**: Add nats-py to requirements, test import
3. **TypeScript**: Add nats.ws to package.json, test import
4. **Swift**: Add SwiftNATS to Package.swift, test import

### Phase B: Core Integration (Can do in parallel)
1. **C++**: Implement NATS connection and publishing in TWS client
2. **Python**: Implement NATS subscription and publishing in strategy runner
3. **TypeScript**: Implement NATS connection and subscriptions in frontend
4. **Swift**: Implement NATS connection and subscriptions in iPad app

### Phase C: Testing & Validation (Can do in parallel)
1. **C++**: Test market data publishing with nats CLI
2. **Python**: Test strategy signal/decision publishing
3. **TypeScript**: Test real-time UI updates
4. **Swift**: Test real-time UI updates on device

---

## Success Criteria

### C++ Integration
- ✅ Market data ticks published to `market-data.tick.{symbol}`
- ✅ Messages in correct JSON format
- ✅ Automatic reconnection on failure
- ✅ No impact on existing TWS client functionality

### Python Integration
- ✅ Subscribes to market data topics
- ✅ Publishes strategy signals to `strategy.signal.>`
- ✅ Publishes strategy decisions to `strategy.decision.>`
- ✅ Handles async message processing

### TypeScript Integration
- ✅ Connects to NATS via WebSocket
- ✅ Subscribes to market data and strategy topics
- ✅ Updates UI in real-time
- ✅ Handles connection failures gracefully

### Swift Integration
- ✅ Connects to NATS server
- ✅ Subscribes to market data and strategy topics
- ✅ Updates UI in real-time
- ✅ Handles app lifecycle (background/foreground)

---

## Reference Implementation

The Rust backend integration (`agents/backend/services/backend_service/src/nats_integration.rs`) serves as the reference for:
- Message format (JSON with metadata)
- Topic naming conventions
- Error handling patterns
- Reconnection logic
- DLQ handling

---

## Next Steps

1. **Start with Phase A** - Set up dependencies for all 4 languages
2. **Move to Phase B** - Implement core integration in parallel
3. **Complete Phase C** - Test and validate each integration
4. **Integration Testing** - Test all components together

---

## Notes

- All integrations should follow the same message format
- Use the topic registry from `agents/backend/crates/nats_adapter/src/topics.rs`
- Implement graceful degradation (continue without NATS if unavailable)
- Add logging for debugging
- Follow existing code style for each language
