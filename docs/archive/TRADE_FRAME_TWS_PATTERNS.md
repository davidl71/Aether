# Trade-Frame TWS Integration Patterns

**Date:** 2025-01-27
**Source:** Trade-Frame repository and TRADE_FRAME_LEARNINGS.md
**Purpose:** Extract key TWS connection establishment patterns from Trade-Frame for improving our implementation

---

## Overview

Trade-Frame's `TFInteractiveBrokers` library provides excellent patterns for TWS API integration, especially:

- Connection establishment
- EReader threading
- Multi-leg order management (ComboTrading example)
- Thread safety patterns

**Repository:** <https://github.com/rburkholder/trade-frame>
**Key Application:** `ComboTrading` - Perfect example for 4-leg box spreads

---

## 1. Connection Establishment Pattern

### Trade-Frame's Approach

**Key Patterns:**

1. **Dedicated EReader Thread:**
   - Separate thread for EReader message processing
   - Prevents blocking main application thread
   - Handles all TWS callbacks asynchronously

2. **Connection Lifecycle:**
   - Proper connection acknowledgment waiting
   - Connection state tracking
   - Automatic reconnection support
   - Graceful disconnection handling

3. **Error Handling:**
   - Comprehensive error handling
   - Connection state management on errors
   - Automatic recovery where possible

### Comparison with Our Implementation

| Aspect | Trade-Frame | Our Current | Status |
|--------|------------|-------------|--------|
| **EReader Thread** | ✅ Dedicated thread | ✅ Dedicated thread | ✅ **ALREADY CORRECT** |
| **Mutex Strategy** | ✅ Separate mutexes | ✅ Separate mutexes | ✅ **ALREADY CORRECT** |
| **Connection Waiting** | ✅ Proper acknowledgment | ✅ Condition variable | ✅ **ALREADY CORRECT** |
| **Error Handling** | ✅ Comprehensive | ⚠️ Needs improvement | **IMPROVE THIS** |
| **Reconnection** | ✅ Automatic | ✅ Exponential backoff | ✅ **ALREADY GOOD** |

**Key Finding:** Our TWS connection patterns are already aligned with Trade-Frame's best practices! ✅

**What to Improve:**

1. Add more error codes to error guidance map
2. Enhance connection state management
3. Improve error context logging

---

## 2. EReader Threading Pattern

### Trade-Frame's Pattern

```cpp
// Trade-Frame approach (from TRADE_FRAME_LEARNINGS.md):

1. Dedicated EReader Thread:
   - Separate thread for EReader message processing
   - Prevents blocking main application thread
   - Handles all TWS callbacks asynchronously

2. Thread Safety:
   - Mutex protection for shared data structures
   - Separate mutexes for different data types:
     * Market data mutex
     * Order state mutex
     * Position mutex
     * Account mutex
```

### Our Current Implementation

We follow this pattern in the **Rust** IBKR adapter. The legacy C++ template (`docs/TWS_INTEGRATION_TEMPLATE.cpp`) was removed when the native build was retired; equivalent reader/event-loop logic lives in **`agents/backend/crates/ib_adapter`** (Rust).

---

## 3. Multi-Leg Order Management (ComboTrading Example)

### Trade-Frame's ComboTrading Application

**Perfect for Box Spreads!** Box spreads are 4-leg orders, exactly what ComboTrading demonstrates.

**Key Patterns:**

1. **Combo Order Creation:**

   ```cpp
   // Use IBKR Combo Orders (BAG secType) for atomic execution
   - All legs in single order
   - Guaranteed all-or-nothing execution
   - No partial fills possible
   ```

2. **Leg Synchronization:**

   ```cpp
   // Track all legs together
   - Monitor fill status across all legs
   - Check if combo order is complete
   - Handle order status updates
   ```

3. **Rollback Logic:**

   ```cpp
   // If using individual orders (fallback):
   - If any leg fails, cancel remaining legs
   - Track order IDs for rollback
   - Rapid cancellation capability
   ```

### Application to Box Spreads

**Box Spread Structure:**

- Long call at K1
- Short call at K2
- Long put at K2
- Short put at K1

**Trade-Frame Pattern Application:**

1. **Use Combo Orders (Recommended):**

   ```cpp
   // Create BAG (basket) order with 4 legs
   Contract combo;
   combo.symbol = "SPY";
   combo.secType = "BAG";  // Basket/combo order

   // Add all 4 legs
   ComboLeg leg1, leg2, leg3, leg4;
   // ... configure legs ...

   combo.comboLegs.push_back(leg1);
   combo.comboLegs.push_back(leg2);
   combo.comboLegs.push_back(leg3);
   combo.comboLegs.push_back(leg4);

   // Place as single order - atomic execution!
   placeOrder(orderId, combo, order);
   ```

2. **Leg Tracking (if using individual orders):**

   ```cpp
   // Track all 4 legs together
   struct BoxSpreadOrder {
       OrderId long_call_id;
       OrderId short_call_id;
       OrderId long_put_id;
       OrderId short_put_id;
       bool all_filled = false;
   };

   // Monitor fill status
   void onOrderFilled(OrderId orderId) {
       // Check if this is part of box spread
       // Update fill status
       // If all filled, mark complete
       // If any failed, cancel remaining
   }
   ```

3. **Rollback Strategy:**

   ```cpp
   // If combo order rejected, no rollback needed (atomic)
   // If using individual orders:
   void rollback_box_spread(BoxSpreadOrder& spread) {
       if (spread.long_call_id > 0) cancelOrder(spread.long_call_id);
       if (spread.short_call_id > 0) cancelOrder(spread.short_call_id);
       if (spread.long_put_id > 0) cancelOrder(spread.long_put_id);
       if (spread.short_put_id > 0) cancelOrder(spread.short_put_id);
   }
   ```

**Recommendation:** Study Trade-Frame's ComboTrading example in detail - it's perfect for box spreads!

---

## 4. Thread Safety Patterns

### Trade-Frame's Approach

```cpp
// Separate mutexes for different concerns:

1. Market Data Mutex:
   - Protects tick data storage
   - Prevents race conditions on price updates
   - Allows concurrent reads where safe

2. Order Mutex:
   - Protects order state
   - Prevents race conditions on order updates
   - Ensures order status consistency

3. Position Mutex:
   - Protects position tracking
   - Prevents race conditions on position updates
   - Ensures position accuracy

4. Account Mutex:
   - Protects account data
   - Prevents race conditions on account updates
   - Ensures account balance accuracy
```

### Our Current Implementation

We already follow this pattern! ✅

```cpp
// Our current implementation:

class TWSClient::Impl {
private:
    std::mutex connection_mutex_;  // Connection state
    std::mutex data_mutex_;        // Market data
    std::mutex order_mutex_;       // Orders
    std::mutex position_mutex_;    // Positions
    std::mutex account_mutex_;     // Account info
    std::mutex error_mutex_;       // Error tracking
};
```

**Status:** ✅ **ALREADY CORRECT** - We're using the same pattern as Trade-Frame!

---

## 5. Key Learnings for Our Implementation

### What We're Already Doing Right ✅

1. **EReader Threading** - Dedicated thread, correct pattern
2. **Mutex Strategy** - Separate mutexes for different concerns
3. **Connection Waiting** - Condition variable for acknowledgment
4. **Reconnection** - Exponential backoff implemented

### What to Improve ⚠️

1. **Error Handling:**
   - Add more error codes to guidance map
   - Improve error context logging
   - Better error recovery strategies

2. **Multi-Leg Orders:**
   - Study ComboTrading example in detail
   - Implement combo orders (BAG secType) for atomic execution
   - Add rollback logic if using individual orders

3. **Connection State Management:**
   - Enhance state synchronization
   - Add position/account sync after reconnection
   - Track reconnection state

---

## 6. Action Items

### Immediate (This Week)

- [ ] **Study Trade-Frame's ComboTrading example** - Perfect for box spreads!
- [ ] **Review Trade-Frame's TWS connection code** - Connection establishment patterns
- [ ] **Compare our error handling** - See what Trade-Frame does differently
- [ ] **Review ComboTrading multi-leg patterns** - Apply to box spread implementation

### Short-Term (Next 2 Weeks)

- [ ] Implement combo orders (BAG secType) for box spreads
- [ ] Add rollback logic for individual orders (fallback)
- [ ] Enhance error handling based on Trade-Frame patterns
- [ ] Improve connection state management

---

## 7. Resources

- **Trade-Frame Repository:** <https://github.com/rburkholder/trade-frame>
- **ComboTrading Example:** See `ComboTrading/` directory in Trade-Frame
- **TFInteractiveBrokers Library:** See `lib/TFInteractiveBrokers/` directory
- **Our Learnings Document:** `docs/TRADE_FRAME_LEARNINGS.md`

---

## 8. Conclusion

**Key Findings:**

1. ✅ **Our TWS connection patterns are already correct!** - We're following Trade-Frame's best practices
2. ✅ **Our threading patterns are correct!** - Dedicated EReader thread, separate mutexes
3. ⚠️ **Multi-leg orders need improvement** - Study ComboTrading example for box spreads
4. ⚠️ **Error handling can be enhanced** - Learn from Trade-Frame's comprehensive approach

**Recommendation:**

- **Don't change connection/threading patterns** - They're already correct
- **Study ComboTrading example** - Perfect for implementing box spread orders
- **Enhance error handling** - Add more error codes and better recovery
- **Implement combo orders** - Use BAG secType for atomic box spread execution

---

**Last Updated:** 2025-01-27
**Status:** ✅ Analysis Complete - Ready for Implementation
