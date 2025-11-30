# IBKR Pro Commission Rates Reference

**Source**: <https://www.interactivebrokers.com/en/pricing/commissions-home.php>
**Account Type**: IBKR Pro
**Last Updated**: 2025-11-30

---

## Options Commissions (US Listed)

### Standard IBKR Pro Pricing

**Per Contract Fee**: $0.65 per contract
**Minimum per Order**: $1.00 (waived if commissions exceed $30/month)
**Maximum per Order**: 1% of trade value (for orders under $1.00/share)

### Volume-Based Tiered Pricing (Monthly)

For accounts with high monthly volume, IBKR offers tiered pricing:

| Monthly Volume | Per Contract Fee |
|----------------|------------------|
| 0 - 10,000 contracts | $0.65 |
| 10,001 - 50,000 contracts | $0.60 |
| 50,001 - 100,000 contracts | $0.55 |
| 100,001+ contracts | $0.50 |

### Additional Fees

- **Regulatory Fees**: Varies by exchange (e.g., OCC, SEC fees)
- **Exchange Fees**: Varies by exchange
- **Market Data Fees**: Optional subscriptions

---

## Box Spread Commission Calculation

For a box spread (4 legs):

**Basic Calculation**:

- Entry: 4 contracts × $0.65 = $2.60
- Exit: 4 contracts × $0.65 = $2.60
- **Total**: $5.20 per box spread round trip

**With Volume Discount** (50,000+ contracts/month):

- Entry: 4 contracts × $0.55 = $2.20
- Exit: 4 contracts × $0.55 = $2.20
- **Total**: $4.40 per box spread round trip

**Per Contract** (for rate calculations):

- Basic: $0.65 per contract
- Tiered (high volume): $0.50-$0.60 per contract

---

## Important Notes

1. **Minimum Order Fee**: If total commission is less than $1.00 and monthly commissions are less than $30, a $1.00 minimum applies
2. **Maximum Order Fee**: Orders under $1.00/share are capped at 1% of trade value
3. **Exchange Fees**: Additional fees may apply based on exchange
4. **Regulatory Fees**: SEC, OCC fees vary but are typically small per contract

---

## Implementation Recommendation

For box spread calculations:

- Use **$0.65 per contract** as default (conservative estimate)
- Allow configuration for volume-based tiers
- Account for round-trip costs (entry + exit)
- Consider minimum order fees for small positions
