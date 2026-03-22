# Investment Strategy Framework

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document defines a comprehensive investment strategy framework that integrates convexity optimization, volatility skew considerations, cash
management, ETF allocation, T-bill/bond targets, and spare cash allocation through box spreads.

## Strategy Components

### 1. Portfolio Allocation Framework

The strategy divides capital into distinct allocation buckets. **Important:** Portfolio allocation should consider net portfolio value after
accounting for existing liabilities (loans).

**Net Portfolio Value Calculation:**

```text
Net Portfolio Value = IBKR Assets + Israeli Broker Assets + Discount Bank Balance (ILS→USD) - Existing Loan Liabilities
```

Where:

- **Discount Bank Balance:** Current balance from latest reconciliation file header (converted ILS → USD)
- **Account:** 535-0000-276689
- **Ledger Account:** `Assets:Bank:Discount:276689`

**Position Sources:**

- **IBKR Positions:** Retrieved via IBKR API (TWS or Client Portal API)
- **Israeli Broker Positions:** Imported via Excel (static, RTD, DDE) or web scraping (see `docs/ISRAELI_BROKER_POSITION_IMPORT.md`)
- **Discount Bank Account (535-0000-276689):** Imported via reconciliation files (see `docs/DISCOUNT_BANK_RECONCILIATION_FORMAT.md`)
  - Balance: Current ILS balance from latest header record
  - Interest: 3.00% credit (positive balance), 10.30% debit (negative balance)
  - Ledger Account: `Assets:Bank:Discount:276689`

- **Currency Conversion:** Israeli positions (ILS) converted to USD for unified portfolio view

**Existing Loan Liabilities (User-Specific):**

- **Variable Rate Loans (SHIR-based):** Loans with interest rate = SHIR (Shekel Interbank Rate) + spread/addition
  - Currency: Israeli Shekel (ILS)
  - Interest Rate Risk: Variable, tied to SHIR fluctuations
  - Impact: Monthly payments increase when SHIR rises

- **Fixed Rate CPI-Linked Loans (Israel):**
  - Currency: Israeli Shekel (ILS)
  - Interest Rate: Fixed, but principal/value adjusts with CPI
  - Inflation Hedge: Principal value increases with inflation
  - Impact: Real value of loan decreases in inflationary periods

**Allocation Framework (Based on Net Portfolio Value):**

```text
Net Portfolio Value (After Loan Liabilities)
├── IBKR Positions (from IBKR API)
│   └── Included in total portfolio value
├── Israeli Broker Positions (from Excel/Web scraping)
│   └── Imported and converted to USD (see ISRAELI_BROKER_POSITION_IMPORT.md)
├── Discount Bank Account (535-0000-276689)
│   └── ILS balance converted to USD, earns 3% credit / 10.30% debit
├── Core Investments (70-80% of net portfolio value)
│   ├── Equity ETFs (40-50%)
│   └── Bond ETFs (30-40%)
│       ├── Short-Term Bonds (15-20%) - for convexity barbell
│       └── Long-Term Bonds (15-20%) - for convexity barbell
├── Cash Management (10-15% of net portfolio value)
│   ├── Immediate Cash (3-5%) - liquidity buffer + loan payment reserve
│   └── Spare Cash (7-10%) - short box spreads
├── T-Bill/Bond Ladder (10-15% of net portfolio value)
│   └── Target Rate Allocation
└── Loan Payment Reserve (Separate - factored into cash management)
    └── Monthly loan payments for SHIR-based and CPI-linked loans
```

### 2. Convexity Optimization Strategy

**Objective:** Enhance portfolio convexity to benefit from interest rate movements while managing risk.

**Implementation:**

- **Barbell Strategy:** Allocate bond holdings between short-term (2-3 years) and long-term (20+ years) maturities
- **Avoid Intermediate Bonds:** Minimize or eliminate intermediate-term bonds (5-10 years) that provide less convexity
- **ETF Selection:**
  - Short-term: SHY (iShares 1-3 Year Treasury Bond ETF) or BIL (SPDR Bloomberg 1-3 Month T-Bill ETF)
  - Long-term: TLT (iShares 20+ Year Treasury Bond ETF) or EDV (Vanguard Extended Duration Treasury ETF)

**Convexity Calculation:**

```cpp
// Portfolio convexity is weighted average of bond holdings
PortfolioConvexity = Σ(Weight_i × Convexity_i)
```

**Rebalancing Rules:**

- Rebalance when portfolio convexity deviates >10% from target
- Adjust barbell weights based on interest rate environment
- Long-term bonds increase in rising rate volatility environments

### 3. Volatility Skew Management

**Objective:** Incorporate positive-skew assets to enhance risk-return profile.

**Implementation:**

- **Equity ETFs:** Allocate to broad-market ETFs (SPY, QQQ) with natural positive skew
- **Options Overlay:** Consider protective puts or covered calls for downside protection
- **Skew Monitoring:** Track implied volatility skew (25-delta put/call ratio) for market sentiment

**Skew Metrics:**

- Target positive skewness in portfolio return distribution
- Monitor VIX/VXV ratio (fear gauge)
- Adjust allocation when skew becomes excessively negative

### 4. Cash Management Strategy

#### Tier 1: Immediate Cash (3-5% of portfolio)

- **Purpose:** Liquidity buffer for unexpected needs, margin requirements, loan payments, or opportunities
- **Components:**
  1. **Emergency Reserve:** 2-3% for unexpected needs
  2. **Loan Payment Reserve:** Reserve for monthly loan payments (SHIR-based + CPI-linked loans)
     - Calculate: (Monthly SHIR loan payment × 2) + (Monthly CPI-linked loan payment × 2)
     - Maintain 2 months of payments as buffer for rate changes
  3. **Margin Buffer:** Reserve sufficient cash to cover margin requirements for existing positions and potential new trades

- **Vehicles:**
  - **IBKR Cash Balance:** Earns interest on positive settled cash balances ([IBKR Interest Rates](https://www.interactivebrokers.com/en/accounts/fees/pricing-interest-rates.php))
  - **Discount Bank Account (535-0000-276689):** Israeli Shekel (ILS) account
    - **Credit Balance (Positive):** Earns 3.00% per year on positive balances
    - **Debit Balance (Negative/Margin):** Charges 10.30% per year on negative balances
    - **Purpose:** Direct loan payment access, ILS liquidity, earns competitive interest on credit balances
    - **Integration:** Imported via Discount Bank reconciliation files (see `docs/DISCOUNT_BANK_RECONCILIATION_FORMAT.md`)
    - **Current Balance:** Tracked via ledger system (Account: `Assets:Bank:Discount:276689`)
  - High-yield savings account
  - Money market fund (MMF)
  - Ultra-short T-bills (0-1 month)
  - **Israeli Shekel (ILS) accounts:** For direct loan payment access (if loans are in ILS)

- **IBKR Interest:** IBKR pays interest on positive cash balances; rates vary and should be compared against alternatives
- **Discount Bank Interest:** 3.00% credit rate competitive with T-bills; 10.30% debit rate is high - avoid negative balances
- **Loan Payment Timing:** Factor in loan payment dates when planning cash allocation
- **Cash Flow Awareness:** Use cash flow forecasts to ensure sufficient liquidity for upcoming payments (see `docs/CASH_FLOW_FORECASTING_SYSTEM.md`)
- **Rebalancing:** Monthly review, maintain minimum threshold

**Cash Flow Forecasting:**

- **Future Cash Flows Calculated:**
  - **Loan Payments (Outflows):** Monthly SHIR-based + CPI-linked loan payments
  - **Option Expirations (Inflows/Outflows):** Cash returned from long options, cash required for short options (including box spreads)
  - **Bond Coupons (Inflows):** Periodic coupon payments from bonds/bond ETFs
  - **Bond Maturities (Inflows):** Principal returned at bond maturity
  - **Dividends (Inflows):** Scheduled dividend payments from stocks/ETFs

- **Cash Flow Timeline:** Generate timeline of all future cash flows (next 12 months)
- **Liquidity Planning:** Ensure sufficient cash reserves to cover upcoming outflows
- **Allocation Optimization:** Adjust spare cash allocation based on upcoming cash flows (favor liquidity if large outflows,
  favor yield if large inflows)
- See `docs/CASH_FLOW_FORECASTING_SYSTEM.md` for detailed cash flow calculation methodology

#### Tier 2: Spare Cash (7-10% of portfolio)

- **Purpose:** Enhanced yield through short box spreads targeting T-bill-equivalent returns
- **Implementation:**
  - Use existing box spread strategy (`native/src/box_spread_strategy.cpp`)
  - Target rate: Current T-bill rate + 0-50 bps spread
  - Duration: 1-3 month box spreads (aligns with T-bill ladder)
  - Allocation algorithm: See section 7

### 5. ETF Allocation Framework

**Equity ETFs (40-50% of portfolio):**

- **Core Holdings (70% of equity allocation):**
  - SPY or VOO (S&P 500) - 40%
  - QQQ (Nasdaq 100) - 30%

- **International Diversification (20% of equity allocation):**
  - VEA (Developed Markets) - 10%
  - VWO (Emerging Markets) - 10%

- **Sector/Style Tilt (10% of equity allocation):**
  - Based on market regime and skew preferences

**Bond ETFs (30-40% of portfolio):**

- **Short-Term (50% of bond allocation):** SHY, BIL, or VGSH
  - Duration: 1-3 years
  - Purpose: Liquidity, convexity barbell component

- **Long-Term (50% of bond allocation):** TLT, EDV, or VGLT
  - Duration: 20+ years
  - Purpose: Yield, convexity barbell component

**Rebalancing:**

- Quarterly rebalancing for strategic allocation
- Threshold-based rebalancing when deviations >5%
- Tax-efficient rebalancing (use new contributions first)

### 6. T-Bill/Bond Target Rate Strategy

**Objective:** Maintain T-bill/bond ladder to target specific short-term rates for cash-like returns.

**Ladder Structure:**

- **Maturity Schedule:** Staggered maturities every 1-3 months
- **Total Allocation:** 10-15% of portfolio
- **Individual Positions:** 2-3% per maturity bucket

**Target Rate Calculation:**

```cpp
// Target rate for T-bill/bond ladder
TargetRate = max(
    CurrentTBillRate,           // Market rate
    UserDefinedMinimumRate,     // User threshold
    BoxSpreadEquivalentRate - 0.20%  // Box spread rate minus spread
)
```

**Selection Criteria:**

1. **T-Bills:** When rates exceed box spread equivalent by >20 bps
2. **Short-Term Bonds:** When rates exceed box spread by >50 bps and duration risk acceptable
3. **Box Spreads:** When rates competitive (within 20 bps) and liquidity available

**Rolling Strategy:**

- As T-bills mature, compare rates across:
  - New T-bills
  - Short-term bonds
  - Box spread opportunities

- Allocate to highest yield vehicle after accounting for:
  - Liquidity needs
  - Transaction costs
  - Tax efficiency

### 7. Spare Cash Allocation Algorithm

**Objective:** Dynamically allocate spare cash between box spreads, T-bills, and short-term bonds to maximize yield while maintaining liquidity.

**Decision Framework:**

```cpp
struct SpareCashAllocation {
    double box_spread_percent;     // Allocation to short box spreads
    double tbill_percent;          // Allocation to T-bills
    double short_bond_percent;     // Allocation to short-term bonds
    double ibkr_cash_percent;      // Allocation to IBKR cash (earns interest)
    double discount_bank_ils_percent; // Allocation to Discount Bank ILS account (3% credit, 10.30% debit)
    double immediate_cash_percent; // Emergency liquidity buffer
};

SpareCashAllocation calculate_allocation(
    double total_spare_cash,
    double current_tbill_rate,
    double best_box_spread_rate,
    double short_bond_rate,
    double ibkr_cash_rate,         // IBKR interest rate on cash balances
    double discount_bank_credit_rate, // Discount Bank credit rate (3.00% = 0.03)
    double discount_bank_debit_rate, // Discount Bank debit rate (10.30% = 0.103)
    double discount_bank_balance,  // Current Discount Bank balance (ILS, converted to USD)
    double margin_interest_rate,   // IBKR margin interest rate (if borrowing)
    double margin_requirement,     // Current margin requirement
    double excess_liquidity,       // Current excess liquidity
    int days_to_next_opportunity,
    double liquidity_needs
) {
    // Step 1: Reserve liquidity buffer (10% minimum) and margin buffer
    double liquidity_buffer = total_spare_cash * 0.10;

    // Step 2: Reserve margin buffer to maintain excess liquidity > 25% of net liquidation
    // Ensure we have sufficient cash to cover margin requirements
    double margin_buffer = std::max(0.0, margin_requirement * 0.15); // 15% buffer above margin req

    // Step 3: Calculate available for investment
    double investable = total_spare_cash - liquidity_buffer - margin_buffer;

    // Step 4: Adjust box spread rate for margin costs if margin is required
    double effective_box_spread_rate = best_box_spread_rate;
    if (margin_requirement > 0 && margin_interest_rate > 0) {
        // Net return = box spread return - margin cost
        // Only use margin if box spread return exceeds margin rate + spread
        if (best_box_spread_rate > margin_interest_rate + 0.0050) {  // 50 bps spread
            effective_box_spread_rate = best_box_spread_rate - (margin_interest_rate * 0.5); // Assume 50% margin utilization
        } else {
            // Box spread not profitable after margin costs - reduce allocation
            effective_box_spread_rate = 0.0;
        }
    }

    // Step 5: Rate comparison and allocation (continued)
    // Include IBKR cash rate and Discount Bank credit rate in comparison
    // Use effective box spread rate (adjusted for margin costs)
    // Only consider Discount Bank if balance is positive (credit rate applies)
    double discount_bank_rate = (discount_bank_balance > 0) ? discount_bank_credit_rate : 0.0;
    double max_rate = max(current_tbill_rate, effective_box_spread_rate, short_bond_rate, ibkr_cash_rate, discount_bank_rate);
    double rate_spread = max_rate - min(current_tbill_rate, effective_box_spread_rate, short_bond_rate, ibkr_cash_rate, discount_bank_rate);

    // Step 6: Check margin constraints
    // Ensure excess liquidity remains > 25% of net liquidation
    bool margin_ok = (excess_liquidity > total_spare_cash * 2.5); // Conservative check

    // Step 7: Allocate based on rate advantage and constraints
    // Compare all rates: IBKR cash, T-bills, box spreads (adjusted), short bonds

    if (effective_box_spread_rate > 0 &&
        effective_box_spread_rate >= current_tbill_rate - 0.0020 &&
        effective_box_spread_rate >= ibkr_cash_rate - 0.0010 &&
        margin_ok) {  // Box spread competitive and margin OK
        // Favor box spreads if rate competitive
        if (liquidity_needs < investable * 0.30) {  // Low liquidity needs
            return {
                .box_spread_percent = 0.60,
                .tbill_percent = 0.15,
                .short_bond_percent = 0.10,
                .ibkr_cash_percent = 0.00,  // Minimal if box spreads competitive
                .discount_bank_ils_percent = 0.05,  // Small allocation if rate competitive
                .immediate_cash_percent = 0.10
            };
        } else {
            // Higher liquidity needs - favor T-bills, IBKR cash, and Discount Bank
            return {
                .box_spread_percent = 0.30,
                .tbill_percent = 0.25,
                .short_bond_percent = 0.10,
                .ibkr_cash_percent = 0.15,  // Higher liquidity, earns IBKR interest
                .discount_bank_ils_percent = 0.10,  // ILS liquidity for loan payments
                .immediate_cash_percent = 0.10
            };
        }
    } else if (ibkr_cash_rate >= current_tbill_rate - 0.0015 &&
               ibkr_cash_rate >= best_box_spread_rate - 0.0010) {
        // IBKR cash rate competitive (within 15 bps of T-bill, 10 bps of box spread)
        // Favor IBKR cash for simplicity and liquidity
        return {
            .box_spread_percent = 0.20,
            .tbill_percent = 0.20,
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.35,  // High allocation if IBKR rate competitive
            .discount_bank_ils_percent = 0.05,  // Small allocation for ILS liquidity
            .immediate_cash_percent = 0.10
        };
    } else if (current_tbill_rate > best_box_spread_rate + 0.0020 &&
               current_tbill_rate > ibkr_cash_rate + 0.0015) {
        // T-bills significantly better - favor T-bills
        return {
            .box_spread_percent = 0.15,
            .tbill_percent = 0.50,
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.10,
            .discount_bank_ils_percent = 0.05,  // Small allocation for ILS liquidity
            .immediate_cash_percent = 0.10
        };
    } else if (discount_bank_rate >= current_tbill_rate - 0.0020 &&
               discount_bank_rate >= ibkr_cash_rate - 0.0015 &&
               discount_bank_balance > 0) {
        // Discount Bank credit rate competitive (3% = 0.03)
        // Good for ILS liquidity and loan payments
        return {
            .box_spread_percent = 0.25,
            .tbill_percent = 0.25,
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.15,
            .discount_bank_ils_percent = 0.15,  // Higher allocation if rate competitive
            .immediate_cash_percent = 0.10
        };
    } else {
        // Balanced allocation across all vehicles
        return {
            .box_spread_percent = 0.30,
            .tbill_percent = 0.25,
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.20,  // Moderate IBKR cash for liquidity
            .discount_bank_ils_percent = 0.05,  // Small allocation for ILS liquidity
            .immediate_cash_percent = 0.10
        };
    }
}
```

**Constraints:**

- Minimum 10% immediate cash buffer
- Maximum 70% in box spreads (liquidity risk)
- Maximum 60% in T-bills (diversification)
- Maximum 50% in IBKR cash (opportunity cost if rates improve)
- Maximum 20% in Discount Bank ILS (ILS liquidity, avoid negative balances due to 10.30% debit rate)
- Box spreads require minimum 30 DTE and maximum 90 DTE
- IBKR cash earns interest on positive settled balances; compare rate to alternatives
- **Discount Bank:** 3.00% credit rate (competitive), 10.30% debit rate (avoid negative balances)
- **Discount Bank Balance:** Monitor via reconciliation file imports; maintain positive balance to earn 3% interest

**Rebalancing Triggers:**

1. Rate changes >50 bps (T-bill, box spread, IBKR cash, Discount Bank, or margin rate)
2. Box spread opportunities discovered
3. T-bill maturities
4. IBKR interest rate updates (check monthly)
5. Margin requirement changes (excess liquidity drops below 30% of net liquidation)
6. Margin interest rate changes affecting box spread net returns
7. **SHIR rate changes >25 bps** (affects variable loan payments; adjust cash reserves)
8. **Israeli CPI changes** (affects CPI-linked loan principal; adjust allocation if significant)
9. **ILS/USD exchange rate changes >5%** (affects loan payment costs if portfolio is USD-denominated)
10. **Discount Bank balance changes** (import reconciliation file; adjust allocation if balance significantly changes)
11. **Discount Bank balance goes negative** (immediate action: transfer funds to avoid 10.30% debit rate)
12. **Cash Flow Triggers:**
    - Upcoming large cash outflow (loan payment, option expiration) exceeds available cash in next 7 days
    - Upcoming large cash inflow (bond maturity, option expiration) changes allocation targets
    - Cumulative cash balance projected to go negative in next 30 days (insufficient liquidity)
    - Cash flow timing mismatch (inflows don't cover outflows in time window)
    - Option expiration approaching within 7 days (decision alert: close vs. let expire)
13. Monthly review cycle

**Future Cash Flow Considerations:**

- **Loan Returns (Payments):** Monthly SHIR-based + CPI-linked loan payments (outflows)
- **Option Expiration:** Cash returned from long options, cash required for short options (inflows/outflows)
  - Box spreads: Guaranteed cash flow = Strike width × Contract multiplier × Quantity
  - Regular options: Projected intrinsic value based on current underlying price

- **Bond Coupons:** Periodic coupon payments from bonds/bond ETFs (inflows)
- **Bond Maturities:** Principal returned at bond maturity (large inflows)
- **Dividends:** Scheduled dividend payments from stocks/ETFs (inflows)
- See `docs/CASH_FLOW_FORECASTING_SYSTEM.md` for comprehensive cash flow calculation methodology

**IBKR-Specific Considerations:**

- IBKR pays interest on positive settled cash balances ([IBKR Interest Rates](https://www.interactivebrokers.com/en/accounts/fees/pricing-interest-rates.php))
- Rates may vary and should be monitored monthly
- For accounts with 10M+ USD and ECP qualification, FX Swap Program available with 2-10 bps spreads
- Accrued interest tracked via Account Summary API (`AccruedCash` tag)
- Compare IBKR cash rate against T-bills, box spreads, and Discount Bank when making allocation decisions

**Discount Bank Account (535-0000-276689) Considerations:**

- **Credit Balance (Positive):** Earns 3.00% per year - competitive with T-bills
- **Debit Balance (Negative):** Charges 10.30% per year - avoid negative balances
- **Integration:** Import balance via Discount Bank reconciliation files (see `docs/DISCOUNT_BANK_RECONCILIATION_FORMAT.md`)
- **Ledger Account:** `Assets:Bank:Discount:276689` (ILS currency)
- **Use Cases:**
  - Direct loan payment access (SHIR-based and CPI-linked loans in ILS)
  - ILS liquidity buffer
  - Competitive yield on positive balances (3% ≈ T-bill rates)

- **Risk Management:**
  - Monitor balance via reconciliation file imports
  - Maintain positive balance to earn interest (avoid 10.30% debit rate)
  - Factor into cash allocation decisions (ILS liquidity needs)
  - Convert to USD for portfolio allocation calculations (use current ILS/USD exchange rate)

**Currency Carry Trade Considerations (Advanced - Multi-Currency Accounts):**

- **FX Swap Program:** For qualifying accounts (10M+ USD, ECP status),
  IBKR's FX Swap Program is essentially a currency carry trade mechanism that can provide 2-10 bps spreads per currency ([Investopedia:
  Currency Carry Trade](https://www.investopedia.com/terms/c/currencycarrytrade.asp))
- **Strategy:** Borrow in low-interest-rate currencies (e.g., JPY) to invest in higher-yielding currencies (e.g., USD, AUD)
- **Benefits:** Can enhance yield on multi-currency cash positions beyond local currency rates
- **Significant Risks:**
  - **Exchange Rate Risk:** Currency value changes can wipe out interest rate gains (as demonstrated in 2024 JPY carry trade unwinding)
  - **Interest Rate Shifts:** Central bank decisions can trigger rapid unwinding (e.g., Bank of Japan rate hikes in 2024)
  - **Market Sentiment:** "Safe haven" currency flows can reverse positions quickly
  - **Leverage Risk:** Amplifies losses when markets turn (margin calls on currency positions)

- **Recommendation:** Only for sophisticated investors with:
  - Large multi-currency accounts (10M+ USD)
  - ECP qualification
  - Strong understanding of currency and interest rate risks
  - Ability to monitor and adjust positions quickly

- **Integration:** Consider alongside box spreads, T-bills, and IBKR cash as an advanced cash allocation strategy,
  but limit exposure due to currency risk

**Margin Considerations:**

- **Margin Rates:** IBKR charges margin interest on borrowed funds ([IBKR Margin Rates](https://www.interactivebrokers.com/en/trading/margin-rates.php))
- **Margin Requirements:** Box spreads use margin/collateral;
  monitor margin requirements via Account Summary API (`InitMarginReq`, `MaintMarginReq`, `AvailableFunds`, `ExcessLiquidity`)
- **Net Return Calculation:** When box spreads require margin, net return = Box Spread Return - Margin Interest Cost
- **Cash Buffer:** Reserve sufficient cash buffer above margin requirements to avoid margin calls
- **Margin Efficiency:** Box spreads that don't require additional margin (fully collateralized) are preferred
- **Margin Monitoring:** Track margin usage and ensure excess liquidity > 25% of net liquidation value (maintain `Cushion` > 25%)

**Broker-Specific Margin Notes:**

- **IBKR-Specific:** This framework is designed for Interactive Brokers; margin requirements, rates, and calculations are IBKR-specific
- **Other Brokers:** Margin requirements vary significantly by broker (e.g.,
  [Alpaca Margin Account](https://alpaca.markets/support/determine-margin-account));
  concepts like maintenance margin and pattern day trading (PDT) rules are universal, but specific requirements differ
- **Pattern Day Trading (PDT):** Accounts with <$25k equity are subject to PDT restrictions;
  this affects strategy execution frequency (IBKR and most brokers enforce PDT rules)
- **Maintenance Margin:** Minimum equity required to maintain positions;
  varies by broker and asset class (options typically require higher maintenance margin than stocks)
- **Multi-Broker Strategies:** If expanding to multiple brokers, account for broker-specific margin requirements, rates,
  and API capabilities in allocation decisions

**Currency Risk (Multi-Currency Accounts):**

- **FX Exposure:** Multi-currency accounts face exchange rate risk on non-base currency positions
- **Currency Carry Trades:** FX Swap Program and currency carry strategies can enhance yields but introduce significant currency risk
- **Risk Management:** Limit currency carry exposure, hedge when appropriate, monitor currency correlations
- **2024 Lessons:** Japanese yen carry trade unwinding demonstrated how quickly these strategies can reverse (see [Investopedia:
  Currency Carry Trade](https://www.investopedia.com/terms/c/currencycarrytrade.asp))

**Existing Loan Liabilities (User-Specific):**

- **Variable SHIR-Based Loans (Israel):**
  - **Structure:** Interest rate = SHIR (Shekel Interbank Rate) + spread/addition
  - **Currency:** Israeli Shekel (ILS)
  - **Interest Rate Risk:** Variable rate tied to SHIR fluctuations; payments increase when SHIR rises
  - **Cash Flow Impact:** Monthly payments fluctuate with SHIR changes; reserve 2 months of payments as buffer
  - **Monitoring:** Track SHIR rate changes and adjust cash reserves accordingly
  - **Hedging Considerations:** Consider ILS/USD hedging if portfolio is USD-denominated; rate increases can reduce available capital

- **Fixed Rate CPI-Linked Loans (Israel):**
  - **Structure:** Fixed interest rate, but principal value adjusts with CPI (Consumer Price Index)
  - **Currency:** Israeli Shekel (ILS)
  - **Inflation Hedge:** Principal increases with inflation, reducing real value of debt in inflationary periods
  - **Cash Flow Impact:** Monthly payments may adjust with CPI; principal value increases over time
  - **Real Value Impact:** Loan becomes "cheaper" in real terms during inflation; consider this when allocating assets
  - **Monitoring:** Track Israeli CPI changes and principal adjustments

- **Loan Payment Reserve Strategy:**
  - **Calculation:** Total monthly loan payments × 2 months (buffer for rate changes)
  - **Allocation:** Factor into Tier 1 Immediate Cash allocation
  - **Currency Matching:** Consider maintaining ILS reserves for direct loan payments (if loans are in ILS)
  - **Interest Optimization:** Balance between maintaining ILS reserves vs. earning higher USD rates (account for FX risk)

- **Impact on Portfolio Allocation:**
  - **Net Portfolio Value:** Calculate allocations based on net portfolio value (assets minus loan liabilities)
  - **Effective Cash Flow:** Consider monthly loan payments as a liability reducing available cash flow
  - **Interest Rate Sensitivity:** Variable SHIR loans create interest rate exposure; monitor correlation with portfolio holdings
  - **Inflation Sensitivity:** CPI-linked loans provide natural inflation hedge; align with inflation-sensitive assets

### 8. Risk Management Rules

**Portfolio-Level:**

- Maximum total exposure: 95% of account value (5% cash buffer)
- Maximum single ETF position: 20% of portfolio
- Maximum box spread exposure: 10% of portfolio
- Maximum T-bill/bond ladder: 15% of portfolio
- **Margin Cushion:** Maintain excess liquidity > 25% of net liquidation value (`Cushion` > 25%)
- **Available Funds:** Ensure `AvailableFunds` > 5% of account value for margin buffer
- **Margin Utilization:** Monitor `ExcessLiquidity` to avoid margin calls
- **Loan Payment Reserve:** Maintain 2 months of loan payments (SHIR + CPI-linked) in cash buffer
- **Net Portfolio Value:** All allocations calculated on net portfolio value (assets minus loan liabilities)

**Portfolio Greeks (Risk Sensitivities):**

- **Delta (Price Sensitivity):** Maximum ±50% of portfolio value (target delta-neutral for hedged strategies)
- **Gamma (Delta Acceleration):** Monitor for large positive/negative gamma (high sensitivity to large price movements)
- **Vega (Volatility Sensitivity):** Maximum $10,000 per 1% volatility change (monitor in high-volatility environments)
- **Theta (Time Decay):** Maximum -$500 per day time decay cost (monitor for option-heavy portfolios)
- **Rho (Interest Rate Sensitivity):** Maximum ±$50,000 per 1% rate change (monitor correlation with SHIR-based loans)
- **Greeks Aggregation:** Sum Greeks across all positions (IBKR + Israeli brokers) with currency conversion
- See `docs/PORTFOLIO_GREEKS_SYSTEM.md` for detailed Greeks calculation methodology

**Box Spread Specific:**

- Minimum ROI: 0.5% (existing strategy parameter)
- Maximum position size: $10,000 per box spread (existing parameter)
- Maximum total box spread exposure: 10% of portfolio
- Liquidity requirements: Minimum 100 volume, 500 OI per leg
- Duration limits: 30-90 DTE
- **Margin Requirements:** Check margin requirements before opening box spreads
- **Net Return:** Calculate net return after margin interest costs (if margin used)
- **Fully Collateralized Preferred:** Prefer box spreads that don't require additional margin
- **Margin Cost:** If margin is required, ensure box spread return > margin interest rate + 50 bps

**Rebalancing Limits:**

- No rebalancing if transaction costs >25 bps of position value
- Tax-efficient rebalancing: Use new contributions and dividends first
- Avoid realizing short-term capital gains unless necessary

### 9. Integration with Existing Codebase

**Configuration Extensions:**

```cpp
// Add to native/include/config_manager.h
struct PortfolioAllocationConfig {
    // Core allocations
    double equity_etf_percent = 0.45;      // 45% equity ETFs
    double bond_etf_percent = 0.35;        // 35% bond ETFs
    double immediate_cash_percent = 0.04;  // 4% immediate cash
    double spare_cash_percent = 0.08;      // 8% spare cash
    double tbill_ladder_percent = 0.08;    // 8% T-bill ladder

    // Convexity targets
    double short_term_bond_percent = 0.175;  // 50% of bond allocation
    double long_term_bond_percent = 0.175;   // 50% of bond allocation
    double target_convexity = 150.0;         // Target portfolio convexity

    // Spare cash allocation
    double box_spread_target_rate_spread = 0.0020;  // 20 bps vs T-bills
    double ibkr_cash_rate_spread = 0.0015;          // 15 bps spread threshold for IBKR cash
    double discount_bank_credit_rate = 0.03;        // 3.00% Discount Bank credit rate
    double discount_bank_debit_rate = 0.103;        // 10.30% Discount Bank debit rate (avoid)
    double min_box_spread_rate = 0.00;              // Minimum rate
    double max_box_spread_allocation = 0.70;        // Max 70% of spare cash
    double max_ibkr_cash_allocation = 0.50;          // Max 50% in IBKR cash
    double max_discount_bank_ils_allocation = 0.20; // Max 20% in Discount Bank ILS

    // T-bill ladder
    double tbill_target_rate = 0.05;        // 5% target rate
    int tbill_ladder_maturities = 4;        // 4 maturity buckets
    int tbill_days_between_maturities = 30; // Staggered monthly
};

struct Config {
    // ... existing config ...
    PortfolioAllocationConfig portfolio_allocation;
};
```

**New Components Needed:**

1. `PortfolioAllocationManager` class
   - Track current allocations
   - Calculate target allocations
   - Trigger rebalancing decisions
   - Integrate with box spread strategy
   - **Aggregate positions from multiple sources:** IBKR + Israeli brokers (see `docs/ISRAELI_BROKER_POSITION_IMPORT.md`)
   - **Currency conversion:** ILS to USD for unified portfolio view
   - **Net portfolio value calculation:** Includes all positions minus loan liabilities
   - **Portfolio Greeks calculation:** Aggregate Greeks across all positions (see `docs/PORTFOLIO_GREEKS_SYSTEM.md`)
   - **Cash flow forecasting:** Generate future cash flow timeline (see `docs/CASH_FLOW_FORECASTING_SYSTEM.md`)

2. `ConvexityCalculator` class
   - Calculate portfolio convexity
   - Optimize barbell allocation
   - Monitor convexity drift

3. `SpareCashAllocator` class
   - Execute spare cash allocation algorithm
   - Compare rates across vehicles (IBKR cash, T-bills, box spreads, bonds)
   - Fetch IBKR cash interest rate from account summary
   - Fetch margin requirements and rates from account summary (`InitMarginReq`, `MaintMarginReq`, `AvailableFunds`, `ExcessLiquidity`)
   - **Cash flow awareness:** Adjust allocation based on upcoming cash flows (favor liquidity if large outflows, favor yield if large inflows)
   - See `docs/CASH_FLOW_FORECASTING_SYSTEM.md` for cash flow integration

4. `CashFlowCalculator` class (NEW)
   - Calculate loan payment cash flows (SHIR-based + CPI-linked)
   - Calculate option expiration cash flows (including box spreads)
   - Calculate bond coupon and maturity cash flows
   - Calculate dividend cash flows
   - Generate aggregated cash flow timeline
   - Project cumulative cash balance over time
   - Currency conversion for foreign cash flows (ILS → USD)
   - See `docs/CASH_FLOW_FORECASTING_SYSTEM.md` for detailed implementation

5. `TBillLadderManager` class
   - Track T-bill positions and maturities
   - Calculate target rates
   - Execute rolling strategy

## Implementation Roadmap

### Phase 1: Core Allocation Framework (Weeks 1-2)

- [ ] Extend config_manager.h with PortfolioAllocationConfig
- [ ] Implement PortfolioAllocationManager class
- [ ] Create allocation tracking and reporting
- [ ] Add configuration validation
- [ ] Integrate Israeli broker position import (see `docs/ISRAELI_BROKER_POSITION_IMPORT.md`)
- [ ] Add currency conversion (ILS → USD) for portfolio aggregation

### Phase 2: Spare Cash Allocation (Weeks 3-4)

- [ ] Implement SpareCashAllocator class
- [ ] Integrate with existing box spread strategy
- [ ] Add IBKR cash interest rate fetching from Account Summary API
- [ ] Add margin requirements fetching (`InitMarginReq`, `MaintMarginReq`, `AvailableFunds`, `ExcessLiquidity`)
- [ ] Add margin interest rate tracking (if available via API)
- [ ] Implement margin cost adjustment for box spread net returns
- [ ] Add margin constraint checking (excess liquidity > 25%)
- [ ] Add rate comparison logic (IBKR cash, T-bills, box spreads adjusted for margin, bonds)
- [ ] Implement allocation execution

### Phase 3: Convexity Optimization (Weeks 5-6)

- [ ] Implement ConvexityCalculator class
- [ ] Add barbell allocation logic
- [ ] Create rebalancing triggers
- [ ] Test convexity calculations

### Phase 4: T-Bill Ladder (Weeks 7-8)

- [ ] Implement TBillLadderManager class
- [ ] Add maturity tracking
- [ ] Implement rolling strategy
- [ ] Integrate with IBKR API for T-bill orders

### Phase 5: ETF Integration (Weeks 9-10)

- [ ] Add ETF position tracking
- [ ] Implement rebalancing logic
- [ ] Add transaction cost analysis
- [ ] Create tax-efficient rebalancing

### Phase 6: Testing & Validation (Weeks 11-12)

- [ ] Backtest allocation strategy
- [ ] Validate against historical data
- [ ] Paper trading implementation
- [ ] Performance monitoring

## Performance Metrics

**Target Metrics:**

- Portfolio yield: T-bill rate + 50-100 bps (via box spreads and optimization)
- Convexity: Maintain within ±10% of target
- Skewness: Positive portfolio skewness
- Cash efficiency: <5% idle cash
- Rebalancing frequency: Quarterly strategic, monthly tactical

**Monitoring:**

- Track actual vs. target allocations
- Monitor rate capture (box spreads vs. T-bills)
- Measure convexity drift
- Calculate realized yield

## Risk Considerations

**Interest Rate Risk:**

- Long-term bonds vulnerable to rate increases
- Barbell strategy provides convexity protection
- Regular rebalancing mitigates duration drift

**Liquidity Risk:**

- Box spreads may be less liquid than T-bills
- Maintain minimum cash buffer
- Diversify across maturities

**Execution Risk:**

- Box spread fills may differ from theoretical
- Use existing confidence scores and liquidity filters
- Fallback to T-bills if execution uncertain

**Tax Considerations:**

- Box spreads: 60/40 long-term/short-term capital gains treatment (if held >1 year)
- T-bills: State tax exempt
- ETFs: Distributions may be taxable
- Rebalancing: Minimize short-term capital gains

## References

1. Research Document: `docs/INVESTMENT_STRATEGY_RESEARCH.md`
2. Box Spread Strategy: `native/src/box_spread_strategy.cpp`
3. Risk Calculator: `native/src/risk_calculator.cpp`
4. Config Manager: `native/include/config_manager.h`
5. Israeli Broker Position Import: `docs/ISRAELI_BROKER_POSITION_IMPORT.md`
6. Portfolio Greeks System: `docs/PORTFOLIO_GREEKS_SYSTEM.md`
7. Cash Flow Forecasting System: `docs/CASH_FLOW_FORECASTING_SYSTEM.md`
8. [Wikipedia: Greeks (finance)](https://en.wikipedia.org/wiki/Greeks_(finance))
9. [Net Present Value (NPV)](https://en.wikipedia.org/wiki/Net_present_value)

---

**Next Steps:**

1. Review and approve framework
2. Document user requirements and assumptions (T-61)
3. Design Israeli broker position import system (T-62)
4. Begin Phase 1 implementation
