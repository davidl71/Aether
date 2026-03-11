# Feature Tracking: Historical TUI vs Web

> Historical note
>
> As of March 11, 2026, the React web frontend is retired from the active runtime
> surface. The current supported frontends are the Rust TUI and the native CLI.
> This document compares the older Python/Textual TUI against the retired web app
> for historical parity context only.

## Historical Status Labels

- `Shared` — implemented on both frontends when web was still active
- `TUI-only` — intentionally terminal-only or only implemented in the Textual TUI
- `Web-only` — intentionally browser-only or only implemented in the web app
- `Partial` — exists on both sides but differs materially
- `Missing` — expected for parity but absent on one side

## Historical Scope

The Python/Textual TUI and the React web app consume the same platform concepts:

- snapshot/status data
- positions, historic positions, orders, alerts
- scenario and benchmark data
- service/back-end health

Parity does not mean identical interaction design. The TUI is keyboard-first and operations-heavy. The web app is browser/PWA-oriented and favors richer panels, charts, and pointer-driven navigation.

## Current Matrix

### Shared data and status

| Capability | TUI | Web | Status | Notes |
|------------|-----|-----|--------|-------|
| Shared snapshot model | Yes | Yes | Shared | Both are documented as using the same snapshot schema family. |
| Provider/backend health display | Yes | Yes | Partial | Both surface backend status, but the TUI has a denser operator-focused status bar. |
| Mode/account/strategy status | Yes | Yes | Shared | Present in the TUI status bar and web header/status flows. |
| REST-backed live data | Yes | Yes | Shared | Both support live snapshot/API-backed usage. |
| Static/sample fallback flows | Yes | Yes | Shared | TUI has mock/file/cache flows; web has static JSON/PWA fallback. |

### Core trading views

| Capability | TUI | Web | Status | Notes |
|------------|-----|-----|--------|-------|
| Dashboard / symbol overview | Yes | Yes | Shared | Both expose symbol-level overview and selection. |
| Current positions | Yes | Yes | Shared | Both show active positions. |
| Historic positions | Yes | Yes | Shared | Both expose historic/closed positions. |
| Orders | Yes | Yes | Shared | Both show order history/timeline. |
| Alerts | Yes | Yes | Shared | Both show alert feeds. |
| Box spread scenarios table | Yes | Yes | Shared | TUI has a scenarios tab; web has scenario summary + table. |
| Scenario summary statistics | Limited | Yes | Partial | Web is richer here today. |

### Banking / financing views

| Capability | TUI | Web | Status | Notes |
|------------|-----|-----|--------|-------|
| Bank accounts display | Yes | Yes | Shared | Both consume bank account data, but in different layouts. |
| Unified positions including bank accounts | Yes | Yes | Shared | TUI uses `UnifiedPositionsTab`; web uses `UnifiedPositionsPanel`. |
| Cash flow projection | Yes | Yes | Shared | Present in TUI and web panels. |
| Opportunity simulation | Yes | Yes | Shared | Present in both, but not exposed in the same top-level navigation model. |
| Relationship visualization | Yes | Yes | Shared | Present in both with different presentation styles. |
| Loan management / loan import UI | Yes | No | TUI-only | TUI has loan entry/list management; web does not expose equivalent UI. |

### Benchmarks, setup, logs, and operator tooling

| Capability | TUI | Web | Status | Notes |
|------------|-----|-----|--------|-------|
| Benchmarks / rates view | Yes | Yes | Shared | TUI has a benchmarks tab; web has rates/benchmarks panels. |
| Setup / provider switching UI | Yes | Limited | Partial | TUI has a dedicated setup screen; web relies more on env/config and service panels. |
| Application logs view | Yes | Limited | Partial | TUI has a dedicated logs tab; web has service/log guidance and limited log surfacing. |
| Export flows | Yes | Limited | Partial | TUI has explicit export actions; web parity is weaker and should be tracked separately if made first-class. |
| QA screenshot harness | Yes | No | TUI-only | TUI has an automated screenshot harness; web does not have an equivalent documented QA capture flow. |

### Interaction model differences

| Capability | TUI | Web | Status | Notes |
|------------|-----|-----|--------|-------|
| Keyboard-first navigation | Yes | Limited | TUI-only | TUI is designed around bindings and function keys. |
| Mouse/touch-first navigation | No | Yes | Web-only | Browser/PWA interaction model. |
| Installable PWA behavior | No | Yes | Web-only | This is specific to the browser client. |
| Terminal operator footer/help | Yes | No | TUI-only | TUI-specific affordance. |

## Historical Gaps Worth Tracking

### Former web gaps relative to TUI

- loan management and import workflows
- richer operator setup/provider controls
- dedicated logs/operator diagnostics surface
- some export/operator workflows

### Former TUI gaps relative to web

- richer scenario summary presentation
- browser/PWA-specific install/offline UX
- broader pointer/touch-oriented interaction patterns

### Intentional differences

- TUI remains the operator-focused terminal surface.
- The retired web client was the browser/PWA surface and could keep browser-native capabilities
  that did not need TUI equivalents.

## Historical Maintenance Rules

This matrix is no longer an active maintenance target while the web frontend is retired.
Keep it only as reference if the browser client is revived later.

## Related Docs

- [python/tui/README.md](../python/tui/README.md)
- [web/README.md](../web/README.md)
- [agents/shared/API_CONTRACT.md](../agents/shared/API_CONTRACT.md)
- [AGENTS.md](../AGENTS.md)

**Last Updated**: 2026-03-10
