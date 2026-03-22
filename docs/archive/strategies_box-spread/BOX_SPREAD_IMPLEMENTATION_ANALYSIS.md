# Box Spread Implementation Analysis vs Industry Best Practices

**Date**: 2026-03-19
**Sources**: CBOE article (Long-Dated Box Spreads: A Better Way to Buy a Home), SyntheticFi platform, OCC Options Education PDF
**Purpose**: Compare our implementation against industry standards for box spread financing

---

## 1. Industry Overview

### SyntheticFi (Current Market Leader)
- **Fixed rate**: 3.70% (5-year term)
- **Floating rate**: 3.95% (line of credit)
- **Underlying**: SPX options (CBOE)
- **Max term**: 5 years
- **Margin**: 50% Reg T, 85% Portfolio Margin
- **Tax**: Section 1256 - "interest" treated as capital loss (60% long-term, 40% short-term)

### Key Insights from CBOE Article
1. **Box = loan**: Combine Long Call K1 + Short Call K2 + Long Put K2 + Short Put K1 → fixed payout at expiration
2. **Net debit** = upfront premium = loan principal
3. **Implied rate** ≈ Fed Funds + spread (30-50 bps for liquid SPX boxes)
4. **Advantages over mortgages**:
   - No down payment (margin collateral only)
   - Lower rates (~4.6% vs 7%+)
   - Fully tax-deductible (no cap like mortgage $750K)
   - Stay fully invested

---

## 2. Our Implementation Analysis

### 2.1 Box Spread Identification ✅ GOOD
**File**: `combo_strategy.rs`
**What we do**:
- Parse option symbols to extract stem, strike, call/put
- Group positions by `(account_id, strategy, symbol_stem)`
- Infer Box (4 legs, 2 strikes, 2 calls + 2 puts) vs Vertical (2 legs, same type)
- Fallback to broker `strategy` field for BAG positions

**Gap**: None significant. Correctly identifies Box vs Vertical.

### 2.2 Box Spread Pricing ✅ GOOD
**File**: `quant/src/lib.rs:518-595`
**What we do**:
```rust
pub fn calculate_box_spread(&self, s, k_low, k_high, t_years, r, sigma)
```
- Computes synthetic leg cost: `(long_call - long_put).abs()`
- Computes actual leg cost: `s - k_high` (intrinsic value via put-call parity)
- Net cost = synthetic_leg_cost - actual_leg_cost
- Annualized rate = `(net_cost / actual_leg_cost) * (365 / days)`
- Flags arbitrage when `rate > r * 1.5`

**Gap**: None. Formula is correct for Box spread implied rate.

### 2.3 Combo Net Bid/Ask from Marks ⚠️ PARTIAL
**File**: `combo_strategy.rs:199-209`
**What we do**:
- `combo_net_bid/ask = sum(leg.mark * leg.quantity)`
- Source = "leg_sum_mark"

**Issue**: Using mark price (last/trade) instead of bid-ask. Mark can be stale.
**Missing**: Per-leg bid/ask when available → "leg_sum" source.

### 2.4 Yield Curve from TWS ✅ GOOD (when enabled)
**File**: `tws_yield_curve/` crate
**What we do**:
- Connect to TWS, request option chain
- Subscribe to 4 legs of box
- Compute `BoxSpreadInput` with bid/ask/net_debit/net_credit

**Gap**: None when TWS is connected. Synthetic fallback when not.

### 2.5 Strategy Execution ❌ MISSING
**File**: `ib_adapter/src/types.rs:96-118`
**What we do**:
- Define `ComboLeg` and `ComboOrder` types for BAG orders
- Support 4 legs with ratio/action

**Gap**: We can *describe* box spreads but not *construct* them programmatically.
- No function to generate 4-leg order from (k_low, k_high, expiry, long/short)
- No put-call parity validation before order

### 2.6 Margin Calculation ✅ GOOD
**File**: `margin.rs`
**What we do**:
- `BoxSpreadMarginInput` with net_debit
- Reg T and Portfolio Margin calculations
- Maintenance requirement calculations

**Gap**: None significant.

---

## 3. Implementation Gaps Summary

| Component | Status | Gap |
|-----------|--------|-----|
| Box identification | ✅ | None |
| Box pricing (implied rate) | ✅ | None |
| Combo net bid/ask | ⚠️ | Using marks, not bid-ask |
| TWS yield curve | ✅ | None (when enabled) |
| **Box order construction** | ❌ | No 4-leg order generator |
| **Put-call parity validation** | ❌ | No pre-trade check |
| **Section 1256 tax tracking** | ❌ | Not implemented |
| **Term loan structure (fixed rate)** | ❌ | No maturity matching |

---

## 4. Recommended Actions

### High Priority
1. **Box order generator** (`ib_adapter`): Create function to build 4-leg BAG order from parameters
   ```rust
   pub fn box_spread_order(
       symbol: &str,
       k_low: f64,
       k_high: f64,
       expiry: &str,
       is_long: bool,  // long box = borrow, short box = lend
       quantity: i32,
   ) -> ComboOrder
   ```

2. **Pre-trade validation** (`quant`): Verify put-call parity before box execution
   - Check that `call - put ≈ s - k` within tolerance

### Medium Priority
3. **Bid-ask net quote** (`combo_strategy`): Use bid/ask when available instead of marks
   - Add `combo_quote_source = "twap"` or `"bid_ask"` variants

4. **Section 1256 tracking** (`ledger`): Mark-to-market P&L as capital gains/losses
   - 60% long-term, 40% short-term treatment

### Low Priority
5. **Term matching**: For fixed-rate loans, match box expiry to loan maturity
   - Currently we have 5-year SPX options but no term structure

---

## 5. Reference Calculations

### Box Implied Rate (from SyntheticFi article)
```
Loan = Box Width - Net Premium Received
Implied Rate = (Loan / Box Width) ^ (365 / Days) - 1
```

### Example (SPX 5000/5100, 5-year)
- Box width: $100
- Net premium: $96 (implying 4% rate)
- Annualized: `(100-96)/100 * 365/1825 ≈ 0.8%` (plus spread)

### Our Calculation (`quant/src/lib.rs:548`)
```rust
let annualized_rate = if net_cost > 0.0 {
    (net_cost / actual_leg_cost) * (365.0 / days as f64)
} else {
    0.0
};
```
Where `actual_leg_cost = s - k_high` (net debit paid upfront)

---

## 6. Files Reference

| File | Purpose |
|------|---------|
| `agents/backend/crates/api/src/combo_strategy.rs` | Box/Vertical identification |
| `agents/backend/crates/quant/src/lib.rs:518-595` | Box spread pricing |
| `agents/backend/crates/quant/src/margin.rs` | Margin calculations |
| `agents/backend/crates/api/src/finance_rates.rs` | Yield curve building |
| `agents/backend/crates/tws_yield_curve/` | TWS option chain → box spread |
| `agents/backend/crates/ib_adapter/src/types.rs` | BAG order types |
| `agents/backend/crates/ib_adapter/src/lib.rs:268` | Place bag order |
| `agents/backend/services/tui_service/src/ui/positions.rs` | Combo grouping UI |

---

## 7. Conclusion

Our implementation covers **pricing**, **identification**, and **margin** for box spreads. The core implied rate calculation is correct and aligns with industry practice.

**Critical missing piece**: programmatic **order construction** (building the 4-leg BAG from strike parameters). This is needed to actually execute box spread trades through IBKR.

**Secondary gaps**: bid-ask instead of marks for net quote, Section 1256 tax treatment, and term structure matching for fixed-rate loans.
