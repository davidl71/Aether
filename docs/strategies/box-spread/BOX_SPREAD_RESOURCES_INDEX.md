# Box Spread Resources Index

<!--
@index: box-spread-resources
@category: box-spread
@tags: box-spread, arbitrage, options-strategies, borrowing-lending, cboe, cme
@last-updated: 2025-01-27
-->

**Purpose**: Focused index of all box spread trading resources, research, and educational materials.

**Full Documentation**: See `API_DOCUMENTATION_INDEX.md` for complete details.

---

## Quick Reference

### Research & Educational Resources

| Resource | Type | Focus | Key Takeaway |
|----------|------|-------|--------------|
| **Cboe – Box Spreads as Alternative Borrowing & Lending** | Article | Borrowing/lending via box spreads | Competitive rates vs T-bills, OCC clearing reduces risk |
| **OCC Options Education – Box Spread Strategies** | PDF Guide | Comprehensive box spread guide | Construction, implied rates, tax treatment (Section 1256) |
| **SyntheticFi** | Implementation Example | Box spread-based lending | YC-backed fintech using SPX box spreads for loans |
| **CME Group – Capital Efficiencies** | Whitepaper | Capital efficiency comparison | Compare box spreads vs futures-based strategies |

### Market Structure Resources

| Resource | Type | Focus |
|----------|------|-------|
| **Cboe Quoted Spread Book (QSB)** | Service | Box spread quoting on CBOE |
| **Cboe Frequent Trader Program (FTID)** | Fee Program | Fee rebates for high-volume options trading |
| **Cboe EDGX Fee Schedule** | Fee Structure | Market data and connectivity fees |
| **CME Group Fee Schedules** | Fee Structure | Futures and options fee structures |

---

## Decision Tree

### Which Resource Should I Use?

```
Need to understand box spread mechanics?
  → OCC Options Education PDF (comprehensive guide)

Need to compare box spreads to alternatives?
  → Cboe article (vs T-bills) or CME whitepaper (vs futures)

Need implementation examples?
  → SyntheticFi (YC-backed fintech example)

Need CBOE-specific information?
  → Cboe QSB FAQ, FTID program, EDGX fee schedule

Need CME-specific information?
  → CME Group whitepapers and fee schedules
```

---

## Resource Details

### Educational Resources

#### Cboe – Box Spreads as Alternative Borrowing & Lending

- **URL**: <https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/>
- **Author**: Dr. Wesley R. Gray (Alpha Architect)
- **Key Points**:
  - Box spreads replicate risk-free borrowing/lending via put-call parity
  - Competitive rates vs Treasury bills
  - OCC clearing mitigates counterparty risk

- **Documentation**: `../API_DOCUMENTATION_INDEX.md#cboe-box-spreads`

#### OCC Options Education – Box Spread Strategies

- **PDF**: <https://www.optionseducation.org/getmedia/2ae6c8bd-9a8e-4d2f-8168-19b6ff9e3589/listed-options-box-spread-strategies-for-borrowing-or-lending-cash.pdf>
- **Key Topics**:
  - Box spread construction (bull call + bear put)
  - Implied interest rate calculations
  - Tax treatment (Section 1256)
  - Capital efficiency through portfolio margining

- **Documentation**: `../API_DOCUMENTATION_INDEX.md#occ-box-spread-strategies`

### Implementation Examples

#### SyntheticFi

- **Website**: <https://www.syntheticfi.com/>
- **YC Profile**: <https://www.ycombinator.com/companies/syntheticfi>
- **Description**: YC-backed fintech using box spreads for securities-backed lending
- **Key Features**:
  - Loans at rates 1-3% lower than traditional lenders
  - Uses SPX options for box spread construction
  - Full tax-deductible interest expenses

- **Documentation**: `../API_DOCUMENTATION_INDEX.md#syntheticfi`

### Market Structure

#### Cboe Quoted Spread Book (QSB)

- **Document**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>
- **Focus**: Box spread quoting on CBOE Complex Order Books
- **Key Features**:
  - Box spreads on SPX contracts at 4000 and 5000 strikes
  - Market Maker rest orders in Complex Order Books
  - ~10 quotable instruments daily

- **Documentation**: `../API_DOCUMENTATION_INDEX.md#cboe-qsb`

#### Cboe Frequent Trader Program (FTID)

- **Document**: <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf>
- **Focus**: Fee rebate program for high-volume options trading
- **Benefits**: Reduced fees for qualifying traders
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#cboe-ftid`

---

## Use Cases

### Learning Box Spread Mechanics

- **Start**: OCC Options Education PDF
- **Deep Dive**: Cboe article by Dr. Wesley R. Gray
- **Implementation**: SyntheticFi example

### Comparing Strategies

- **Box Spreads vs T-bills**: Cboe article
- **Box Spreads vs Futures**: CME Group whitepapers
- **Capital Efficiency**: CME Group AIR TRFs whitepaper

### CBOE Integration

- **Quoting**: Cboe QSB FAQ
- **Fee Optimization**: Cboe FTID program
- **Fee Structure**: Cboe EDGX fee schedule

### CME Integration

- **Financing Comparison**: CME Group whitepapers
- **Fee Structure**: CME Group fee schedules
- **Integration**: CME Client Systems Wiki

---

## See Also

- **Full Documentation**: `../API_DOCUMENTATION_INDEX.md#market-structure-efficiency-references`
- **Summary**: `../API_DOCUMENTATION_SUMMARY.md`
- **NotebookLM Suggestions**: `../NOTEBOOKLM_API_DOCUMENTATION_SUGGESTIONS.md`
- **CME Research**: `../CME_RESEARCH.md`
- **SyntheticFi Analysis**: `../SYNTHETICFI_LENDING_BORROWING_ANALYSIS.md`
