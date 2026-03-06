# NATS Topics Registry

Complete registry of all NATS topics used in the system. All topics follow the hierarchical naming convention: `{domain}.{entity}.{action}.{identifier}`

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
- **Schema**: `MarketDataTick.json`

### `market-data.candle.{symbol}`

- **Purpose**: OHLCV candle data
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine, analytics
- **Example**: `market-data.candle.XSP`
- **Schema**: `MarketDataCandle.json`

### `market-data.quote.{symbol}`

- **Purpose**: Bid/ask quote updates
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine
- **Example**: `market-data.quote.NDX`
- **Schema**: `MarketDataQuote.json`

### `market-data.volume.{symbol}`

- **Purpose**: Volume updates
- **Publisher**: Market data provider
- **Subscribers**: Analytics
- **Example**: `market-data.volume.SPY`
- **Schema**: `MarketDataVolume.json`

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
- **Schema**: `StrategySignal.json`

### `strategy.decision.{symbol}`

- **Purpose**: Trading decisions from strategy engine
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine, Order manager, Frontends
- **Example**: `strategy.decision.SPY`
- **Schema**: `StrategyDecision.json`

### `strategy.status`

- **Purpose**: Strategy state changes (start/stop/pause)
- **Publisher**: Strategy controller
- **Subscribers**: Frontends, monitoring
- **Schema**: `StrategyStatus.json`

### `strategy.control`

- **Purpose**: Control commands (start/stop/pause)
- **Publisher**: Frontends, REST API
- **Subscribers**: Strategy controller
- **Schema**: `StrategyControl.json`

### Subscription Patterns

- `strategy.signal.>` - All strategy signals
- `strategy.decision.>` - All strategy decisions

## Order Topics

### `orders.new`

- **Purpose**: New order requests
- **Publisher**: Strategy engine (via risk engine)
- **Subscribers**: Order manager, TWS client
- **Schema**: `OrderRequest.json`

### `orders.status.{order_id}`

- **Purpose**: Order status updates
- **Publisher**: Order manager, TWS client
- **Subscribers**: Strategy engine, Frontends
- **Example**: `orders.status.ORD-123`
- **Schema**: `OrderStatus.json`

### `orders.fill.{order_id}`

- **Purpose**: Order fill notifications
- **Publisher**: TWS client
- **Subscribers**: Strategy engine, Position manager
- **Example**: `orders.fill.ORD-123`
- **Schema**: `OrderFill.json`

### `orders.cancel.{order_id}`

- **Purpose**: Order cancellation
- **Publisher**: Order manager, Frontends
- **Subscribers**: TWS client, Strategy engine
- **Example**: `orders.cancel.ORD-123`
- **Schema**: `OrderCancel.json`

### Subscription Patterns

- `orders.status.>` - All order status updates
- `orders.fill.>` - All order fills

## Position Topics

### `positions.update.{symbol}`

- **Purpose**: Position changes
- **Publisher**: Position manager
- **Subscribers**: Frontends, Risk engine
- **Example**: `positions.update.SPY`
- **Schema**: `PositionUpdate.json`

### `positions.snapshot`

- **Purpose**: Full position snapshot
- **Publisher**: Position manager
- **Subscribers**: Frontends (on request)
- **Schema**: `PositionSnapshot.json`

### Subscription Patterns

- `positions.update.>` - All position updates

## Risk Topics

### `risk.check`

- **Purpose**: Risk validation requests
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine
- **Schema**: `RiskCheck.json`

### `risk.decision`

- **Purpose**: Risk check results
- **Publisher**: Risk engine
- **Subscribers**: Strategy engine, Order manager
- **Schema**: `RiskDecision.json`

### `risk.limit.{type}`

- **Purpose**: Risk limit events
- **Publisher**: Risk engine
- **Subscribers**: Frontends, Alerts
- **Example**: `risk.limit.position`
- **Schema**: `RiskLimitEvent.json`

### `risk.violation`

- **Purpose**: Risk limit violations
- **Publisher**: Risk engine
- **Subscribers**: Alerts, Frontends
- **Schema**: `RiskViolation.json`

## System Topics

### `system.health`

- **Purpose**: System health status
- **Publisher**: Backend service
- **Subscribers**: Monitoring, Frontends
- **Schema**: `HealthStatus.json`

### `system.events`

- **Purpose**: System-wide events
- **Publisher**: All components
- **Subscribers**: Monitoring, Logging
- **Schema**: `SystemEvent.json`

### `system.alerts`

- **Purpose**: Alert notifications
- **Publisher**: All components
- **Subscribers**: Frontends, Monitoring
- **Schema**: `Alert.json`

### `system.config`

- **Purpose**: Configuration updates
- **Publisher**: Config manager
- **Subscribers**: All components
- **Schema**: `ConfigUpdate.json`

## Backend snapshot and health (Python backends)

### `snapshot.{backend_id}`

- **Purpose**: Backend snapshot payload (same shape as REST `/api/snapshot` or `/api/v1/snapshot`)
- **Publisher**: Backend service (IB, Alpaca, TradeStation, Tastytrade) when NATS publish enabled
- **Subscribers**: TUI/PWA NATS provider, snapshot aggregator
- **Examples**: `snapshot.ib`, `snapshot.alpaca`, `snapshot.tradestation`
- **Payload (canonical)**: Proto `SystemSnapshot` (see `proto/messages.proto`). REST and NATS should use the same shape; TUI and PWA consume generated types (Python `python/generated/`, TypeScript when ts-proto wired per project tasks).

### Subscription patterns (snapshot)

- `snapshot.>` — All backend snapshots
- `snapshot.ib` — IB snapshot only

### `system.health`

- **Purpose**: Health status from any backend (single topic; payload includes `backend` id)
- **Publisher**: Backend service on each health check when NATS publish enabled
- **Subscribers**: Monitoring, TUI health aggregator, frontends
- **Payload (canonical)**: Proto `BackendHealth` (see `proto/messages.proto`). JSON equivalent: `{"backend": "ib", "status": "ok", "updated_at": "...", "error": "", "hint": "", "extra": {...}}`. Legacy: ad-hoc JSON with `backend`, `status`, `ts`, and backend-specific fields until all publishers use proto.

## RPC (Request/Reply) Topics

### `rpc.strategy.status`

- **Purpose**: Request strategy status
- **Request**: Frontend, Monitoring
- **Reply**: Strategy controller
- **Schema**: `StrategyStatus.json`

### `rpc.system.snapshot`

- **Purpose**: Request system snapshot
- **Request**: Frontend
- **Reply**: Backend service
- **Schema**: `SystemSnapshot.json`

## Dead Letter Queue Topics

### `system.dlq.{component}.{error_type}`

- **Purpose**: Failed messages that couldn't be processed
- **Publisher**: Message processors (on failure)
- **Subscribers**: Monitoring, Debugging
- **Example**: `system.dlq.strategy.deserialization_error`
- **Schema**: `DeadLetterMessage.json`

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
