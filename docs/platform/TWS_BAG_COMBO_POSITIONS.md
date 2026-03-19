# TWS API BAG / synthetic combo positions

**Last updated:** 2026-03-18  
**Purpose:** How TWS represents synthetic instruments (spreads, combos) and how we map them in the UI.

## TWS API behaviour

- **secType = "BAG"** is used for synthetic instruments: spreads, combos, custom multi-leg instruments.
- A BAG contract is defined with:
  - **Contract:** `secType = "BAG"`, symbol (often the primary underlying, e.g. "SPX", "WTI"), exchange (often "SMART").
  - **comboLegs:** one `ComboLeg` per component (conId, ratio, action BUY/SELL, exchange).
- Supported for option spreads, EFP, SSF (Single Stock Future) spreads, and other combos.

**References:** [Interactive Brokers GitHub](https://github.com/interactivebrokers) – synthetic symbols, BAG, ComboLeg.

## How we handle BAG in this codebase

| Layer | Behaviour |
|-------|-----------|
| **Order placement** | `ib_adapter::place_bag_order` builds a BAG contract (secType BAG, comboLegs with conId/ratio/action) and submits via ibapi. Supports box spread (4 legs); exchange defaults to SMART; paper port 7497, live trading gated by config. See [TWS_COMPLEX_ORDER_ECOSYSTEM.md](TWS_COMPLEX_ORDER_ECOSYSTEM.md). |
| **Positions** | Positions from TWS/Client Portal can be (a) one row per leg (OPT) or (b) one row per BAG contract. We accept both. |
| **Combo strategy type** | `api::combo_strategy` infers Box / Vertical from leg count and strike structure when symbols parse (e.g. `SPX 20250321C5000`). When inference fails (e.g. BAG with no per-leg symbols), we fall back to the broker `strategy` field: if it contains "box" (case-insensitive), we set `derived_strategy_type = "Box"`. |
| **Single BAG** | A single position with `position_type = "BAG"` and `strategy` containing "box" gets `derived_strategy_type = "Box"` so the TUI shows "Box" in the Type column and in combo headers. |
| **TUI** | Combo header shows "▶ Box SPX (n legs)" when derived or broker strategy indicates box. Type column shows "Box" (not "Spread") for BAG positions that are box spreads. |

## Ensuring box spreads show as "Box"

- **Broker sends `strategy`** containing "box" (e.g. "Box", "Box spread") → we use it for labelling.
- **Four OPT legs** with parseable symbols and 2 strikes / 2 calls + 2 puts → we infer Box without broker strategy.
- **One BAG position** with `strategy` containing "box" → we set derived type to Box and show "Box" in the UI.

If your box still shows as "Combo" or "Spread", either the broker is not sending a strategy with "box", or the symbol format is not parsed (we can extend the parser for more TWS formats).
