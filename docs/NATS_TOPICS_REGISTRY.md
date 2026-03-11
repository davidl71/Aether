# NATS Topics Registry

Complete registry of all NATS topics used in the system. All topics follow the hierarchical naming convention: `{domain}.{entity}.{action}.{identifier}`

**Note**: All topics use protobuf (`NatsEnvelope` with inner message). JSON schemas below are deprecated — see [`proto/messages.proto`](../../proto/messages.proto) for canonical definitions.

## Topic Naming Convention

- **Hierarchical**: `domain.entity.action.identifier`
- **Wildcards**:
  - `>` matches one or more tokens (e.g., `market-data.>` matches all market data)
  - `*` matches exactly one token (e.g., `market-data.*.SPY` matches any action for SPY)

- **Validation**: All topics are validated using `nats_adapter::topics::validate_topic()`

## Market Data Topics

### `market-data.tick.{symbol}`

- **Purpose**: Real-time price tick updates
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine, frontends
- **Example**: `market-data.tick.SPY`
- **Proto**: `MarketDataTick` (from `proto/messages.proto`)

### `market-data.candle.{symbol}`

- **Purpose**: OHLCV candle data
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine, analytics
- **Example**: `market-data.candle.XSP`
- **Proto**: `MarketDataCandle`

### `market-data.quote.{symbol}`

- **Purpose**: Bid/ask quote updates
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine
- **Example**: `market-data.quote.NDX`
- **Proto**: `MarketDataQuote`

### `market-data.volume.{symbol}`

- **Purpose**: Volume updates
- **Publisher**: Market data provider
- **Subscribers**: Analytics
- **Example**: `market-data.volume.SPY`
- **Proto**: `MarketDataVolume`

### Subscription Patterns

- `market-data.>` - All market data for all symbols
- `market-data.tick.>` - All tick updates
- `market-data.tick.SPY` - Tick updates for SPY only

## Strategy Topics

### `strategy.signal.{symbol}`

- **Purpose**: Market signals for strategy evaluation
- **Publisher**: Market data provider → Strategy engine
- **Subscribers**: Strategy engine
- **Example**: `strategy.signal.XSP`
- **Proto**: `StrategySignal`

### `strategy.decision.{symbol}`

- **Purpose**: Trading decisions from strategy engine
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine, Order manager, Frontends
- **Example**: `strategy.decision.SPY`
- **Proto**: `StrategyDecision`

### `strategy.status`

- **Purpose**: Strategy state changes (start/stop/pause)
- **Publisher**: Strategy controller
- **Subscribers**: Frontends, monitoring
- **Proto**: `StrategyStatus`

### `strategy.control`

- **Purpose**: Control commands (start/stop/pause)
- **Publisher**: Frontends, REST API
- **Subscribers**: Strategy controller
- **Proto**: `StrategyControl`

### Subscription Patterns

- `strategy.signal.>` - All strategy signals
- `strategy.decision.>` - All strategy decisions

## Order Topics

### `orders.new`

- **Purpose**: New order requests
- **Publisher**: Strategy engine (via risk engine)
- **Subscribers**: Order manager, TWS client
- **Proto**: `OrderRequest`

### `orders.status.{order_id}`

- **Purpose**: Order status updates
- **Publisher**: Order manager, TWS client
- **Subscribers**: Strategy engine, Frontends
- **Example**: `orders.status.ORD-123`
- **Proto**: `OrderStatus`

### `orders.fill.{order_id}`

- **Purpose**: Order fill notifications
- **Publisher**: TWS client
- **Subscribers**: Strategy engine, Position manager
- **Example**: `orders.fill.ORD-123`
- **Proto**: `OrderFill`

### `orders.cancel.{order_id}`

- **Purpose**: Order cancellation
- **Publisher**: Order manager, Frontends
- **Subscribers**: TWS client, Strategy engine
- **Example**: `orders.cancel.ORD-123`
- **Proto**: `OrderCancel`

### Subscription Patterns

- `orders.status.>` - All order status updates
- `orders.fill.>` - All order fills

## Position Topics

### `positions.update.{symbol}`

- **Purpose**: Position changes
- **Publisher**: Position manager
- **Subscribers**: Frontends, Risk engine
- **Example**: `positions.update.SPY`
- **Proto**: `PositionUpdate`

### `positions.snapshot`

- **Purpose**: Full position snapshot
- **Publisher**: Position manager
- **Subscribers**: Frontends (on request)
- **Proto**: `PositionSnapshot`

### Subscription Patterns

- `positions.update.>` - All position updates

## Risk Topics

### `risk.check`

- **Purpose**: Risk validation requests
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine
- **Proto**: `RiskCheck`

### `risk.decision`

- **Purpose**: Risk check results
- **Publisher**: Risk engine
- **Subscribers**: Strategy engine, Order manager
- **Proto**: `RiskDecision`

### `risk.limit.{type}`

- **Purpose**: Risk limit events
- **Publisher**: Risk engine
- **Subscribers**: Frontends, Alerts
- **Example**: `risk.limit.position`
- **Proto**: `RiskLimitEvent`

### `risk.violation`

- **Purpose**: Risk limit violations
- **Publisher**: Risk engine
- **Subscribers**: Alerts, Frontends
- **Proto**: `RiskViolation`

## System Topics

### `system.health`

- **Purpose**: System health status
- **Publisher**: Backend service
- **Subscribers**: Monitoring, Frontends
- **Proto**: `HealthStatus`

### `system.events`

- **Purpose**: System-wide events
- **Publisher**: All components
- **Subscribers**: Monitoring, Logging
- **Proto**: `SystemEvent`

### `system.alerts`

- **Purpose**: Alert notifications
- **Publisher**: All components
- **Subscribers**: Frontends, Monitoring
- **Proto**: `Alert`

### `system.config`

- **Purpose**: Configuration updates
- **Publisher**: Config manager
- **Subscribers**: All components
- **Proto**: `ConfigUpdate`

## Backend snapshot and health (Python backends)

### `snapshot.{backend_id}`

- **Purpose**: Backend snapshot payload (same shape as REST `/api/snapshot` or `/api/v1/snapshot`)
- **Publisher**: Backend service (IB, Alpaca, Tastytrade) when NATS publish enabled
- **Subscribers**: TUI/PWA NATS provider, snapshot aggregator
- **Examples**: `snapshot.ib`, `snapshot.alpaca`
- **Payload (canonical)**: Proto `SystemSnapshot` (see `proto/messages.proto`). REST and NATS should use the same shape; TUI and PWA consume generated types (Python `python/generated/`, TypeScript when ts-proto wired per project tasks).

### Subscription patterns (snapshot)

- `snapshot.>` — All backend snapshots
- `snapshot.ib` — IB snapshot only

### `system.health`

- **Purpose**: Health status from any backend (single topic; payload includes `backend` id)
- **Publisher**: Backend service on each health check when NATS publish enabled
- **Subscribers**: Monitoring, TUI health aggregator, frontends
- **Payload (canonical)**: Proto `BackendHealth` (see `proto/messages.proto`). All publishers must use protobuf; the legacy JSON format is deprecated.

## RPC (Request/Reply) Topics

### `rpc.strategy.status`

- **Purpose**: Request strategy status
- **Request**: Frontend, Monitoring
- **Reply**: Strategy controller
- **Proto**: `StrategyStatus`

### `rpc.system.snapshot`

- **Purpose**: Request system snapshot
- **Request**: Frontend
- **Reply**: Backend service
- **Proto**: `SystemSnapshot`

## Dead Letter Queue Topics

### `system.dlq.{component}.{error_type}`

- **Purpose**: Failed messages that couldn't be processed
- **Publisher**: Message processors (on failure)
- **Subscribers**: Monitoring, Debugging
- **Example**: `system.dlq.strategy.deserialization_error`
- **Proto**: `DeadLetterMessage`

## Usage in Code

```rust
use nats_adapter::topics;

// Generate topic names
let tick_topic = topics::market_data::tick("SPY");
let signal_topic = topics::strategy::signal("XSP");

// Validate topics
topics::validate_topic(&tick_topic)?;

// Use with publisher/subscriber
let publisher = bridge.create_publisher(
    topics::market_data::tick("SPY"),
    "backend",
    "MarketDataTick"
);
```

## Topic Validation

All topics must pass validation:

- Not empty
- Not start/end with `.`
- No consecutive `.`
- Valid characters only (alphanumeric, `.`, `-`, `_`, `>`, `*`)
- Max 256 characters

Use `nats_adapter::topics::validate_topic()` before publishing/subscribing.

## Collision Prevention

- All topics are defined in this registry
- Use topic generation functions (e.g., `topics::market_data::tick()`) instead of string literals
- Validation prevents invalid topic names
- Documentation ensures consistent usage
