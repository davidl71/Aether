# Boxtrades.com Reference

**URL:** <https://www.boxtrades.com/>  
**Purpose:** External reference for box spread yields by underlying and expiration. Use for UX alignment (yield curve view, expiry buckets) and as a market reference when comparing our engine’s implied rates.

## What Boxtrades.com Provides

- **SPX (S&P 500 Index):** Box spread yield curve by expiration. Links to per-expiration pages (e.g. 20 Mar 26, 17 Apr 26, … out to ~6 years).
- **ES (E-mini S&P 500 Futures):** Same idea for futures-related expirations (e.g. 17 Dec 27).
- **Structure:** One page per underlying; expirations grouped in buckets (short-dated → long-dated). No execution; display/reference only.

## Alignment With This Project

| Aspect | Boxtrades | This project |
|--------|-----------|--------------|
| **Concept** | Box spreads as yield (T-bill alternative) | Same: box spreads for spare-cash yield (TUI/CLI, strategy). |
| **Underlyings** | SPX, ES | Configurable (e.g. SPX, XSP); see shared config `strategy.symbols`. |
| **Expiry buckets** | 5 days, ~1 month, 2 months, … ~6 years | We use similar labels for TUI/API: see expiry bucket labels below. |
| **Yield curve** | One view per symbol, many expirations | NATS `api.finance_rates.build_curve`, `api.finance_rates.yield_curve`; TUI “Yield” tab (when wired). |
| **Data source** | Their own data | Backend (NATS); optional FRED for benchmarks. |

## Expiry Bucket Labels (aligned with Boxtrades)

Use these labels when displaying expirations in the TUI or API (e.g. Yield curve tab, scenario explorer):

| Approx DTE range | Label (short) | Example expiry |
|------------------|---------------|----------------|
| 0–7 | 5 days | Weekly / nearby |
| 8–25 | about 1 month | ~1 month out |
| 26–45 | 2 months | 2-month tenor |
| 46–75 | 3 months | 3-month |
| 76–105 | 4 months | 4-month |
| 106–135 | 5 months | 5-month |
| 136–165 | 6 months | 6-month |
| 166–200 | 7 months | 7-month |
| 201–235 | 8 months | 8-month |
| 236–270 | 9 months | 9-month |
| 271–320 | 10 months | 10-month |
| 321–380 | 11 months | 11-month |
| 351–380 | about 1 year | 1-year |
| 381–730 | over 1 year | 1–2 years |
| 731–1095 | almost 2 years | 2 years |
| 1096–1460 | almost 3 years | 3 years |
| 1461–1825 | almost 4 years | 4 years |
| 1826–2190 | almost 5 years | 5 years |
| 2191+ | almost 6 years | 6 years |

Implementation: see `agents/backend/services/tui_service/src/expiry_buckets.rs` (or shared helper in `crates/common` / `crates/api`) for a function that maps `days_to_expiry` (or expiry date) to the label string.

## When to Use This Reference

- **TUI Yield curve tab:** Show symbol → expirations with APR; use bucket labels for column or grouping. Reference link in UI or help: “Compare: boxtrades.com”.
- **Scenario explorer:** Sort/filter by expiry; display bucket label next to date (e.g. “20 Mar 26 (about 1 month)”).
- **Docs:** Point to this file from TUI_LEGACY_DESIGN_LEARNINGS, NATS_API (finance_rates), and any “market reference” section.
- **Testing / validation:** Manually compare our implied rates (from `api.finance_rates.build_curve` or strategy) to boxtrades for the same symbol/expiry when checking reasonableness.

## Pre-populating the yield curve from public sources

The backend **pre-populates** the yield curve at startup (one write immediately, then on an interval) so the TUI Yield tab has data without waiting. Data source:

- **Default:** Synthetic curve points (no live option chain). Good for demos and development.
- **Optional public URL:** Set `YIELD_CURVE_SOURCE_URL` to a URL that returns a **JSON array** of curve points. The backend fetches that URL and writes `yield_curve.{symbol}` from the response. If the fetch fails or returns empty, it falls back to synthetic.

**Boxtrades.com:** They do not offer a public API or data feed. Use their site for **visual comparison** (same expiries, similar buckets). To use real-world-style data, host your own JSON file (e.g. from CBOE’s box spread tools, manual entry, or another provider) and set `YIELD_CURVE_SOURCE_URL` to that URL.

### JSON schema for `YIELD_CURVE_SOURCE_URL`

The URL must return a JSON **array** of objects. Each object (snake_case) must include:

| Field               | Type   | Required | Description |
|---------------------|--------|----------|-------------|
| `symbol`            | string | yes      | Underlying (e.g. SPX, XSP). |
| `expiry`            | string | yes      | Expiry date (e.g. `2026-03-20`). |
| `days_to_expiry`    | number | yes      | Integer days to expiry. |
| `buy_implied_rate`  | number | yes      | Implied rate (decimal, e.g. 0.045). |
| `sell_implied_rate` | number | yes      | Implied rate (decimal). |
| `strike_width`      | number | no       | Default 5. |
| `net_debit`         | number | no       | Default 0. |
| `net_credit`       | number | no       | Default 0. |
| `liquidity_score`   | number | no       | Default 50. |
| `spread_id`         | string | no       | Optional id. |

Example response (one symbol, two expiries):

```json
[
  { "symbol": "SPX", "expiry": "2026-03-20", "days_to_expiry": 30, "buy_implied_rate": 0.044, "sell_implied_rate": 0.052, "strike_width": 5, "liquidity_score": 70 },
  { "symbol": "SPX", "expiry": "2026-06-19", "days_to_expiry": 90, "buy_implied_rate": 0.046, "sell_implied_rate": 0.054 }
]
```

Points are grouped by `symbol`; each symbol is written to `yield_curve.{symbol}` in the NATS KV bucket. See `NATS_KV_USAGE_AND_RECOMMENDATIONS.md` and `yield_curve_writer.rs` for implementation details.

## Links

- **Home:** <https://www.boxtrades.com/>
- **SPX example (one expiry):** <https://www.boxtrades.com/SPX/20MAR26>
- **ES example:** <https://www.boxtrades.com/EW3Z7/17DEC27>
