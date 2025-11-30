# Quick Start Guide

## Current Status: ✅ Framework Complete

Your IBKR Box Spread Generator is **built and tested** but needs TWS API integration to connect to Interactive Brokers.

---

## What Works Now

✅ **Build System**: Universal binary (Intel + Apple Silicon)
✅ **Configuration**: JSON-based with validation
✅ **Strategy Engine**: Box spread detection framework
✅ **Risk Management**: Position sizing, limits, VaR calculations
✅ **Order Management**: Multi-leg order support
✅ **Logging**: Comprehensive with spdlog
✅ **Testing**: 29/29 tests passing (100%)
✅ **Dry-Run Mode**: Safe simulation

⚠️ **Stub TWS Client**: Not connected to real broker

---

## Next Steps Summary

### Step 1: Download TWS API ⏳

```bash

# Visit and download

open https://interactivebrokers.github.io/
```

### Step 2: Extract API ⏳

```bash
mkdir -p native/third_party/tws-api
unzip ~/Downloads/twsapi_macunix*.zip -d native/third_party/tws-api/
```

### Step 3: Implement Client ⏳

- Modify `src/tws_client.cpp`
- Implement EWrapper callbacks
- See: `docs/IMPLEMENTATION_GUIDE.md`

### Step 4: Test Paper Trading ⏳

```bash

# Port 7497 = Paper Trading

./build/bin/ib_box_spread --config config/config.json
```

### Step 5: Validate Data ⏳

- Run for 30+ days
- Verify all metrics
- Check performance

### Step 6: Live Trading (EXTREME CAUTION) ⏳

- Only after 30+ days of successful paper trading
- Start with $500 max position
- Monitor continuously

---

## Quick Commands

### Build

```bash
./scripts/build_universal.sh
```

### Test

```bash
cd build && ctest --output-on-failure
```

### Run (Dry-Run)

```bash
./build/bin/ib_box_spread --config config/config.json --dry-run
```

### Validate Config

```bash
./build/bin/ib_box_spread --config config/config.json --validate
```

### View Logs

```bash
tail -f logs/ib_box_spread.log
```

---

## Important Files

| File | Purpose |
|------|---------|
| `config/config.json` | Your settings |
| `docs/IMPLEMENTATION_GUIDE.md` | Detailed steps 1-6 |
| `src/tws_client.cpp` | Needs TWS API integration |
| `logs/ib_box_spread.log` | Application logs |

---

## Safety Reminders

⚠️ **ALWAYS start with paper trading** (port 7497)
⚠️ **NEVER skip testing** - run for 30+ days minimum
⚠️ **START SMALL** - $500 max when going live
⚠️ **MONITOR CLOSELY** - watch every trade
⚠️ **STOP IF ERRORS** - any error = stop immediately

---

## Get Help

- **Full Guide**: `docs/IMPLEMENTATION_GUIDE.md`
- **README**: `README.md`
- **IBKR API Docs**: <https://interactivebrokers.github.io/tws-api/>
- **IBKR Support**: 1-877-442-2757

---

**Remember**: This is trading software. You can lose money. Use at your own risk.
