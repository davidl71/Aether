## Risk metrics returns sourcing (VaR/CVaR): options + recommended MVP

### Local building blocks already in repo
- `api::quant::RiskMetricsRequest { returns: Vec<f64>, confidence: Option<f64> }` and `RiskMetricsResponse { var_95, cvar_95, max_loss }` live in `agents/backend/crates/api/src/quant.rs`.
- Backend NATS handler exists: `services/backend_service/src/handlers/calculate.rs` subscribes to `api.calculate.risk_metrics` and calls `api::quant::calculate_risk_metrics`.
- Quant crate already computes simple returns from prices for HV:
  - `crates/quant/src/lib.rs` builds `returns: Vec<f64> = prices.windows(2).map(|w| (w[1]-w[0])/w[0])...`
- Risk crate has additional VaR helpers (`crates/risk/src/var.rs`) but current NATS/API path uses `api::quant`.

### Candidate sources for `returns`
- **A. Chart price history (per-symbol, TUI-side compute)**
  - When the user is already viewing a symbol chart, compute returns from its stored price history and call `api.calculate.risk_metrics`.
  - Pros: minimal backend work; immediate UI value; no new persistence.
  - Cons: per-symbol only; depends on chart history availability.

- **B. Backend-computed returns from market-data history (per-symbol)**
  - Backend fetches price series (from whatever historical store exists / provider) and computes returns.
  - Pros: consistent; reusable by CLI/TUI.
  - Cons: requires deciding/implementing a history source.

- **C. Portfolio PnL / equity curve (portfolio-level)**
  - Compute returns on a portfolio equity curve derived from ledger or positions snapshots.
  - Pros: more meaningful risk measure.
  - Cons: needs an actual equity curve definition and sampling policy.

### Recommended MVP
- Start with **A (chart price history)**: the TUI already has a chart-focused workflow; we can surface VaR/CVaR contextually without committing to a global returns store.

### Follow-up implementation tasks (suggested)
- Add a small TUI panel (or detail overlay) on Charts tab: compute returns from displayed price series, call risk-metrics NATS, render `var_95/cvar_95/max_loss` + confidence + sample size.
- Later: decide whether to formalize B or C as a product surface.
