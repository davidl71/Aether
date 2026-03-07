# Financing Optimizer — Design (Phase 4)

**Task:** T-1772887222913841962 (EPIC Financing Optimizer — NLopt multi-instrument optimization)  
**Status:** Phase 4 — first C++ optimizer class wiring NLopt  
**References:** `docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md`, `native/src/convexity_calculator.cpp`, `docs/research/integration/NLOPT_INTEGRATION_GUIDE.md`

---

## 1. Objective

Minimize **effective financing cost** (blended annualized rate) across multiple instruments:

- Box spreads (synthetic financing)
- T-bills
- Bank loans
- Pension loans
- FX swaps

The optimizer chooses **allocation weights** (fractions of total notional) per instrument so that the **weighted average effective rate** is minimized, subject to constraints (e.g. weights sum to 1, optional min/max per instrument).

---

## 2. Variables and Objective

- **Variables:** \( x_i \in [0,1] \) for \( i = 0..N-1 \) (allocation weight for instrument \( i \)).
- **Objective (minimize):**  
  \( f(x) = \sum_i x_i \cdot r_i \)  
  where \( r_i \) is the effective annual rate (e.g. APR, after fees) for instrument \( i \), in decimal (e.g. 0.05 for 5%).
- **Equality constraint:** \( \sum_i x_i = 1 \).
- **Bounds:** \( 0 \le x_i \le 1 \) (and optionally instrument-specific min/max via further constraints).

This is a **linearly constrained linear objective**; NLopt handles it (e.g. SLSQP or LD_MMA). We use **NLopt C++ API** (`nlopt.hpp`), same pattern as `ConvexityCalculator::optimize_barbell_allocation`.

---

## 3. Instrument Set (Phase 4)

We support exactly five instrument slots (matching the epic description):

| Index | Instrument   | Description                    |
|-------|--------------|--------------------------------|
| 0     | Box spread   | Synthetic financing (options)  |
| 1     | T-bill       | Treasury bill / cash-like      |
| 2     | Bank loan    | Direct bank financing          |
| 3     | Pension loan | Secured pension fund loan      |
| 4     | FX swap      | Cross-currency financing       |

Each slot has:

- `effective_rate` — annualized effective cost (decimal)
- Optional `min_weight`, `max_weight` for later extensions

---

## 4. NLopt Wiring

- **Algorithm:** `nlopt::LD_SLSQP` (equality + bounds), same as convexity calculator. Alternative: `nlopt::LD_MMA` if we add many inequality constraints later.
- **Dimension:** \( n = 5 \) (one weight per instrument).
- **Objective:** C-style callback `effective_cost_objective(unsigned n, const double* x, double* grad, void* data)`; gradient is \( \nabla f = (r_0,\ldots,r_{n-1}) \).
- **Equality constraint:** `sum_weights_equality` → \( \sum_i x_i - 1 = 0 \).
- **Bounds:** `set_lower_bounds(0)`, `set_upper_bounds(1)`.
- **Tolerances:** `set_xtol_rel(1e-4)`, `set_ftol_rel(1e-6)`.
- **Initial guess:** e.g. \( x_i = 1/N \) (equal weights).

---

## 5. Result and API

- **Result struct:** `FinancingOptimizerResult` with:
  - `weights[5]` — optimal allocation per instrument
  - `effective_cost` — minimized blended rate (decimal)
  - `success` — whether NLopt converged
  - `error_message` — if failed

- **Input struct:** `FinancingOptimizerInput` with:
  - `effective_rates[5]` — \( r_i \) per instrument (decimal)
  - Optional per-instrument min/max weight (for future use)

- **Method:**  
  `FinancingOptimizerResult optimize(const FinancingOptimizerInput& input) const;`

---

## 6. Dependencies and Build

- **NLopt:** Already in build via FetchContent (`NLopt::nlopt`), used by `convexity_calculator.cpp`.
- **New files:**
  - `native/include/financing_optimizer.h` — declarations and input/result structs
  - `native/src/financing_optimizer.cpp` — NLopt objective, constraint, `optimize()` implementation
- **CMake:** Add `financing_optimizer.cpp` / `financing_optimizer.h` to main `SOURCES` / `HEADERS` and link `NLopt::nlopt` (already linked for `ib_box_spread`).

---

## 7. Extension Path (Post–Phase 4)

- **E3 (Asset Relationship Graph):** Replace fixed 5-slot input with `FinancingInstrumentRegistry` + `AssetRelationshipGraph`; optimizer then takes a list of instruments and rates from the registry.
- **Collateral / constraints:** Add inequality constraints (e.g. max allocation to one instrument, liquidity limits) and optional `CollateralValuator` integration.
- **Multi-currency:** Add FX terms to effective cost and constraints (e.g. max FX exposure) as in `SYNTHETIC_FINANCING_ARCHITECTURE.md`.

---

## 8. Testing (Future)

- Unit test: known rates → expect optimizer to put full weight on cheapest instrument (subject to constraints).
- Sanity: equal rates → equal weights; two instruments with one much cheaper → weight concentrated on cheaper one.
