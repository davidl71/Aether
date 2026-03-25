# Provider Validity Review — 2026-03-24

This review captures the current backend-owned market-data truth path for
Aether while the product remains in read-only exploration mode.

## Current Valid Path

- The supported TWS provider path is `ib_adapter`.
- The backend owns quote resolution through `MarketDataAggregator`.
- The TUI no longer uses a direct Polygon side channel; it consumes only
  backend-published `market-data.tick.*`, `market-data.candle.*`, and
  `system.alerts`.

## Source Priority Rules

Provider priority is currently defined in
`agents/backend/crates/market_data/src/aggregator/mod.rs`.

- `nautilus` = 100
- `tws` = 100
- `polygon` = 70
- `fmp` = 60
- `yahoo` = 50
- `mock` = 0

`MarketDataAggregator::process_event()` keeps the highest-priority quote for a
symbol, and on equal priority prefers the fresher timestamp. Quotes become
stale after the configured TTL, which is why the TUI can correctly show waiting
or stale states even when a provider is configured.

## What The Operator Can Trust Today

- The TUI status bar displays the winning live source as `SOURCE@PRIORITY`
  plus age, derived from live tick events.
- Snapshot market-data source and live tick metadata now come from the backend
  path, not local TUI provider code.
- Charts explicitly refuse to fabricate synthetic candles when no live history
  is available.
- Alerts can arrive live over `system.alerts` and be rendered from the active
  snapshot state.

## Remaining Gaps

- Provider validity is visible, but not yet fully explainable, in the TUI.
  Operators can see the winning provider and age, but not why a lower-priority
  provider lost or whether a source was rejected for quality reasons.
- The backend still emits some strategy/execution-oriented comments and logs in
  active code paths, which can weaken operator trust in read-only mode.
- Yield-curve live validation still requires a running backend/provider stack;
  it is not verifiable from static repo inspection alone.

## Recommended Next Steps

- Keep provider-truth work focused on operator explainability: source,
  priority, age, stale status, and degraded-provider visibility.
- Add or preserve regression coverage for waiting/stale/no-synthetic chart
  states and live alerts.
- Perform live yield validation only against an actual running backend with the
  intended providers enabled; do not infer that result from static code alone.
