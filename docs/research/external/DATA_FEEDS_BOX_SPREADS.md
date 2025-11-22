# Data Feeds for Quoted Box Spreads

**Date**: 2025-01-27
**Purpose**: Document which data feeds can get data for quoted box spreads, especially CBOE and CME

---

## Summary

### Current Status: ❌ No Direct Quoted Box Spread Support

The codebase currently **does not** have direct integration for quoted box spreads from any exchange. Box spreads are built from individual option leg quotes.

### Available Data Feeds

| Data Feed | Individual Options | Quoted Box Spreads | Status |
|-----------|-------------------|-------------------|--------|
| **IBKR TWS API** | ✅ Yes | ❌ No (combo orders not implemented) | Primary |
| **ORATS** | ✅ Yes | ❌ No | Fallback |
| **Nautilus Trader** | ✅ Yes (framework) | ❌ No | Framework |
| **CBOE QSB** | ❌ N/A | ⚠️ Available but not integrated | Not integrated |
| **CME** | ❌ No | ❌ No | Not integrated |
| **Polygon** | ❌ No (stocks only) | ❌ No | Rust backend only |

---

## CBOE Quoted Spread Book (QSB)

### What is CBOE QSB?

CBOE's Quoted Spread Book (QSB) service allows Market Makers to rest orders directly in Complex Order Books (COBs) for select spread instruments during Regular Trading Hours (RTH).

### Box Spread Availability

**Instruments Available**:

- **Box Spreads**: First four serial, first three quarterly, and first three December standard SPX contracts at 4000 and 5000 strikes
- **Approximately**: ~10 quotable box spread instruments daily
- **Exchange**: CBOE (CBOE, CBOE2, EDGX)

**Additional Instruments**:

- **Box Swaps**: ~25 quotable instruments daily
- **Jelly Rolls**: ~120 quotable instruments daily

### How to Access QSB Quotes

#### Option 1: CBOE Reference Data (JSON/HTML)

- **URL**: Cboe U.S. Options Reference Data webpage
- **Format**: JSON/HTML
- **Availability**: Complete list available by 7:00 a.m. ET each trading day
- **Update Frequency**: Daily
- **Use Case**: Symbol discovery, instrument identification

#### Option 2: Complex PITCH/TOP Feeds (Real-time)

- **Feeds**: Complex PITCH and TOP feeds
- **Message Type**: EDCID (Exchange Designated Complex Instrument Definition)
- **Update Frequency**: Real-time
- **Use Case**: Real-time QSB instrument tracking, live quotes
- **Access**: Requires CBOE market data subscription

#### Option 3: TWS API (Indirect)

- **Status**: Not directly supported
- **Workaround**: Build box spreads from individual option legs
- **Limitation**: No direct access to QSB quotes

### Integration Requirements

To integrate CBOE QSB quotes:

1. **Market Data Subscription**:
   - Subscribe to CBOE Complex PITCH/TOP feeds
   - Obtain CBOE market data license
   - Set up feed connectivity (direct or via vendor)

2. **EDCID Message Parsing**:
   - Parse EDCID messages from Complex PITCH/TOP feeds
   - Extract box spread instrument definitions
   - Track QSB instrument availability

3. **Reference Data Integration**:
   - Parse QSB reference data (JSON/HTML)
   - Maintain QSB instrument list
   - Update daily at 7:00 a.m. ET

4. **Quote Subscription**:
   - Subscribe to QSB COB quotes
   - Monitor bid/ask for box spread instruments
   - Track market depth if available

### Documentation

- **QSB FAQ**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>
- **Reference Data**: Cboe U.S. Options Reference Data webpage
- **Complex Feeds**: CBOE Complex PITCH/TOP feed documentation
- **Contact**: `cboelabs@cboe.com` for additional instruments

### Integration Status

**Current**: ❌ Not integrated
**Priority**: Medium (if CBOE box spreads are target)
**Effort**: High (requires market data subscription, feed integration, message parsing)

---

## CME Options Data

### Current Status

**CME Options**: ❌ Not integrated
**CME Futures Options**: ❌ Not integrated
**CME Market Data**: ❌ Not integrated

### CME Research

The codebase includes CME research documentation (`docs/CME_RESEARCH.md`), but no actual CME data feed integration exists.

### CME Market Data Access

**Licensed Distributors**:

- **Directory**: <https://www.cmegroup.com/market-data/license-data/licensed-market-data-distributors.html>
- **Purpose**: Official directory of authorized CME market data distributors
- **Use Case**: Sourcing compliant CME market data feeds and understanding licensing partners

**CME Client Systems Wiki**:

- **Portal**: <https://cmegroupclientsite.atlassian.net/wiki/spaces/EPICSANDBOX/overview?homepageId=457314687>
- **Scope**: Reference data, Globex connectivity, clearing services, test environments
- **Access**: Some content requires authenticated CME client credentials
- **Use Case**: Integration workflows, API specs, settlement schedules

### Integration Requirements

To integrate CME options data:

1. **Market Data License**:
   - Obtain CME market data license
   - Choose licensed distributor
   - Set up feed connectivity

2. **CME Globex Connectivity**:
   - Connect to CME Globex market data feed
   - Parse market data messages
   - Handle options-specific data structures

3. **Reference Data**:
   - Access CME reference data
   - Track options contracts, expirations, strikes
   - Monitor contract roll schedules

4. **Clearing Integration** (if needed):
   - Integrate with CME clearing systems
   - Handle margin requirements
   - Process settlement data

### Integration Status

**Current**: ❌ Not integrated
**Priority**: Low (system focuses on equity options)
**Effort**: Very High (requires CME licensing, Globex connectivity, options data parsing)

---

## IBKR TWS API Combo Orders

### Current Status

**Individual Options**: ✅ Supported
**Combo Orders**: ⚠️ Placeholder (not implemented)
**Quoted Box Spreads**: ❌ Not supported

### TWS API Combo Order Support

The TWS API supports combo orders via the `ComboLeg` structure, but this is **not yet implemented** in the codebase.

**Placeholder Code**:

```cpp
// python/integration/order_factory.py:235-256
def create_combo_order(
    self,
    legs: List[dict],
    time_in_force: TimeInForce = TimeInForce.DAY,
) -> Optional[Order]:
    """
    Create a combo order (if supported by venue).

    Note: IBKR combo orders require special handling.
    This is a placeholder for future implementation.
    """
    # TODO: Implement IBKR combo order support
    # IBKR combo orders use ComboLeg structure
    logger.warning("Combo orders not yet implemented, using individual orders")
    return None
```

### Implementation Plan

From `docs/ACTION_PLAN.md`:

**Priority 2: Implement Atomic Execution (All-or-Nothing)**

**Option A (Preferred)**: Use IBKR combo orders

- Create `ComboLeg` structures for all 4 legs
- Place as single combo order
- Guarantees all-or-nothing execution

**Option B (Fallback)**: Implement rollback logic

- Place all 4 orders rapidly
- Monitor fill status
- If any leg fails, cancel remaining orders

### TWS API Combo Order Structure

```cpp
// native/third_party/tws-api/IBJts/source/cppclient/client/Order.h:181-185
// order combo legs
typedef std::vector<OrderComboLegSPtr> OrderComboLegList;
typedef std::shared_ptr<OrderComboLegList> OrderComboLegListSPtr;

OrderComboLegListSPtr orderComboLegs;
```

### Integration Status

**Current**: ⚠️ Placeholder (not implemented)
**Priority**: High (from ACTION_PLAN.md)
**Effort**: Medium (requires TWS API combo order implementation)

---

## Recommended Integration Priority

### 1. IBKR Combo Orders (High Priority)

- **Why**: Atomic execution, better fill guarantees
- **Effort**: Medium
- **Benefit**: Improved execution quality, reduced partial fill risk
- **Status**: Placeholder exists, needs implementation

### 2. CBOE QSB Integration (Medium Priority)

- **Why**: Direct access to quoted box spreads on CBOE
- **Effort**: High (requires market data subscription, feed integration)
- **Benefit**: Access to QSB quotes, better execution opportunities
- **Status**: Not integrated, requires significant work

### 3. CME Options Data (Low Priority)

- **Why**: Expand to futures options if needed
- **Effort**: Very High (requires CME licensing, Globex connectivity)
- **Benefit**: Access to CME options, futures-based box spreads
- **Status**: Not integrated, not a current focus

---

## Alternative Approaches

### Current Approach: Build from Individual Legs

**How it works**:

1. Get individual option quotes from TWS API
2. Calculate box spread price from 4 legs
3. Place 4 separate orders (or combo order when implemented)

**Advantages**:

- Works with existing TWS API integration
- No additional market data subscriptions
- Flexible (can build any strike combination)

**Disadvantages**:

- No direct access to exchange-quoted box spreads
- Execution risk (partial fills)
- May miss QSB opportunities on CBOE

### Future Approach: Direct QSB Quotes

**How it would work**:

1. Subscribe to CBOE QSB quotes via Complex PITCH/TOP feeds
2. Receive direct box spread quotes from exchange
3. Execute against QSB quotes (better execution)

**Advantages**:

- Direct access to exchange quotes
- Better execution (lit Market Maker quotes)
- Reduced execution risk

**Disadvantages**:

- Requires CBOE market data subscription
- Limited to QSB instruments (~10 daily)
- Additional integration complexity

---

## Next Steps

### Immediate (Week 1-2)

1. **Implement IBKR Combo Orders**:
   - Add `ComboLeg` structure support
   - Implement `create_combo_order` method
   - Test with TWS API

### Short-term (Week 3-4)

1. **Evaluate CBOE QSB Integration**:
   - Research CBOE market data subscription requirements
   - Assess cost vs. benefit
   - Design integration architecture

### Medium-term (Month 2-3)

1. **CBOE QSB Integration** (if approved):
   - Subscribe to CBOE Complex PITCH/TOP feeds
   - Implement EDCID message parsing
   - Integrate QSB quote subscription
   - Test with live data

### Long-term (Future)

1. **CME Options Integration** (if needed):
   - Evaluate CME licensing requirements
   - Design CME integration architecture
   - Implement CME options data feed

---

## References

- **CBOE QSB FAQ**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>
- **CBOE Reference Data**: Cboe U.S. Options Reference Data webpage
- **CME Licensed Distributors**: <https://www.cmegroup.com/market-data/license-data/licensed-market-data-distributors.html>
- **CME Client Systems Wiki**: <https://cmegroupclientsite.atlassian.net/wiki/spaces/EPICSANDBOX/overview?homepageId=457314687>
- **IBKR Combo Orders**: <https://interactivebrokers.github.io/tws-api/combo_orders.html>
- **TWS API Documentation**: <https://interactivebrokers.github.io/tws-api/>

---

## Notes

- **Current Focus**: Equity options (SPX/SPXW) via TWS API
- **QSB Integration**: Requires CBOE market data subscription and feed integration
- **CME Integration**: Not a current focus, would require significant effort
- **Combo Orders**: High priority for atomic execution, but not yet implemented
- **Alternative**: Continue building box spreads from individual legs until QSB integration is complete

---

**Last Updated**: 2025-01-27
