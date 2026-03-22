# Box Spread & Cash-Equivalent ETF Alternatives

<!--
@index: trading-concepts
@category: reference
@tags: box-spread, etf, boxx, t-bill, cash-equivalent, risk-free-rate
@last-updated: 2026-03-05
-->

Quick reference for **ETF alternatives** to executing box spreads yourself: box-spread ETFs, T-bill ETFs, and related cash-like products. For full detail on hedging and tax treatment, see [T_BILLS_AND_FUTURES_GUIDE.md](T_BILLS_AND_FUTURES_GUIDE.md).

---

## Box-spread / options-based T-bill alternatives

| Ticker | Name | Strategy | Tax | Notes |
|--------|------|----------|-----|--------|
| **BOXX** | [Alpha Architect 1-3 Month Box ETF](https://www.alphaarchitect.com/etfs/boxx) | Box spreads to replicate 1–3 month T-bill returns | Capital gains | Main “box spread in an ETF” product; ~\$8B+ AUM; high liquidity. |

**BOXX** is the primary listed product that uses box spreads to deliver T-bill-like returns with **capital gains treatment** instead of interest income. Useful as:

- Tax-efficient alternative to T-bills in taxable accounts
- Benchmark to compare your own box spread implied rates
- One-click exposure without managing options legs

Other “options income” ETFs (e.g. JEPI, XYLD, covered-call strategies) are **not** box-spread or risk-free substitutes; they have equity and volatility risk.

---

## T-bill ETFs (interest income, no options)

Direct T-bill exposure, no box spreads. Interest taxed as ordinary income.

| Ticker | Name | Focus | Expense ratio (typical) |
|--------|------|--------|---------------------------|
| **SGOV** | iShares 0-3 Month Treasury Bond ETF | 0–3 month T-bills | ~0.07% |
| **BIL**  | SPDR Bloomberg 1-3 Month T-Bill ETF | 1–3 month T-bills | ~0.14% |
| **SHV**  | iShares Short Treasury Bond ETF | Short Treasuries | ~0.15% |

Use when you want **convenient T-bill exposure** and liquidity without using options; tax treatment is like physical T-bills.

---

## Comparison (when to use what)

| Goal | Prefer |
|------|--------|
| Tax-efficient T-bill-like return (taxable account) | **BOXX** |
| Simple T-bill exposure, minimal cost | **SGOV**, **BIL**, **SHV** |
| Direct control, custom expirations | Your own box spreads (this project) |
| Hedging box spread positions | T-bills, T-bill futures, or T-bill ETFs; BOXX as tax-efficient alternative — see [T_BILLS_AND_FUTURES_GUIDE.md](T_BILLS_AND_FUTURES_GUIDE.md). |

---

## References

- **Project:** [T_BILLS_AND_FUTURES_GUIDE.md](T_BILLS_AND_FUTURES_GUIDE.md) — BOXX, leveraged ETFs, hedging, integration with box spread trading.
- **External:** [Picture Perfect Portfolios: BOXX ETF Review](https://pictureperfectportfolios.com/boxx-etf-review-alpha-architect-1-3-month-box-strategy/) (cited in [INVESTMENT_STRATEGY_PLAN.md](INVESTMENT_STRATEGY_PLAN.md)).
