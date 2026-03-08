# Historical Positions and Activity from IBKR

**Question:** Is it possible to pull **historical** positions from IB (Client Portal)?

---

## Short answer

- **Client Portal (Gateway, port 5001)** – **No.** The Portal REST API only exposes **current** positions and **recent** session activity. There is no documented "positions as of date X" or full trade history by date range.
- **Historical positions and full trade history** – **Yes**, but via **Flex Queries** (or the newer Web API reporting), not the Client Portal REST API this project uses for live snapshots.

---

## What the Client Portal (Gateway) provides

| Data | Endpoint (or path) | Notes |
|------|--------------------|--------|
| **Current positions** | `GET /iserver/account/{accountId}/positions` or `GET /v1/api/portfolio/{accountId}/positions` | Live only; no date parameter. Used by `ibkr_portal_client.get_portfolio_positions()`. |
| **Recent trades** | `GET /iserver/account/trades` | Rate limit: 1 req / 5 sec. Typically *recent/session* trades, not full history by date. |
| **Account summary** | `GET /iserver/account/{accountId}/summary` | Current balances and margin. |
| **Ledger (cash)** | `GET /v1/api/portfolio/{accountId}/ledger` | Current cash by currency. |
| **PnL (partitioned)** | `/iserver/account/pnl/partitioned` | 1 req / 5 sec; PnL breakdown, not position history. |
| **Portfolio Analyst** | `/pa/performance`, `/pa/summary` | 1 req / 15 min; performance metrics, not position snapshots. |

So from the **Portal alone** you cannot pull "positions as of 2024-01-15" or a full historical list of positions.

---

## How to get historical positions and activity (Flex / reporting)

1. **Flex Queries (classic)**  
   - In **Account Management** → **Reports** → **Flex Queries** you define a query (e.g. "Positions", "Trades", "PnL") with a **date range**.  
   - IB generates a report (XML/CSV). You can:  
     - Download it manually from the portal, or  
     - Use the **Flex Web Service** (token-based HTTPS) to request the report by query ID and date range.  
   - This is the standard way to get "positions as of date X", closed positions, and full trade history.  
   - Not part of the Client Portal REST API; separate auth and endpoints.

2. **Web API (newer)**  
   - IB's newer **Web API** (OAuth 2.0, different from the Gateway) includes **Reporting** and **Account Management**.  
   - Documentation: [Web API](https://www.interactivebrokers.com/campus/ibkr-api-page/web-api/).  
   - If "historical positions" or "position history" are exposed there, they would be in the Reporting/Account Management side, not in the local Client Portal Gateway we use for live data.

3. **Build your own history**  
   - Poll **current** positions from the Portal on a schedule (e.g. daily) and store them in your own DB (e.g. QuestDB, PostgreSQL).  
   - You then have "historical" positions as of each snapshot time. This is the only way to get history **using only the Client Portal** API.

---

## Practical recommendation

- **Live TUI/dashboard** – Keep using the Client Portal (Gateway) for **current** positions and account summary; no change.
- **Historical positions / "Hist" tab** – Either:  
  - **Option A:** Implement **Flex Web Service** (or Web API reporting) to pull historical positions/trades by date and feed the Historic tab, or  
  - **Option B:** Add a **scheduled job** that calls `get_portfolio_positions()` (and optionally account summary) and writes snapshots to your own store; the Historic tab reads from that store.

Rate limits for the Portal are in `docs/IBKR_RATE_LIMITS.md`. Flex and Web API have their own limits and auth (see IBKR Campus).
