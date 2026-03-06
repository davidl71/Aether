# Complex Order Book (Quoted Combos) via IB

**Can the client pull quotes from the complex order book (quoted combos from CME/CBOE)?**

---

## Short answer

**Not in a reliable, documented way.** Today the client gets **leg-level** quotes from IB and builds combo prices in-app. True **exchange complex order book** (CBOE QSB, CME quoted combos) is either not exposed by IB as a single “combo quote” or is undocumented; direct COB quotes require exchange/vendor feeds.

---

## 1. What we do today

- **Client Portal:** `GET /iserver/marketdata/snapshot?conids=...` is used with **single-contract conids** (stocks, single options). We do **not** request a combo conid for snapshot.
- **TWS API:** We request market data per leg and place **BAG combo orders** for execution. We do **not** call `reqMktData()` with a BAG contract to get a single combo quote.
- **Combo price:** Built in-app from the four leg bids/asks (see `box_spread_strategy.cpp`, `combo_detector.py`).

So today the client **does not** pull “complex order book” as a single instrument; it pulls legs and synthesizes the combo.

---

## 2. Can IB give a single “combo quote”?

### Client Portal (REST)

- Snapshot is **conid-based**. If IB assigns a **conid to a combo** (e.g. after defining a BAG via contract search/structure), you could in theory call `/iserver/marketdata/snapshot?conids=<combo_conid>`.
- IB’s Web API docs do **not** clearly state that:
  - a combo has a snapshot-able conid, or  
  - that snapshot for a combo returns the **exchange complex order book** (CBOE/CME quoted bid/ask) rather than a derived or leg-based value.
- So: **Unclear / undocumented.** You’d have to try combo conids and compare to exchange COB; risk of derived/leg-based quote rather than true COB.

### TWS API (socket)

- You can call **`reqMktData()` with a BAG contract** (combo with legs). So “can the client request market data for a combo?” → yes, at the API level.
- Our research (`docs/VIRTUAL_SECURITIES_INTEGRATION_RESEARCH.md`) notes that BAG market data **“may not provide synthetic pricing (shows individual leg prices)”**. So you may get leg-level or aggregated data, not necessarily the **exchange’s complex order book** quote.
- So: **Possible to request combo market data; not guaranteed to be the CBOE/CME complex order book.** Likely leg-based or IB-derived.

---

## 3. Where the real complex order book lives

### CBOE (e.g. SPX/SPXW)

- **CBOE Quoted Spread Book (QSB)** / Complex Order Book (COB) is served by CBOE’s **Complex PITCH/TOP** (and related) feeds.
- Access is via **CBOE market data subscription** and feed integration (EDCID, etc.), not via a generic “combo snapshot” in IB’s REST/socket API.
- IB routes orders to the exchange but does **not** document exposing QSB/COB as a first-class “combo quote” in the standard API. So “client pulls quotes from CBOE complex order book” via IB alone is **not** something we can rely on.

See: `docs/strategies/box-spread/DATA_FEEDS_BOX_SPREADS.md` (CBOE QSB, integration options).

### CME

- CME has its own market data and licensing. Quoted combos on CME would come from **CME-licensed feeds/distributors**, not automatically from IB’s generic snapshot/combo API.
- Same idea: **client cannot assume** that “pull quotes from IB” equals “pull CME complex order book.”

---

## 4. Practical recommendation

| Goal | Approach |
|------|----------|
| **Combo price for display/strategy** | Keep **leg-level** snapshot + in-app combo calculation (current design). |
| **Single “combo quote” from IB** | **Experimental:** Try TWS `reqMktData(BAG)` and/or Client Portal snapshot with a combo conid (if you can resolve one). Treat as possibly leg-derived until IB documents behavior. |
| **True CBOE/CME complex order book** | Use **exchange/vendor feeds** (e.g. CBOE Complex PITCH/TOP for QSB). Not provided by “IB client pull” in a documented way. |

So: the client **can** pull leg quotes from IB and build combos; it **cannot** rely on IB to supply the CBOE/CME complex order book as a single quoted combo in a documented, first-class way. For true COB quotes, use the relevant exchange/vendor data feeds and see `docs/strategies/box-spread/DATA_FEEDS_BOX_SPREADS.md`.
