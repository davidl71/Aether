# NATS Integration - Project Split Impact Analysis

**Date**: 2025-11-22
**Purpose**: Analyze how project separation and broker-agnostic refactoring affects NATS message queue integration

---

## Executive Summary

**NATS integration is largely unaffected by the project split** because:
1. ✅ NATS is broker-agnostic by design (coordination layer, not broker-specific)
2. ✅ NATS adapter crate is already independent and reusable
3. ✅ Message schemas are broker-agnostic (market data, strategy signals, decisions)
4. ✅ Topic registry is broker-agnostic

**However, Phase 2 integration work is now more complex** because:
- ⚠️ Extracted `box-spread-cpp` and `box-spread-python` need NATS client integration
- ⚠️ Multiple repositories need NATS client libraries (C++, Python, TypeScript, Swift)
- ⚠️ Message schemas need to be shared across repositories

---

## Current NATS Architecture

### Code Location
```
agents/backend/
├── crates/
│   └── nats_adapter/          # ✅ Broker-agnostic crate
│       ├── src/
│       │   ├── bridge.rs      # Tokio ↔ NATS bridge
│       │   ├── client.rs      # NATS client wrapper
│       │   ├── topics.rs      # Topic registry (broker-agnostic)
│       │   └── serde.rs       # Message serialization
│       └── tests/
└── services/
    └── backend_service/
        └── src/
            └── nats_integration.rs  # Integration module
```

### Dependencies
- ✅ **Broker-agnostic**: Uses `strategy` crate types (`StrategySignal`, `Decision`)
- ✅ **No broker-specific code**: No TWS, IBKR, or Alpaca references
- ✅ **Standard libraries**: `async-nats`, `serde`, `tokio`

### Message Types (All Broker-Agnostic)
1. **Market Data Tick**: `{symbol, bid, ask, timestamp}` - No broker-specific fields
2. **Strategy Signal**: `StrategySignal` from strategy crate - Broker-agnostic
3. **Strategy Decision**: `Decision` from strategy crate - Broker-agnostic

---

## Impact of Project Split

### ✅ Positive Impacts

#### 1. Broker-Agnostic Architecture Aligns with NATS
- The `IBroker` interface refactoring makes NATS integration cleaner
- All broker adapters (TWS, Alpaca, IB Client Portal) can publish to the same NATS topics
- No broker-specific message formats needed

#### 2. Clear Separation of Concerns
- **NATS adapter crate**: Pure coordination layer (can stay in main repo or be extracted)
- **Broker adapters**: Implement `IBroker` interface, publish to NATS
- **Strategy logic**: Broker-agnostic, publishes decisions to NATS

#### 3. Multi-Repository Support
- Each extracted repository can have its own NATS client
- Message schemas can be shared via documentation or shared types
- Topic registry can be duplicated or shared

### ⚠️ Challenges Introduced

#### 1. Phase 2 Integration Complexity

**Before Split:**
- C++ NATS client: Integrate into `native/src/tws_client.cpp`
- Python NATS client: Integrate into `python/integration/`
- TypeScript NATS client: Integrate into `web/src/`

**After Split:**
- C++ NATS client: Integrate into `box-spread-cpp` library OR main repo's `native/src/`
- Python NATS client: Integrate into `box-spread-python` package OR main repo's `python/`
- TypeScript NATS client: Integrate into main repo's `web/src/` (not extracted)
- Swift NATS client: Integrate into main repo's `ios/` (not extracted)

**Decision Needed**: Should NATS clients be in extracted libraries or main repo?

#### 2. Message Schema Sharing

**Problem**: Message schemas need to be consistent across:
- Rust backend (main repo)
- C++ library (`box-spread-cpp`)
- Python package (`box-spread-python`)
- TypeScript frontend (main repo)
- Swift iPad app (main repo)

**Options**:
1. **Documentation-only**: Define schemas in `docs/message_schemas/` (current approach)
2. **Shared types**: Extract message types to a shared repository
3. **Code generation**: Generate types from a schema definition (JSON Schema, Protobuf)

**Recommendation**: Keep documentation approach for now, consider code generation if drift occurs.

#### 3. Topic Registry Duplication

**Current**: Topic registry in `agents/backend/crates/nats_adapter/src/topics.rs`

**After Split**: Each repository needs topic definitions:
- Rust backend: `nats_adapter/src/topics.rs` ✅
- C++ library: Need topic constants/definitions
- Python package: Need topic constants/definitions
- TypeScript frontend: Need topic constants/definitions
- Swift iPad app: Need topic constants/definitions

**Options**:
1. **Duplicate**: Copy topic definitions to each repository
2. **Shared docs**: Document topics in `trading-api-docs` repository
3. **Code generation**: Generate topic constants from a source of truth

**Recommendation**: Document in `trading-api-docs`, duplicate constants in each language.

---

## Recommendations

### 1. Keep NATS Adapter in Main Repo (For Now)

**Rationale**:
- NATS adapter is tightly coupled to Rust backend service
- It's not a standalone library (depends on `strategy` crate)
- Main repo is where coordination happens

**Future Consideration**: If NATS adapter becomes reusable across projects, extract to `trading-nats-adapter` repository.

### 2. Integrate NATS Clients into Extracted Libraries

**For `box-spread-cpp`**:
- Add optional NATS client integration
- Publish market data, strategy signals to NATS
- Use `nats.c` library (official C client)
- Make it optional (compile-time flag: `USE_NATS`)

**For `box-spread-python`**:
- Add optional NATS client integration
- Publish strategy signals, decisions to NATS
- Use `nats.py` library (asyncio client)
- Make it optional (runtime check: `if nats_available:`)

**Rationale**: Extracted libraries should be self-contained and optionally support NATS.

### 3. Document Message Schemas in `trading-api-docs`

**Action**: Add NATS message schema documentation to `trading-api-docs` repository:
- JSON Schema definitions
- Type definitions for each language
- Example messages
- Versioning strategy

### 4. Create Topic Registry Documentation

**Action**: Document topic registry in `trading-api-docs`:
- Complete topic list
- Topic naming conventions
- Wildcard patterns
- Usage examples per language

### 5. Phase 2 Integration Plan Update

**Updated Tasks**:
- **T-Phase2-C++**: Integrate `nats.c` into `box-spread-cpp` library
- **T-Phase2-Python**: Integrate `nats.py` into `box-spread-python` package
- **T-Phase2-TypeScript**: Integrate `nats.js` into main repo's `web/`
- **T-Phase2-Swift**: Integrate `nats.swift` into main repo's `ios/`

**New Task**: **T-Phase2-Schemas**: Document and validate message schemas across all repositories

---

## Architecture After Split

### Message Flow (Broker-Agnostic)

```
┌─────────────────────────────────────────────────────────────┐
│                    NATS Server                              │
│              (nats://localhost:4222)                        │
│                                                              │
│  Topics:                                                     │
│  - market-data.tick.{symbol}                                 │
│  - strategy.signal.>                                         │
│  - strategy.decision.>                                       │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼──────┐   ┌────────▼────────┐  ┌──────▼──────┐
│ Rust Backend │   │ box-spread-cpp  │  │ box-spread- │
│ (Main Repo)  │   │ (Extracted)     │  │ python      │
│              │   │                  │  │ (Extracted) │
│ - Publishes  │   │ - Optional NATS │  │ - Optional  │
│   market     │   │   client         │  │   NATS      │
│   data       │   │ - Publishes      │  │   client    │
│ - Publishes  │   │   market data    │  │ - Publishes │
│   signals    │   │ - Uses IBroker   │  │   signals   │
│ - Publishes  │   │   interface      │  │             │
│   decisions  │   │                  │  │             │
└──────────────┘   └──────────────────┘  └─────────────┘
        │
        │ (Subscribes)
        │
┌───────▼──────┐
│ Web Frontend │
│ (Main Repo)  │
│              │
│ - Subscribes │
│   to topics  │
│ - Real-time  │
│   updates    │
└──────────────┘
```

### Key Points

1. **All components use broker-agnostic message formats**
2. **Broker adapters (TWS, Alpaca, etc.) implement `IBroker` and publish to NATS**
3. **Extracted libraries optionally support NATS**
4. **Main repo coordinates via NATS**

---

## Action Items

### Immediate (No Changes Needed)
- ✅ NATS adapter crate remains in main repo
- ✅ Current integration continues to work
- ✅ Broker-agnostic refactoring doesn't break NATS

### Short Term (Documentation)
1. **Add NATS schemas to `trading-api-docs`**:
   - Message type definitions
   - JSON Schema files
   - Language-specific examples

2. **Document topic registry in `trading-api-docs`**:
   - Complete topic list
   - Naming conventions
   - Usage patterns

### Medium Term (Phase 2 Integration)
1. **Integrate NATS into `box-spread-cpp`**:
   - Add `nats.c` dependency (optional)
   - Publish market data, signals
   - Document in library README

2. **Integrate NATS into `box-spread-python`**:
   - Add `nats.py` dependency (optional)
   - Publish signals, decisions
   - Document in package README

3. **Integrate NATS into frontends**:
   - TypeScript: `nats.js` in `web/`
   - Swift: `nats.swift` in `ios/`

### Long Term (If Needed)
1. **Extract NATS adapter** to `trading-nats-adapter` repository if it becomes reusable
2. **Code generation** for message schemas if drift occurs
3. **Shared topic registry** via code generation if duplication becomes problematic

---

## Conclusion

**NATS integration is well-positioned for the project split**:
- ✅ Already broker-agnostic
- ✅ No breaking changes required
- ✅ Architecture aligns with extracted libraries

**Main impact**: Phase 2 integration work is more distributed but follows the same patterns. The broker-agnostic refactoring actually makes NATS integration cleaner and more consistent.

**Next Steps**:
1. Document message schemas in `trading-api-docs`
2. Plan Phase 2 NATS client integration for extracted libraries
3. Update Phase 2 tasks to reflect multi-repository structure
