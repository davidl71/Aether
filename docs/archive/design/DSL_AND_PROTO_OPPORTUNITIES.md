# DSL and Protobuf: Where They Fit

**Purpose**: Clarify where a DSL adds value vs where more protobuf (or structured config) is enough, and how they work together.

---

## Current state

| Area | Today | Format |
|------|--------|--------|
| **Strategy parameters** | Numeric thresholds (min_roi, min_dte, max_spread, etc.) | JSON config → C++ `StrategyParams` / proto `StrategyParams` |
| **Box spread / financing scenarios** | Historical Python embedded DSL design | documented historical path; not part of the current repo layout |
| **Multi-asset relationships** | Design only | `docs/research/architecture/MULTI_ASSET_RELATIONSHIP_DSL_DESIGN.md` |
| **Wire format / cross-language** | Proto messages | `proto/messages.proto` — market data, positions, strategy, risk, box spread, yield curve |

---

## Where a DSL helps (and where it doesn’t)

**Use a DSL when:**

- You need **expressions or rules** that non-developers (or config) can change without code: e.g. “only trade when `roi >= 4.0 AND spread_bps < 20 AND symbol IN allowed_list`”.
- You want **declarative strategy logic**: filters, ranking, conditions, simple cash-flow or relationship rules.
- You want **one source of truth** that can generate code (C++/Rust), tests, or docs (existing DSL design).

**Don’t use a DSL for:**

- **Data shapes and wire format** — use protobuf (and optionally JSON) for messages, config DTOs, and cross-language contracts.
- **Fixed numeric knobs** — keep these as structured fields (proto or JSON); no need for an expression language.

So: **more protobuf** is the right tool for schema and messaging; **a DSL** is for expressions, rules, and declarative strategy logic.

---

## Protobuf: what to add (no DSL)

- **Already in proto**: `StrategyParams` (min_days_to_expiry, min_roi_percent, max_bid_ask_spread, etc.), box spread and yield curve messages, risk/alert/system snapshots.
- **Reasonable additions** (still data, not expressions):
  - More risk/strategy **parameter messages** if new strategies or limits appear (e.g. per-symbol overrides, time windows).
  - **Envelope or RPC payloads** for NATS/REST so every client uses the same types (Python/TS/Go generated from `proto/messages.proto`).
  - **Done:** `StrategyParams` extended; `RiskConfig`; Discount Bank (`BankAccount`, `DiscountBankBalance`, `DiscountBankTransaction`); Risk-Free Rate (`RiskFreeRatePoint`, `RiskFreeRateCurve`). C++ proto_adapter converts `config::StrategyParams` and `config::RiskConfig` to/from proto.

Keep using proto for all “structured data + cross-language” needs; no need to turn it into an expression language.

---

## DSL opportunities (beyond current Python DSL)

1. **Strategy filter / condition expressions**
   - **Problem**: Today filters are fixed fields (min_roi, min_dte, …). Changing logic means code or many new fields.
   - **Idea**: Optional **expression string** (e.g. CEL or a tiny custom DSL) evaluated over a context (symbol, roi, spread_bps, dte, …). Example: `roi_percent >= 4.0 && spread_bps < 20 && symbol in ["SPX","XSP"]`.
   - **Place**: Config (e.g. `strategy.filter_expr`) or a small “strategy rule” proto message that carries an expression string + optional params. Evaluated in C++ (CEL C++ lib), Rust, or Python (CEL/googlex) depending on where the strategy runs.
   - **Alternative**: Stay with structured params only; add more named fields when needed (simpler, no expression engine).

2. **Alert / risk rules**
   - **Problem**: “Alert when X” is often hardcoded or a fixed set of flags.
   - **Idea**: Rule DSL or expression, e.g. `net_liq < 100000 || margin_requirement > 0.8 * excess_liquidity`. Same pattern: optional expression in config or proto, evaluated at runtime.
   - **Place**: TUI/PWA alert config, or risk service config; proto can carry the rule (e.g. `AlertRule { expression = "..." }`).

3. **Extend historical DSL ideas if revived**
   - **Current**: only design notes remain in docs; no active `python/dsl/` tree exists in this repo layout.
   - **Possible**: Reintroduce a relationship/strategy DSL deliberately, with export to proto or JSON if other services need the same structure.

4. **External / code-gen DSL (later)**
   - Docs already describe external DSL → C++ generation. Only worth it if you need non-developers to edit strategy or relationship definitions in a dedicated editor; otherwise Python embedded DSL + optional expressions is enough.

---

## Recommended split

| Need | Use | Example |
|------|-----|--------|
| Message shapes, API contracts, config DTOs | **Protobuf** (and JSON for config files) | `StrategyParams`, `BoxSpreadLeg`, `RiskLimit`, `SystemSnapshot` |
| Numeric/boolean strategy knobs | **Proto or JSON** (structured fields) | min_roi_percent, max_days_to_expiry, enable_stop_loss |
| User- or config-driven conditions | **Optional expression DSL** (e.g. CEL) | `filter_expr: "roi_percent >= 4.0 && spread_bps < 20"` in config or in a proto `StrategyRule` |
| Scenario/strategy/cash-flow modeling | **Existing Python DSL** (+ optional export to proto/JSON) | BoxSpread, FinancingStrategy, CashFlowModel |
| Multi-asset relationships | **Relationship DSL** (design in docs; implement when needed) | loan → margin → box spread |

**Summary**: Use **more protobuf** for all structured data and cross-language contracts. Add a **small expression/rule DSL** (e.g. CEL) only where you want configurable conditions (strategy filters, alert/risk rules) without code changes. Keep the Python DSL for scenario and strategy modeling; proto can carry the result if other languages need it.

---

## References

- `proto/messages.proto` — current messages
- historical DSL notes in `docs/research/architecture/` — Box spread / financing / cash flow DSL
- `docs/research/architecture/DSL_ARCHITECTURE_DESIGN.md` — three-tier DSL and code gen
- `docs/research/architecture/MULTI_ASSET_RELATIONSHIP_DSL_DESIGN.md` — relationship DSL
- `config/config.example.json` — `strategy` and `risk` sections (today all numeric/structured)
