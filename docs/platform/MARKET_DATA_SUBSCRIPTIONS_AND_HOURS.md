# Market Data Subscriptions and Market Hours

**Purpose:** Clarify **what we're subscribed to** (symbols, contract type, tick types, NATS topics) and how **market hours** apply today and how they could be configured. Use this when debugging live data, adding symbols, or restricting to RTH.

**Related:** [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_LEARNINGS_FROM_SIBLING_REPO.md) (tick types, useRTH, tradingHours), [NATS_API.md](NATS_API.md) (topics), [NATS_TOPICS_REGISTRY.md](../NATS_TOPICS_REGISTRY.md).

---

## 1. What we're subscribed to

### 1.1 Configuration

- **Source:** `agents/backend/config/default.toml` (overridable by env or other config).
- **Section:** `[market_data]`
  - **provider:** `"mock"` | `"polygon"` | `"tws"`. Default `"mock"`.
  - **symbols:** List of symbols. Default `["SPX", "XSP", "NDX"]` (European-style indices; American-style SPY/QQQ/IWM are available but not default).
  - **poll_interval_ms:** Used for mock and Polygon; TWS is stream-based.

When **provider = "tws"**:

- **backend_service** calls `tws_market_data::spawn_tws_market_data(symbols, ...)`.
- Connection: **TWS_HOST** (default `127.0.0.1`), **TWS_PORT** (optional; if unset, tries 7497, 4002, 7496, 4001), **TWS_CLIENT_ID** (default 0 for market data).

### 1.2 TWS subscription (contract and request)

- **Contract:** For each symbol we build a **stock** contract: `Contract::stock(&symbol).build()` (e.g. SPX, XSP, NDX as underlyings).
- **Request:** We call `client.market_data(&contract).subscribe()` (ibapi crate). That corresponds to TWS **reqMktData**: one subscription per symbol, default options (no explicit genericTickList or snapshot flag in our code).
- **Result:** A stream of ticks. We do **not** currently pass:
  - **genericTickList** (e.g. "100,101,104,105,106" for optional tick types),
  - **snapshot** (one-shot vs streaming),
  - or any **useRTH**-style filter (that applies to historical/real-time bars in the TWS API, not to the streaming reqMktData in the same way; see §3).

So **we're subscribed to**:

- **Instruments:** Stock contracts for each symbol in `market_data.symbols` (default SPX, XSP, NDX).
- **Data:** Whatever the ibapi crate requests and TWS returns for those contracts (typically streaming bid/ask/last and sizes). We only **use** bid, ask, and last in code (see §1.3).

### 1.3 Tick types we consume

In **`agents/backend/services/backend_service/src/tws_market_data.rs`** we handle:

- **TickTypes::Price(p)** with:
  - **TickType::Bid** → store `per_symbol[symbol].bid = price`
  - **TickType::Ask** → store `per_symbol[symbol].ask = price`
  - **TickType::Last** → used only to backfill bid/ask when either is missing (`if entry.bid <= 0.0` / `if entry.ask <= 0.0`).

We **ignore** all other tick types (sizes, option computations, delayed, etc.). Downstream we only emit when **both** bid and ask are present (`s.bid > 0.0 && s.ask > 0.0`), then build:

- **MarketDataEvent** { symbol, bid, ask, timestamp }
- Forwarded to **handle_market_event** → shared snapshot and NATS.

### 1.4 NATS and downstream

- **Topic:** Market data is published on **market-data.tick** (and per-symbol patterns like `market-data.tick.{symbol}`); see **nats_adapter/src/topics.rs** and **NATS_TOPICS_REGISTRY.md**.
- **Consumers:** Strategy logic, TUI, and any client that subscribes to `market-data.tick.>` or `market-data.tick.{symbol}`.
- **Snapshot:** The shared **SystemSnapshot** holds the latest bid/ask per symbol (and related state); **snapshot_publisher** and REST/NATS snapshot endpoints expose it.

**Summary table**

| Item | Value |
|------|--------|
| Config symbols (default) | SPX, XSP, NDX |
| Contract type | Stock (STK) |
| TWS request | reqMktData equivalent via ibapi `market_data(contract).subscribe()` |
| Tick types we use | Bid, Ask, Last (Last only to fill missing bid/ask) |
| NATS topic | market-data.tick, market-data.tick.{symbol} |
| genericTickList / snapshot | Not set in our code (crate/TWS defaults) |

---

## 2. Market hours: current behaviour

- We **do not** set **useRTH** (Regular Trading Hours) anywhere in the market data subscription path. **useRTH** in the TWS API applies to **reqHistoricalData**, **reqRealTimeBars**, **reqHeadTimeStamp**, **reqHistogramData**, **reqHistoricalTicks** — not to the streaming **reqMktData** tick stream itself.
- For **reqMktData**, TWS typically sends ticks whenever the exchange produces them (pre-market, RTH, after-hours). So with current code we are effectively subscribed to **all sessions** for those symbols (whatever TWS sends).
- We do **not** read **tradingHours** or **liquidHours** from ContractDetails in the subscription path; those are metadata from contract details, not a filter on the stream.

So:

- **Understanding what we're subscribed to:** We're subscribed to **streaming stock market data (bid/ask/last)** for the configured symbols, with **no explicit session filter**; session is determined by TWS and the exchange.
- **If you need RTH-only:** Options include (a) filtering ticks by time in our code using the contract’s **tradingHours** (from reqContractDetails) or a fixed RTH window, or (b) using delayed data or historical/bar requests with useRTH=1 for RTH-only analytics, while keeping the live stream as-is for display.

---

## 3. Market hours: what the TWS API offers (reference)

From the sibling **tws-api** repo and [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_LEARNINGS_FROM_SIBLING_REPO.md):

| Concept | Where it applies | Use in our code |
|--------|-------------------|------------------|
| **tradingHours** / **liquidHours** | ContractDetails (reqContractDetails) | Not used in subscription path; could be used to know “regular session” for a symbol. |
| **useRTH** | reqHistoricalData, reqRealTimeBars, reqHeadTimeStamp, reqHistogramData, reqHistoricalTicks | We don’t use these for the live tick feed; they could be used for RTH-only historical or bar data. |
| **LAST_RTH_TRADE** | Tick type: last trade during RTH | We don’t request or handle it; could be added if we ever request generic tick list 236. |
| **outsideRth** (order) | Order placement | Allows execution outside RTH; separate from market data subscription. |

So “what we’re subscribed to” is **streaming ticks for the configured symbols, all sessions**, and “market hours” today are **not** restricted in code; they depend on TWS and the exchange. To make “market hours” explicit, we could:

- **Document or log tradingHours per symbol (from ContractDetails)** — When we have ContractDetails (e.g. from `reqContractDetails`), log or expose **tradingHours** (and optionally **liquidHours**) per symbol for operator reference (e.g. "20240614:0930-1600;20240617:0930-1600"). In any path that receives ContractDetails, log `trading_hours` for the symbol; or add a debug/admin endpoint that returns trading hours per configured symbol. See [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_LEARNINGS_FROM_SIBLING_REPO.md) §3.1 for ContractDetails fields.
- **Time filter (implemented):** Set `market_data.use_rth = true` to only forward ticks when time is within 09:30–16:00 ET (fixed window in `tws_market_data.rs`). Per-symbol tradingHours from ContractDetails are not yet used; deferred until we have tradingHours per symbol.
- Use **useRTH** on any new historical/bar requests if we add them.

**Where to implement logging:** The current market-data subscription path (`backend_service/src/tws_market_data.rs`) does **not** request ContractDetails; it builds `Contract::stock(&symbol)` and calls `client.market_data(&contract).subscribe()` only. To log tradingHours per symbol, either (a) in `run_tws_subscriptions`, call `client.contract_details(&contract)` for each symbol before or after subscribing and log `trading_hours` / `liquid_hours` from the returned details (ibapi returns a list of contract-detail structs; field names may be `trading_hours` / `liquid_hours` — see TWS_API_LEARNINGS §3.1), or (b) add a debug/admin endpoint (REST or NATS) that requests contract details for configured symbols and returns trading hours for operator reference. An existing path that already receives ContractDetails is `agents/backend/crates/tws_yield_curve/src/lib.rs` (`resolve_index_conid`, `resolve_option_contract`); that code could be extended to log tradingHours when details are received, or the market-data path can add a dedicated contract_details call for the stock symbols.

---

## 4. Quick reference: config and code

| What | Where |
|------|--------|
| Symbols list | `config/default.toml` → `[market_data]` → `symbols` |
| Provider (mock/polygon/tws) | `config/default.toml` → `[market_data]` → `provider` |
| TWS host/port/clientId | Env: TWS_HOST, TWS_PORT, TWS_CLIENT_ID |
| Subscription spawn | `backend_service/src/main.rs` → `spawn_market_data_provider` → `tws_market_data::spawn_tws_market_data` |
| Per-symbol subscribe | `backend_service/src/tws_market_data.rs` → `run_tws_subscriptions` → `Contract::stock(&symbol)`, `client.market_data(&contract).subscribe()` |
| Tick handling (Bid/Ask/Last) | `tws_market_data.rs` → `TickTypes::Price`, `TickType::Bid` / `Ask` / `Last` |
| RTH filter (optional) | `config` → `market_data.use_rth`; when true, only 09:30–16:00 ET ticks forwarded |
| NATS publish | `handle_market_event` (and nats_integration); topic `market-data.tick` |

This gives a single place to look when asking “what are we subscribed to?” and “how do market hours apply?”.
