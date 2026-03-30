# R analytics sidecar (Aether)

**Plan task:** T-1774201866539327000 — historical backtesting MVP in-repo: `box_spread_historical_backtest.R` (MTM CSV → P&L, Sharpe, max drawdown; optional `ggplot2` / `plotly` / `quantstrat`). Full QuestDB option-chain replay remains a later export-contract exercise.

This folder implements offline **export + `Rscript`** analytics, following [`docs/R_RUST_BOUNDARY.md`](../docs/R_RUST_BOUNDARY.md) (no embedding in Rust hot paths).

## Phase map

| Phase | Input | Output | Status |
|-------|--------|--------|--------|
| 0 | `fixtures/sample_equity_bars_v1.csv` or Rust-exported bars with `date,close` | Sharpe (log), vol, max drawdown, total return | **Scaffold present** |
| Yield curve | Sparse zero pillars (JSON) via **plumber** | Smooth zero/forward grid (`yield_curve/`) | **Scaffold present** — T-1774201865476785000 |
| Box MTM backtest | `fixtures/sample_box_mtm_v1.csv` or export with `date,box_mtm` | Cumulative P&L, Sharpe, max DD; optional PNG/HTML; optional **quantstrat** (`AETHER_QUANTSTRAT=1`) | **MVP present** — T-1774201866539327000 |
| 1 | Parquet/CSV exported from QuestDB (quotes or consolidated underlyings) | Same metrics + alignment checks | Not started |
| 2 | Full multi-leg / chain replay | Richer exposure attribution | Future |

See **`yield_curve/README.md`** for term-structure sidecar (R `termstrc` / `YieldCurve` optional follow-up).

## Run (requires R)

```bash
cd analytics/r
Rscript phase0_equity_metrics.R
Rscript phase0_equity_metrics.R /path/to/your/bars.csv

Rscript box_spread_historical_backtest.R
Rscript box_spread_historical_backtest.R /path/to/box_mtm.csv --plot=/tmp/mtm.png
# Optional plotly: --plotly=/tmp/mtm.html (needs plotly + htmlwidgets)
# Optional quantstrat: AETHER_QUANTSTRAT=1 Rscript box_spread_historical_backtest.R
```

From repo root (if `Rscript` is on `PATH`):

```bash
just r-analytics-smoke
```

### `box_spread_historical_backtest.R` contract

- **`date`**: calendar date
- **`box_mtm`**: daily mark-to-market in USD for **one** static long box (exported series; source may be QuestDB or synthetic)

Expected columns for phase0: **`date`** (ISO), **`close`** (numeric). Optional columns (e.g. `symbol`) are ignored.

## Data contract (v1 bars for equity-like series)

- `date`: calendar date
- `close`: mark price for that date (synthetic or last print)

Version this contract when Rust export lands; bump folder or filename (`bars_v2`) per [`docs/R_RUST_BOUNDARY.md`](../docs/R_RUST_BOUNDARY.md).
