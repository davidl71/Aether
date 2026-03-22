# TWS API log → Rust yield-curve mapping

This document maps raw TWS API message logs (e.g. `api1.md`) to the Rust `tws_yield_curve` crate so we can interpret logs and fix bugs (e.g. delayed data not filling bid/ask).

## Log format

- **`<-`** = client → TWS (our requests).
- **`->`** = TWS → client (responses).
- Messages are `[msg_id; req_id; ...fields...]`.

## Message ID → code path

| Log msg_id | TWS meaning | Rust code path |
|------------|-------------|----------------|
| **9** | Request contract details (reqContractDetails) | `resolve_option_contract()` → `client.contract_details(contract)`; option chain uses different flow. |
| **10** | Contract details (symbol, expiry, strike, right, exchange, conid, etc.) | Returned by `contract_details()` / option chain stream. |
| **52** | Tick size (min tick) | From `market_data().subscribe()` stream (not used for bid/ask in our code). |
| **1** (client) | Request market data (reqMktData) | `client.market_data(&contract).subscribe().await` in `get_option_bid_ask()`. |
| **81** | **Tick price, type 81** = Delayed Ask Option | Handled in `get_option_bid_ask()` — must match `TickType::DelayedAskOption` (see below). |
| **4** | Informational / warning (e.g. 10090, 10167) | “Requested market data is not subscribed. Displaying delayed market data.” — can be ignored for logic; confirms delayed feed. |

## Tick type IDs (TWS → ibapi `TickType`)

When **delayed** market data is used (`YIELD_CURVE_USE_CLOSING=1` or out-of-hours), TWS sends **delayed** tick types, not real-time. The yield-curve code must handle both.

| Tick Id | TWS name | ibapi `TickType` | Used in yield-curve for bid/ask |
|--------|-----------|-------------------|----------------------------------|
| 1 | Bid Price | `TickType::Bid` | ✓ bid |
| 2 | Ask Price | `TickType::Ask` | ✓ ask |
| 4 | Last Price | `TickType::Last` | ✓ fallback bid/ask |
| 9 | Close Price | `TickType::Close` | ✓ fallback bid/ask |
| **66** | **Delayed Bid** | `TickType::DelayedBid` | ✓ must handle |
| **67** | **Delayed Ask** | `TickType::DelayedAsk` | ✓ must handle |
| **68** | **Delayed Last** | `TickType::DelayedLast` | ✓ fallback |
| **75** | **Delayed Close** | `TickType::DelayedClose` | ✓ fallback |
| **80** | **Delayed Bid Option** | `TickType::DelayedBidOption` | ✓ must handle |
| **81** | **Delayed Ask Option** | `TickType::DelayedAskOption` | ✓ must handle (seen in api1.md) |
| **82** | **Delayed Last Option** | `TickType::DelayedLastOption` | ✓ fallback |

Reference: [TWS API tick types](https://interactivebrokers.github.io/tws-api/tick_types.html), [ibapi TickType](https://docs.rs/ibapi/latest/ibapi/contracts/tick_types/enum.TickType.html).

## api1.md example

- **Line 3–4**: Client sends contract details request `[9;8;9012;...]` for SPX OPT 20260416 6000 C → TWS replies with `[10;9012;...]` and `[52;1;9012]`.
- **Line 6–7**: Client requests market data `[1;11;9013;...]` (conid 770324907) → TWS sends **tick 81** `[81;9013;0.05;c70003;1]` (Delayed Ask Option, price 0.05).
- **Line 8–9**: TWS sends warning 10090 / 10167 (“Displaying delayed market data”).

So in delayed mode we receive **tick type 81** (and possibly 80, 66, 67, etc.). The Rust code previously only matched `Bid`/`Ask`/`Last`/`Close` (1, 2, 4, 9), so **81 was ignored** (`_ => {}`) and bid/ask never set → “no bid/ask within 12000ms”.

## Code change

In `crates/tws_yield_curve/src/lib.rs`, `get_option_bid_ask()`:

- Treat **DelayedBid** (66) and **DelayedBidOption** (80) like **Bid**.
- Treat **DelayedAsk** (67) and **DelayedAskOption** (81) like **Ask**.
- Treat **DelayedLast** (68), **DelayedClose** (75), **DelayedLastOption** (82) like **Last**/**Close** (fallback for missing bid/ask).

After this, delayed ticks from a log like api1.md will populate bid/ask and the yield-curve CLI can complete when using `YIELD_CURVE_USE_CLOSING=1` or when the market is closed.

---

## Additional findings (session 2026-03-18)

### 1. ibapi decodes tick 81 as RequestParameters

In practice, TWS sends `[81;reqId;0.05;c70003;1]` (Delayed Ask Option, price 0.05) but the ibapi crate delivers it as **`TickTypes::RequestParameters(TickRequestParameters { min_tick: 0.05, bbo_exchange: "c70003", snapshot_permissions: 1 })`**, not as `TickTypes::Price(..., DelayedAskOption, 0.05)`. So we also handle `RequestParameters` in `get_option_bid_ask()`: when `min_tick > 0`, set both bid and ask to `min_tick` so we get a quote without waiting for a separate Price tick.

### 2. Parallel fetches (one connection)

- **Per expiry:** The four legs (c_low, c_high, p_low, p_high) are requested in parallel via `tokio::join!`, so we wait at most one timeout (~12s) per expiry instead of four.
- **Across expiries:** Up to three expiries are fetched in parallel via `futures::future::join_all`, so total wait is ~12s instead of 3×12s. All subscriptions use the same TWS connection.

### 3. Session logging

- **Env:** `YIELD_CURVE_LOG_FILE=/path/to/file` — tracing logs are also written to that file (no ANSI).
- **Shell:** Redirect full session (stdout + stderr) with `... > yield_curve_session.log 2>&1` or `... | tee session.log`.

### 4. Debug logging for unparsed data

- Any tick that is not `Price` or `PriceSize` is logged as `tick_variant = ?...` so we can see Notice, Size, RequestParameters, etc.
- Price ticks with an unhandled `TickType` (e.g. DelayedHigh, DelayedLow) are logged as `tick_type = ?..., price = ...`.
- Stream errors are logged.
