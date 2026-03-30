## VaR/CVaR risk metrics (backend exists; TUI needs sourcing + surface)

### Summary
Backend supports `api.calculate.risk_metrics` via NATS request/reply and uses `api::quant::calculate_risk_metrics`. Missing pieces are:
- defining where `returns: Vec<f64>` comes from in Aether,
- a minimal TUI surface to show `var_95`, `cvar_95`, `max_loss` along with assumptions.

### Local findings
- Handler: `agents/backend/services/backend_service/src/handlers/calculate.rs` (topic `api.calculate.risk_metrics`)
- Types + calculation: `agents/backend/crates/api/src/quant.rs` (`RiskMetricsRequest`, `RiskMetricsResponse`)

### Internet references (2026)
- Risk API shapes (example): https://docs.rs/risk-metrics/latest/risk_metrics/
- VaR/CVaR concept presence in a Monte Carlo crate: https://docs.rs/aprender-monte-carlo/latest/aprender_monte_carlo/all.html

### Recommendation
- Add a focused “returns sourcing” design task: choose a source (price series, portfolio PnL, etc).
- Then add a focused TUI view: show metrics + confidence + sample size, with a short assumptions blurb.
