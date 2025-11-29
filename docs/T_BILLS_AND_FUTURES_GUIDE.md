# Treasury Bills (T-Bills) and T-Bill Futures Guide

<!--
@index: trading-concepts
@category: reference
@tags: t-bills, treasury-bills, risk-free-rate, futures, fixed-income, arbitrage
@last-updated: 2025-01-27
-->

This guide provides comprehensive information about Treasury Bills (T-bills) and T-bill futures, with particular relevance to box spread trading and risk-free rate calculations.

## Overview

Treasury bills (T-bills) are short-term debt securities issued by the U.S. government, typically maturing in one year or less. They are considered one of the safest investments, backed by the full faith and credit of the U.S. government.

## Key Characteristics

### Basic Structure

- **Maturities**: 4, 8, 13, 17, 26, and 52 weeks
- **Minimum Purchase**: $100
- **Pricing**: Sold at a discount to face value (zero-coupon)
- **Return**: The difference between purchase price and face value at maturity
- **Tax Treatment**:
  - Exempt from state and local taxes
  - Subject to federal taxation

### How T-Bills Work

1. **Purchase**: Investors buy T-bills at a discount to face value
2. **Holding Period**: No periodic interest payments
3. **Maturity**: Receive full face value at maturity
4. **Return**: The discount represents the interest earned

**Example**: Purchase a $10,000 T-bill for $9,800. At maturity, receive $10,000. The $200 difference is the interest earned.

## Purchasing T-Bills

### Direct Purchase

- **TreasuryDirect.gov**: Direct purchase from U.S. Treasury
- **Auctions**: T-bills are sold through regular auctions
- **Minimum**: $100 increments

### Brokerage Purchase

- Available through most brokerage accounts (including Interactive Brokers)
- May have different minimums or fees
- Often more convenient for active traders

## T-Bill Futures

### CME Group T-Bill Futures

T-bill futures are exchange-traded contracts that allow traders to:

- Hedge exposure to T-bill rates
- Speculate on interest rate movements
- Execute arbitrage strategies

### Key Factors Driving T-Bill Futures (2025)

According to CME Group analysis, key factors include:

1. **T-Bill Issuance Changes**
   - Reductions due to debt ceiling constraints
   - Impact on supply and demand dynamics
   - Affects T-bill yields

2. **Treasury General Account (TGA) Cash Flows**
   - Cash inflows/outflows affect market liquidity
   - Influences short-term interest rates
   - Creates trading opportunities

3. **Federal Open Market Committee (FOMC) Policy**
   - Interest rate decisions
   - Banking regulations
   - Monetary policy announcements

4. **Secured Overnight Financing Rate (SOFR)**
   - Relationship between T-bill rates and SOFR
   - Inter-commodity spread opportunities
   - Basis trading strategies

### Trading Strategies

#### Inter-Commodity Spreads

- **T-Bill Futures vs. SOFR Futures**: Capitalize on rate differentials
- **Calendar Spreads**: Trade different contract months
- **Basis Trading**: Exploit pricing inefficiencies

#### Arbitrage Opportunities

- **Cash-Futures Arbitrage**: Between physical T-bills and futures
- **Box Spread vs. T-Bill Arbitrage**: Compare risk-free rates
- **Cross-Market Arbitrage**: Between different exchanges or instruments

## Relevance to Box Spread Trading

### Risk-Free Rate Calculations

T-bills are commonly used as the risk-free rate benchmark for:

1. **Option Pricing Models**
   - Black-Scholes and other models use risk-free rate
   - T-bill rates provide short-term risk-free rate
   - Matching maturity to option expiration

2. **Box Spread Valuation**
   - Box spreads should price to risk-free rate
   - Compare box spread implied rate to T-bill rate
   - Identify arbitrage opportunities

3. **Profitability Analysis**
   - Calculate expected returns vs. risk-free rate
   - Assess whether box spread premium justifies risk
   - Determine optimal position sizing

### Box Spread vs. T-Bill Arbitrage Opportunities

#### Rate Comparison

When box spread implied rate differs from T-bill rate:

- **Box Spread Rate > T-Bill Rate**: Potential arbitrage opportunity
- **Box Spread Rate < T-Bill Rate**: May indicate mispricing or risk premium
- **Transaction Costs**: Must account for commissions and fees

#### Implementation Considerations

1. **Maturity Matching**: Match T-bill maturity to option expiration
2. **Liquidity**: Ensure both markets are liquid enough
3. **Execution Risk**: Simultaneous execution in both markets
4. **Capital Requirements**: Margin requirements for both positions

## Alternative Hedging Instruments

Beyond physical T-bills and T-bill futures, several ETF-based alternatives offer different risk-return profiles and implementation approaches for hedging box spread positions.

### BOXX ETF (Alpha Architect 1-3 Month Box ETF)

**Overview**: BOXX uses box spreads to replicate T-bill returns, providing a unique alternative to direct T-bill investment.

**Key Characteristics**:

- **Strategy**: Employs box spread options positions to replicate short-term U.S. Treasury bill returns
- **Tax Treatment**: Converts interest income into capital gains (potential tax advantage)
- **Assets Under Management**: ~$8.8 billion as of November 2025 (significant growth since late 2022 inception)
- **Liquidity**: High daily trading volume (~1.7M shares/day)
- **Expense Ratio**: Check current prospectus for management fees

**Advantages**:

- **Tax Efficiency**: Capital gains treatment vs. ordinary income for T-bill interest
- **Liquidity**: ETF structure provides easy entry/exit
- **No Direct T-Bill Purchase**: Avoids TreasuryDirect or auction process
- **Box Spread Expertise**: Managed by professionals who understand box spread mechanics

**Disadvantages**:

- **Options Market Risk**: Performance tied to options market liquidity and pricing
- **Financial Distress Risk**: Box spreads may be less reliable during market stress
- **Not True Cash Equivalent**: May not maintain value during extreme market conditions
- **Management Fees**: ETF expense ratio reduces net returns
- **Tracking Error**: May not perfectly track T-bill rates

**Use Cases**:

- Tax-advantaged alternative to T-bills for high-income investors
- Hedge for box spread positions when tax efficiency matters
- Short-term cash alternative with potential capital gains treatment
- Portfolio allocation tool for risk-free rate exposure

### Leveraged T-Bill and Bond Index ETFs

**Overview**: ETFs that amplify the daily returns of T-bill or bond indexes using leverage.

**Key Characteristics**:

- **Leverage Ratios**: Typically 2x (SEC Rule 18f-4 generally limits to 2x)
- **Daily Rebalancing**: Leverage resets daily, causing compounding effects
- **Regulatory Status**: SEC uncertainty about 3x and 5x leveraged products
- **Risk Profile**: Amplifies both gains and losses

**Advantages**:

- **Amplified Returns**: 2x leverage can enhance returns in favorable markets
- **Liquidity**: ETF structure provides easy trading
- **Diversification**: Can track broad bond indexes, not just T-bills

**Disadvantages**:

- **Amplified Losses**: Leverage works both ways
- **Compounding Effects**: Daily rebalancing can cause performance decay over time
- **Short-Term Focus**: Best suited for short-term strategies
- **Regulatory Risk**: Higher leverage products may face regulatory challenges
- **Volatility**: Higher volatility than underlying index

**Use Cases**:

- Short-term tactical hedging positions
- Enhanced exposure to interest rate movements
- Speculative positions on rate direction
- **Not Recommended**: Long-term hedging or cash equivalent purposes

### Non-Leveraged T-Bill ETFs

**Common Examples**: SHV (iShares Short Treasury Bond ETF), SGOV (iShares 0-3 Month Treasury Bond ETF), BIL (SPDR Bloomberg 1-3 Month T-Bill ETF)

**Key Characteristics**:

- **Direct T-Bill Exposure**: Hold actual T-bills or very short-term Treasury securities
- **Low Expense Ratios**: Typically 0.10-0.20% annually
- **High Liquidity**: Easy to buy/sell throughout trading day
- **Tax Treatment**: Interest income subject to federal tax (like physical T-bills)

**Advantages**:

- **Convenience**: Easier than direct T-bill purchase
- **Liquidity**: Tradeable throughout market hours
- **Diversification**: Hold multiple T-bill maturities
- **Low Cost**: Minimal expense ratios
- **True T-Bill Exposure**: Direct ownership of Treasury securities

**Disadvantages**:

- **Expense Ratio**: Small drag on returns vs. direct T-bill purchase
- **Tax Treatment**: Interest income, not capital gains
- **Tracking Error**: May not perfectly match specific T-bill maturity
- **Market Price Fluctuations**: ETF price may deviate from NAV

**Use Cases**:

- Convenient T-bill exposure for hedging
- Cash management tool
- Risk-free rate benchmark proxy
- Portfolio allocation to short-term Treasuries

### Comparison Matrix

| Instrument                     | Tax Treatment   | Liquidity           | Risk       | Leverage | Best For                         |
| ------------------------------ | --------------- | ------------------- | ---------- | -------- | -------------------------------- |
| **Physical T-Bills**           | Interest income | Low (auction-based) | Lowest     | None     | Long-term, direct ownership      |
| **T-Bill Futures**             | Capital gains   | High                | Low-Medium | Implicit | Short-term hedging, leverage     |
| **BOXX ETF**                   | Capital gains   | High                | Low-Medium | None     | Tax-efficient T-bill alternative |
| **Leveraged ETFs**             | Capital gains   | High                | High       | 2x       | Short-term tactical positions    |
| **T-Bill ETFs (SHV/SGOV/BIL)** | Interest income | High                | Low        | None     | Convenient T-bill exposure       |

### Hedging Strategy Considerations

#### When to Use Each Instrument

1. **Physical T-Bills**
   - Long-term hedging (matching exact maturity)
   - Maximum safety and direct ownership
   - Tax-advantaged accounts (IRA, 401k)

2. **T-Bill Futures**
   - Short-term hedging with leverage
   - Active trading strategies
   - Basis trading opportunities

3. **BOXX ETF**
   - Tax-efficient hedging for taxable accounts
   - When capital gains treatment is advantageous
   - Short-term positions (1-3 months)

4. **Leveraged ETFs**
   - Short-term tactical positions only
   - Enhanced exposure to rate movements
   - **Avoid for**: Long-term hedging, cash equivalents

5. **T-Bill ETFs (SHV/SGOV/BIL)**
   - Convenient hedging tool
   - Daily liquidity needs
   - Portfolio cash management

#### Risk Considerations

**BOXX-Specific Risks**:

- Options market stress can impact box spread effectiveness
- Not a true cash equivalent during financial distress
- Relies on options market liquidity and pricing efficiency

**Leveraged ETF Risks**:

- Daily rebalancing causes compounding effects
- Performance decay over longer holding periods
- Regulatory uncertainty for higher leverage products
- Not suitable for long-term hedging

**General ETF Risks**:

- Tracking error vs. underlying assets
- Market price vs. NAV deviations
- Expense ratio drag on returns
- Counterparty risk (minimal for T-bill ETFs)

### Integration with Box Spread Trading

#### Hedging Box Spread Positions

1. **Long Box Spread Hedging**
   - Use T-bills or T-bill ETFs to hedge interest rate risk
   - BOXX provides tax-efficient alternative
   - Match maturity to option expiration

2. **Short Box Spread Hedging**
   - Use T-bill futures for leveraged hedge
   - T-bill ETFs for convenience
   - Consider leveraged ETFs only for very short-term

3. **Arbitrage Opportunities**
   - Compare box spread rates to BOXX returns
   - Monitor T-bill ETF yields vs. box spread implied rates
   - Identify cross-market pricing inefficiencies

#### Alternative Hedging Implementation Considerations

1. **Tax Optimization**
   - Use BOXX in taxable accounts for capital gains treatment
   - Use T-bills/T-bill ETFs in tax-advantaged accounts
   - Consider tax implications of hedging strategy

2. **Liquidity Management**
   - ETFs provide daily liquidity
   - Physical T-bills require holding to maturity or secondary market
   - Futures require margin management

3. **Cost Analysis**
   - Compare expense ratios (ETFs) vs. transaction costs (T-bills/futures)
   - Account for bid-ask spreads
   - Consider tax implications of each approach

4. **Risk Matching**
   - Match instrument maturity to box spread expiration
   - Consider credit risk (minimal for all options)
   - Evaluate liquidity risk during market stress

## Market Data Sources

### T-Bill Rates

- **TreasuryDirect.gov**: Official auction results
- **Federal Reserve Economic Data (FRED)**: Historical rates
- **Bloomberg/Reuters**: Real-time rates
- **Interactive Brokers**: Market data subscriptions

### T-Bill Futures Data

- **CME Group**: Exchange data and specifications
- **Brokerage Platforms**: Real-time quotes and historical data
- **Market Data Vendors**: Professional-grade feeds

## Integration with IB Box Spread Application

### Potential Enhancements

1. **Risk-Free Rate Lookup**
   - Integrate T-bill rate data
   - Match maturity to option expiration
   - Use in box spread valuation

2. **Arbitrage Detection**
   - Compare box spread rates to T-bill rates
   - Flag opportunities when spread exceeds threshold
   - Account for transaction costs

3. **T-Bill Futures Integration**
   - Add T-bill futures contracts to analysis
   - Compare box spread rates to futures-implied rates
   - Cross-market arbitrage detection

4. **Portfolio Optimization**
   - Include T-bills as alternative investment
   - Risk-adjusted return comparisons
   - Capital allocation decisions

## Key Resources

### Educational Resources

- **CME Group Article**: [Key Factors That Will Drive T-Bill Futures and How Traders Can Prepare](https://www.cmegroup.com/articles/2025/key-factors-that-will-drive-t-bill-futures-and-how-traders-can-prepare.html)
- **Interactive Brokers**: [T-Bills 101](https://www.interactivebrokers.com/campus/traders-insight/t-bills-101/)
- **Investopedia**: [Treasury Bill (T-Bill) Definition](https://www.investopedia.com/terms/t/treasurybill.asp)

### Official Sources

- **TreasuryDirect.gov**: Direct purchase and auction information
- **Federal Reserve**: Economic data and policy information
- **CME Group**: Futures contract specifications and market data

## Best Practices

### For Box Spread Traders

1. **Monitor T-Bill Rates**: Regularly check rates matching option expirations
2. **Compare Rates**: Always compare box spread implied rates to T-bill rates
3. **Account for Costs**: Include all transaction costs in arbitrage calculations
4. **Consider Liquidity**: Ensure both markets are liquid enough for execution
5. **Risk Management**: Understand that arbitrage opportunities may be limited by:
   - Execution risk
   - Capital requirements
   - Regulatory constraints

### For T-Bill Futures Traders

1. **Stay Informed**: Monitor FOMC announcements and economic data
2. **Watch TGA Flows**: Treasury General Account activity affects rates
3. **Understand Basis**: Learn the relationship between cash and futures
4. **Manage Margin**: T-bill futures require margin management
5. **Use Spreads**: Consider inter-commodity spreads for lower risk

## International Risk-Free Rate Equivalents

While U.S. T-bills are the standard risk-free rate benchmark for U.S. markets, other countries have equivalent instruments that serve similar purposes for their respective markets.

### SHIR (Shekel Overnight Interest Rate) - Israel

**Overview**: SHIR is the Israeli equivalent of a risk-free rate, administered by the Bank of Israel. It serves as the benchmark overnight interest rate for the Israeli Shekel (ILS) market.

**Key Characteristics**:

- **Administrator**: Bank of Israel (or its successor)
- **Publication**: Published daily by 11:00 AM on the Bank of Israel website
- **Rate**: Equal to the Bank of Israel interest rate
- **Non-Publication Days**: Saturdays, Sundays, and additional dates as published
- **Fallback**: On non-publication days, SHIR value equals the last published value
- **Reference**: <https://www.boi.org.il/en/economic-roles/financial-markets/shir/>

**Use Cases**:

- Risk-free rate benchmark for Israeli options pricing
- Reference rate for Israeli interest rate derivatives
- Hedging Israeli market exposure
- Multi-currency trading strategies involving ILS

**Derivative Products**:

- **ILS SHIR-OIS Compound Swap**: Fixed-to-floating interest rate swap based on SHIR
  - Available on Bloomberg SEF (Swap Execution Facility)
  - Cleared through LCH.Clearnet Ltd.
  - Follows 2021 ISDA Interest Rate Derivatives Definitions
  - Tenor: 7 days to 4050 days
  - Trading: 00:01-24:00 Sunday-Friday (Eastern Time)
  - Documentation: <https://assets.bbhub.io/professional/sites/27/ILS-SHIR-OIS-Compound-Fixed-to-Floating-Interest-Rate-Swap-Contract-as-filed.pdf>

**Relevance to Box Spread Trading**:

- Israeli traders can use SHIR as risk-free rate for ILS-denominated options
- Compare box spread implied rates to SHIR for arbitrage opportunities
- Cross-currency arbitrage between USD box spreads and ILS risk-free rates
- Hedging strategies for multi-currency portfolios

### Other International Risk-Free Rates

Similar instruments exist in other markets:

- **EUR**: EONIA (Euro Overnight Index Average) or €STR (Euro Short-Term Rate)
- **GBP**: SONIA (Sterling Overnight Index Average)
- **JPY**: TONAR (Tokyo Overnight Average Rate)
- **CHF**: SARON (Swiss Average Rate Overnight)

Each serves as the risk-free rate benchmark for options pricing and derivative valuation in their respective currencies.

## Conclusion

Understanding T-bills and T-bill futures is essential for:

- Accurate risk-free rate calculations in option pricing
- Identifying arbitrage opportunities between box spreads and T-bills
- Making informed decisions about capital allocation
- Understanding the broader fixed income market context

For box spread traders, T-bill rates provide the benchmark against which box spread profitability should be measured, helping to identify true arbitrage opportunities versus risk-adjusted returns.

**International Considerations**: Traders operating in multiple currencies should use the appropriate risk-free rate for each currency (e.g., SHIR for ILS, T-bills for USD) when pricing options and evaluating arbitrage opportunities.

## See Also

- [Box Spread Comprehensive Guide](strategies/box-spread/BOX_SPREAD_COMPREHENSIVE_GUIDE.md) - Complete box spread mechanics and strategies
- [TWS Integration Status](research/integration/TWS_INTEGRATION_STATUS.md) - Interactive Brokers API integration details
- [API Documentation Index](API_DOCUMENTATION_INDEX.md) - External APIs and resources
