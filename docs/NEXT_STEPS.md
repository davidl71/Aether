# Next Steps: Buy/Sell Disparity Feature Implementation

## ✅ Completed

1. **Core Data Structures**
   - Added buy/sell fields to `BoxSpreadLeg` (buy_net_debit, sell_net_credit, buy_profit, sell_profit, etc.)
   - Added buy/sell fields to TUI data structures (`HistoryEntry`, `YieldCurvePoint`)
   - Added buy/sell fields to Web UI TypeScript interfaces

2. **Calculation Functions**
   - `calculate_buy_net_debit()` - Using ASK for long legs, BID for short legs
   - `calculate_sell_net_credit()` - Using BID for long legs, ASK for short legs
   - `calculate_buy_sell_disparity()` - Difference in profitability
   - `calculate_put_call_parity_violation()` - Put-call parity violation detection

3. **Web UI Visualization**
   - Added "Buy Profit" and "Sell Profit" columns
   - Added "Disparity" column with color-coding
   - Added "P-C Viol (bps)" column for put-call parity violations
   - Color-coded to highlight which side (buy vs sell) is more favorable

4. **Documentation**
   - Created `docs/BUY_SELL_DISPARITY_ANALYSIS.md` explaining factors and implementation

## 📋 Next Steps

### 1. Update the Running PWA

The PWA needs to be rebuilt to include the new buy/sell disparity features.

**Option A: Development Mode (Hot Reload)**

```bash
cd web
npm run dev
```

This will:

- Start Vite dev server on `http://localhost:5173`
- Auto-reload when files change
- Show the new buy/sell columns in the table

**Option B: Production Build**

```bash
cd web
npm run build
npm run preview
```

This will:

- Build optimized production bundle
- Preview on `http://localhost:4173`
- Include service worker for PWA features

**Option C: If PWA is Already Installed**
The PWA will auto-update when you rebuild (service worker auto-update enabled).
Just rebuild and refresh the browser.

### 2. Build and Run the TUI

The TUI needs to be rebuilt to include the new buy/sell calculations.

**Quick Build (if CMake is already configured):**

```bash
cd build
cmake --build . --target ib_box_spread_tui
./bin/ib_box_spread_tui --mock
```

**Full Build (if needed):**

```bash

# From project root

./scripts/build_universal.sh

# Or fast build with caching

./scripts/build_fast.sh

# Then run TUI

./build/bin/ib_box_spread_tui --mock
```

**Run with Mock Data:**

```bash
./build/bin/ib_box_spread_tui --mock
```

**Run with Live Backend:**

```bash

# Terminal 1: Start backend (if you have one running)
# Terminal 2: Run TUI

./build/bin/ib_box_spread_tui
```

### 3. Verify Features Work

**Web UI Checks:**

1. ✅ See "Buy Profit" column showing profit from buying box spread
2. ✅ See "Sell Profit" column showing profit from selling box spread
3. ✅ See "Disparity" column showing difference (positive = buy better, negative = sell better)
4. ✅ See "P-C Viol (bps)" column showing put-call parity violations
5. ✅ Color-coding: Green for profitable, bold for significant disparity
6. ✅ Filtering: European-style options shown by default

**TUI Checks:**

1. ✅ Data structures include buy/sell fields
2. ✅ Mock provider populates buy/sell disparity
3. ⚠️ **Note**: TUI display components still need to be updated to show buy/sell columns
   - Currently data is populated but not displayed in tables
   - Need to update `RenderHistoric()` and yield curve displays

### 4. Update TUI Display Components (TODO)

The TUI currently has the data but doesn't display it yet. Need to:

1. **Update `RenderHistoric()` in `tui_app.cpp`**:
   - Add columns: "Buy Profit", "Sell Profit", "Disparity", "P-C Viol"
   - Color-code based on which side is more favorable

2. **Update Yield Curve Display** (if exists):
   - Show buy vs sell implied rates
   - Highlight disparity

3. **Add Buy/Sell Comparison View** (optional):
   - Side-by-side comparison
   - Time-of-day analysis

### 5. Test with Real Data (When Available)

Once connected to live market data:

1. **Verify Bid/Ask Prices**:
   - Check that buy calculations use ASK for long legs
   - Check that sell calculations use BID for long legs
   - Verify spreads are calculated correctly

2. **Verify Put-Call Parity Violations**:
   - Check that violations are detected when call/put implied rates differ
   - Verify violation amounts are reasonable (typically <100 bps)

3. **Monitor Intraday Changes**:
   - Track how buy/sell disparity changes throughout the day
   - Identify optimal times for buy vs sell opportunities

### 6. Optional Enhancements

1. **Time-of-Day Analysis**:
   - Track average disparity by hour
   - Identify patterns (e.g., wider spreads at market open/close)

2. **Historical Tracking**:
   - Store buy/sell disparity over time
   - Chart disparity trends

3. **Alerts**:
   - Alert when buy/sell opportunity appears
   - Alert when disparity exceeds threshold

4. **Sorting/Filtering**:
   - Sort by disparity
   - Filter to show only profitable buy or sell opportunities

## Quick Reference Commands

```bash

# Update PWA (Development)

cd web && npm run dev

# Build PWA (Production)

cd web && npm run build && npm run preview

# Build TUI

cd build && cmake --build . --target ib_box_spread_tui

# Run TUI with Mock Data

./build/bin/ib_box_spread_tui --mock

# Full Rebuild (if needed)

./scripts/build_universal.sh
```

## Current Status

- ✅ **Core Calculations**: Complete
- ✅ **Data Structures**: Complete
- ✅ **Web UI**: Complete and ready to view
- ⚠️ **TUI Display**: Data populated but display components need updating
- ⏳ **Testing**: Ready for testing once apps are rebuilt

## Documentation

- Full analysis: `docs/BUY_SELL_DISPARITY_ANALYSIS.md`
- This file: `docs/NEXT_STEPS.md`
