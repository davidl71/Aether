# LEAN Integration Analysis for PWA/TUI

**Date**: 2025-11-18
**Status**: Analysis Complete
**Purpose**: Evaluate whether LEAN can be used in PWA/TUI and identify duplicate functionality

---

## Executive Summary

**Key Finding**: LEAN is a **headless execution engine** - it does NOT provide a web UI or TUI. However, LEAN can handle backend execution, and you may be duplicating some functionality that LEAN already provides internally.

**Recommendation**: Use LEAN for execution, but you still need to build:

1. REST API wrapper around LEAN to expose data to UIs
2. PWA/TUI frontends (these are still needed)
3. Real-time data bridge from LEAN to UIs

---

## What LEAN Provides

### ✅ Execution Engine Capabilities

1. **Algorithm Execution**
   - Backtesting with historical data
   - Paper trading with real market data
   - Live trading with real execution
   - Multi-broker support (IBKR, Alpaca, etc.)

2. **Market Data Management**
   - Subscription to options chains
   - Real-time quote handling
   - Historical data access
   - Data normalization

3. **Order Management** (Internal)
   - Order placement
   - Order tracking
   - Fill handling
   - Position management
   - Portfolio tracking

4. **Risk Management** (Internal)
   - Position limits
   - Margin calculations
   - Portfolio risk metrics

### ❌ What LEAN Does NOT Provide

1. **No Web UI** - LEAN is headless, no built-in web interface
2. **No TUI** - No terminal UI provided
3. **No REST API** - LEAN doesn't expose HTTP endpoints for UIs
4. **No WebSocket** - No real-time push notifications for UIs
5. **No Dashboard** - No visualization or analytics UI

**LEAN is purely an execution engine** - you run algorithms, it executes trades, but you need to build UIs separately.

---

## Current Architecture

### Current Stack

```
┌─────────────────────────────────────────────────────────┐
│                    PWA (React/TypeScript)               │
│  - Dashboard UI                                          │
│  - Charts/Visualization                                  │
│  - Strategy Controls                                     │
└──────────────────────┬──────────────────────────────────┘
                       │ REST API
┌──────────────────────▼──────────────────────────────────┐
│              Rust Backend (agents/backend/)             │
│  - REST API (/api/v1/snapshot, /strategy/start, etc.)  │
│  - Market data ingestion                                │
│  - Strategy execution (NautilusTrader or LEAN)         │
│  - Position/order tracking                              │
│  - Risk management                                      │
└──────────────────────┬──────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│   TWS API    │ │  Alpaca   │ │   ORATS   │
│   (IBKR)     │ │   API     │ │    API    │
└──────────────┘ └───────────┘ └───────────┘
```

### Current REST API Endpoints

From `agents/backend/crates/api/src/rest.rs`:

- `GET /health` - Health check
- `GET /api/v1/snapshot` - System snapshot (positions, orders, metrics)
- `POST /api/v1/strategy/start` - Start strategy
- `POST /api/v1/strategy/stop` - Stop strategy

From `agents/shared/API_CONTRACT.md`:

- `GET /api/v1/account/summary` - Account summary
- `GET /api/v1/account/positions` - Positions
- `GET /api/v1/strategy/status` - Strategy status
- `GET /api/v1/strategy/stats` - Strategy statistics
- `GET /api/v1/orders/recent` - Recent orders
- `POST /api/v1/orders/cancel` - Cancel order
- `POST /api/v1/combos/buy` - Buy combo order
- `POST /api/v1/combos/sell` - Sell combo order

---

## LEAN Integration Architecture

### Proposed Architecture with LEAN

```
┌─────────────────────────────────────────────────────────┐
│                    PWA (React/TypeScript)               │
│  - Dashboard UI (STILL NEEDED)                          │
│  - Charts/Visualization (STILL NEEDED)                  │
│  - Strategy Controls (STILL NEEDED)                     │
└──────────────────────┬──────────────────────────────────┘
                       │ REST API (STILL NEEDED)
┌──────────────────────▼──────────────────────────────────┐
│         REST API Wrapper (NEW - Python/Rust)            │
│  - Expose LEAN data via REST                            │
│  - Bridge LEAN events to WebSocket                      │
│  - Strategy control endpoints                           │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              LEAN Engine (C#)                           │
│  - Algorithm execution                                  │
│  - Market data handling                                 │
│  - Order execution                                      │
│  - Position/portfolio tracking (INTERNAL)              │
└──────────────────────┬──────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│   TWS API    │ │  Alpaca   │ │   ORATS   │
│   (IBKR)     │ │   API     │ │    API    │
└──────────────┘ └───────────┘ └───────────┘
```

### Key Difference

**With LEAN**: You need to build a **REST API wrapper** that:

1. Runs LEAN algorithm
2. Exposes LEAN's internal state (positions, orders, portfolio) via REST
3. Provides strategy control endpoints (start/stop)
4. Bridges LEAN events to WebSocket for real-time updates

**Without LEAN**: You build everything yourself (current approach with Rust backend).

---

## Duplicate Functionality Analysis

### What You're Currently Building That LEAN Already Does

| Feature | Current Implementation | LEAN Provides | Duplication? |
|---------|----------------------|---------------|--------------|
| **Order Management** | Rust backend tracks orders | ✅ LEAN tracks orders internally | ⚠️ **Partial** - LEAN tracks internally, but you need to expose via REST |
| **Position Tracking** | Rust backend tracks positions | ✅ LEAN tracks positions internally | ⚠️ **Partial** - LEAN tracks internally, but you need to expose via REST |
| **Portfolio Management** | Rust backend calculates P&L | ✅ LEAN calculates P&L internally | ⚠️ **Partial** - LEAN calculates internally, but you need to expose via REST |
| **Market Data Subscription** | Rust backend subscribes to TWS | ✅ LEAN handles subscriptions | ✅ **Yes** - LEAN can handle this |
| **Order Execution** | Rust backend places orders | ✅ LEAN places orders | ✅ **Yes** - LEAN can handle this |
| **Multi-Broker Support** | Building adapters (T-35, T-36, T-37) | ✅ LEAN has adapters | ✅ **Yes** - LEAN already has this |
| **Risk Management** | Rust backend risk checks | ✅ LEAN has risk management | ⚠️ **Partial** - LEAN has basic risk, you may need custom logic |
| **Backtesting** | Not implemented | ✅ LEAN has backtesting | ✅ **Yes** - LEAN provides this |
| **Dashboard UI** | React PWA | ❌ LEAN has no UI | ❌ **No** - You still need to build this |
| **TUI** | C++ FTXUI | ❌ LEAN has no TUI | ❌ **No** - You still need to build this |
| **REST API** | Rust backend REST API | ❌ LEAN has no REST API | ❌ **No** - You still need to build this |

### Key Insight

**LEAN does a lot internally, but you can't access it directly from UIs.** You need to build a bridge layer.

---

## What You Still Need to Build (Even with LEAN)

### 1. REST API Wrapper (Required)

**Purpose**: Expose LEAN's internal state to PWA/TUI

**What it needs to do**:

- Query LEAN's portfolio/positions
- Query LEAN's order history
- Provide strategy control (start/stop LEAN algorithm)
- Expose metrics (P&L, ROI, etc.)

**Implementation Options**:

**Option A: Python Wrapper**

```python
# python/lean_integration/api_wrapper.py
from fastapi import FastAPI
from lean_integration.lean_client import LeanClient

app = FastAPI()
lean = LeanClient()

@app.get("/api/v1/snapshot")
async def get_snapshot():
    # Query LEAN's portfolio
    portfolio = lean.get_portfolio()
    positions = lean.get_positions()
    orders = lean.get_orders()

    return {
        "positions": positions,
        "orders": orders,
        "metrics": calculate_metrics(portfolio)
    }

@app.post("/api/v1/strategy/start")
async def start_strategy():
    lean.start_algorithm()
    return {"status": "started"}
```

**Option B: Rust Wrapper (Call Python)**

```rust
// agents/backend/crates/lean_api/src/lib.rs
// Call Python LEAN wrapper via PyO3
```

### 2. Real-Time Updates (Required)

**Purpose**: Push LEAN events to PWA/TUI via WebSocket

**What it needs to do**:

- Subscribe to LEAN order events
- Subscribe to LEAN position updates
- Push updates to connected clients

**Implementation**:

```python
# python/lean_integration/websocket_bridge.py
from fastapi import WebSocket
from lean_integration.lean_client import LeanClient

async def bridge_lean_events(websocket: WebSocket):
    lean = LeanClient()

    # Subscribe to LEAN events
    async for event in lean.event_stream():
        # Forward to WebSocket clients
        await websocket.send_json(event.to_dict())
```

### 3. PWA/TUI Frontends (Still Required)

**Purpose**: User interface for viewing and controlling strategy

**What they provide**:

- Dashboard visualization
- Charts and analytics
- Strategy controls (start/stop)
- Position/order viewing
- Real-time updates

**These are still needed** - LEAN doesn't provide UIs.

---

## Comparison: Current vs LEAN Approach

### Current Approach (Rust Backend)

**Pros**:

- ✅ Full control over implementation
- ✅ Direct REST API (no wrapper needed)
- ✅ Best performance (Rust)
- ✅ Already partially implemented
- ✅ No C# dependency

**Cons**:

- ❌ Need to build multi-broker adapters (T-35, T-36, T-37)
- ❌ Need to implement order/position tracking
- ❌ Need to implement backtesting (if desired)
- ❌ More code to maintain

### LEAN Approach

**Pros**:

- ✅ Multi-broker support built-in
- ✅ Order/position tracking built-in
- ✅ Backtesting built-in
- ✅ Large community and documentation
- ✅ Battle-tested execution engine

**Cons**:

- ❌ Need to build REST API wrapper
- ❌ Need to bridge LEAN events to WebSocket
- ❌ C# runtime dependency
- ❌ Additional latency (C# interop)
- ❌ Less control over implementation
- ❌ Migration effort (5-7 weeks)

---

## Recommendation

### Option 1: Continue Current Approach (Recommended)

**Rationale**:

- Already partially implemented
- Best performance (Rust)
- Full control
- No C# dependency
- Lower latency

**What to do**:

- Complete multi-broker adapters (T-35, T-36, T-37)
- Keep existing REST API
- Keep PWA/TUI as-is
- Use LEAN only for backtesting (optional)

### Option 2: Migrate to LEAN (If Multi-Broker is Priority)

**Rationale**:

- Excellent multi-broker support
- Built-in backtesting
- Less code to maintain (for execution)

**What to do**:

- Build REST API wrapper around LEAN
- Build WebSocket bridge for real-time updates
- Keep PWA/TUI (they're still needed)
- Use LEAN for execution
- Keep C++ calculations (via Cython)

**Architecture**:

```
PWA/TUI → REST API Wrapper → LEAN → Brokers
              ↑
         C++ Calculations (via Cython)
```

---

## What You're NOT Duplicating

### ✅ Still Need to Build

1. **PWA Frontend** - LEAN has no web UI
2. **TUI Frontend** - LEAN has no terminal UI
3. **REST API** - LEAN has no REST API
4. **WebSocket Bridge** - LEAN has no WebSocket
5. **Dashboard Visualization** - LEAN has no charts/analytics
6. **User Controls** - LEAN has no UI controls

### ⚠️ Partially Duplicating

1. **Order/Position Tracking** - LEAN does this internally, but you need to expose it
2. **Portfolio Management** - LEAN does this internally, but you need to expose it
3. **Risk Management** - LEAN has basic risk, you may need custom logic

### ✅ Can Use LEAN For

1. **Multi-Broker Execution** - LEAN has adapters
2. **Order Placement** - LEAN handles this
3. **Market Data Subscription** - LEAN handles this
4. **Backtesting** - LEAN provides this

---

## Conclusion

### Can LEAN be used in PWA/TUI?

**Short answer**: LEAN can handle **execution**, but PWA/TUI are still needed for **user interface**.

**Long answer**:

- ✅ LEAN can replace backend execution logic
- ❌ LEAN cannot replace PWA/TUI (it has no UI)
- ⚠️ You still need to build REST API wrapper to expose LEAN data
- ⚠️ You still need to build WebSocket bridge for real-time updates

### Are you duplicating functionality?

**Yes, partially**:

- Order/position tracking (LEAN does this internally, but you're also tracking)
- Multi-broker adapters (LEAN has these, you're building your own)
- Market data subscription (LEAN handles this, you're also handling it)

**But**:

- PWA/TUI are NOT duplicates - LEAN has no UI
- REST API is NOT a duplicate - LEAN has no REST API
- Dashboard/visualization are NOT duplicates - LEAN has no visualization

### Final Recommendation

**For PWA/TUI**: Keep building them - they're still needed regardless of execution engine.

**For Backend**:

- **Option A (Recommended)**: Continue with Rust backend, complete multi-broker adapters
- **Option B**: Migrate to LEAN if multi-broker support is critical and you're willing to build REST wrapper

**Key Point**: Using LEAN doesn't eliminate the need for PWA/TUI - it only changes what the backend does. You still need UIs and REST APIs.

---

## Next Steps

1. **Decision**: Choose execution engine (Rust backend vs LEAN)
2. **If LEAN**: Build REST API wrapper (new task)
3. **If LEAN**: Build WebSocket bridge (new task)
4. **Either way**: Continue PWA/TUI development (still needed)
5. **Either way**: Continue REST API development (still needed)

---

## References

- [LEAN GitHub](https://github.com/QuantConnect/Lean)
- [LEAN Documentation](https://www.quantconnect.com/docs/v2/lean-engine)
- [LEAN Python API](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/algorithm-structure)
- Current REST API: `agents/backend/crates/api/src/rest.rs`
- Current API Contract: `agents/shared/API_CONTRACT.md`
