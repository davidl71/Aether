# TWS, Client Portal & QuestDB – Quick Reference

How these pieces fit in the active platform: **TWS** and **Client Portal** for IBKR connectivity, **QuestDB** for time-series storage.

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

ORATS is **not** part of the active supported runtime.

- keep it as a historical/future-integration reference only
- do not treat historical ORATS references as proof of a live integration
- if revived later, treat it as a new integration effort

---

## 4. QuestDB

| What | Time-series DB for quotes and trades (ILP + REST SQL) |
|------|--------------------------------------------------------|
| **Role** | Persist validated quotes/trades; historical data for charts and backtests |
| **Ports** | ILP 9009 (writes), HTTP 9000 (REST queries) |
| **Code** | `python/integration/questdb_client.py`, `python/integration/market_data_handler.py`, `python/integration/strategy_runner.py` |
| **Config** | `config/config.json` → `questdb.enabled`, `questdb.ilp_host`, `questdb.ilp_port`, etc. |
| **Docs** | `README.md` (QuestDB Archiving), `docs/research/external/FINANCIAL_DATA_SOURCES_RESEARCH.md`, [QuestDB docs](https://questdb.com/docs/) |

Snapshot metrics include `questdb_ok`. **NATS → QuestDB:** the Rust backend can now write QuestDB ILP directly when `QUESTDB_ILP_ADDR` is set, using decoded `MarketDataEvent` envelopes from NATS. Use the normal Rust backend runtime with `NATS_URL` and `QUESTDB_ILP_ADDR` configured. For current subject/payload expectations, see **`docs/NATS_SETUP.md`** and **`docs/NATS_TOPICS_REGISTRY.md`**.

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
│ TWS API         │                             │ QuestDB         │
│ (C++ socket     │     │ options analytics│     │ (optional)      │
│  7496/7497)     │     │ liquidity / IV   │     │ quotes/trades   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

- **Live snapshot for PWA:** Gateway running + logged in → IB service running → `VITE_API_URL=http://127.0.0.1:8002/api/snapshot` and dev server restarted.
- **QuestDB:** Set `questdb.enabled` and run QuestDB (e.g. Docker); strategy/market handler write quotes/trades; notebooks/backtests can query via `notebooks/utils/data_loaders.py` or `questdb_client`.

---

## See also

- **API index:** `docs/API_DOCUMENTATION_INDEX.md` (TWS, historical ORATS references, dxFeed, data stack)
- **IB + PWA:** `web/README.md` – “Connect PWA to IB Gateway”
- **ORATS usage:** `python/ORATS_USAGE.md` (historical/future only)
- **QuestDB / data stack:** `docs/research/external/FINANCIAL_DATA_SOURCES_RESEARCH.md`, `docs/MCP_INTERACTIVE_TOOLS.md` (QuestDB MCP)
