# Backlog Priority — 2026-03-24

Active backlog ordering for Aether while the product remains in
read-only exploration mode.

## Near-Term Order

1. Backlog and scope cleanup
2. Backend provider truth and market-data validation
3. TUI trust and discoverability foundation
4. Read-only TUI enrichments that improve comparison and operator understanding
5. Deferred research and execution-era tails

## What To Prioritize Now

- Task and doc cleanup that removes execution-era ambiguity
- Provider validation, yield validity, and truthful stale/waiting/error states
- TUI discoverability, selection clarity, loading feedback, and operator trust
- High-value read-only views such as Greeks, P&L breakdown, and Discount Bank data

## What To De-Emphasize

- Strategy start/stop/cancel flows
- Scenario execution UX
- OMS-style supervision or command-lifecycle work added only for disabled trading paths
- Research spikes that do not improve current operator trust or data validity

## Working Rule

If a task mixes read-only operator value with execution-era behavior, narrow it
to the read-only slice before implementation. Prefer truthful, explainable
surfaces over feature breadth.
