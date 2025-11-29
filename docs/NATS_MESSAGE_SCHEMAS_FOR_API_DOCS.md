# NATS Message Schemas - For trading-api-docs Repository

**Date**: 2025-11-22
**Purpose**: Complete NATS message schema documentation to be added to `trading-api-docs` repository
**Status**: Ready for copy to `trading-api-docs/docs/nats/message_schemas/`

---

## Overview

This document contains all NATS message schemas used in the trading system. These schemas define the structure of messages published to and subscribed from NATS topics, enabling cross-language communication between C++, Python, Rust, TypeScript, and Swift components.

## Schema Format

All schemas follow **JSON Schema Draft 7** format and include:
- Message type identifier
- Required fields
- Field types and constraints
- Example payloads
- Metadata (id, timestamp, source, type)

## Common Message Structure

All NATS messages follow this common structure:

```json
{
  "id": "uuid-v4",
  "timestamp": "2025-11-20T10:00:00Z",
  "source": "component-name",
  "type": "MessageType",
  "payload": { ... }
}
```

### Common Fields

- **id**: Unique message identifier (UUID v4)
- **timestamp**: ISO 8601 timestamp (UTC)
- **source**: Component that published the message (e.g., "tws-client", "python-strategy", "rust-backend")
- **type**: Message type identifier (e.g., "MarketDataTick", "StrategyDecision")
- **payload**: Message-specific data (varies by message type)

---

## Message Types

### Market Data Messages

#### MarketDataTick
- **Topic**: `market-data.tick.{symbol}`
- **Purpose**: Real-time price tick updates
- **Publisher**: Market data provider (TWS client, Polygon, etc.)
- **Subscribers**: Strategy engine, frontends
- **Schema**: See `MarketDataTick.json`

#### MarketDataCandle
- **Topic**: `market-data.candle.{symbol}`
- **Purpose**: OHLCV candle data
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine, analytics
- **Schema**: `MarketDataCandle.json` (to be created)

#### MarketDataQuote
- **Topic**: `market-data.quote.{symbol}`
- **Purpose**: Bid/ask quote updates
- **Publisher**: Market data provider
- **Subscribers**: Strategy engine
- **Schema**: `MarketDataQuote.json` (to be created)

---

### Strategy Messages

#### StrategySignal
- **Topic**: `strategy.signal.{symbol}`
- **Purpose**: Market signals for strategy evaluation
- **Publisher**: Market data provider → Strategy engine
- **Subscribers**: Strategy engine
- **Schema**: `StrategySignal.json` (to be created)

#### StrategyDecision
- **Topic**: `strategy.decision.{symbol}`
- **Purpose**: Trading decisions from strategy engine
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine, Order manager, Frontends
- **Schema**: See `StrategyDecision.json`

#### StrategyStatus
- **Topic**: `strategy.status`
- **Purpose**: Strategy state changes (start/stop/pause)
- **Publisher**: Strategy controller
- **Subscribers**: Frontends, monitoring
- **Schema**: `StrategyStatus.json` (to be created)

---

### Order Messages

#### OrderRequest
- **Topic**: `orders.new`
- **Purpose**: New order requests
- **Publisher**: Strategy engine (via risk engine)
- **Subscribers**: Order manager, TWS client
- **Schema**: `OrderRequest.json` (to be created)

#### OrderStatus
- **Topic**: `orders.status.{order_id}`
- **Purpose**: Order status updates
- **Publisher**: Order manager, TWS client
- **Subscribers**: Strategy engine, Frontends
- **Schema**: See `OrderStatus.json`

#### OrderFill
- **Topic**: `orders.fill.{order_id}`
- **Purpose**: Order fill notifications
- **Publisher**: TWS client
- **Subscribers**: Strategy engine, Position manager
- **Schema**: `OrderFill.json` (to be created)

---

### Risk Messages

#### RiskCheck
- **Topic**: `risk.check`
- **Purpose**: Risk validation requests
- **Publisher**: Strategy engine
- **Subscribers**: Risk engine
- **Schema**: `RiskCheck.json` (to be created)

#### RiskDecision
- **Topic**: `risk.decision`
- **Purpose**: Risk check results
- **Publisher**: Risk engine
- **Subscribers**: Strategy engine, Order manager
- **Schema**: See `RiskDecision.json`

---

## Schema Files

The following JSON Schema files should be copied to `trading-api-docs/docs/nats/message_schemas/`:

1. ✅ `MarketDataTick.json` - Real-time price tick updates
2. ✅ `StrategyDecision.json` - Trading decisions from strategy engine
3. ✅ `OrderStatus.json` - Order status updates
4. ✅ `RiskDecision.json` - Risk check results
5. ⏳ `MarketDataCandle.json` - OHLCV candle data (to be created)
6. ⏳ `MarketDataQuote.json` - Bid/ask quote updates (to be created)
7. ⏳ `StrategySignal.json` - Market signals for strategy evaluation (to be created)
8. ⏳ `StrategyStatus.json` - Strategy state changes (to be created)
9. ⏳ `OrderRequest.json` - New order submission (to be created)
10. ⏳ `OrderFill.json` - Order fill notifications (to be created)
11. ⏳ `RiskCheck.json` - Risk validation requests (to be created)

---

## Usage Guidelines

### For Developers

1. **Validate messages** against schemas before publishing
2. **Use consistent field names** across all languages
3. **Include all required fields** in every message
4. **Use ISO 8601 timestamps** (UTC)
5. **Generate UUIDs** for message IDs

### For Code Generation

These schemas can be used to generate:
- TypeScript types (`typescript-json-schema`)
- Rust structs (`schemars`)
- Python dataclasses (`dataclasses-json`)
- C++ structs (manual or code generation)
- Swift structs (manual or code generation)

### For Testing

- Use schemas to validate test messages
- Generate mock messages from schemas
- Verify message structure in integration tests

---

## Versioning

**Current Version**: 1.0
**Schema Format**: JSON Schema Draft 7

**Versioning Strategy**:
- Breaking changes: Increment major version
- New optional fields: Increment minor version
- Bug fixes: Increment patch version

**Backward Compatibility**:
- New fields should be optional
- Deprecated fields should be marked but not removed immediately
- Version number included in message metadata (future enhancement)

---

## Integration with Extracted Libraries

### box-spread-cpp
- Use schemas to define C++ structs for message types
- Validate messages before publishing
- Document topic usage in library README

### box-spread-python
- Use schemas to define Python dataclasses
- Validate messages before publishing
- Document topic usage in package README

### Frontends (TypeScript/Swift)
- Use schemas to generate types
- Validate messages on receipt
- Document subscription patterns

---

## Next Steps

1. ✅ Copy existing schema files to `trading-api-docs`
2. ⏳ Create missing schema files (MarketDataCandle, StrategySignal, etc.)
3. ⏳ Add code generation examples for each language
4. ⏳ Add validation examples
5. ⏳ Document topic-to-schema mapping

---

## References

- [NATS Topics Registry](research/../NATS_TOPICS_REGISTRY.md)
- [NATS Integration Architecture](research/../research/architecture/MESSAGE_QUEUE_ARCHITECTURE.md)
- [JSON Schema Specification](https://json-schema.org/specification.html)

