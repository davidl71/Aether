# LEAN Box Spread Strategy Testing Guide

**Date**: 2025-11-18
**Status**: Testing Guide
**Purpose**: End-to-end testing procedures for LEAN box spread strategy

---

## Overview

This guide provides comprehensive testing procedures for the LEAN box spread strategy, including backtesting, paper trading, and validation steps.

---

## Testing Strategy

### Phase 1: Backtesting (Historical Data)

Test the algorithm with historical data to verify:

- Algorithm logic works correctly
- Data conversion functions properly
- Opportunity detection works
- No runtime errors

### Phase 2: Paper Trading (Real Market Data)

Test with real market data in paper trading mode:

- Real-time market data reception
- Order placement
- Position tracking
- Error handling

### Phase 3: Validation

Validate results:

- Order execution accuracy
- Position tracking correctness
- P&L calculations
- Performance metrics

---

## Prerequisites

1. ✅ LEAN CLI installed and working
2. ✅ Broker configured (IBKR or Alpaca)
3. ✅ TWS/IB Gateway running (for IBKR)
4. ✅ API keys configured (for Alpaca)
5. ✅ Strategy configuration files created

---

## Phase 1: Backtesting

### 1.1 Create Test Algorithm

Create a simple test algorithm to verify basic functionality:

```python
# Main/test_basic.py
from AlgorithmImports import *

class TestBasic(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2025, 1, 1)
        self.SetEndDate(2025, 1, 31)
        self.SetCash(100000)

        # Subscribe to SPY options
        option = self.AddOption("SPY")
        option.SetFilter(lambda u: u.Strikes(-5, +5).Expiration(0, 30))

    def OnData(self, slice):
        option_chain = slice.OptionChains.get("SPY", None)
        if option_chain:
            self.Log(f"Received {len(option_chain)} option contracts")
```

### 1.2 Run Backtest

```bash
# Activate LEAN environment
source python/venv312/bin/activate

# Run backtest
lean backtest Main/test_basic.py

# Check results
ls -la results/
```

### 1.3 Verify Results

- Check algorithm logs for errors
- Verify option chain data received
- Check backtest results in `results/` folder

---

## Phase 2: Paper Trading

### 2.1 Configure Broker

**For IBKR:**

1. Ensure TWS/IB Gateway is running
2. Verify API is enabled
3. Check port matches configuration (7497 for paper)

**For Alpaca:**

1. Verify API keys are correct
2. Check base URL (paper-api.alpaca.markets)
3. Verify account permissions

### 2.2 Test Connection

```bash
# Test IBKR connection
lean live --brokerage InteractiveBrokers --data-provider InteractiveBrokers

# Test Alpaca connection
lean live --brokerage Alpaca --data-provider Alpaca
```

### 2.3 Run Box Spread Strategy

```bash
# Run with IBKR
lean live Main/box_spread_algorithm.py --brokerage InteractiveBrokers

# Run with Alpaca
lean live Main/box_spread_algorithm.py --brokerage Alpaca
```

### 2.4 Monitor Execution

Watch for:

- Algorithm initialization
- Market data reception
- Opportunity detection
- Order placement
- Order fills
- Position tracking
- Error messages

---

## Phase 3: Validation

### 3.1 Order Execution Validation

**Check:**

- ✅ Orders are placed correctly
- ✅ Combo orders have 4 legs
- ✅ Quantities are correct (1, -1, 1, -1)
- ✅ Order status updates correctly
- ✅ Fills are tracked

**Validation Steps:**

1. Monitor `OnOrderEvent` logs
2. Verify order IDs are tracked
3. Check order status transitions
4. Verify fill prices

### 3.2 Position Tracking Validation

**Check:**

- ✅ Positions are created on fill
- ✅ Position data is correct
- ✅ P&L is calculated
- ✅ Positions are tracked in `active_positions`

**Validation Steps:**

1. Check `active_positions` dictionary
2. Verify position entry time
3. Verify position entry price
4. Check P&L calculations

### 3.3 Data Conversion Validation

**Check:**

- ✅ LEAN data converts to C++ format correctly
- ✅ C++ contracts convert to LEAN Symbols correctly
- ✅ Market data is preserved
- ✅ Dates are formatted correctly

**Validation Steps:**

1. Log converted data
2. Verify contract details match
3. Check market data values
4. Verify date formatting

### 3.4 Opportunity Detection Validation

**Check:**

- ✅ Opportunities are detected
- ✅ Profitability checks work
- ✅ Risk checks work
- ✅ Best opportunities are selected

**Validation Steps:**

1. Log detected opportunities
2. Verify profitability calculations
3. Check risk scores
4. Verify opportunity ranking

---

## Test Scenarios

### Scenario 1: Basic Functionality

**Objective:** Verify algorithm runs without errors

**Steps:**

1. Start algorithm
2. Wait for market data
3. Check logs for errors
4. Verify algorithm continues running

**Expected Results:**

- No errors in logs
- Market data received
- Algorithm continues running

### Scenario 2: Opportunity Detection

**Objective:** Verify box spreads are detected

**Steps:**

1. Run algorithm with symbols that have options
2. Wait for market data
3. Check logs for opportunity detection
4. Verify opportunities are logged

**Expected Results:**

- Opportunities detected (if available)
- Opportunities logged with details
- Profitability calculated

### Scenario 3: Order Placement

**Objective:** Verify orders are placed correctly

**Steps:**

1. Run algorithm
2. Wait for opportunity
3. Verify order is placed
4. Check order details

**Expected Results:**

- Order placed via ComboMarketOrder
- Order has 4 legs
- Quantities are (1, -1, 1, -1)
- Order ID is tracked

### Scenario 4: Order Fill

**Objective:** Verify order fills are tracked

**Steps:**

1. Place order
2. Wait for fill
3. Check OnOrderEvent
4. Verify position created

**Expected Results:**

- Order status changes to Filled
- Position added to active_positions
- Entry time and price recorded
- Statistics updated

### Scenario 5: Error Handling

**Objective:** Verify error handling works

**Steps:**

1. Simulate connection error (stop TWS)
2. Check error handling
3. Verify algorithm continues or reconnects
4. Check error logs

**Expected Results:**

- Errors are logged
- Algorithm handles errors gracefully
- Reconnection attempted (if configured)

---

## Test Checklist

### Pre-Testing

- [ ] LEAN CLI installed and working
- [ ] Broker configured (IBKR or Alpaca)
- [ ] TWS/IB Gateway running (for IBKR)
- [ ] API keys configured (for Alpaca)
- [ ] Configuration files created
- [ ] Strategy code deployed

### Backtesting

- [ ] Backtest runs without errors
- [ ] Historical data loads correctly
- [ ] Option chains received
- [ ] Algorithm logic executes
- [ ] Results generated

### Paper Trading

- [ ] Connection established
- [ ] Market data received
- [ ] Options data available
- [ ] Algorithm runs continuously
- [ ] No connection errors

### Order Execution

- [ ] Opportunities detected
- [ ] Orders placed correctly
- [ ] Order status tracked
- [ ] Fills received
- [ ] Positions created

### Validation

- [ ] Order details correct
- [ ] Position tracking accurate
- [ ] P&L calculated correctly
- [ ] Statistics updated
- [ ] Error handling works

---

## Troubleshooting

### No Market Data

**Symptoms:** Algorithm runs but no option chain data received

**Solutions:**

- Check broker connection
- Verify market data subscriptions
- Check symbol is correct
- Verify market is open
- Check data provider configuration

### Order Rejections

**Symptoms:** Orders placed but immediately rejected

**Solutions:**

- Check contract details (symbol, expiry, strike, right)
- Verify account permissions
- Check buying power
- Verify market is open
- Check order parameters

### Connection Errors

**Symptoms:** Connection lost or cannot connect

**Solutions:**

- Verify TWS/Gateway is running (IBKR)
- Check API is enabled
- Verify port number
- Check IP address is trusted
- Restart TWS/Gateway

### Data Conversion Errors

**Symptoms:** Errors in data conversion

**Solutions:**

- Check LEAN data format
- Verify C++ bindings are available
- Check date formatting
- Verify contract matching logic
- Review error logs

---

## Performance Metrics

### Monitor During Testing

- **Latency:** Time from market data to order placement
- **Throughput:** Number of opportunities evaluated per second
- **Order Fill Rate:** Percentage of orders that fill
- **Error Rate:** Number of errors per hour
- **Memory Usage:** Algorithm memory consumption
- **CPU Usage:** Algorithm CPU usage

### Target Metrics

- **Latency:** < 100ms from opportunity to order
- **Throughput:** > 10 opportunities/second
- **Fill Rate:** > 80% for profitable opportunities
- **Error Rate:** < 1 error per hour
- **Memory:** < 500MB
- **CPU:** < 50% on single core

---

## Test Results Template

```markdown
## Test Results - [Date]

### Configuration
- Broker: [IBKR/Alpaca]
- Mode: [Paper/Live]
- Symbols: [SPY, SPX, etc.]
- Duration: [X hours]

### Results
- Opportunities Found: [X]
- Trades Executed: [X]
- Orders Filled: [X]
- Total Profit: $[X]
- Errors: [X]

### Issues
- [List any issues encountered]

### Performance
- Average Latency: [X]ms
- Throughput: [X] opportunities/sec
- Fill Rate: [X]%
- Error Rate: [X] errors/hour
```

---

## Next Steps

After successful testing:

1. ✅ **Document Results**: Record test results
2. ⏳ **Fix Issues**: Address any problems found
3. ⏳ **Optimize**: Improve performance if needed
4. ⏳ **Production Ready**: Prepare for live trading (if desired)

---

## References

- [LEAN Backtesting Guide](https://www.quantconnect.com/docs/v2/lean-cli/backtesting)
- [LEAN Live Trading Guide](https://www.quantconnect.com/docs/v2/lean-cli/live-trading)
- [LEAN Testing Best Practices](https://www.quantconnect.com/docs/v2/writing-algorithms/testing)
- [LEAN IBKR Setup](docs/LEAN_IBKR_SETUP.md)
- [LEAN Alpaca Setup](docs/LEAN_ALPACA_SETUP.md)

---

## Status

- ✅ Testing guide created
- ✅ Test procedures documented
- ✅ Validation criteria defined
- ✅ Troubleshooting guide included
- ⏳ Ready for execution
