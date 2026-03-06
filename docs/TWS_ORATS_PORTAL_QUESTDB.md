# TWS, ORATS, Client Portal & QuestDB – Quick Reference

How these four pieces fit in the platform: **TWS** and **Client Portal** for IBKR connectivity, **ORATS** for options analytics, **QuestDB** for time-series storage.

---

## 1. TWS (Trader Workstation) / TWS API

| What | C++ socket API to IBKR; execution + market data |
|------|------------------------------------------------|
| **Role** | Connect to TWS or IB Gateway, send orders, receive ticks and account data |
| **Port** | 7496 (live), 7497 (paper) – socket, not HTTP |
| **Code** | `native/src/tws_client.cpp`, `native/include/brokers/` |
| **Docs** | [TWS API](https://interactivebrokers.github.io/tws-api/), `docs/API_DOCUMENTATION_INDEX.md` (search “TWS API”) |

**Note:** TWS API is the **classic** C++/Java socket API. The project also uses the **Client Portal** (REST) for the PWA/IB service.

**Important:** Client Portal and TWS are **exclusive** — only one can be logged in at a time. Use either the Client Portal Gateway (port 5001) or TWS/Gateway socket (7496/7497), not both simultaneously.

---

## 2. Client Portal (IB Gateway REST)

| What | REST API served by IB Client Portal Gateway (same login as TWS) |
|------|-----------------------------------------------------------------|
| **Role** | REST endpoints for snapshot, portfolio, orders; used by the **IB service** and PWA |
| **Port** | 5001 (default); login at https://localhost:5001 |
| **Code** | `python/integration/ib_service.py` (calls Portal), `ib-gateway/run-gateway.sh` |
| **Docs** | [CP Web API](https://interactivebrokers.github.io/cpwebapi/), `web/README.md` (Connect PWA to IB Gateway) |

**Flow:** IB Gateway (logged in) → Client Portal API (5001) ← IB service (8002) ← PWA (`VITE_API_URL`).

**Important:** Client Portal and TWS are **exclusive** — only one can be logged in at a time. If you need TWS socket (7496/7497), log out of the Client Portal Gateway (or vice versa).

---

## 3. ORATS

| What | Options analytics API: Greeks, IV, liquidity, earnings/dividend calendars |
|------|----------------------------------------------------------------------------|
| **Role** | Enrich option chains, liquidity scoring, risk filters (earnings/dividend blackouts) |
| **API** | REST, https://api.orats.io (token required) |
| **Code** | `python/integration/orats_client.py`, `python/integration/strategy_runner.py` (optional enricher) |
| **Config** | `config/config.json` → `orats.enabled`, `orats.api_token`, etc. |
| **Docs** | `python/ORATS_USAGE.md`, `docs/research/external/ORATS_INTEGRATION.md`, [orats.com/docs](https://orats.com/docs) |

Snapshot metrics include `orats_ok` (PWA/HeaderStatus); strategy can use ORATS for liquidity and risk checks.

---

## 4. QuestDB

| What | Time-series DB for quotes and trades (ILP + REST SQL) |
|------|--------------------------------------------------------|
| **Role** | Persist validated quotes/trades; historical data for charts and backtests |
| **Ports** | ILP 9009 (writes), HTTP 9000 (REST queries) |
| **Code** | `python/integration/questdb_client.py`, `python/integration/market_data_handler.py`, `python/integration/strategy_runner.py` |
| **Config** | `config/config.json` → `questdb.enabled`, `questdb.ilp_host`, `questdb.ilp_port`, etc. |
| **Docs** | `README.md` (QuestDB Archiving), `docs/research/external/FINANCIAL_DATA_SOURCES_RESEARCH.md`, [QuestDB docs](https://questdb.com/docs/) |

Snapshot metrics include `questdb_ok`. **NATS → QuestDB:** (1) **Python writer** – `python/integration/questdb_nats_writer.py` subscribes to Core NATS `market-data.tick.>`, parses JSON ticks, writes to QuestDB ILP (table `market_data`). Run via `./scripts/run_questdb_nats_writer.sh` or `./scripts/service.sh start questdb_nats`. (2) **Go bridge** – `agents/go/cmd/nats-questdb-bridge/` subscribes to JetStream `market.data.>` (JSON shape: symbol, bid, ask, last, volume, timestamp) and writes ILP. For full pipeline options (Core NATS vs JetStream, protobuf vs JSON), see **`docs/NATS_USE_OPPORTUNITIES.md`**.

---

## How they work together

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ IB Gateway      │     │ IB service      │     │ PWA / TUI       │
│ (Client Portal  │◀────│ (Python 8002)   │◀────│ VITE_API_URL    │
│  https://:5001) │     │                 │     │ snapshot        │
└────────┬────────┘     └────────┬────────┘     └─────────────────┘
         │                        │
         │ (same login as TWS)     │ optional
         │                        ▼
┌────────┴────────┐     ┌──────────────────┐     ┌─────────────────┐
│ TWS API         │     │ ORATS (optional) │     │ QuestDB         │
│ (C++ socket     │     │ options analytics│     │ (optional)      │
│  7496/7497)     │     │ liquidity / IV   │     │ quotes/trades   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

- **Live snapshot for PWA:** Gateway running + logged in → IB service running → `VITE_API_URL=http://127.0.0.1:8002/api/snapshot` and dev server restarted.
- **ORATS:** Set `orats.enabled` and `orats.api_token` in config; strategy and snapshot can expose `orats_ok`.
- **QuestDB:** Set `questdb.enabled` and run QuestDB (e.g. Docker); strategy/market handler write quotes/trades; notebooks/backtests can query via `notebooks/utils/data_loaders.py` or `questdb_client`.

---

## See also

- **API index:** `docs/API_DOCUMENTATION_INDEX.md` (TWS, ORATS, dxFeed, data stack)
- **IB + PWA:** `web/README.md` – “Connect PWA to IB Gateway”
- **ORATS usage:** `python/ORATS_USAGE.md`
- **QuestDB / data stack:** `docs/research/external/FINANCIAL_DATA_SOURCES_RESEARCH.md`, `docs/MCP_INTERACTIVE_TOOLS.md` (QuestDB MCP)
