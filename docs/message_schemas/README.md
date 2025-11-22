# NATS Message Schemas

This directory contains JSON schema definitions for all NATS messages used in the system.

## Schema Format

All schemas follow JSON Schema Draft 7 format and include:
- Message type identifier
- Required fields
- Field types and constraints
- Example payloads

## Message Types

### Market Data Messages
- `MarketDataTick.json` - Real-time price tick updates
- `MarketDataCandle.json` - OHLCV candle data
- `MarketDataQuote.json` - Bid/ask quote updates

### Strategy Messages
- `StrategySignal.json` - Market signals for strategy evaluation
- `StrategyDecision.json` - Trading decisions from strategy engine
- `StrategyStatus.json` - Strategy state changes

### Order Messages
- `OrderRequest.json` - New order submission
- `OrderStatus.json` - Order status updates
- `OrderFill.json` - Order fill notifications

### Position Messages
- `PositionUpdate.json` - Position changes
- `PositionSnapshot.json` - Full position snapshot

### Risk Messages
- `RiskCheck.json` - Risk validation requests
- `RiskDecision.json` - Risk check results
- `RiskLimitEvent.json` - Risk limit events

### System Messages
- `SystemEvent.json` - System-wide events
- `Alert.json` - Alert notifications
- `HealthStatus.json` - System health status

## Usage

These schemas can be used for:
- Code generation (TypeScript types, Rust structs, etc.)
- Message validation
- API documentation
- Testing

## Validation

Messages should be validated against these schemas before publishing to NATS.
