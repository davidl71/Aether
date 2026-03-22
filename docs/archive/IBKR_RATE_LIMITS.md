# Interactive Brokers API Rate Limits

Summary of **Interactive Brokers’ recommended or enforced** rate limits and how this project applies them.

**Sources:** IBKR Campus (TWS API doc, Market Data Subscriptions, Web API v1 / Client Portal API). Limits can change; check [IBKR API Home](https://www.interactivebrokers.com/campus/ibkr-api-page/ibkr-api-home/) and [Web API v1](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/) for current values.

---

## 1. TWS API (socket – TWS/Gateway)

### General message pacing

- **IB does not publish a single “messages per second” number** in the main TWS API doc.
- Common practice and many implementations use **up to 50 messages per second** to avoid disconnects or pacing errors.
- **Error 162**: “Historical data request pacing violation” — reduce request frequency; the rate limiter should prevent this.

### Historical data

- From TWS API Reference: **“no more than 60 API queries in more than 600 seconds”** for historical data (and real-time bars under the same pacing).
- Implies **~1 historical request per 10 seconds** when at the limit (60 in 600 s).

### Market data lines

- From [Market Data Subscriptions](https://www.interactivebrokers.com/campus/ibkr-api-page/market-data-subscriptions/):
  - **Default: 100 concurrent market data lines** per user (TWS watchlist + API share this pool).
  - After the first month, allocation is the **greater of**: 100, `(USD equity × 100 / 1,000,000)`, or `USD monthly commissions / 8`.
  - Tick-by-tick and market depth (Level II) have **lower caps** that scale with line count (see table on that page).

### This project’s TWS rate limiter

| Limit                    | Default in code | Notes                                      |
|--------------------------|-----------------|--------------------------------------------|
| Messages per second      | 50              | `native/include/rate_limiter.h`            |
| Simultaneous historical  | 50              | Avoids exceeding historical pacing        |
| Market data lines        | 100             | Matches IB’s default line allocation       |

- Implemented in `native/src/rate_limiter.cpp` and used by the TWS connection / client (e.g. `tws_connection.cpp`, `tws_client.cpp`).
- Error 162 is mapped in `native/src/tws_error_codes.cpp` to guidance: “Historical data request pacing violation. Rate limiter should prevent this. Reduce request frequency.”

---

## 2. Client Portal API (REST – Gateway / Web API)

From [Web API v1.0 Documentation – Pacing](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/):

- **Global:** **10 requests per second** (for endpoints not listed in the pacing table).
- If exceeded: **429 Too Many Requests**; violator IP can be put in a **penalty box for 15 minutes**; repeat violators can be **permanently blocked** until resolved.

### Selected endpoint-specific limits

| Endpoint                               | Limit              |
|----------------------------------------|--------------------|
| Most endpoints (default)               | 10 req/s global    |
| `/iserver/marketdata/snapshot`         | 10 req/s           |
| `/iserver/marketdata/history`         | 5 concurrent req   |
| `/iserver/account/orders`             | 1 req / 5 sec     |
| `/iserver/account/trades`              | 1 req / 5 sec     |
| `/iserver/account/pnl/partitioned`     | 1 req / 5 sec     |
| `/portfolio/accounts`                  | 1 req / 5 sec     |
| `/portfolio/subaccounts`               | 1 req / 5 sec     |
| `/iserver/scanner/run`                 | 1 req/s            |
| `/tickle`                              | 1 req/s (call ~every 1 min to avoid timeout) |
| `/iserver/scanner/params`             | 1 req / 15 min     |
| `/pa/performance`, `/pa/summary`, etc. | 1 req / 15 min     |

### This project’s Client Portal usage

- Python: `python/integration/ibkr_portal_client.py` — no built-in rate limiter; callers should throttle (e.g. snapshot at 10 req/s, account/orders at 1 per 5 s).
- Design doc: `docs/research/external/IB_CLIENT_PORTAL_API_INTEGRATION_DESIGN.md` mentions 429 handling and backoff.
- For high-frequency polling, stay under **10 req/s** overall and respect the per-endpoint table above.

---

## 3. Recommendations

1. **TWS (C++):** Keep using the existing rate limiter (50 msg/s, 50 historical, 100 lines). If you see error 162, ensure historical requests are spaced (e.g. not more than ~60 in 600 s) and that `record_message()` is used for every outgoing API message.
2. **Client Portal (Python/REST):** Throttle to **≤10 req/s** globally; for `/iserver/account/orders`, `/iserver/account/trades`, and similar, use **≤1 request per 5 seconds** per endpoint. Call `/tickle` about once per minute to avoid session timeout.
3. **Market data:** Respect the 100-line default (or your account’s allocation); count TWS watchlist + API subscriptions together.
4. **Check official docs** before changing limits; IB can update pacing and add new endpoint-specific rules.
