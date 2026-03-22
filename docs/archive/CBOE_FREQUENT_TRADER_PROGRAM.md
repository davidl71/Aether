# CBOE Frequent Trader Program (FTID)

**Date**: 2025-01-27
**Source**: <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf>
**Registration**: <https://www.cboe.com/FTID/registration.aspx>

---

## Overview

The CBOE Frequent Trader Program provides fee rebates for high-volume trading activity in RUT, VIX®, SPX, and SPXW options. Customers obtain a unique Frequent Trader ID (FTID) that can be appended to orders to qualify for transaction fee rebates.

---

## Eligibility

Available to **non-Trading Permit Holder, non-broker/dealer** (i.e., customer) users such as:

- Asset Managers
- Corporate Treasuries
- ETF/ETN Providers
- Family Offices
- Hedge Funds
- Individuals
- Insurance Companies
- Mutual Funds
- Pensions
- Proprietary Trading Firms

---

## How It Works

1. **Registration**: Customers register at <https://www.cboe.com/FTID/registration.aspx> to obtain a unique FTID
2. **Order Tagging**: FTID is appended to orders delivered to CBOE during both Regular Trading Hours (RTH) and Global Trading Hours (GTH)
3. **Volume Aggregation**: Volume associated with each FTID is aggregated to qualify for rebates
4. **Rebate Calculation**: Rebates are calculated based on monthly contract volume and tier thresholds
5. **Payment**: Rebates can be paid to executing agent or directly to customer via EFT

---

## Rebate Schedule

### VIX Options

| Tier | Monthly Contracts Traded | Fee Rebate |
|------|------------------------|-------------|
| 1    | 10,000 - 99,999       | 5%          |
| 2    | 100,000 - 299,999      | 15%         |
| 3    | 300,000 and above      | 25%         |

### SPX / SPXW Options

| Tier | Monthly Contracts Traded | Fee Rebate |
|------|------------------------|-------------|
| 1    | 10,000 - 49,999       | 3%          |
| 2    | 50,000 - 99,999       | 6%          |
| 3    | 100,000 and above     | 9%          |

### RUT Options

| Tier | Monthly Contracts Traded | Fee Rebate |
|------|------------------------|-------------|
| 1    | 10,000 - 24,999       | 10%         |
| 2    | 25,000 - 49,999       | 15%         |
| 3    | 50,000 and above      | 25%         |

---

## Key Features

### Trading Hours Coverage

- **Regular Trading Hours (RTH)**: Standard market hours
- **Global Trading Hours (GTH)**: Extended hours trading
- FTID applies to both RTH and GTH orders

### Volume Aggregation

- Volume is aggregated across all executing agents for a single FTID
- Monthly contract volume determines tier qualification
- Each customer is responsible for ensuring FTID is appended to orders

### Privacy

- FTID is **not visible** to marketplace or trading counterparties
- Recorded only on CBOE's internal order tracking system
- Not disclosed or discoverable by any other party
- No public record of FTID owners
- Personal information handled per Frequent Trader Program Privacy Policy

---

## Registration & Setup

### Step 1: Register for FTID

1. Visit: <https://www.cboe.com/FTID/registration.aspx>
2. Complete registration form
3. Receive unique FTID

### Step 2: Provide FTID to Executing Agent

- Supply FTID to executing agent (broker)
- Agent appends FTID to both manual and electronic orders
- **Important**: Use the same FTID with all executing agents

### Step 3: Monitor Activity

- Receive daily email statements with:
  - Contracts traded
  - Total estimated fees
  - Potential rebate amount

- Request customized reports: <https://www.cboe.org/tphreports/default.aspx>

---

## Rebate Payment

### Payment Options

1. **To Executing Agent**: Rebate paid to broker/agent
2. **Direct to Customer**: Electronic Funds Transfer (EFT) directly to customer

### Payment Instructions

- Submit payment instructions via link on daily statement
- If multiple executing agents used, rebates paid pro rata based on executed volume share

---

## Post-Trade Adjustments

The program allows for post-trade adjustments through:

- Form: <https://www.cboe.com/trading-resources/fee-schedules>
- Submit adjustments for corrections or modifications

---

## FAQs

### 1. What is the process to obtain a Frequent Trader ID?

Register at <https://www.cboe.com/FTID/registration.aspx>

### 2. How does a customer enter their FTID on an order?

Provide FTID to executing agent who appends it to both manual and electronic orders. Use the same FTID with all executing agents.

### 3. Will there be a way for customers to track their activity?

Yes. FTID owners receive:

- Daily email statements with contracts traded and estimated fees/rebates
- Customized reports available at <https://www.cboe.org/tphreports/default.aspx>

### 4. How will a customer receive their rebate?

- Request payment to executing agent, OR
- Electronic Funds Transfer (EFT) directly to customer
- Submit payment instructions via link on daily statement
- Multiple agents: Rebates paid pro rata based on volume share

### 5. Will the FTID be visible or available to the marketplace or trading counterparties?

**No.** FTID is:

- Recorded only on CBOE's internal order tracking system
- Not disclosed or discoverable by any other party
- CBOE will not disclose list or details of FTID owners
- No public record of FTID owners
- Personal information handled per Privacy Policy

### 6. Does the Program allow for post trade adjustments?

Yes. Submit adjustments via: <https://www.cboe.com/trading-resources/fee-schedules>

---

## Integration Considerations

### For Box Spread Trading

The Frequent Trader Program is particularly relevant for box spread strategies because:

1. **SPX/SPXW Options**: Box spreads typically use SPX or SPXW options, which qualify for rebates
2. **High Volume**: Box spread strategies may generate significant monthly volume
3. **Cost Reduction**: Rebates can reduce transaction costs, improving net profitability
4. **Tier Qualification**: Volume thresholds are achievable for active box spread traders

### Implementation Notes

- **FTID Management**: Store FTID securely (never commit to git)
- **Order Tagging**: Ensure FTID is appended to all CBOE orders
- **Volume Tracking**: Monitor monthly volume to track tier progression
- **Rebate Optimization**: Structure trading to maximize rebate tier qualification
- **Multi-Agent Coordination**: If using multiple brokers, coordinate FTID usage

### Configuration

Add FTID to order configuration:

```cpp
// Example: Add FTID to order tag
Order order;
order.m_tagList = "FTID=YOUR_FTID_HERE";
```

---

## Resources

- **Registration**: <https://www.cboe.com/FTID/registration.aspx>
- **Program PDF**: <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf>
- **Custom Reports**: <https://www.cboe.org/tphreports/default.aspx>
- **Post-Trade Adjustments**: <https://www.cboe.com/trading-resources/fee-schedules>
- **Rule Filing**: <http://www.cboe.com/aboutcboe/legal/submittedsecfilings.aspx>

---

## Key Takeaways

1. **Eligibility**: Non-TPH, non-broker/dealer customers only
2. **Coverage**: RUT, VIX®, SPX, and SPXW options
3. **Rebates**: 3-25% depending on product and volume tier
4. **Privacy**: FTID is private and not visible to counterparties
5. **Volume Aggregation**: Monthly contract volume determines tier
6. **Payment**: Can be paid to agent or directly to customer
7. **Tracking**: Daily email statements and custom reports available

---

## Relevance to Box Spread Strategies

The CBOE Frequent Trader Program directly benefits box spread trading:

- **SPX/SPXW Options**: Primary instruments for box spreads qualify for rebates
- **Volume Thresholds**: Achievable for active traders (10,000+ contracts/month)
- **Cost Efficiency**: Rebates reduce transaction costs, improving net spreads
- **Privacy**: FTID doesn't reveal trading strategy to counterparties
- **Flexibility**: Works with both RTH and GTH trading

---

**Note**: Always verify current program terms, rebate schedules, and eligibility requirements on the official CBOE website before relying on this documentation.
