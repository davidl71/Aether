# TUI Dashboard Design (GNU top inspired)

## Goals
- Provide a live, terminal-native view of account/strategy state with minimal keystrokes.
- Mimic `top`/`htop` layout for familiarity while incorporating ideas from `icli`.
- Support quick actions, including entering offsetting box spreads to improve cost basis.

## Layout Overview
```
┌──────────────────────────────────────────────────────────────┐
│ IB BOX SPREAD TERMINAL                Time: 10:24:31          │
│ Mode: DRY-RUN │ Strategy: RUNNING │ Account: DU123456        │
│ TWS: OK │ ORATS: Enabled │ Portal: OK │ QuestDB: OK           │
├──────── Tabs ────────────────────────────────────────────────┤
│ Dashboard │ Current Positions │ Historic Positions │ Orders │ Alerts │
├──────────────────────────────────────────────────────────────┤
│ (Dashboard metrics / symbol table)                           │
├───────────────────────┬──────────────────────────────────────┤
│ Active Positions      │ Recent Orders                        │
└───────────────────────┴──────────────────────────────────────┘
│ Alerts                                                    ↑ │
└──────────────────────────────────────────────────────────────┘
```

### Tabs
- **Dashboard**: symbols, profitability metrics, subsystem status.
- **Current Positions**: open box spreads with ROI, maker/taker counts, rebate estimates.
- **Historic Positions**: recently closed spreads (sources: QuestDB + strategy history).
- **Orders**: timeline of live/past orders.
- **Alerts**: scrollable notification feed.

## Key Metrics & Color Scheme
- Headers and healthy statuses: cyan/green.
- ROI, net liquidity, buying power: green for positive, red for negative.
- Maker/taker counts: maker in cyan, taker in magenta.
- Alerts: info blue, warnings yellow, errors red.

## Controls & Shortcuts
| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Cycle focus across tabs |
| Arrow keys | Scroll within lists/panels |
| `Enter` | Open detail popup for selected row |
| `S` | Start (or resume) strategy |
| `T` | Stop strategy gracefully |
| `K` | Cancel all open strategy orders |
| `D` | Toggle dry-run mode |
| `B` | **Buy combo**: prompts for target venue/price, submits additional box spread to lower cost basis |
| `S` (with shift or confirmation) | **Sell combo**: submits offsetting spread to realize gains or flatten |
| `Q` | Quit TUI |

> **Cost Basis Improvement**: The `B`/`Shift+S` quick keys fire prefilled combo orders (matching the currently highlighted spread or symbol). They auto-populate a limit price based on the latest combo quote (with configurable slippage) and allow rapid maker entries to improve average cost.

## Data Sources
- REST endpoints shared with iPad/web frontends (strategy status, positions, orders, alerts).
- Combo quotes from IB combo market data (when enabled).
- QuestDB history for archived positions.
- Mock TWS service for offline testing.

## Implementation Notes
- Language: Go (`tcell` + `tview`) for performance and layout control.
- Modular views to keep panels reusable.
- Config-driven palette to allow monochrome fallback.
- Background goroutines poll REST endpoints or subscribe to an optional WebSocket for push updates.

