# Primary Currency and Cross-Venue Hedging (IB + TASE)

**Status:** Planning  
**Last updated:** 2025-03-05

## 1. Goals

1. **Primary currency per account** – User defines the reporting/decision currency for each account (e.g. ILS for Israeli accounts, USD for US accounts). All P&L, exposure, and hedging suggestions are expressed in that currency where relevant.
2. **Find/suggest hedging** – Suggest combinations such as:
   - **Long USD** (e.g. box spread on IB in USD) ↔ **hedge with TASE** (e.g. buy/sell put/call on TA-35, TA-125, or **TASE options on USD** such as ILS/USD currency options).
   - So the system can propose: "You have X USD exposure from box spread on IB; consider hedging with Y on TASE (index option or **options on USD**)."

---

## 2. Primary Currency for Account

### 2.1 Current state

- **Config:** No explicit "primary currency" or "reporting currency" for accounts. `config.example.json` has `dataSources.primary` (data source name, not currency).
- **Code:**
  - `HedgeManager` uses `base_currency` / `hedge_currency` (e.g. USD/ILS) for currency hedges; strategy has `hedge_currency_code` (e.g. `"ILS"`).
  - TWS/positions assume USD for key fields (`NetLiquidation`, `TotalCashBalance`, etc.) when `currency == "USD"`.
  - Multi-account design doc mentions "Currency conversion (ILS → USD) for position valuation" but no single "primary currency" per account.
- **Loans:** `LoanPosition` has `get_usd_value(ils_usd_rate)` for ILS→USD conversion; currency is implicit per loan.

### 2.2 Proposed: primary currency in config

**Location:** Account-level or portfolio-level in shared config.

**Option A – Per account (preferred for multi-account)**

```json
{
  "accounts": [
    {
      "id": "IB-123",
      "broker": "ib",
      "primary_currency": "USD"
    },
    {
      "id": "Meitav-456",
      "broker": "meitav",
      "primary_currency": "ILS"
    }
  ]
}
```

**Option B – Single global (simpler)**

```json
{
  "portfolio": {
    "primary_currency": "ILS"
  }
}
```

**Recommendation:** Support both: `accounts[].primary_currency` when present, else fall back to `portfolio.primary_currency` (default `"USD"`).

**Use of primary currency**

- **Reporting:** Convert and show P&L, NAV, and exposures in primary currency when different from position currency (e.g. ILS account: show USD box spread value in ILS using configured or live FX).
- **Hedging decisions:** When suggesting hedges, treat "account primary = ILS" as "user cares about ILS outcome"; suggest TASE or FX hedges to reduce ILS volatility. When "primary = USD", focus on USD-rate/vol hedges.
- **HedgeManager / strategy:** Already has `hedge_currency_code`; can be defaulted from account/portfolio `primary_currency` when the hedge is for "reporting currency" risk.
- **UI:** TUI/PWA can show "Account currency: ILS" and convert all relevant numbers to ILS when primary ≠ USD.

**Implementation order**

1. Add `primary_currency` to shared config schema (portfolio and optionally per account).
2. Load and expose in TUI config and backend (e.g. `SharedConfigLoader`, C++ config if needed).
3. Use in reporting/aggregation (convert to primary currency) and pass into hedging logic as "reporting currency".

---

## 3. Hedging Suggestion: IB Box Spread + TASE Put/Call

### 3.1 Idea

- **IB (USD):** User holds or considers a box spread (e.g. SPX/XSP) → long USD cash flow / USD interest rate exposure.
- **TASE (ILS):** User can hedge or express views via:
  - **TASE index options** (TA-35, TA-125) – put/call; correlation with ILS and local rates.
  - **TASE options on USD** – e.g. **ILS/USD (or USD/ILS) currency options** listed on TASE; direct FX hedge for USD exposure. The Israeli market also lists other USD-related derivatives.
  - **TASE stock/other options** as relevant.
- **Suggestion:** "Given your box spread exposure on IB (notional, tenor, currency), here are suggested TASE option trades (put/call on index or **on USD**) to hedge currency or local-rate risk."

### 3.2 Current state

- **IB box spread:** Full support in C++/Python (strategy, pricing, margin, yield curve). Currency of box = USD.
- **TASE:**
  - Docs: `ISRAELI_BROKER_POSITION_IMPORT.md`, `ISRAELI_FUND_TYPES.md`, `API_DOCUMENTATION_INDEX.md` (TASE data vendors).
  - Models: `python/integration/israeli_broker_models.py` – TASE exchange, TA-35/TA-125, `is_tase_option()`.
  - No live TASE options chain or pricing API in repo yet; position import from files/Excel/RTD.
- **Hedging:** `HedgeManager` does currency hedge (USD/ILS notional, exchange rate) and interest-rate hedge (SOFR futures). No "suggest TASE option" logic yet.

### 3.3 Data needed for suggestions

| Need | Source | Status |
|------|--------|--------|
| Box spread notional, tenor, currency | IB / existing engine | Available |
| Account primary currency | Config (above) | To add |
| USD/ILS (and other) FX rate | HedgeManager stub; TWS/FRED/API | Stub exists; improve later |
| TASE option chain (TA-35, TA-125, **options on USD** / ILS/USD) | **TASE Data Hub API** (<https://datahubapi.tase.co.il/>), broker API, or vendors | Not in repo |
| TASE option quotes (bid/ask, IV) | Same | Not in repo |
| Correlation or hedge ratio (box vs TASE option) | Model or heuristic | New |

### 3.4 Suggestion logic (high level)

1. **Inputs**
   - Current or proposed box spread(s) on IB: notional USD, tenor, implied rate.
   - Account primary currency (e.g. ILS).
   - Optional: current TASE positions (from Israeli broker import).
2. **Exposure**
   - USD exposure from box = notional (and optionally duration for rate sensitivity).
   - If primary = ILS, translate to "ILS equivalent" using FX for display.
3. **Matching**
   - **Currency hedge:** Suggest **TASE options on USD** (e.g. ILS/USD or USD/ILS currency options on TASE) – buy USD put / ILS call to hedge USD depreciation vs ILS.
   - **Index hedge:** Suggest TA-35 / TA-125 puts or calls depending on desired correlation (e.g. hedge local equity risk while keeping USD financing).
   - **Tenor:** Prefer TASE expiries close to box expiry where possible.
4. **Output**
   - List of "suggested hedges": instrument (exchange, underlying, type put/call), direction (buy/sell), approximate size (notional or contracts), rationale (e.g. "Currency hedge for X USD box notional").

### 3.5 Where to implement

- **Primary currency:** Config + shared loader; optional C++ config; TUI/PWA read and use for display + hedging context.
- **Hedge suggestion service:** New module (e.g. `python/integration/hedge_suggestion.py` or `native` if heavy logic):
  - Input: box spread(s), account primary currency, FX rate(s).
  - Output: list of suggested hedges (description, venue, instrument, direction, size, rationale).
- **TASE data:**
  - Phase 1: No live chain; suggestions can be "template" (e.g. "Consider TA-35 put with expiry near box expiry; notional ~ X ILS") using notional and tenor only.
  - Phase 2: Integrate TASE option chain via **TASE Data Hub API** (<https://datahubapi.tase.co.il/>) or broker API when available; refine hedge ratios and sizes.
- **UI:** New section or tab "Hedging" / "Suggestions": show primary currency, box exposure in primary currency, and list of suggested TASE (and later other) hedges with rationale.

### 3.6 TASE-specific notes (from existing docs)

- **Index options:** TA-35, TA-125, TA-90, TA-Banks5.
- **Options on USD:** TASE lists **currency options on USD** (e.g. ILS/USD, USD/ILS) and other USD-related derivatives; use these to hedge USD exposure from IB box spreads without leaving the Israeli market.
- **Other currency options:** ILS/EUR, etc.
- **Clearing:** MAOF.
- **Data:** **TASE Data Hub API** (<https://datahubapi.tase.co.il/>) is the official developer portal for programmatic TASE data; use for option chain (index options, options on USD) when integrating. Alternatively, authorized vendors (e.g. TNS) or Israeli broker APIs. **IBKR (TWS API)** provides USD/ILS and other FX via `secType=CASH`, `exchange=IDEALPRO`; Israeli equities and TASE options (if available) can be verified via [IBKR Symbol and Exchange Search](https://www.interactivebrokers.com/en/trading/products-exchanges.php)—see `docs/API_DOCUMENTATION_INDEX.md` § IBKR Israeli product data. Israeli broker position import (Excel/RTD/scrape) can provide existing TASE positions for "current exposure" in suggestions.

---

## 4. Implementation order (concise)

1. **Primary currency**
   - Add `portfolio.primary_currency` and optional `accounts[].primary_currency` to shared config schema and loader.
   - Default `primary_currency` to `"USD"` when unset.
   - Use in reporting (convert to primary) and pass into hedging as "reporting currency".
2. **Hedge suggestion (stub)**
   - New module: input = box spread(s) + primary currency + FX; output = list of suggested hedges (e.g. "TASE TA-35 put", "TASE options on USD (ILS/USD currency option)") with notional/tenor and short rationale.
   - No TASE live data yet; use notional and tenor only (templates).
3. **UI**
   - Show primary currency in Setup or Dashboard.
   - Add "Hedging suggestions" section/tab: show box exposure in primary currency and list of suggested TASE hedges (and later currency/rate hedges).
4. **Later**
   - Integrate TASE option chain when available; refine hedge ratios and sizes.
   - Add correlation or regression-based hedge ratios if data allows.

---

## 5. References

- **TASE Data Hub API**: <https://datahubapi.tase.co.il/> – official developer/data portal for TASE market data (securities, derivatives, options on USD). **API docs**: <https://datahubapi.tase.co.il/docs/1626b30a-9369-4f6e-b0ec-b0340d8515bf/1748180086472>. **API guide (PDF, English)**: <https://content.tase.co.il/media/l5xjhjmz/2000_api_guide_eng.pdf>. **Data file distribution**: <https://www.tase.co.il/en/content/data/file_distribution>. **Derivatives daily file format 28 (PDF, English)**: [sh28eng.pdf](https://content.tase.co.il/media/0munlqqi/sh28eng.pdf) – record layout for “Derivatives - Daily Trading Results” (header 01, derivative details 02, additional 03/04, trailer 99). **Market data (Hebrew)**: <https://market.tase.co.il/he/market_data>. **Derivatives major data (Hebrew)**: <https://market.tase.co.il/he/market_data/derivatives/major_data/details>. **Derivatives EoD history (10y)**: <https://datahubapi.tase.co.il/spec/f5196aac-357f-49e1-8984-2a93d0160758/c77e97c5-120d-4bf8-b54d-e6e8c6d359da#/APIs/getDerivativesEoDHistory10YearsData>. **Third-party scraper**: [algonell/tase](https://github.com/algonell/tase) (Python, TA35, Jupyter).
- `docs/research/analysis/CURRENCY_EXCHANGE_RISK.md` – Currency risk and hedging.
- `native/include/hedge_manager.h`, `native/src/hedge_manager.cpp` – Currency and rate hedging.
- `docs/ISRAELI_BROKER_POSITION_IMPORT.md` – TASE instruments, MAOF, position import.
- `docs/ISRAELI_FUND_TYPES.md` – TASE indices, ETFs.
- `docs/platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md` – Account model, currency conversion.
- `python/integration/israeli_broker_models.py` – TASE exchange, TA-35/TA-125 helpers.
- `config/config.example.json` – Current config shape; add `portfolio.primary_currency` and optional `accounts`.
