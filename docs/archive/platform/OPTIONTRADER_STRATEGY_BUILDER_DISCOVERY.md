# OptionTrader-style strategy builder – discovery

**Last updated:** 2026-03-18  
**Purpose:** Summarize TWS OptionTrader/Strategy Builder capabilities and protocols; recommend integrate vs build-our-own for box/vertical strategies. Discovery and documentation only.

**Related:** [TWS_COMPLEX_ORDER_ECOSYSTEM.md](TWS_COMPLEX_ORDER_ECOSYSTEM.md), [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md), [IB_ADAPTER_REVIEW.md](IB_ADAPTER_REVIEW.md).

---

## 1. OptionTrader & Strategy Builder – capabilities

### 1.1 What OptionTrader is

- **TWS UI:** Standalone window (Mosaic: New Window → OptionTrader; Classic: Trading Tools → OptionTrader). Shows one underlying at a time.
- **Features:** Option chains (tabbed or list), quote panel, orders/activity/portfolio, **Strategy Builder** tab, statistics panel, configurable buttons, Model Navigator / Options Analytics integration.
- **Strategy Builder (within OptionTrader):** Build multi-leg combo orders by clicking in the option chain (bid = sell leg, ask = buy leg). Supports:
  - Adding legs from the chain (calls on left, puts on right).
  - Modifying action, last trading day, strike, put/call via dropdowns in the Combo window.
  - **Add Stock** (stock leg), **Make Delta Neutral** (hedge leg from system delta), **Add to Quote Panel**, **Transmit**).
- **Leg count:** Project reference ([TWS_COMPLEX_ORDER_ECOSYSTEM.md](TWS_COMPLEX_ORDER_ECOSYSTEM.md)) states “up to 16 legs”; IB user guides do not publish a hard maximum in the sources reviewed.

*Sources: [IBKR OptionTrader / Strategy Builder](https://www.ibkrguides.com/tws/usersguidebook/specializedorderentry/optiontrader_strategybuilder.htm), [Strategy Builder (QA)](https://qa.interactivebrokers.ca/en/software/tws/usersguidebook/mosaic/strategybuilder.htm).*

### 1.2 Protocols / APIs (programmatic vs UI)

| Layer | Description | Relevance to us |
|-------|-------------|------------------|
| **TWS UI (OptionTrader)** | GUI only; no public API to “drive” Strategy Builder from code. | UX reference only; cannot integrate the UI itself. |
| **TWS API (EClient/EWrapper)** | Contract + order API. For combos: `Contract` with `secType = "BAG"`, `comboLegs` list of `ComboLeg` (conId, ratio, action, exchange). Place via `placeOrder(orderId, contract, order)`. | **This is what we use.** Already planned in `ib_adapter`: BAG contract + ComboLegs; see [IB_ADAPTER_REVIEW.md](IB_ADAPTER_REVIEW.md), [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md). |
| **Client Portal API** | REST; snapshot/orders. Combo orders possible via equivalent structures; positions may return OPT legs rather than BAG. | Alternative to TWS API for execution; same logical model (BAG/combo legs). |
| **RTD / Complex syntax** | Excel RTD: combo defined as `cmb=conid#ratio#action#exchange;...` (legs separated by `;`). | Data/quoting in Excel; not an order-placement API for our stack. |

*Sources: [TWS API Options](https://interactivebrokers.github.io/tws-api/options.html), [TWS API ComboLeg](https://interactivebrokers.github.io/tws-api/classIBApi_1_1ComboLeg.html), [RTD Complex Syntax](https://interactivebrokers.github.io/tws-api/rtd_complex_syntax.html), [Python complex orders (IBKR Campus)](https://ibkrcampus.com/campus/trading-lessons/python-complex-orders/).*

### 1.3 What we already have in-code

- **Combo strategy type:** `api::combo_strategy` infers Box / Vertical from leg count and strikes; supports BAG and grouped OPT legs ([combo_strategy.rs](../../agents/backend/crates/api/src/combo_strategy.rs)).
- **BAG order type:** `ib_adapter::place_bag_order` builds a BAG-style request with legs; **currently a stub** (returns `Ok(0)`); wiring to ibapi is the remaining work ([IB_ADAPTER_REVIEW.md](IB_ADAPTER_REVIEW.md)).
- **Positions/TUI:** Combo view, combo key, derived strategy type for Box; see [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md).

---

## 2. Integrate vs build our own (box / vertical)

### 2.1 “Integrate” OptionTrader

- **What it could mean:** Use TWS OptionTrader/Strategy Builder as the *human* workflow (operator builds combos in TWS; we don’t replicate that UI).
- **What it cannot mean:** There is **no programmatic API to the Strategy Builder UI** (no “send strategy to OptionTrader” or “read Strategy Builder state”). So “integrate” = **rely on TWS for manual combo building and execution**; our stack would use the **same execution path** (BAG orders via TWS API or Client Portal) when we place combos from code.

### 2.2 Build our own (strategy builder UX + execution)

- **Scope for box/vertical:** We only need a small subset of OptionTrader:
  - **Box:** 4 legs (2 calls, 2 puts, 2 strikes); fixed structure.
  - **Vertical:** 2 legs (same expiry, same type, two strikes).
- **We already have:** Combo inference (Box/Vertical), BAG request shape in `ib_adapter`, and TUI combo view. **Missing:** Real BAG placement (wire `place_bag_order` to ibapi) and, if desired, a dedicated “strategy builder” UX (e.g. pick expiry/strikes → preview net price → place BAG).

### 2.3 Recommendation

| Approach | Use when | Comment |
|----------|----------|---------|
| **Rely on TWS OptionTrader for manual combos** | Operators prefer building/tweaking combos in TWS. | No code to “integrate” with the Strategy Builder UI; execution still goes through BAG via API when we send orders. |
| **Build our own (box/vertical only)** | We want structured box/vertical flows in our TUI/CLI (pick strikes/expiry → quote → place). | **Recommended for box/vertical:** Our needs are 2–4 legs and well-defined structures. Wire BAG in `ib_adapter`; add a small “strategy builder” flow in the app (e.g. box/vertical template → legs → place BAG). No need to mimic full 16-leg OptionTrader. |
| **Full 16-leg builder** | We need arbitrary multi-leg strategies (e.g. butterflies, condors) in our app. | Defer. OptionTrader UI is the right place for that today; if we ever need it in-app, we can extend our builder later. |

**Summary recommendation:** **Build our own for box and vertical only.** Use TWS API BAG + ComboLeg (already designed in `ib_adapter`); complete `place_bag_order` wiring and add a focused strategy builder UX for box/vertical. Do not attempt to integrate with the OptionTrader GUI; treat OptionTrader as the reference for behaviour and as the UI for ad‑hoc complex strategies we don’t support in-app.

---

## 3. References

- [OptionTrader / Strategy Builder (IBKR)](https://www.ibkrguides.com/tws/usersguidebook/specializedorderentry/optiontrader_strategybuilder.htm)
- [TWS API – Options](https://interactivebrokers.github.io/tws-api/options.html)
- [TWS API – ComboLeg](https://interactivebrokers.github.io/tws-api/classIBApi_1_1ComboLeg.html)
- [TWS API – RTD Complex Syntax (cmb=)](https://interactivebrokers.github.io/tws-api/rtd_complex_syntax.html)
- [IBKR Campus – Python complex orders](https://ibkrcampus.com/campus/trading-lessons/python-complex-orders/)
- In-repo: [TWS_COMPLEX_ORDER_ECOSYSTEM.md](TWS_COMPLEX_ORDER_ECOSYSTEM.md), [TWS_BAG_COMBO_POSITIONS.md](TWS_BAG_COMBO_POSITIONS.md), [IB_ADAPTER_REVIEW.md](IB_ADAPTER_REVIEW.md), [BOX_SPREAD_YIELD_CURVE_TWS.md](BOX_SPREAD_YIELD_CURVE_TWS.md)
