# R yield curve estimation sidecar

**Plan task:** T-1774201865476785000 (deferred / low). Fits a **smooth zero curve** from sparse annualized zero rates using **base R** (`stats::spline`), with an optional path to CRAN **`termstrc`** / **`YieldCurve`** in a later iteration.

Aligned with [`docs/R_RUST_BOUNDARY.md`](../../docs/R_RUST_BOUNDARY.md): R runs out-of-process; Rust would call this via HTTP only on non-latency-critical paths.

## Contract

**POST `/estimate`** — JSON body:

| Field | Type | Required |
|-------|------|----------|
| `maturities_years` | number[] | yes — strictly positive; duplicated tenors averaged |
| `zero_rates` | number[] | yes — annualized decimals (e.g. `0.052`) |
| `output_grid` | number[] | no — points in years within `[min(T), max(T)]` |

**Response:** `method`, `grid_years`, `zero_rates`, `forward_rates` (simple discrete approximation), `data_source`, `note`.

**GET `/health`** — reports whether optional packages are installed.

## Dependencies

```r
install.packages(c("plumber", "jsonlite"))
# optional, for future NS/Svensson-style fits:
# install.packages(c("termstrc", "YieldCurve"))
```

## Run

```bash
cd analytics/r/yield_curve
Rscript run_plumber.R
# optional port: Rscript run_plumber.R 8766
```

Smoke test:

```bash
curl -s http://127.0.0.1:8765/health
curl -s -X POST http://127.0.0.1:8765/estimate \
  -H 'Content-Type: application/json' \
  -d @fixtures/sample_zeros_v1.json | head -c 400
```

## Rust integration (not implemented here)

When prioritized: `reqwest` (or similar) from `backend_service` / CLI **off event loop**, env e.g. `R_YIELD_CURVE_URL=http://127.0.0.1:8765`, map JSON response into existing `CurveResponse` / finance_rates types. Reuse pillar data exported from `tws_yield_curve` or finance_rates handlers.

## Offline (no server)

```bash
cd analytics/r/yield_curve
Rscript -e 'source("fit_term_structure.R"); str(estimate_term_structure(c(1,5,10), c(0.05,0.055,0.058)))'
```
