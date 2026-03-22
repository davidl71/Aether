# Virtual Securities Integration Research

**Date**: 2025-11-17
**Status**: Research Complete
**Purpose**: Research integration possibilities for Interactive Brokers Virtual Securities with TWS API

---

## Executive Summary

**Key Finding**: Virtual Securities in TWS are **read-only visualization tools** and **cannot be created or managed programmatically via the TWS API**. However, there are alternative approaches for creating synthetic instruments for analysis and monitoring.

---

## What Are Virtual Securities?

According to the [Interactive Brokers Virtual Securities Documentation](https://www.ibkrguides.com/traderworkstation/virtual-securities.htm):

**Virtual Securities** allow you to:

- Create synthetic financial instruments by combining existing securities using an equation builder
- View current pricing and chart historical pricing for these synthetic instruments
- Use them in quote monitors and analytical tools within TWS
- **Cannot be used in any trading tools** (read-only)

**Example Use Cases:**

- Create custom spreads: `"SPY" - "QQQ"` (SPY minus QQQ)
- Create synthetic indices: `("AAPL" + "MSFT" + "GOOGL") / 3`
- Create ratio spreads: `"SPY" / "VIX"`
- Monitor box spread synthetic positions (theoretical)

---

## TWS API Limitations

### ❌ No Direct API Support

**Research Findings:**

1. **No Virtual Security secType**: The TWS API `Contract` structure supports secTypes like:
   - `STK` (Stock)
   - `OPT` (Option)
   - `BAG` (Combo/Leg)
   - `CASH` (Forex)
   - `FUT` (Future)
   - `BOND` (Bond)
   - But **no `VIRTUAL` secType**

2. **No API Methods**: The TWS API (`EClient.h`) has no methods for:
   - Creating Virtual Securities
   - Managing Virtual Security equations
   - Querying Virtual Security definitions

3. **Read-Only in TWS**: Virtual Securities are created manually in TWS UI and stored locally

### ✅ What the API CAN Do

The TWS API **can**:

- Request market data for Virtual Securities **if they already exist in TWS**
- Use `reqMktData()` with a Virtual Security ticker (created manually in TWS)
- Subscribe to real-time quotes for Virtual Securities
- Request historical data for Virtual Securities

**Example:**

```cpp
// If "BOX_SPX_5000_5100" was created manually in TWS as a Virtual Security
Contract contract;
contract.symbol = "BOX_SPX_5000_5100";
contract.secType = "STK";  // Virtual Securities appear as STK type
contract.exchange = "SMART";
contract.currency = "USD";

// Request market data
client->reqMktData(tickerId, contract, "", false, false, TagValueListSPtr());
```

---

## Alternative Approaches

### Option 1: Manual Creation + API Monitoring

**Approach:**

1. Manually create Virtual Securities in TWS UI for box spreads
2. Use TWS API to request market data for these Virtual Securities
3. Monitor prices programmatically

**Pros:**

- Works with existing TWS API
- Real-time pricing from TWS
- Historical data available

**Cons:**

- Requires manual setup in TWS
- Not scalable (must create each Virtual Security manually)
- Cannot create dynamically

**Implementation:**

```cpp
// After creating "BOX_SPX_5000_5100" manually in TWS
int tickerId = request_market_data_for_virtual_security("BOX_SPX_5000_5100");
```

---

### Option 2: Calculate Synthetically in Application

**Approach:**

1. Request market data for individual box spread legs
2. Calculate synthetic box spread price in application
3. Store and display calculated values

**Pros:**

- Fully programmatic
- No TWS UI dependency
- Can create any combination dynamically
- Can store in QuestDB for historical analysis

**Cons:**

- Must calculate prices yourself
- No TWS historical data for synthetic instrument
- Requires managing all legs

**Implementation:**

```cpp
// Request market data for all 4 legs
auto long_call_data = request_market_data(long_call_contract);
auto short_call_data = request_market_data(short_call_contract);
auto long_put_data = request_market_data(long_put_contract);
auto short_put_data = request_market_data(short_put_contract);

// Calculate box spread price
double box_price = long_call_data.bid - short_call_data.ask +
                   long_put_data.bid - short_put_data.ask;
```

**Current Implementation:**
This is **already what the project does** in `box_spread_strategy.cpp` - it calculates box spread prices from individual leg prices.

---

### Option 3: Use Combo Orders (BAG secType)

**Approach:**

1. Use TWS API combo order functionality
2. Create `BAG` (combo) contracts with 4 legs
3. Request market data for combo contracts

**Pros:**

- Native TWS API support
- Real-time pricing from TWS
- Can be used for actual trading

**Cons:**

- Requires contract IDs (conId) for each leg
- More complex setup
- May not provide synthetic pricing (shows individual leg prices)

**Implementation:**

```cpp
// Create combo contract
Contract combo;
combo.symbol = "SPX";
combo.secType = "BAG";
combo.currency = "USD";
combo.exchange = "SMART";

// Add combo legs
combo.comboLegs = std::make_shared<ComboLegList>();
// ... add 4 legs with conIds

// Request market data
client->reqMktData(tickerId, combo, "", false, false, TagValueListSPtr());
```

**Note:** This is what the project is implementing for atomic order execution (T-5).

---

### Option 4: Hybrid Approach (Recommended)

**Approach:**

1. **For Analysis**: Calculate synthetic prices in application (Option 2)
2. **For Trading**: Use combo orders (Option 3)
3. **For Visualization**: Optionally create Virtual Securities manually in TWS for monitoring

**Benefits:**

- Best of all worlds
- Programmatic calculation for strategy
- Real trading via combo orders
- Optional TWS visualization

---

## Integration Recommendations

### For Box Spread Strategy

**Current Approach (Recommended):**

- ✅ Calculate box spread prices from individual leg market data
- ✅ Store calculated prices in application
- ✅ Use for strategy decisions
- ✅ Execute via combo orders (when implemented)

**Virtual Securities Use Case:**

- ⚠️ **Not recommended** for strategy execution
- ✅ **Optional** for manual monitoring in TWS
- ✅ **Useful** for educational/visualization purposes

### Implementation Strategy

**Phase 1: Current (Already Implemented)**

- Calculate box spread prices from leg prices
- Use for opportunity detection
- Execute individual orders (or combo when available)

**Phase 2: Enhanced Monitoring (Future)**

- Store calculated box spread prices in QuestDB
- Create synthetic ticker feed for monitoring
- Optional: Create Virtual Securities manually in TWS for visualization

**Phase 3: Advanced Analysis (Future)**

- Build yield curve from box spread prices
- Compare calculated vs. Virtual Security prices (if manually created)
- Historical analysis in QuestDB

---

## Code Integration Points

### Current Codebase

**Box Spread Calculation:**

- Location: `native/src/box_spread_strategy.cpp`
- Method: `calculate_arbitrage_profit()`, `calculate_roi()`
- Already calculates synthetic box spread prices

**Market Data:**

- Location: `native/src/tws_client.cpp`
- Method: `request_market_data()`
- Can request data for individual legs

**Combo Orders:**

- Location: `native/src/order_manager.cpp`
- Status: Being implemented (T-5)
- Will support atomic execution

### Potential Enhancements

**1. Virtual Security Monitoring (Optional)**

```cpp
// If Virtual Security exists in TWS, request its market data
int request_virtual_security_data(const std::string& virtual_ticker) {
    Contract contract;
    contract.symbol = virtual_ticker;
    contract.secType = "STK";  // Virtual Securities appear as STK
    contract.exchange = "SMART";
    contract.currency = "USD";

    return request_market_data(contract, callback);
}
```

**2. Synthetic Price Feed**

```cpp
// Create synthetic price feed from calculated box spreads
void publish_synthetic_box_spread_price(
    const std::string& symbol,
    const BoxSpreadLeg& spread,
    double calculated_price
) {
    // Store in QuestDB or publish to subscribers
    // Can be used for monitoring/analysis
}
```

---

## Limitations & Constraints

### Virtual Securities Limitations

1. **Read-Only**: Cannot trade Virtual Securities
2. **Manual Creation**: Must be created in TWS UI
3. **No API Creation**: Cannot create programmatically
4. **Local Storage**: Stored locally in TWS, not on IB servers
5. **Not Scalable**: Must create each one manually

### Alternative Approach Benefits

1. **Fully Programmatic**: Calculate in application
2. **Dynamic**: Create any combination on-the-fly
3. **Scalable**: Can generate thousands of combinations
4. **Tradable**: Can execute actual trades
5. **Storable**: Can store in database for analysis

---

## Use Cases & Recommendations

### ✅ Recommended: Application-Level Calculation

**Use For:**

- Strategy execution
- Opportunity detection
- Real-time monitoring
- Historical analysis
- Yield curve construction

**Implementation:**

- Already implemented in `box_spread_strategy.cpp`
- Continue using this approach

### ⚠️ Optional: Manual Virtual Securities

**Use For:**

- Visual monitoring in TWS
- Educational purposes
- Quick manual checks
- TWS charting

**Implementation:**

- Create manually in TWS UI
- Use API to request market data (if needed)
- Not required for strategy execution

### ❌ Not Recommended: Relying on Virtual Securities

**Why:**

- Cannot create programmatically
- Not scalable
- Read-only (cannot trade)
- Manual setup required

---

## Future Possibilities

### Potential TWS API Enhancements

**If IB adds Virtual Security API support:**

- `reqCreateVirtualSecurity()` - Create Virtual Security
- `reqVirtualSecurityEquation()` - Get/Set equation
- `reqVirtualSecurityList()` - List all Virtual Securities
- Virtual Security `secType` in Contract structure

**Current Status:**

- No such API methods exist
- No indication from IB that this is planned
- Would require TWS API version update

### Workarounds

**Until API support exists:**

1. Continue using application-level calculation (current approach)
2. Use combo orders for execution
3. Optionally create Virtual Securities manually for visualization
4. Store synthetic prices in QuestDB for analysis

---

## References

- [Interactive Brokers Virtual Securities Guide](https://www.ibkrguides.com/traderworkstation/virtual-securities.htm)
- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [TWS API Initial Setup](https://interactivebrokers.github.io/tws-api/initial_setup.html)
- Project files:
  - `native/src/box_spread_strategy.cpp` - Box spread calculation
  - `native/src/tws_client.cpp` - Market data requests
  - `native/include/types.h` - Box spread data structures

---

## Conclusion

**Virtual Securities are not suitable for programmatic box spread trading** because:

1. Cannot be created via API
2. Read-only (cannot trade)
3. Require manual setup

**Recommended Approach:**

- ✅ Continue using application-level box spread calculation (already implemented)
- ✅ Use combo orders for atomic execution (being implemented)
- ✅ Store synthetic prices in QuestDB for analysis
- ⚠️ Optionally create Virtual Securities manually in TWS for visualization only

**The current implementation approach is correct and optimal for programmatic trading.**

---

**Document Status**: ✅ Complete - Comprehensive research on Virtual Securities integration possibilities
