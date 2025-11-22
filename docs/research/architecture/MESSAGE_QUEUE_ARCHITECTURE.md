# NATS Message Queue Integration Architecture

**Date**: 2025-11-20
**Purpose**: Detailed architecture design for integrating NATS message queue into the IBKR Box Spread multi-language trading application.

## Executive Summary

This document defines the architecture for integrating NATS as the central message queue coordination layer, enabling event-driven communication between C++, Python, Rust, Go, and TypeScript components. The design prioritizes low latency, simplicity, and gradual migration from existing coordination mechanisms.

## Architecture Overview

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         NATS Server                              │
│                    (nats://localhost:4222)                       │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Core NATS  │  │  JetStream   │  │   WebSocket  │         │
│  │   (Pub/Sub)  │  │ (Persistence)│  │   (Frontend) │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼──────┐   ┌────────▼────────┐  ┌──────▼──────┐
│   C++ TWS    │   │  Rust Backend   │  │  Python     │
│   Client     │   │   Service       │  │  Strategy   │
│              │   │                 │  │  Runner     │
│  [nats.c]    │   │  [async-nats]   │  │  [nats.py]  │
└───────┬──────┘   └────────┬────────┘  └──────┬──────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼──────┐   ┌────────▼────────┐  ┌──────▼──────┐
│ TypeScript   │   │   Swift iPad    │  │   Go        │
│ Web Frontend │   │      App        │  │  Services   │
│              │   │                 │  │             │
│ [nats.js]    │   │  [nats.swift]   │  │ [nats.go]   │
└──────────────┘   └─────────────────┘  └─────────────┘
```

## Topic Hierarchy

### Subject Naming Convention

NATS uses hierarchical subject names with dot (`.`) separators. Our convention:

```
{domain}.{entity}.{action}.{identifier}
```

### Core Topics

#### Market Data Topics
```
market-data.tick.{symbol}              # Real-time tick updates
market-data.candle.{symbol}            # OHLCV candle updates
market-data.quote.{symbol}             # Bid/ask quote updates
market-data.volume.{symbol}            # Volume updates
```

#### Strategy Topics
```
strategy.signal.{symbol}               # Market signals for strategy
strategy.decision.{symbol}             # Trading decisions
strategy.status                        # Strategy state changes
strategy.control                       # Control commands (start/stop)
```

#### Order Topics
```
orders.new                             # New order requests
orders.status.{order_id}               # Order status updates
orders.fill.{order_id}                # Order fill notifications
orders.cancel.{order_id}               # Order cancellation
```

#### Position Topics
```
positions.update.{symbol}              # Position changes
positions.snapshot                     # Full position snapshot
```

#### Risk Topics
```
risk.check                             # Risk check requests
risk.decision                          # Risk check results
risk.limit.{type}                     # Risk limit events
risk.violation                         # Risk limit violations
```

#### System Topics
```
system.health                          # System health status
system.events                          # System-wide events
system.alerts                          # Alert notifications
system.config                          # Configuration updates
```

#### Request/Reply Topics
```
rpc.snapshot                           # Request full snapshot
rpc.positions                          # Request positions
rpc.account                            # Request account info
rpc.strategy.status                    # Request strategy status
```

## Message Schemas

All messages use JSON encoding for cross-language compatibility. See `docs/message_schemas/` for detailed schemas.

### Common Message Structure

```json
{
  "id": "uuid-v4",
  "timestamp": "2025-11-20T10:00:00Z",
  "source": "component-name",
  "type": "message-type",
  "payload": { ... }
}
```

### Key Message Types

1. **MarketDataTick** - Real-time price updates
2. **StrategySignal** - Market signals for strategy evaluation
3. **StrategyDecision** - Trading decisions from strategy engine
4. **OrderRequest** - New order submission
5. **OrderStatus** - Order status updates
6. **PositionUpdate** - Position changes
7. **RiskCheck** - Risk validation requests
8. **RiskDecision** - Risk check results
9. **SystemEvent** - System-wide events
10. **Alert** - Alert notifications

See `docs/message_schemas/` for complete schema definitions.

## Integration Points by Language

### 1. Rust Backend Service

**Location**: `agents/backend/services/backend_service/src/main.rs`

**Integration Strategy**:
- Replace Tokio channels with NATS adapters
- Bridge existing channels to NATS topics
- Maintain backward compatibility during migration

**Key Changes**:
```rust
// Before: Tokio channels
let (strategy_signal_tx, strategy_signal_rx) = mpsc::unbounded_channel();

// After: NATS adapter
let nats_client = async_nats::connect("nats://localhost:4222").await?;
let strategy_signal_tx = NatsAdapter::new(nats_client, "strategy.signal".into());
```

**Components to Integrate**:
- Market data provider → `market-data.*` topics
- Strategy engine → `strategy.*` topics
- Risk engine → `risk.*` topics
- Order manager → `orders.*` topics
- REST/gRPC servers → Subscribe to relevant topics

**New Module**: `agents/backend/crates/nats_adapter/`
- NATS client wrapper
- Channel-to-NATS bridge
- Message serialization/deserialization
- Error handling and reconnection

### 2. C++ TWS Client

**Location**: `native/src/tws_client.cpp`

**Integration Strategy**:
- Add NATS C client (`nats.c`)
- Bridge TWS callbacks to NATS messages
- Publish market data events
- Subscribe to order commands

**Key Changes**:
```cpp
// Add NATS client
#include <nats/nats.h>
natsConnection* nc = nullptr;
natsConnection_ConnectTo(&nc, "nats://localhost:4222");

// In TWS callback
void TWSClient::onTickPrice(TickPrice tick) {
    // Existing logic...

    // Publish to NATS
    natsMsg* msg = createMarketDataMessage(tick);
    natsConnection_Publish(nc, "market-data.tick.SPY", msg);
    natsMsg_Destroy(msg);
}
```

**New Files**:
- `native/include/nats_bridge.h` - NATS bridge interface
- `native/src/nats_bridge.cpp` - NATS bridge implementation
- Update `CMakeLists.txt` to link `libnats.a`

**CMake Integration**:
```cmake
find_library(NATS_LIB nats PATHS /usr/local/lib)
target_link_libraries(ib_box_spread ${NATS_LIB})
```

### 3. Python Strategy Runner

**Location**: `python/integration/strategy_runner.py`

**Integration Strategy**:
- Add `nats-py` dependency
- Subscribe to strategy signals
- Publish strategy decisions
- Async/await pattern

**Key Changes**:
```python
import asyncio
from nats.aio.client import Client as NATS

async def strategy_runner():
    nc = NATS()
    await nc.connect("nats://localhost:4222")

    # Subscribe to signals
    async def signal_handler(msg):
        signal = json.loads(msg.data)
        decision = evaluate_strategy(signal)
        await nc.publish("strategy.decision", json.dumps(decision))

    await nc.subscribe("strategy.signal.>", cb=signal_handler)
```

**Dependencies**:
- Add `nats-py` to `requirements.txt`
- Update `python/setup.py` if needed

### 4. TypeScript Web Frontend

**Location**: `web/src/`

**Integration Strategy**:
- Use NATS WebSocket connection
- Replace REST polling with NATS subscriptions
- Real-time updates for all data

**Key Changes**:
```typescript
import { connect, NatsConnection } from 'nats.ws';

const nc: NatsConnection = await connect({
  servers: ['ws://localhost:4222'],
});

// Subscribe to snapshot updates
const sub = nc.subscribe('system.snapshot', {
  callback: (err, msg) => {
    if (msg) {
      const snapshot = JSON.parse(new TextDecoder().decode(msg.data));
      updateUI(snapshot);
    }
  },
});
```

**New Files**:
- `web/src/services/natsClient.ts` - NATS client wrapper
- `web/src/hooks/useNatsSubscription.ts` - React hook for subscriptions
- Update `web/package.json` to include `nats.ws`

### 5. Swift iPad App

**Location**: `ios/BoxSpreadIPad/`

**Integration Strategy**:
- Use `nats.swift` client
- Replace REST polling with NATS subscriptions
- Real-time position and order updates

**Key Changes**:
```swift
import Nats

let connection = try await NatsConnection.connect(url: "nats://localhost:4222")

// Subscribe to positions
let subscription = try await connection.subscribe(subject: "positions.update.>") { message in
    let position = try JSONDecoder().decode(PositionUpdate.self, from: message.data)
    updateUI(position)
}
```

**Dependencies**:
- Add `nats.swift` via Swift Package Manager
- Update `Package.swift`

### 6. Go Services (Future)

**Location**: Future Go components

**Integration Strategy**:
- Use `nats.go` client
- Standard Go patterns

**Example**:
```go
nc, err := nats.Connect("nats://localhost:4222")
if err != nil {
    log.Fatal(err)
}
defer nc.Close()

nc.Subscribe("market-data.tick.>", func(msg *nats.Msg) {
    // Process message
})
```

## Deployment Architecture

### Single Server Deployment (Development)

```
┌─────────────────────────────────────┐
│         Development Machine        │
│                                     │
│  ┌──────────────┐                  │
│  │  NATS Server │  :4222           │
│  │  (Standalone)│                  │
│  └──────────────┘                  │
│                                     │
│  ┌──────────────┐  ┌─────────────┐ │
│  │ Rust Backend │  │  C++ TWS    │ │
│  │   Service    │  │   Client    │ │
│  └──────────────┘  └─────────────┘ │
│                                     │
│  ┌──────────────┐  ┌─────────────┐ │
│  │ Python       │  │ TypeScript │ │
│  │ Strategy     │  │   Web      │ │
│  └──────────────┘  └─────────────┘ │
└─────────────────────────────────────┘
```

### Clustered Deployment (Production)

```
┌─────────────────────────────────────────────────────┐
│              NATS Cluster                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ NATS-1   │  │ NATS-2   │  │ NATS-3   │         │
│  │ :4222    │  │ :4222    │  │ :4222    │         │
│  └──────────┘  └──────────┘  └──────────┘         │
└─────────────────────────────────────────────────────┘
         │              │              │
    ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
    │ Backend │    │  TWS    │    │Strategy │
    │ Service │    │ Client  │    │ Runner  │
    └─────────┘    └─────────┘    └─────────┘
```

### Configuration

**NATS Server Config** (`nats-server.conf`):
```conf
port: 4222
http_port: 8222
cluster {
  port: 6222
  routes = [
    "nats://nats-1:6222"
    "nats://nats-2:6222"
    "nats://nats-3:6222"
  ]
}

jetstream {
  store_dir: "/var/lib/nats/jetstream"
}
```

**Client Configuration**:
- Connection URL: `nats://localhost:4222` (dev) or cluster URLs
- Reconnect: Automatic with exponential backoff
- Timeout: 5 seconds
- Max reconnect attempts: Unlimited

## Integration Sequence Diagrams

### Market Data Flow

```
C++ TWS Client          NATS Server          Rust Backend          TypeScript Frontend
     │                      │                      │                      │
     │──Tick Price─────────>│                      │                      │
     │                      │──market-data.tick───>│                      │
     │                      │                      │──Update State────────>│
     │                      │                      │                      │
     │                      │                      │<─Subscribe───────────│
     │                      │                      │                      │
     │                      │<─market-data.tick────│                      │
     │                      │                      │                      │
     │                      │──────────────────────┼──market-data.tick───>│
     │                      │                      │                      │
```

### Strategy Decision Flow

```
Market Data            NATS Server          Python Strategy        Rust Backend
     │                      │                      │                      │
     │──Tick───────────────>│                      │                      │
     │                      │──strategy.signal───>│                      │
     │                      │                      │                      │
     │                      │                      │──Evaluate───────────>│
     │                      │                      │                      │
     │                      │<─strategy.decision───│                      │
     │                      │                      │                      │
     │                      │──strategy.decision──>│                      │
     │                      │                      │                      │
     │                      │                      │──Risk Check─────────>│
     │                      │                      │                      │
     │                      │<─risk.decision───────│                      │
     │                      │                      │                      │
     │                      │──orders.new─────────>│                      │
```

### Order Execution Flow

```
Rust Backend          NATS Server          C++ TWS Client          TypeScript Frontend
     │                      │                      │                      │
     │──orders.new─────────>│                      │                      │
     │                      │──orders.new─────────>│                      │
     │                      │                      │                      │
     │                      │                      │──Place Order────────>│
     │                      │                      │                      │
     │                      │<─orders.status───────│                      │
     │                      │                      │                      │
     │<─orders.status───────│                      │                      │
     │                      │                      │                      │
     │                      │──────────────────────┼──orders.status──────>│
     │                      │                      │                      │
```

## Error Handling and Retry Strategies

### Connection Errors

**Strategy**: Automatic reconnection with exponential backoff

```rust
// Rust example
let nc = async_nats::connect("nats://localhost:4222")
    .await
    .context("Failed to connect to NATS")?;

// Automatic reconnection built into async-nats
```

**Backoff Schedule**:
- Initial: 1 second
- Max: 30 seconds
- Multiplier: 2x

### Message Delivery Errors

**Strategy**: Dead Letter Queue (DLQ) for failed messages

```
Topic: system.dlq.{component}.{error_type}
```

**Retry Policy**:
- Max retries: 3
- Retry delay: 1s, 5s, 30s
- After max retries: Send to DLQ

### Circuit Breaker Pattern

**Implementation**: Per-component circuit breaker

```rust
struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: Mutex<Instant>,
    state: AtomicU8, // Closed, Open, HalfOpen
}
```

**States**:
- **Closed**: Normal operation
- **Open**: Failures exceed threshold, reject requests
- **HalfOpen**: Test if service recovered

## Migration Strategy

### Phase 1: Foundation (Week 1-2)
1. Deploy NATS server
2. Create Rust NATS adapter crate
3. Integrate Rust backend with NATS (parallel to existing channels)
4. Test with mock data

### Phase 2: Core Integration (Week 3-4)
1. Bridge C++ TWS client to NATS
2. Connect Python strategy runner
3. Replace REST polling in TypeScript frontend
4. Monitor performance and latency

### Phase 3: Full Migration (Week 5-6)
1. Remove Tokio channels (replace with NATS)
2. Add Swift iPad app integration
3. Implement JetStream for persistence (if needed)
4. Performance optimization

### Phase 4: Production Hardening (Week 7-8)
1. Add monitoring and alerting
2. Implement circuit breakers
3. Load testing
4. Documentation and runbooks

## Performance Considerations

### Latency Targets
- Market data: < 1ms (publish to subscribe)
- Strategy decisions: < 5ms (end-to-end)
- Order execution: < 10ms (request to confirmation)

### Throughput Targets
- Market data: 10,000 messages/second
- Strategy signals: 1,000 messages/second
- Order updates: 500 messages/second

### Optimization Strategies
1. **Message Batching**: Batch multiple updates into single message
2. **Compression**: Enable message compression for large payloads
3. **Connection Pooling**: Reuse connections across components
4. **Selective Subscriptions**: Subscribe only to needed topics

## Security Considerations

### Authentication
- Use NATS credentials file for authentication
- Per-component credentials with least privilege

### Authorization
- Subject-based permissions
- Read-only vs read-write access

### Encryption
- TLS for all connections (production)
- Certificate-based authentication

### Example Config
```conf
authorization {
  users = [
    { user: "backend", password: "$2a$...", permissions: { publish: ["strategy.*", "orders.*"], subscribe: ["market-data.*"] } }
    { user: "frontend", password: "$2a$...", permissions: { subscribe: ["system.*", "positions.*"] } }
  ]
}
```

## Monitoring and Observability

### Metrics to Track
- Message publish rate (per topic)
- Message subscribe rate (per topic)
- Message latency (publish to receive)
- Connection status (per component)
- Error rates (per component)
- Queue depths (if using JetStream)

### Tools
- NATS monitoring: `nats-server --http_port 8222`
- Prometheus exporter: `nats-prometheus-exporter`
- Grafana dashboards: Custom dashboards for trading metrics

### Logging
- Structured logging (JSON)
- Correlation IDs for message tracing
- Component-level log aggregation

## Testing Strategy

### Unit Tests
- Message serialization/deserialization
- NATS adapter logic
- Error handling

### Integration Tests
- End-to-end message flow
- Multi-component coordination
- Failure scenarios

### Performance Tests
- Latency benchmarks
- Throughput tests
- Load testing

### Test Environment
- Local NATS server for development
- Docker Compose for integration tests
- Staging cluster for pre-production

## Dependencies

### New Dependencies

**Rust**:
```toml
[dependencies]
async-nats = "0.32"
serde_json = "1.0"
```

**C++**:
- `libnats.a` (NATS C client library)
- CMake integration

**Python**:
```txt
nats-py>=2.6.0
```

**TypeScript**:
```json
{
  "dependencies": {
    "nats.ws": "^2.0.0"
  }
}
```

**Swift**:
- `nats.swift` via Swift Package Manager

## Next Steps

1. **Review and Approve**: Architecture review with team
2. **Prototype**: Build minimal NATS integration proof-of-concept
3. **Implement Phase 1**: Foundation setup
4. **Iterate**: Gradual migration following phases

## References

- [NATS Documentation](https://docs.nats.io/)
- [NATS Architecture Guide](https://docs.nats.io/nats-concepts/architecture)
- [Message Queue Research](./MESSAGE_QUEUE_RESEARCH.md)
- [Component Coordination Analysis](./COMPONENT_COORDINATION_ANALYSIS.md)
