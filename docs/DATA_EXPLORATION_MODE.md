# Data Exploration Mode

**Status:** Active product direction as of 2026-03-24

## Intent

Aether is currently operating as a **read-only relative-value exploration
console**.

The near-term product goal is to make market data, yield curves, positions,
comparative analytics, and operator trust **valid and explainable** before
reintroducing any execution workflows.

This mode is not a temporary UI toggle. It is the default architectural and
product assumption for current work unless a later decision explicitly reopens
execution.

## What Stays In Scope

- Real positions, balances, orders, and account state may remain visible for
  inspection
- Market-data ingestion, provider validation, and candle/OHLCV truth paths
- Yield-curve construction, benchmark comparison, and financing parity views
- Cross-instrument comparison across T-bills, bonds, synthetic boxes, ETFs,
  and bank loans
- TUI configuration, daemon health, and operator-facing diagnostics
- Explicit stale, unavailable, and degraded-data states

## What Is Out Of Scope For Now

- Strategy start/stop flows
- Scenario execution
- Order placement, cancel, or cancel-all workflows
- New command-lifecycle or execution-status infrastructure built only to
  support trading controls
- UI affordances that imply execution is active when it is intentionally
  deprecated

## Product And UX Rules

- Prefer wording such as `exploration`, `comparison`, `validation`,
  `market-data truth`, and `yield validity`.
- Do not present Aether as a generic trading terminal.
- Do not report synthetic or fallback data as if it were live.
- If data is missing, stale, provider-specific, or inferred, label it
  explicitly in the TUI and docs.
- Keep real positions visible when they help explain financing exposure or
  provider parity.

## Architecture Implications

The current direction does **not** require a major Aether architecture rewrite.

It does require a clearer boundary:

- **Data plane:** provider ingestion, normalization, snapshots, NATS subjects,
  analytics, yield curves, and read models
- **Execution plane:** deprecated for now and kept behind explicit capability
  boundaries

Practical implications:

- Avoid expanding command/execution subsystems while execution is off
- Keep execution endpoints and TUI controls disabled or visibly deprecated
- Prioritize provider- and snapshot-truth work over OMS-like infrastructure
- Treat external integrations skeptically; add providers only when they improve
  data validity or operator understanding

## Dependency Guidance

When choosing dependencies or integrations:

- Minimize external includes and new vendors unless they materially improve
  market-data or yield-curve validity
- Prefer existing Rust workspace crates and current provider paths over new
  abstraction experiments
- Remove or de-emphasize integration layers that only support disabled
  execution flows

## Backlog Guidance

Bias the active backlog toward:

- market-data truth
- yield-curve validity
- provider audit and reconciliation
- TUI trust, visibility, and comparative analytics
- docs/runbooks that reflect read-only behavior

Defer or cancel backlog items whose only value is execution robustness while
execution remains deprecated.
