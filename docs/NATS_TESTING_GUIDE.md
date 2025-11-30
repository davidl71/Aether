# NATS Integration Testing Guide

**Date**: 2025-11-20
**Status**: ✅ Complete

## Overview

This guide covers testing the NATS integration with mock data, including manual testing procedures, automated tests, and performance validation.

## Prerequisites

1. **NATS Server Running**

   ```bash
   ./scripts/start_nats.sh
   ```

2. **nats CLI Tool** (for manual testing)

   ```bash
   brew install nats-io/nats-tools/nats
   ```

3. **Backend Service Built**

   ```bash
   cd agents/backend
   cargo build -p backend_service
   ```

## Test Scripts

### Automated Test Script

Run the comprehensive test suite:

```bash
./scripts/test_nats_integration.sh [test-type]
```

**Test Types:**

- `basic` - Basic publish/subscribe
- `market-data` - Market data topic validation
- `strategy` - Strategy topic validation
- `performance` - Latency measurement
- `error` - Error handling verification
- `all` - Run all tests (default)

**Example:**

```bash

# Run all tests

./scripts/test_nats_integration.sh

# Run only performance tests

./scripts/test_nats_integration.sh performance
```

### Integration Tests (Rust)

Run Rust integration tests:

```bash
cd agents/backend
cargo test -p backend_service --test integration_test -- --ignored
```

**Note:** Tests are marked with `#[ignore]` and require a running NATS server.

## Manual Testing Procedures

### 1. Test Market Data Publishing

**Terminal 1 - Subscribe:**

```bash
nats sub "market-data.tick.>"
```

**Terminal 2 - Start Backend:**

```bash
cd agents/backend
cargo run -p backend_service
```

**Expected:** Market data ticks appear in Terminal 1 for each symbol (SPY, XSP, NDX).

### 2. Test Strategy Signals

**Terminal 1 - Subscribe:**

```bash
nats sub "strategy.signal.>"
```

**Terminal 2 - Start Backend:**

```bash
cd agents/backend
cargo run -p backend_service
```

**Expected:** Strategy signals appear when market data triggers strategy evaluation.

### 3. Test Strategy Decisions

**Terminal 1 - Subscribe:**

```bash
nats sub "strategy.decision.>"
```

**Terminal 2 - Start Backend:**

```bash
cd agents/backend
cargo run -p backend_service
```

**Expected:** Strategy decisions appear after risk checks pass.

### 4. Test Error Handling (NATS Down)

**Terminal 1 - Stop NATS:**

```bash
./scripts/stop_nats.sh
```

**Terminal 2 - Start Backend:**

```bash
cd agents/backend
cargo run -p backend_service
```

**Expected:**

- Service starts successfully
- Logs: "NATS integration unavailable, continuing without NATS"
- Service continues to function normally
- No crashes or blocking

**Terminal 1 - Restart NATS:**

```bash
./scripts/start_nats.sh
```

**Expected:** Service continues without reconnection (graceful degradation).

### 5. Test Message Format

**Publish test message:**

```bash
echo '{"symbol":"SPY","bid":100.0,"ask":100.1,"timestamp":"2025-01-01T00:00:00Z"}' | \
  nats pub "market-data.tick.SPY" --stdin
```

**Subscribe and verify:**

```bash
nats sub "market-data.tick.SPY"
```

**Expected:** Message appears with proper JSON structure including metadata:

```json
{
  "id": "...",
  "timestamp": "...",
  "source": "backend",
  "type": "MarketDataTick",
  "payload": {
    "symbol": "SPY",
    "bid": 100.0,
    "ask": 100.1,
    "timestamp": "2025-01-01T00:00:00Z"
  }
}
```

## Performance Testing

### Latency Measurement

Run performance test:

```bash
./scripts/test_nats_integration.sh performance
```

**Expected Results:**

- Average latency < 10ms (local)
- Sub-millisecond latency for high-frequency messages
- No message loss

### Throughput Testing

**Publish many messages:**

```bash
for i in {1..1000}; do
  echo "{\"test\":$i}" | nats pub "test.throughput" --stdin
done
```

**Subscribe and count:**

```bash
nats sub "test.throughput" | wc -l
```

**Expected:** All 1000 messages received.

## Phase 2 Integration Testing

### C++ TWS Client Integration ✅

- [x] NATS wrapper implemented (`native/include/nats_client.h`)
- [x] Integrated into TWSClient (`native/src/tws_client.cpp`)
- [x] Market data publishing in `tickPrice()` callback
- [ ] Build with `ENABLE_NATS=ON` and test
- [ ] Verify market data messages published correctly

### Python Strategy Runner Integration ✅

- [x] NATS client wrapper implemented (`python/integration/nats_client.py`)
- [x] Integrated into strategy runner
- [x] Strategy signal publishing tested and passing
- [x] Strategy decision publishing tested and passing
- [ ] End-to-end test with running strategy

### TypeScript Frontend Integration ✅

- [x] NATS service implemented (`web/src/services/nats.ts`)
- [x] NATS hook created (`web/src/hooks/useNATS.ts`)
- [x] Integrated into HeaderStatus component
- [ ] Test in browser with running NATS server
- [ ] Verify real-time message reception

## Validation Checklist

### ✅ Basic Functionality

- [x] NATS server starts successfully
- [x] Backend connects to NATS
- [x] Market data publishes to correct topics
- [x] Strategy signals publish to correct topics
- [x] Strategy decisions publish to correct topics
- [x] Messages have correct format (JSON with metadata)

### ✅ Topic Validation

- [ ] Market data: `market-data.tick.{symbol}`
- [ ] Strategy signals: `strategy.signal.{symbol}`
- [ ] Strategy decisions: `strategy.decision.{symbol}`
- [ ] Wildcard subscriptions work (`market-data.>`, `strategy.signal.>`)

### ✅ Error Handling

- [ ] Service starts when NATS is down
- [ ] Service continues when NATS disconnects
- [ ] Publish failures are logged but don't crash
- [ ] No blocking operations

### ✅ Performance

- [ ] Latency < 10ms (local)
- [ ] No message loss
- [ ] High throughput (1000+ messages/sec)

### ✅ Integration

- [ ] Existing Tokio channels still work
- [ ] gRPC streaming still works
- [ ] REST API still works
- [ ] No breaking changes

## Troubleshooting

### NATS Server Not Running

```bash

# Check if NATS is running

curl http://localhost:8222/healthz

# Start NATS

./scripts/start_nats.sh

# Check logs

tail -f /tmp/nats-server.log
```

### Connection Errors

```bash

# Check NATS URL

echo $NATS_URL  # Should be: nats://localhost:4222

# Test connection

nats server check
```

### Message Not Received

1. Verify subscription topic matches publish topic
2. Check NATS server logs
3. Verify message format (JSON)
4. Check for wildcard matching issues

### Performance Issues

1. Check NATS server resources
2. Verify network latency
3. Check message size
4. Monitor NATS server metrics: `curl http://localhost:8222/varz`

## Test Results Template

```markdown

## Test Results - [Date]

### Environment
- NATS Version: [version]
- Backend Version: [version]
- OS: [OS version]

### Test Results
- Basic Publish/Subscribe: ✅/❌
- Market Data Topics: ✅/❌
- Strategy Topics: ✅/❌
- Performance (latency): [X]ms
- Error Handling: ✅/❌

### Issues Found
- [List any issues]

### Notes
- [Additional observations]
```

## Next Steps

After successful testing:

1. Proceed with Phase 2 integration (C++, Python, TypeScript, Swift)
2. Implement dead letter queue (T-195)
3. Add circuit breakers
4. Performance optimization
5. Production deployment

## References

- [NATS Documentation](https://docs.nats.io/)
- [NATS CLI Tool](https://docs.nats.io/using-nats/nats-tools/nats)
- [NATS Topics Registry](./NATS_TOPICS_REGISTRY.md)
- [NATS Setup Guide](./NATS_SETUP.md)
