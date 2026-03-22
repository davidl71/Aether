# TWS complex order ecosystem (reference)

**Last updated:** 2026-03-18  
**Purpose:** Reference for TWS/IB complex-order features (COB, COA, NSM, strategy builder) when designing or integrating box spreads and multi-leg strategies.

## TWS complex order concepts

| Concept | Description |
|--------|-------------|
| **Complex Order Book (COB)** | Resting multi-leg orders in a separate book from single-leg books; complex orders sit in COB until execution. |
| **OptionTrader Strategy Builder** | Build, quote, and manage multi-leg strategies (up to 16 legs). |
| **Legging in** | Complex orders can execute against individual leg liquidity when net price matches. |
| **Spread bid/ask quote** | Net debit/credit for the combo (e.g. butterfly $4.32 bid). |
| **COA (Complex Order Auction)** | Auction for marketable complex orders to seek price improvement. |
| **NSM (National Spread Market)** | Uses NBBO across exchanges to compute best net price for a complex order. |
| **Performance profile** | Risk/return and greeks on the complex quote. |

## COB, COA, NSM and BAG order flow

### COB (Complex Order Book)

- **What it is:** A separate order book for multi-leg (complex) orders. Resting complex orders sit in the COB rather than in single-leg option books.
- **Relation to BAG:** When we place a **BAG** (combo) order, the broker can route it to the exchange’s complex order book. The BAG contract (secType BAG, comboLegs) is the same representation used for COB; whether an order rests in COB depends on exchange, product, and routing (e.g. SMART vs direct to an exchange that supports COB).
- **In this codebase:** We do not currently request COB quotes as a single instrument; we build net price from leg-level data (see [IBKR_COMPLEX_ORDER_BOOK_QUOTES.md](../IBKR_COMPLEX_ORDER_BOOK_QUOTES.md)). COB is the destination book for live BAG order placement when that is implemented.

### COA (Complex Order Auction)

- **What it is:** An auction mechanism for **marketable** complex orders (e.g. buy at or above the current best ask). The order is exposed briefly for price improvement before execution.
- **Relation to BAG:** A marketable BAG order can be sent to COA when the exchange supports it (e.g. CBOE), so the order may get a better net price than the current spread.
- **Integration point:** When extending BAG order placement, routing can optionally prefer COA (e.g. via order type or exchange/routing parameter) so marketable combos participate in the auction. Current `place_bag_order` uses limit orders and does not set COA-specific routing.

### NSM (National Spread Market)

- **What it is:** Aggregation of NBBO across exchanges to derive the best net price for a complex strategy. Used for quoting and price discovery of the spread as a whole.
- **Relation to BAG:** NSM provides the “best” composite bid/ask for the combo; relevant when displaying spread quotes or when routing BAG orders to achieve best execution.
- **Integration point:** Optional use of NSM-derived prices for limit prices or for display (e.g. “national best” combo bid/ask). Today we synthesize combo price from leg NBBOs in-app; an NSM integration would consume exchange/vendor NSM data when available.

### Flow summary (BAG → COB/COA/NSM)

| Stage | Role of COB / COA / NSM |
|-------|--------------------------|
| **Quote / display** | NSM (or leg-based synthesis) → net bid/ask for the combo. |
| **Order placement** | BAG order sent to TWS; routing may send marketable orders to COA, resting orders to COB (exchange-dependent). |
| **Resting** | Orders rest in COB until filled or legged against single-leg liquidity. |
| **Execution** | Fill can occur in COB (match with contra complex order) or via legging (net price matches leg markets). |

### Optional stub flags for COA/NSM (docs only)

When implementing BAG order placement and/or combo quoting, the following configuration flags can be considered (no implementation in this task; documentation only):

| Flag (suggested name) | Purpose | Default |
|-----------------------|---------|--------|
| `use_coa` (or `prefer_complex_order_auction`) | When true, route marketable BAG orders to Complex Order Auction where supported (e.g. CBOE). | `false` (or broker default) |
| `use_nsm` (or `prefer_nsm_quotes`) | When true, use NSM-derived combo quotes for display or limit price when available; otherwise use leg-based synthesis. | `false` (leg-based only) |

These are integration points only; actual behaviour depends on TWS/IB API parameters (e.g. order type, exchange, routing) and exchange support for COA/NSM on the given product.

## Scanners and tools

### Complex Orders Scanner and Predefined Scanner (reference)

- **Complex Orders and Trades Scanner** – TWS UI tool that lists complex combination strategies (spreads, butterflies, etc.) by underlying. It returns strategies that have a native bid/ask quote or have traded that day; results can be sorted by volume. In TWS Mosaic: New Window → Complex Orders and Trades Scanner. In Classic TWS: Analytical Tools → Complex Orders and Trades Scanner. The underlying scan type is exposed in the API (see below).
- **Predefined Scanner** – TWS category of built-in scans that filter for active strategy combinations, high option volume, top gainers, and similar criteria. Various predefined scan types exist; complex-order–related scans (e.g. “Complex Orders and Trades”) are a subset. Scan codes and filter parameters can be discovered via the API.

**TWS API exposure of scanner results:** The TWS API **does** expose scanner results:
- **reqScannerSubscription** (EClient) – Subscribe to a scan using a `ScannerSubscription` (e.g. `ScanCode`, `Instrument`, `LocationCode`, and filters such as `AbovePrice`, `AboveVolume`). TWS v973+ supports additional filters via a TagValue list.
- **scannerData** / **scannerDataEnd** (EWrapper) – Receive results. Each result is a contract (e.g. BAG or leg) with rank, distance, benchmark, projection, and legs string. No market data (bid/ask/volume) is included in the scanner payload; request market data separately if needed.
- **cancelScannerSubscription** – Stop updates for a given request id.
- **reqScannerParameters** – Retrieve available scan codes and filter parameters (e.g. to get the exact code for “Complex Orders and Trades” or other predefined scans).

Limits: max 50 results per scan code, and at most 10 API scanner subscriptions active at once. See [TWS API Market Scanners](https://interactivebrokers.github.io/tws-api/market_scanners.html) and [ScannerSubscription](https://interactivebrokers.github.io/tws-api/classIBApi_1_1ScannerSubscription.html).

### Other TWS tools (reference)

- **Option Strategy Lab** – Generates strategies from price forecasts.
- **BookTrader** – 1-click execution and order book visualization.
- **Risk Navigator** – Real-time risk for complex portfolios.
- **Order presets** – Default strategies per ticker.
- **Bracket orders** – Attached stop-loss / profit-taking.

*Sources: FlexTrade, Interactive Brokers (TWS) documentation, [TWS API Market Scanners](https://interactivebrokers.github.io/tws-api/market_scanners.html), [IBKR API scanners](https://www.interactivebrokers.com/campus/ibkr-quant-news/ibkr-api-scanners/).*

## Relation to this codebase

- **Positions:** We represent multi-leg positions as BAG or grouped OPT legs; see [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md).
- **Orders:** BAG (combo) order placement is implemented in `ib_adapter::place_bag_order` (secType BAG, comboLegs, paper port 7497, live gated). See [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md). COB/COA/NSM remain integration points for routing and quoting.
- **Quoting:** Box spread net debit/credit and strategy builder behaviour are relevant for yield-curve and scenario UX; see [BOX_SPREAD_YIELD_CURVE_TWS.md](BOX_SPREAD_YIELD_CURVE_TWS.md) for TWS-sourced curve context.

### Box spread net debit/credit (spread bid/ask)

We surface **combo net bid/ask** (spread quote) for box spreads in the API and snapshot so the UI can show net debit/credit:

| Data source | Description |
|-------------|-------------|
| **leg_sum_mark** | Computed from leg marks: `sum(leg.mark * leg.quantity)` per combo group. Set by `api::combo_strategy::apply_derived_strategy_types` when positions are grouped and inferred as Box. |
| **leg_sum** | Reserved for when we have per-leg bid/ask (sum of leg bid/ask with correct sign per ratio). |
| **tws** | Reserved for a future TWS combo quote (e.g. COB/NSM net quote) when available from the API. |

**Exposure:** Each position DTO in the snapshot and REST payload includes optional `combo_net_bid`, `combo_net_ask`, and `combo_quote_source`. For box groups these are set on every leg in the group (same values). Proto: `Position.combo_net_bid`, `combo_net_ask`, `combo_quote_source`.

Use this doc as a checklist when adding complex order quoting, COB integration, or strategy builder–style flows.
