# Rust ↔ R integration boundary (Aether)

This document time-boxes the **Phase 5** research outcome from the backlog plan: where R fits relative to the Rust workspace (`agents/backend/`) without blocking core NATS/TUI work.

## Recommended boundary

1. **Rust remains the system of record** for snapshots, NATS subjects, credentials, and anything that must stay in the active trading/exploration path.
2. **R is an optional analytics sidecar**: batch or on-demand jobs that consume **exported artifacts** (CSV, Parquet, or JSON) or call a **narrow HTTP API** served by Rust (or a tiny static file + `plumber`/`plumbed` only if you already standardize on R for a given notebook).
3. **Do not embed R inside `tui_service` or `backend_service` event loops**; keep IPC explicit (files, HTTP, or a job queue) so failures in R never stall the operator console.

## Concrete patterns (pick one per use case)

| Pattern | When to use | Notes |
|--------|-------------|--------|
| **Export + R script** | Backtests, research, ad-hoc curves | Rust CLI or admin endpoint writes time-stamped files; R reads locally or in CI. |
| **HTTP + plumber (or similar)** | Interactive notebooks needing live-ish params | Run R as a separate process; Rust calls it only from non-latency-critical paths. |
| **Batch pipeline** | Nightly reports | `cargo run -p cli …` → files → `Rscript analysis.R`. |

## Out of scope (until product direction changes)

- In-process R (e.g. `extendr`) inside hot request paths.
- R as the primary market-data or health transport.

## Next implementation steps (when prioritized)

1. Define one **stable export schema** (columns + units) for positions, curves, or scenario grids.
2. Add a **single** Rust entry point (CLI flag or gated handler) that writes that schema.
3. Version the schema (`v1`) so R scripts do not break silently when Rust fields evolve.

**Phase 0 (R sidecar scaffold):** `analytics/r/` — `Rscript phase0_equity_metrics.R` reads CSV bars (`date`, `close`) and prints Sharpe, drawdown, and total return; `Rscript box_spread_historical_backtest.R` reads box MTM marks (`date`, `box_mtm`) for cumulative P&L / Sharpe / max DD, with optional `ggplot2`, `plotly`, and `quantstrat` (see `analytics/r/README.md`). Sample data under `analytics/r/fixtures/`. Richer QuestDB option-chain replay stays future export-schema work.

**Yield curve (deferred):** `analytics/r/yield_curve/` — `plumber` HTTP API (`POST /estimate`) smooths sparse zero rates (base R spline); optional CRAN `termstrc` / `YieldCurve` wired later (T-1774201865476785000). Rust integration remains HTTP-only off hot paths.

## Runnable spike

- **`analytics/r/`**: example CSV/JSON fixtures and `scripts/sanity_box_spread.R` (base R for CSV; optional `jsonlite` for JSON). See `analytics/r/README.md`.
