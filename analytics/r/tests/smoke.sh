#!/usr/bin/env bash
# Smoke tests for analytics/r — no-op when Rscript missing (e.g. CI image without R).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT/analytics/r"
if ! command -v Rscript >/dev/null 2>&1; then
  echo "r-analytics-smoke: skip (Rscript not on PATH)"
  exit 0
fi
Rscript phase0_equity_metrics.R
Rscript box_spread_historical_backtest.R --no-plot
echo "r-analytics-smoke: ok"
