# NATS Dead Letter Queue (DLQ) Implementation

**Date**: 2025-11-20
**Task**: T-195
**Status**: ✅ **COMPLETE**

## Overview

Dead Letter Queue (DLQ) implementation for NATS integration provides automatic retry logic with exponential backoff and failed message capture. Messages that fail to publish after retries are sent to a DLQ topic for monitoring and debugging.

## Features

### ✅ Implemented

1. **Automatic Retry Logic**
   - Configurable max retries (default: 3)
   - Exponential backoff with configurable multiplier
   - Configurable initial and max delay

2. **Dead Letter Queue Topics**
   - Topic format: `system.dlq.{component}.{error_type}`
   - Component-specific DLQ topics
   - Error-type specific categorization

3. **Error Classification**
   - `publish_error` - NATS publish failures
   - `serialization_error` - Message serialization failures
   - `connection_error` - NATS connection failures
   - `validation_error` - Topic validation failures
   - `unknown_error` - Other errors

4. **DLQ Message Schema**
   - Original message payload preserved
   - Error details and retry count
   - Timestamp and metadata
   - JSON Schema defined

5. **Integration**
   - DLQ-enabled publishers in backend service
   - Automatic DLQ publishing on failure
   - Non-blocking error handling

## Configuration

### Default Configuration

```rust
DlqConfig {
    max_retries: 3,
    initial_retry_delay_ms: 100,
    max_retry_delay_ms: 5000,
    backoff_multiplier: 2.0,
    enabled: true,
}
```

### Retry Delay Calculation

Retry delays use exponential backoff:

- Attempt 0: 100ms
- Attempt 1: 200ms
- Attempt 2: 400ms
- Attempt 3: 800ms (capped at max_retry_delay_ms)

## Usage

### Creating DLQ-Enabled Publishers

```rust
use nats_adapter::{ChannelBridge, DlqService, NatsClient};

// Create NATS client
let client = NatsClient::connect("nats://localhost:4222").await?;

// Create DLQ service
let dlq_service = DlqService::new(client.clone(), "backend");

// Create bridge
let bridge = ChannelBridge::new(client.clone());

// Create publisher with DLQ
let publisher = bridge.create_publisher_with_dlq(
    "market-data.tick.SPY",
    "backend",
    "MarketDataTick",
    dlq_service,
);

// Publish message (automatic retries + DLQ on failure)
publisher.publish(tick).await?;
```

### Custom DLQ Configuration

```rust
use nats_adapter::{DlqConfig, DlqService};

let config = DlqConfig {
    max_retries: 5,
    initial_retry_delay_ms: 200,
    max_retry_delay_ms: 10000,
    backoff_multiplier: 1.5,
    enabled: true,
};

let dlq_service = DlqService::with_config(client, "backend", config);
```

## DLQ Topics

### Topic Structure

- **Format**: `system.dlq.{component}.{error_type}`
- **Component**: Source component (e.g., "backend", "strategy", "market-data")
- **Error Type**: Type of error (e.g., "publish_error", "serialization_error")

### Examples

- `system.dlq.backend.publish_error` - Backend publish failures
- `system.dlq.strategy.serialization_error` - Strategy serialization failures
- `system.dlq.market-data.connection_error` - Market data connection failures

### Subscribing to DLQ

```bash

# Subscribe to all DLQ messages

nats sub "system.dlq.>"

# Subscribe to backend DLQ messages

nats sub "system.dlq.backend.>"

# Subscribe to specific error type

nats sub "system.dlq.backend.publish_error"
```

## DLQ Message Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-11-20T16:00:00Z",
  "original_topic": "market-data.tick.SPY",
  "component": "backend",
  "error_type": "publish_error",
  "error_message": "Failed to publish message: connection closed",
  "retry_count": 3,
  "original_payload": {
    "symbol": "SPY",
    "bid": 100.5,
    "ask": 100.6,
    "timestamp": "2025-11-20T16:00:00Z"
  },
  "metadata": {
    "source": "backend",
    "message_type": "MarketDataTick"
  }
}
```

## Integration Points

### Backend Service

All NATS publishers in the backend service are now DLQ-enabled:

1. **Market Data Publishers** - Per-symbol publishers with DLQ
2. **Strategy Signal Publisher** - Strategy signals with DLQ
3. **Strategy Decision Publisher** - Strategy decisions with DLQ

### Error Handling Flow

```
1. Publish attempt fails
2. Retry with exponential backoff (up to max_retries)
3. If all retries fail → Send to DLQ
4. Log warning with DLQ topic
5. Return error to caller
```

## Monitoring

### Subscribe to DLQ Messages

```bash

# Monitor all failed messages

nats sub "system.dlq.>" --count 0

# Monitor backend failures

nats sub "system.dlq.backend.>" --count 0
```

### Health Checks

DLQ service status can be monitored via:

- NATS server monitoring endpoints
- Application health endpoints
- DLQ topic subscription counts

## Testing

### Test Scenarios

1. **Connection Failure** - NATS server down
   - Messages retry and eventually go to DLQ
   - Error type: `connection_error`

2. **Serialization Failure** - Invalid message format
   - Immediate DLQ (no retries for serialization errors)
   - Error type: `serialization_error`

3. **Publish Failure** - NATS server rejects message
   - Retries with backoff, then DLQ
   - Error type: `publish_error`

### Test Commands

```bash

# Stop NATS server to test connection errors

./scripts/stop_nats.sh

# Start backend service (will retry and send to DLQ)

cd agents/backend && cargo run -p backend_service

# Monitor DLQ in another terminal

nats sub "system.dlq.>"
```

## Files Created/Modified

### New Files

- `agents/backend/crates/nats_adapter/src/dlq.rs` - DLQ service implementation
- `proto/messages.proto` - Canonical message schemas (replaced JSON schemas)
- `docs/NATS_DLQ_IMPLEMENTATION.md` - This documentation

### Modified Files

- `agents/backend/crates/nats_adapter/src/bridge.rs` - Added DLQ support to Publisher
- `agents/backend/crates/nats_adapter/src/topics.rs` - Added DLQ topic functions
- `agents/backend/crates/nats_adapter/src/lib.rs` - Exported DLQ types
- `agents/backend/services/backend_service/src/nats_integration.rs` - Integrated DLQ into publishers

## Configuration Options

### Environment Variables

DLQ can be configured via environment variables (future enhancement):

- `NATS_DLQ_ENABLED` - Enable/disable DLQ (default: true)
- `NATS_DLQ_MAX_RETRIES` - Max retry attempts (default: 3)
- `NATS_DLQ_INITIAL_DELAY_MS` - Initial retry delay (default: 100)
- `NATS_DLQ_MAX_DELAY_MS` - Max retry delay (default: 5000)

## Future Enhancements

1. **DLQ Replay** - Replay failed messages from DLQ
2. **DLQ Analytics** - Track DLQ message rates and patterns
3. **Automatic Recovery** - Auto-retry DLQ messages after conditions improve
4. **DLQ Expiration** - Auto-delete old DLQ messages
5. **DLQ Alerting** - Alert when DLQ message rate exceeds threshold

## Success Criteria

- ✅ DLQ service implemented
- ✅ Retry logic with exponential backoff
- ✅ DLQ topics defined and documented
- ✅ DLQ message schema created
- ✅ Backend service integrated with DLQ
- ✅ Error classification implemented
- ✅ Documentation complete
- ✅ Code compiles successfully

## Conclusion

Dead Letter Queue implementation is complete and integrated into the NATS adapter. All publishers in the backend service now have automatic retry logic and DLQ support. Failed messages are captured in DLQ topics for monitoring and debugging.

---

**Next Steps**: Runtime testing to verify DLQ behavior under failure conditions.
