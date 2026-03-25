# TUI UX Benchmarks

**Last updated:** 2026-03-24

This document compares Aether's current Rust TUI UX against the most useful
reference apps already documented in this repo. The goal is not to copy another
product's feature set. The goal is to identify terminal interaction patterns
that improve operator confidence, speed, and clarity.

## Scope

Primary Aether references:

- [TUI Audit — 2026-03-24](./TUI_AUDIT_2026_03_24.md)
- [TUI Architecture](./TUI_ARCHITECTURE.md)

Benchmark references:

- [Ticker Terminal UI Analysis](./archive/TICKER_TUI_ANALYSIS.md)
- [FinceptTerminal Financial Platform Analysis](./archive/FINCEPT_TERMINAL_ANALYSIS.md)

Lantern was checked as a possible benchmark source and was not found to be a
relevant trading TUI reference. The common Lantern projects are desktop/network
products rather than terminal-native trading applications, so they are excluded
from the benchmark set.

## Bottom Line

Aether has the strongest trading-domain surface among the documented apps:
positions, orders, alerts, yield, loans, scenarios, settings, and broker-aware
state. The current UX is still behind the best terminal references on three
practical dimensions:

1. Interaction clarity
2. Action confidence
3. Data trustworthiness

The result is that Aether looks broader than the benchmark apps, but it often
feels less reliable and less legible in use.

An additional requirement for Aether, beyond the benchmark apps, is support for
**specific synthetic instruments as chartable objects**. The UX should support
exploration of synthetic boxes and related financing instruments through
instrument-level views and trustworthy **OHLCV candle bars**, not just static
yield/spread tables.

Another Aether-specific requirement is that **configuration and daemon/service
health should be operable from the TUI**. The interface should behave like an
operator console for market state and system state together, not force routine
configuration and health inspection into external file edits or shell-only
workflows.

## Benchmark Roles

### Aether

What it is best at:

- Domain-specific workflows for synthetic financing and broker-connected state
- Multi-surface operator console behavior rather than single-purpose quote
  tracking
- Integration potential across backend, NATS, strategies, and account state

Current UX weaknesses:

- Missing keybinding discoverability and mode visibility
- Weak selection, confirmation, and feedback affordances
- Synthetic or stale data in parts of the UI that appear live
- Inconsistent empty, loading, and error states

### Ticker

Ticker is the best benchmark in this repo for compact terminal presentation and
user-configurable dashboards.

Useful patterns:

- Clean summary statistics layout
- Sorting and grouping as first-class UX controls
- Configurable color schemes
- Clear watchlist/holdings mental model
- Small-screen readability and information density

Less relevant to Aether:

- It is a tracking terminal, not an execution-focused trading workstation
- It does not cover multi-step operational flows such as order review,
  scenarios, service state, or strategy control

### Longbridge Patterns

Longbridge is not broken out in its own standalone comparison doc here, but
`TUI_ARCHITECTURE.md` already identifies the most valuable patterns to adopt.

Useful patterns:

- Dirty render flags
- Dedicated input routing instead of inline event sprawl
- Reusable scrollable table state
- Toast notifications
- Right-aligned numerics
- Conditional row styling and stronger selected-row treatment

These are not cosmetic improvements. They are terminal primitives that make
dense financial data easier to scan and safer to act on.

### FinceptTerminal

Fincept is useful as a product ambition benchmark, not as a terminal UX pattern
benchmark.

Useful lessons:

- Breadth of analytics and multi-domain workflows
- Strong product framing around intelligence, automation, and connected data

Less useful for Aether TUI interaction design:

- It is documented here as a broader desktop platform rather than a terminal
  operator console
- It is a poor source for keyboard-first terminal flow details compared with
  Ticker and Longbridge-inspired terminal patterns

## Comparison Matrix

| Dimension | Aether now | Better benchmark | Why it matters |
|-----------|------------|------------------|----------------|
| Domain-specific workflows | Strong | Aether | Yield, loans, scenarios, broker state are already the right surface |
| Keyboard discoverability | Weak | Longbridge-style patterns | Missing hints and mode labels slow expert use and confuse casual use |
| Table readability | Weak-medium | Ticker + Longbridge-style patterns | Financial terminals need alignment, contrast, and visible selection |
| Async feedback | Weak | Longbridge-style patterns | Users need loading, success, warning, and error feedback for trust |
| Data honesty | Weak in charts/alerts/ticks | Aether must improve itself | Synthetic/live ambiguity is worse than fewer features |
| Small-screen density | Medium | Ticker | Ticker is better at compact summary presentation and configurability |
| Product breadth | Strong | Fincept only as ambition benchmark | Aether already has broad operator workflows, but wiring is incomplete |

## Highest-Value UX Gaps In Aether

### 1. Discoverability Is Too Weak

From the current audit:

- Help overlay is incomplete
- Several tabs have undocumented bindings
- Split-pane mode is not explained
- Search/filter modes often have no clear state indicator

This is the first thing to fix because it lowers the value of every existing
feature.

### 2. Selection And Action Confidence Are Too Weak

Examples from the audit:

- Orders lack visible row selection
- Cancel is not wired
- Scenario execution lacks confirmation flow
- Settings and forms often fail silently or with poor feedback

In a trading terminal, an action should never feel ambiguous.

### 3. Data Trust Is Too Weak

Examples from the audit:

- Charts are synthetic
- Alerts are not fully live
- Position marks are not fully tick-driven
- Polygon bypass creates a side-channel data path in the TUI process

This is a UX issue, not just a backend issue. Users judge the interface by what
it appears to represent.

### 4. Visual Hierarchy Is Too Flat

Current weaknesses:

- Weak row highlighting
- Missing numeric alignment
- Inconsistent active-section emphasis
- Weak empty/loading/error states

This makes scan-heavy tasks slower than they should be.

## Recommended Improvement Themes

### Theme 1: Terminal Interaction Foundation

Adopt the baseline terminal primitives already identified in
[TUI Architecture](./TUI_ARCHITECTURE.md):

- dirty flags
- input routing extraction
- reusable table selection/scroll state
- toasts
- numeric alignment
- conditional row styling

This is the best leverage point because it improves every tab.

### Theme 2: Honest Real-Time UX

Do not allow parts of the UI to look real-time when they are synthetic or
stale.

Required behaviors:

- Label synthetic data when present
- Prefer blank-but-explained states over fake live states
- Wire live alerts and real chart data
- Update position display from live ticks where possible

### Theme 3: Safe Action Workflows

Trading actions need stronger interaction contracts:

- visible selection before action
- confirmation for destructive or multi-leg actions
- progress indication for async work
- result toasts and inline error detail

### Theme 4: Operator Configurability

Borrow from Ticker where it fits:

- sorting options
- grouping controls
- theme or color customization
- summary density controls

These are secondary to correctness and clarity, but they materially improve the
day-to-day experience once the fundamentals are stable.

Configuration should also converge toward a stronger product rule: routine
user-facing settings should be editable from the TUI, and daemon/service health
should be visible there with enough detail to support normal operation without
dropping to separate tooling.

## Recommended Backlog Order

1. Fix discoverability and mode visibility
2. Remove synthetic/live ambiguity and make provider truth visible
3. Improve selection, loading, and inline feedback on existing read-only tabs
4. Apply terminal table and notification primitives across tabs
5. Add user configurability such as sorting, grouping, and themes

## Non-Goals

- Rebuilding the TUI around another framework
- Copying Fincept's desktop product surface into the terminal
- Reintroducing execution-first workflows while read-only exploration mode is active
- Chasing novelty before fixing trust, clarity, and data honesty

## References

- [TUI Audit — 2026-03-24](./TUI_AUDIT_2026_03_24.md)
- [TUI Architecture](./TUI_ARCHITECTURE.md)
- [Ticker Terminal UI Analysis](./archive/TICKER_TUI_ANALYSIS.md)
- [FinceptTerminal Financial Platform Analysis](./archive/FINCEPT_TERMINAL_ANALYSIS.md)
