# IBKR Box Spread Generator - Implementation Guide

This guide walks you through making the application production-ready by integrating with Interactive Brokers TWS API.

## Prerequisites

- ✅ Application built and tested (completed)
- ✅ Configuration file created (completed)
- ⏳ Interactive Brokers account (required)
- ⏳ TWS or IB Gateway installed (required)

---

## Step 1: Download TWS API

### 1.1 Visit IBKR Developer Portal

Go to: <https://interactivebrokers.github.io/>

### 1.2 Download TWS API

1. Navigate to **Downloads** section
2. Choose your platform:
   - **macOS**: Download `TWS API for Mac`
   - **Linux**: Download `TWS API for Linux`
   - **Windows**: Download `TWS API for Windows`

3. Download the latest stable version (currently v10.19+)

### 1.3 Verify Download

```bash

# Check the downloaded file

ls -lh ~/Downloads/twsapi_macunix*.zip

# Verify it's not corrupted

unzip -t ~/Downloads/twsapi_macunix*.zip
```

**Expected file size**: ~10-20 MB

---

## Step 2: Extract TWS API to native/third_party/tws-api/

### 2.1 Create Target Directory

```bash
cd /Users/davidlowes/.claude-squad/worktrees/claude_1873e0c42c155fb0
mkdir -p native/third_party/tws-api
```

### 2.2 Extract Archive

```bash

# Extract to vendor directory

unzip ~/Downloads/twsapi_macunix*.zip -d native/third_party/tws-api/

# Verify extraction

ls -la native/third_party/tws-api/
```

### 2.3 Verify Directory Structure

Expected structure:

```
native/third_party/tws-api/
├── IBJts/
│   ├── source/
│   │   ├── cppclient/
│   │   │   ├── client/          <- Headers we need
│   │   │   │   ├── EClient.h
│   │   │   │   ├── EWrapper.h
│   │   │   │   ├── Contract.h
│   │   │   │   ├── Order.h
│   │   │   │   └── ...
│   │   │   └── src/             <- Source files
│   │   └── ...
│   └── ...
└── samples/
```

### 2.4 Rebuild to Detect API

```bash

# Clean previous build

rm -rf build

# Rebuild - should now detect TWS API

./scripts/build_universal.sh
```

**Expected output**:

```
CMake Status: TWS API found: /path/to/native/third_party/tws-api/source/cppclient/client
```

---

## Step 3: Implement Actual TWS Client

### 3.1 Understanding the TWS API

The TWS API uses a **callback pattern**:

- `EClient`: Your client that sends requests to TWS
- `EWrapper`: Your callbacks that receive data from TWS

### 3.2 Implementation Overview

You need to modify `src/tws_client.cpp` to:

1. **Inherit from EWrapper and EClient**
2. **Implement all EWrapper callbacks**
3. **Create threaded message processing**
4. **Handle connection lifecycle**

### 3.3 Key Files to Modify

```
src/tws_client.cpp       - Main implementation
include/tws_client.h     - Update class definition
```

### 3.4 Implementation Template

Here's the basic structure you'll need:

```cpp
// In tws_client.cpp - Private Implementation
class TWSClient::Impl : public EWrapper, public EClient {
public:
    Impl(const config::TWSConfig& config)
        : EClient(this, &signal_)
        , config_(config) {
    }

    // EWrapper callbacks (100+ methods to implement!)
    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attrib) override {
        // Handle price updates
    }

    void orderStatus(OrderId orderId, const std::string& status,
                    double filled, double remaining,
                    double avgFillPrice, int permId, int parentId,
                    double lastFillPrice, int clientId,
                    const std::string& whyHeld, double mktCapPrice) override {
        // Handle order status updates
    }

    // ... 100+ more callbacks ...

private:
    EReaderOSSignal signal_;
    config::TWSConfig config_;
    std::unique_ptr<std::thread> reader_thread_;
};
```

### 3.5 Essential Callbacks to Implement

**Priority 1 (Critical):**

- `error()` - Error handling
- `connectAck()` - Connection acknowledgment
- `nextValidId()` - Get next order ID
- `connectionClosed()` - Connection closed

**Priority 2 (Market Data):**

- `tickPrice()` - Price updates
- `tickSize()` - Size updates
- `tickOptionComputation()` - Greeks and IV

**Priority 3 (Orders):**

- `orderStatus()` - Order status updates
- `openOrder()` - Open order details
- `execDetails()` - Execution details

**Priority 4 (Positions & Account):**

- `position()` - Position updates
- `updateAccountValue()` - Account updates
- `updatePortfolio()` - Portfolio updates

### 3.6 Reference Implementation

See the TWS API samples:

```bash
cd native/third_party/tws-api/samples/Cpp/TestCppClient/

# Study the example implementation
```

### 3.7 Recommended Resources

- **Official API Guide**: native/third_party/tws-api/IBJts/Guides/
- **Sample Code**: native/third_party/tws-api/samples/Cpp/
- **API Reference**: <https://interactivebrokers.github.io/tws-api/>

---

## Step 4: Test with Paper Trading Account

### 4.1 Setup Paper Trading in TWS

1. **Open TWS or IB Gateway**
2. **Login with paper trading credentials**
3. **Configure API Settings**:
   - Go to: `File` → `Global Configuration` → `API` → `Settings`
   - Enable: `Enable ActiveX and Socket Clients`
   - Port: `7497` (Paper Trading)
   - Master API client ID: Leave blank or set to 0
   - Read-Only API: Disable
   - Trusted IPs: Add `127.0.0.1`

### 4.2 Verify TWS is Running

```bash

# Check if TWS is listening on port 7497

lsof -i :7497

# or

netstat -an | grep 7497
```

**Expected output**:

```
java    12345 user   123u  IPv4 0x1234  TCP *:7497 (LISTEN)
```

### 4.3 Update Configuration

Edit `config/config.json`:

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,              // Paper trading port
    "client_id": 1
  },
  "dry_run": false,            // Use TWS for real (paper) trading
  ...
}
```

### 4.4 Test Connection

```bash

# Run with debug logging

./build/bin/ib_box_spread --config config/config.json --log-level debug
```

**Look for**:

- ✅ "Connected to TWS"
- ✅ "Received nextValidId"
- ✅ Market data flowing
- ❌ Connection errors
- ❌ Authentication failures

### 4.5 Test Scenarios

**Test 1: Connection**

- Start application
- Verify connection established
- Check logs for errors
- Verify graceful shutdown

**Test 2: Market Data**

- Request option chain for SPY
- Verify price updates received
- Check data quality
- Monitor for disconnections

**Test 3: Order Placement**

- Place a test order (small size)
- Verify order acknowledged
- Check order status updates
- Cancel the order
- Verify cancellation confirmed

**Test 4: Error Handling**

- Disconnect TWS mid-session
- Verify auto-reconnect works
- Check error recovery
- Verify data integrity

### 4.6 Paper Trading Best Practices

- ✅ Start with single symbol (SPY only)
- ✅ Use small position sizes ($100-$1000)
- ✅ Monitor logs continuously
- ✅ Test during market hours (9:30 AM - 4:00 PM ET)
- ✅ Let it run for at least 1 full trading day
- ✅ Review all trades manually
- ❌ Don't test overnight (options market closed)
- ❌ Don't test on holidays

---

## Step 5: Validate with Real Market Data

### 5.1 Data Quality Checks

**Create a validation script:**

```bash

#!/bin/bash
# scripts/validate_data.sh

echo "Validating market data quality..."

# Check logs for data gaps

grep "tickPrice" logs/ib_box_spread.log | tail -100

# Check for bid-ask spread quality

grep "bid_ask_spread" logs/ib_box_spread.log | tail -50

# Check for missing data

grep "WARNING\|ERROR" logs/ib_box_spread.log
```

### 5.2 Validation Checklist

**Market Data:**

- [ ] Bid/ask prices updating regularly
- [ ] Bid-ask spreads reasonable (< $0.10 for liquid options)
- [ ] Implied volatility values present
- [ ] Greeks (delta, gamma, etc.) calculated
- [ ] Volume and open interest available

**Option Chain:**

- [ ] All strikes retrieved
- [ ] All expiries in range retrieved
- [ ] Call/put pairs match
- [ ] No missing data for liquid options

**Strategy:**

- [ ] Box spreads identified correctly
- [ ] Arbitrage calculations accurate
- [ ] ROI calculations correct
- [ ] Commission costs included

**Risk Management:**

- [ ] Position limits enforced
- [ ] Exposure limits checked
- [ ] Risk metrics calculated
- [ ] Stop losses would trigger correctly

### 5.3 Backtesting (Optional but Recommended)

Record a full day of data:

```bash

# Run for 1 day, log everything

./build/bin/ib_box_spread --log-level trace > backtest_data.log 2>&1
```

Analyze results:

- How many opportunities found?
- What was average ROI?
- Were opportunities executable?
- What was win rate?

### 5.4 Performance Validation

```bash

# Monitor resource usage

top -pid $(pgrep ib_box_spread)

# Check memory leaks (macOS)

leaks ib_box_spread

# Profile if needed

instruments -t "Time Profiler" ./build/bin/ib_box_spread
```

**Expected performance:**

- CPU: < 5% average
- Memory: < 100 MB
- Network: Minimal (< 1 MB/hour)
- Latency: < 100ms for order placement

---

## Step 6: Live Trading (EXTREME CAUTION)

### ⚠️ WARNING - READ THIS CAREFULLY

**ONLY proceed if:**

- ✅ Paper trading ran flawlessly for 30+ days
- ✅ All test scenarios passed 100%
- ✅ You understand every line of code
- ✅ You have reviewed all risk limits
- ✅ You can afford to lose the capital
- ✅ You have regulatory approval if required
- ✅ You have tested emergency stop procedures

**DO NOT proceed if:**

- ❌ Any test failed or showed errors
- ❌ You haven't tested for at least 30 days
- ❌ You don't fully understand the strategy
- ❌ You're risking money you can't lose
- ❌ Market conditions are volatile or unusual

### 6.1 Pre-Live Checklist

- [ ] 30+ days of successful paper trading
- [ ] Zero critical bugs found
- [ ] All edge cases tested
- [ ] Emergency stop procedure documented
- [ ] Phone number for broker support available
- [ ] Monitoring system in place
- [ ] Alerts configured for errors
- [ ] Capital limits set appropriately

### 6.2 Initial Live Configuration

**Start VERY small:**

```json
{
  "tws": {
    "port": 7496,                  // LIVE TRADING PORT
    "client_id": 1
  },
  "strategy": {
    "symbols": ["SPY"],            // ONE symbol only
    "min_arbitrage_profit": 1.00,  // Higher threshold
    "max_position_size": 500.0     // VERY SMALL initially
  },
  "risk": {
    "max_total_exposure": 1000.0,  // MAXIMUM $1000
    "max_positions": 1,            // ONE position max
    "max_daily_loss": 100.0        // Stop at $100 loss
  },
  "dry_run": false                 // LIVE MODE
}
```

### 6.3 First Live Trade Protocol

1. **Pre-market**: Review configuration
2. **Market open**: Start application, monitor closely
3. **First 30 minutes**: High volatility, no trades
4. **Mid-day**: Calmer, better for first trade
5. **First trade**: Watch every step, verify manually
6. **Position monitoring**: Check every 5 minutes
7. **Before close**: Close any open positions
8. **Post-market**: Review logs, check P&L

### 6.4 Gradual Scale-Up

**Week 1**: $500 max position, 1 trade per day
**Week 2-4**: $1000 max position, 2-3 trades per day
**Month 2**: $2500 max position, 5 trades per day
**Month 3+**: Gradually increase IF profitable

### 6.5 Monitoring in Live Trading

**Real-time monitoring:**

```bash

# Terminal 1: Application

./build/bin/ib_box_spread --config config/config.json

# Terminal 2: Live log monitoring

tail -f logs/ib_box_spread.log | grep -E "ERROR|WARNING|TRADE|FILL"

# Terminal 3: System monitoring

watch -n 5 'ps aux | grep ib_box_spread; netstat -an | grep 7496'
```

### 6.6 Emergency Stop Procedures

**If anything goes wrong:**

```bash

# 1. Stop the application (graceful)

pkill -TERM ib_box_spread

# 2. If not responding (force kill)

pkill -KILL ib_box_spread

# 3. Login to TWS manually
# 4. Cancel ALL open orders
# 5. Close ALL positions
# 6. Review what happened
```

**Emergency contact:**

- IBKR Support: 1-877-442-2757
- Keep your account number ready

### 6.7 Legal and Regulatory

**Disclaimer**: I am not providing financial or legal advice. Consult with:

- Financial advisor
- Tax professional
- Legal counsel if trading professionally
- Regulatory bodies if required

**Consider:**

- Pattern Day Trader rules (PDT)
- Tax implications (wash sales, mark-to-market)
- Record keeping requirements
- Regulatory compliance (if applicable)

---

## Troubleshooting Common Issues

### Issue: Can't connect to TWS

**Solutions:**

1. Verify TWS is running: `lsof -i :7497` or `:7496`
2. Check TWS API settings are enabled
3. Verify IP address is trusted (127.0.0.1)
4. Check firewall isn't blocking connection
5. Try restarting TWS

### Issue: No market data

**Solutions:**

1. Verify you have market data subscriptions
2. Check TWS market data settings
3. Ensure market is open (9:30 AM - 4:00 PM ET)
4. Verify symbols are correct
5. Check TWS data farm connection

### Issue: Orders rejected

**Solutions:**

1. Verify account has options trading enabled
2. Check sufficient buying power
3. Verify option is tradable
4. Check order parameters (price, size, TIF)
5. Review IBKR error codes

### Issue: Performance problems

**Solutions:**

1. Reduce number of symbols
2. Increase loop delay
3. Filter by liquidity (volume, OI)
4. Optimize market data requests
5. Check for memory leaks

---

## Testing Checklist

Before considering live trading, verify ALL of these:

### Functional Testing

- [ ] Application starts without errors
- [ ] Connects to TWS successfully
- [ ] Receives market data
- [ ] Identifies box spread opportunities
- [ ] Calculates arbitrage correctly
- [ ] Places orders successfully
- [ ] Receives order confirmations
- [ ] Handles order fills
- [ ] Tracks positions correctly
- [ ] Calculates P&L accurately
- [ ] Shuts down gracefully

### Edge Cases

- [ ] TWS disconnects mid-session
- [ ] Network interruption
- [ ] Invalid market data
- [ ] Order rejections
- [ ] Partial fills
- [ ] Unusual market conditions
- [ ] After-hours behavior
- [ ] Holiday handling

### Risk Management

- [ ] Position limits enforced
- [ ] Exposure limits checked
- [ ] Daily loss limit works
- [ ] Stop losses trigger
- [ ] Emergency stop works

### Performance

- [ ] CPU usage acceptable
- [ ] Memory stable (no leaks)
- [ ] Network usage reasonable
- [ ] Latency acceptable
- [ ] Runs for 24+ hours stable

---

## Success Metrics

### Paper Trading Phase (Minimum 30 days)

**Must achieve:**

- Uptime: > 99%
- Win rate: > 80%
- Average profit/trade: > $5
- Zero critical errors
- Zero missed opportunities (due to bugs)

### Live Trading Phase (First 90 days)

**Target metrics:**

- Win rate: > 70%
- Average profit/trade: > $10
- Max drawdown: < 5%
- Sharpe ratio: > 1.0
- Uptime: > 99.9%

---

## Resources

### Official Documentation

- TWS API: <https://interactivebrokers.github.io/tws-api/>
- IBKR Knowledge Base: <https://www.interactivebrokers.com/en/support/>
- API Forums: <https://groups.io/g/twsapi>

### Options Trading

- CBOE Options Institute: <https://www.cboe.com/education/>
- OCC (Options Clearing Corp): <https://www.theocc.com/>
- Box Spread Strategy: Research academic papers on arb strategies

### Risk Management

- Position Sizing: Kelly Criterion, Fixed Fractional
- Portfolio Management: Modern Portfolio Theory
- Risk Metrics: VaR, CVaR, Sharpe, Sortino

---

## Final Thoughts

This is **trading software**. Real money is at risk. The implementation provided is a **framework** that requires:

1. Proper TWS API integration
2. Extensive testing
3. Risk management discipline
4. Continuous monitoring
5. Regular review and improvements

**Take your time. Test thoroughly. Start small. Never risk more than you can afford to lose.**

Good luck! 🚀

---

**Last Updated**: 2025-11-01
**Version**: 1.0.0
**Status**: Framework Complete - TWS Integration Required
