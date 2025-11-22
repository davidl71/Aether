# Primary Goals and Requirements

**Date**: 2025-11-19
**Purpose**: Document primary user goals for the synthetic financing platform

---

## Primary Goals

### 1. Unified Position View

**Goal**: See all current positions in one panel (TUI/PWA)

**Instruments to Display**:
- Box spreads (synthetic financing positions)
- Bank loans (e.g., Discount Bank)
- Pension loans
- Bonds/T-bills
- Other financing instruments

**Requirements**:
- Single unified panel showing all positions
- Group by instrument type
- Show key metrics: rate, maturity, cash flow, collateral
- Real-time updates
- Available in both TUI and PWA

**Current State**:
- ✅ Positions tracking exists (PositionSnapshot, PositionsTable)
- ✅ Bank accounts panel exists (BankAccountsPanel)
- ❌ Not unified - separate panels for different instrument types
- ❌ Missing: Box spread positions, pension loans, bonds

**Task**: T-125

---

### 2. Cash Flow Modeling

**Goal**: Model and project cash flows across all positions

**Requirements**:
- Track cash inflows/outflows from all positions
- Project future cash flows based on maturity dates
- Calculate net cash flow at any point in time
- Handle multiple currencies
- Account for interest payments, principal repayments, dividends

**Use Cases**:
- "What's my cash flow next month?"
- "When do I need to repay this loan?"
- "How much cash will I have available for new opportunities?"

**Current State**:
- ✅ Ledger system tracks transactions
- ✅ Position tracking exists
- ❌ No cash flow projection/modeling
- ❌ No future cash flow calculation

**Task**: T-126

---

### 3. Opportunity Simulation

**Goal**: Simulate what-if scenarios for loan usage and optimization

**Key Scenarios**:

#### Scenario 1: Loan Consolidation
- "I have a loan at 5% APR. Can I use cash flow to consolidate other loans?"
- Simulate: Use loan proceeds to pay off higher-rate loans
- Calculate: Net benefit, cash flow impact, risk reduction

#### Scenario 2: Margin for Box Spreads
- "I have a loan at 4% APR. Can I use it as margin for box spreads?"
- Simulate: Use loan as collateral for box spread margin
- Calculate: Effective financing rate, net benefit vs. direct box spread

#### Scenario 3: Investment Fund Strategy
- "I have a loan at 3% APR. Can I invest in a fund and get cheaper loans?"
- Simulate: Use loan to invest in fund, use fund as collateral for cheaper loan
- Calculate: Net benefit, cash flow impact, risk profile

#### Scenario 4: Multi-Instrument Optimization
- "What's the optimal chain: loan → margin → box spread → fund → cheaper loan?"
- Simulate: Complete multi-instrument relationship chain
- Calculate: Total benefit, cash flow, risk, capital efficiency

**Requirements**:
- Interactive what-if analysis
- Real-time simulation as user changes parameters
- Compare multiple scenarios side-by-side
- Show cash flow impact, net benefit, risk metrics
- Visualize relationships between instruments

**Current State**:
- ✅ Opportunity evaluation exists (BoxSpreadStrategy)
- ✅ Risk calculator exists
- ❌ No what-if simulation engine
- ❌ No multi-instrument relationship modeling
- ❌ No scenario comparison

**Task**: T-127

---

### 4. Multi-Instrument Relationship Modeling

**Goal**: Model relationships between instruments (loan → margin → box spread → fund → cheaper loan)

**Relationship Types**:
1. **Collateral Relationships**: Asset can be used as collateral for another
2. **Financing Relationships**: Asset provides financing for another
3. **Cash Flow Relationships**: Asset generates cash flow used by another
4. **Optimization Chains**: Optimal sequence of instrument usage

**Example Chain**:
```
Bank Loan (5% APR)
  ↓ (use as collateral)
Box Spread Margin (4% implied rate)
  ↓ (use proceeds)
Investment Fund (6% return)
  ↓ (use fund as collateral)
Cheaper Loan (3% APR)
  ↓ (net benefit: 2% spread)
```

**Requirements**:
- Model asset relationships (from SYNTHETIC_FINANCING_ARCHITECTURE.md)
- Find optimal chains
- Calculate net benefit of chains
- Visualize relationships
- Real-time updates as positions change

**Current State**:
- ✅ Architecture document exists (SYNTHETIC_FINANCING_ARCHITECTURE.md)
- ✅ Asset relationship graph design exists
- ❌ Not implemented in code
- ❌ No relationship modeling engine

**Task**: T-128

---

## Implementation Priority

### Phase 1: Foundation (High Priority)
1. **T-125**: Unified positions panel
   - Enables visibility into all positions
   - Foundation for other features

2. **T-126**: Cash flow modeling
   - Core requirement for opportunity simulation
   - Needed for all what-if scenarios

### Phase 2: Simulation (High Priority)
3. **T-127**: Opportunity simulation engine
   - Enables what-if analysis
   - Core user value proposition

4. **T-128**: Multi-instrument relationship modeling
   - Enables complex optimization scenarios
   - Builds on cash flow modeling

### Phase 3: Visualization (Medium Priority)
5. **T-129**: Cash flow visualization
   - Enhances user experience
   - Makes cash flow modeling more accessible

---

## Integration Points

### With Existing Systems

1. **Ledger System**:
   - Source of truth for positions
   - Transaction history for cash flow modeling
   - Multi-currency support

2. **Position Tracking**:
   - Current positions from brokers
   - Real-time updates
   - P&L calculations

3. **Risk Calculator**:
   - Risk metrics for scenarios
   - Position sizing
   - Portfolio risk

4. **Synthetic Financing Architecture**:
   - Asset relationship definitions
   - Collateral valuation
   - Financing optimization

---

## User Workflow

### Typical User Flow

1. **View Positions** (T-125):
   - Open unified positions panel
   - See all loans, box spreads, bonds, etc.
   - Review current rates, maturities, cash flows

2. **Model Cash Flow** (T-126):
   - View projected cash flows
   - Identify cash flow gaps
   - Plan for upcoming payments

3. **Simulate Opportunities** (T-127):
   - Select a loan or position
   - Run what-if scenarios:
     - "What if I use this loan to consolidate others?"
     - "What if I use this as margin for box spreads?"
     - "What if I invest in a fund?"
   - Compare scenarios side-by-side

4. **Optimize Relationships** (T-128):
   - System suggests optimal chains
   - User reviews and selects best option
   - Execute optimized strategy

5. **Visualize Results** (T-129):
   - View cash flow charts
   - See relationship diagrams
   - Monitor ongoing positions

---

## Success Criteria

### Unified Positions Panel
- ✅ All instrument types visible in one panel
- ✅ Real-time updates
- ✅ Key metrics displayed (rate, maturity, cash flow)
- ✅ Works in both TUI and PWA

### Cash Flow Modeling
- ✅ Accurate cash flow projections
- ✅ Multi-currency support
- ✅ Handles all instrument types
- ✅ Real-time updates as positions change

### Opportunity Simulation
- ✅ Interactive what-if analysis
- ✅ Multiple scenarios supported
- ✅ Accurate calculations
- ✅ Clear visualization of results

### Multi-Instrument Relationships
- ✅ Relationship chains identified
- ✅ Optimal chains suggested
- ✅ Net benefit calculated
- ✅ Visual relationship diagrams

---

## References

- `docs/SYNTHETIC_FINANCING_ARCHITECTURE.md` - Multi-instrument architecture
- `docs/RISK_FREE_RATE_METHODOLOGY.md` - Rate extraction methodology
- `docs/TODO2_SYNTHETIC_FINANCING_ALIGNMENT_ANALYSIS.md` - Alignment analysis
- `agents/backend/crates/ledger/` - Ledger system implementation
