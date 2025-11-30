# Integration Testing Guide

**Date**: 2025-11-01

**Status**: Ready for TWS Integration

**Version**: 1.0.0

---

## Overview

This document provides comprehensive instructions for integration testing the IBKR Box Spread Generator with Interactive Brokers TWS/Gateway.

## Prerequisites

### Software Requirements

- ✅ **Binary Built**: `build/bin/ib_box_spread` (1.1 MB, x86_64)
- ✅ **All Tests Passing**: 29/29 unit tests pass
- ✅ **Configuration Valid**: `config/config.json` validated
- ✅ **EWrapper Implementation**: Full TWS API integration

### TWS/Gateway Requirements

- Interactive Brokers account (paper trading recommended for initial testing)
- TWS or IB Gateway installed
- API access enabled

---

## Quick Start

### Stage 0: Validate TWS/Gateway with an Official Sample

Before running our stack, take an official IB sample client (or the `scmhub/ibapi` Go sample) and connect to the same TWS/Gateway. Ensure it reaches the `nextValidId` phase—if it fails, fix the environment (API settings, trusted IPs, stale sessions) before debugging our code. Proceed only when the sample succeeds.

### 1. Run Automated Integration Tests

```bash
./scripts/integration_test.sh
```

This script will:

- ✓ Validate configuration
- ✓ Check TWS/Gateway connectivity
- ✓ Test dry-run mode
- ✓ Test TWS connection (if available)

### 1b. Mock TWS Dry Run

The integration wrapper now runs the CLI with `--mock-tws` after validation. This spins up the in-process mock TWS client, exercises the main trading loop for a few seconds (using `timeout`, default 8s), and writes the transcript to `build/integration_logs/mock_run_<timestamp>.log`. Use it to sanity-check orchestration without a live IB Gateway:

```bash
./build/macos-x86_64-release/bin/ib_box_spread \
  --config config/config.json \
  --dry-run \
  --mock-tws \
  --log-level debug
```

### 2. Start TWS Paper Trading

#### Download TWS

- Visit: <https://www.interactivebrokers.com/en/trading/tws.php>
- Download TWS for your platform
- Install and launch

#### Login

- Use paper trading credentials
- Username: Your IB paper trading username
- Password: Your IB paper trading password

#### Enable API Access

1. File → Global Configuration → API → Settings
2. ☑ Enable ActiveX and Socket Clients
3. Socket Port: `7497` (paper trading)
4. Read-Only API: ☐ (unchecked)
5. Click OK

### 3. Run Application

```bash

# Dry-run mode (no real trades)

build/bin/ib_box_spread --config config/config.json --dry-run

# Live mode (executes orders)

build/bin/ib_box_spread --config config/config.json

# Monitor logs

tail -f logs/ib_box_spread.log
```

---

## Configuration

### TWS Connection Settings

**File**: `config/config.json`

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497, // 7497=paper, 7496=live
    "client_id": 1,
    "connection_timeout_ms": 5000,
    "auto_reconnect": true,
    "reconnect_delay_ms": 3000,
    "use_mock": false
  }
}
```

**Port Configuration:**

- **7497**: Paper Trading (TWS)
- **7496**: Live Trading (TWS)
- **4002**: Paper Trading (IB Gateway)
- **4001**: Live Trading (IB Gateway)

---

## Test Scenarios

### Test 1: Connection Test

**Objective**: Verify TWS API connectivity

**Steps:**

1. Start TWS paper trading
2. Enable API access (see Quick Start)
3. Run: `./scripts/integration_test.sh`

**Expected Results:**

```text
[INFO] Connecting to TWS at 127.0.0.1:7497...
[INFO] Connected to TWS
[INFO] Server Version: 178
[INFO] Next valid order ID: 1
```

**Success Criteria:**

- ✓ Connection established
- ✓ Server version received
- ✓ Valid order ID assigned

### Test 2: Market Data Request

**Objective**: Request and receive market data for SPY options

**Steps:**

1. Ensure TWS connection established
2. Application will automatically request option chains
3. Monitor logs for market data callbacks

**Expected Results:**

```text
[INFO] Requesting option chain for SPY
[INFO] Received tick price: SPY, bid=580.25, ask=580.28
[INFO] Received options data: 124 contracts
```

**Success Criteria:**

- ✓ Market data subscription successful
- ✓ Tick data received
- ✓ Option chain populated

### Test 3: Dry-Run Order Placement

**Objective**: Test order creation without execution

**Steps:**

1. Run with `--dry-run` flag
2. Wait for box spread opportunity detection
3. Observe simulated order flow

**Expected Results:**

```text
[INFO] Box spread opportunity found: SPY
[INFO] Expected profit: $1.50, ROI: 1.2%
[DRY RUN] Would place order: BUY 1 SPY CALL 580...
[DRY RUN] Would place order: SELL 1 SPY CALL 585...
[DRY RUN] Would place order: BUY 1 SPY PUT 585...
[DRY RUN] Would place order: SELL 1 SPY PUT 580...
```

**Success Criteria:**

- ✓ Opportunity detected
- ✓ Orders created (not submitted)
- ✓ Risk checks passed

### Test 4: Live Order Placement (Paper Trading)

**⚠️ CAUTION**: Only run with paper trading account

**Objective**: Place actual orders in paper trading environment

**Steps:**

1. **Verify paper trading mode** in TWS
2. Run without `--dry-run` flag
3. Monitor order status updates
4. Verify positions in TWS

**Expected Results:**

```text
[INFO] Placing box spread order: SPY
[INFO] Order submitted: Order ID 1
[INFO] Order status: Submitted
[INFO] Order status: Filled @ 580.25
[INFO] Position opened: +1 SPY 580 CALL
```

**Success Criteria:**

- ✓ Orders submitted successfully
- ✓ Order status updates received
- ✓ Positions tracked correctly
- ✓ P&L calculations accurate

### Test 5: Error Handling

**Objective**: Verify graceful error handling

**Test Cases:**

**5.1 Connection Lost**

- Disconnect network during operation
- Expected: Auto-reconnect attempt, graceful degradation

**5.2 Invalid Order**

- Attempt order with invalid parameters
- Expected: Order rejection, error logged, no crash

**5.3 TWS Restart**

- Restart TWS during operation
- Expected: Reconnection, state recovery

**Success Criteria:**

- ✓ No crashes
- ✓ Clear error messages
- ✓ Automatic recovery where possible

---

## Monitoring and Validation

### Log Monitoring

```bash

# Real-time log monitoring

tail -f logs/ib_box_spread.log

# Search for errors

grep -i error logs/ib_box_spread.log

# Monitor connections

grep -i "connect" logs/ib_box_spread.log

# Track orders

grep -i "order" logs/ib_box_spread.log
```

### TWS Verification

**Check in TWS:**

1. **Account Window**: View positions and P&L
2. **Order Monitor**: Verify order status
3. **Trade Log**: Review execution details
4. **API Status**: File → Global Configuration → API → Status

---

## Troubleshooting

### Connection Errors

**Error**: `Couldn't connect to TWS`

- **Solution**:
  - Verify TWS is running
  - Check API is enabled in TWS settings
  - Confirm port number matches configuration
  - Check firewall settings

**Error**: `Connection refused`

- **Solution**:
  - TWS may not be running
  - Port may be incorrect
  - Try: `netstat -an | grep 7497`

### API Errors

**Error**: `Not connected`

- **Solution**:
  - Wait for connection to establish
  - Check auto_reconnect setting
  - Review connection timeout

**Error**: `Order rejected`

- **Solution**:
  - Check account has sufficient funds
  - Verify contract specifications
  - Review risk limits

### Data Errors

**Error**: `No market data permissions`

- **Solution**:
  - Subscribe to required market data in TWS
  - Verify account has data subscriptions
  - Check symbol is valid

---

## Performance Metrics

### Expected Performance

**Connection:**

- Initial connection: < 5 seconds
- Reconnection: < 3 seconds

**Market Data:**

- Data latency: < 100ms
- Update frequency: Real-time

**Orders:**

- Order submission: < 500ms
- Fill confirmation: < 1 second

---

## Safety Checklist

Before running with real money:

- [ ] All tests pass (29/29)
- [ ] Integration tests successful
- [ ] Paper trading tested for 1+ week
- [ ] Error handling verified
- [ ] Risk limits configured
- [ ] Stop-loss mechanisms tested
- [ ] Account limits set in TWS
- [ ] Monitoring alerts configured

---

## Next Steps

1. **Paper Trading**: Run for 1-2 weeks with paper trading
2. **Data Collection**: Monitor performance and collect metrics
3. **Risk Validation**: Verify risk management works correctly
4. **Live Trading**: Transition to live trading with small position sizes
5. **Scale Up**: Gradually increase position sizes after validation

---

## Current Status

**✅ Completed:**

- Full EWrapper implementation
- Thread-safe operations
- Comprehensive error handling
- Configuration validation
- Integration test suite
- All unit tests passing (29/29)

**⚠️ Requires Testing:**

- Live TWS connectivity
- Market data streaming
- Order execution flow
- Position management
- P&L tracking

**📈 Ready For:**

- Paper trading validation
- Integration testing with live TWS
- Performance monitoring

---

## Support

**Documentation:**

- `docs/EWRAPPER_IMPLEMENTATION.md` - Implementation details
- `docs/EWRAPPER_STATUS.md` - API coverage status
- `README.md` - General usage

**Contact:**

- Issues: Create GitHub issue
- Questions: Check TWS API documentation

---

**Note**: Always start with paper trading. Never test experimental code with real money.
