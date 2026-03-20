---
description: Check status of TUI wave tasks
agent: task
---

List the status of the 5 TUI wave tasks using exarp-go task_workflow:

1. T-1773952110044212000 - Implement dirty flags render optimization in tui_service
2. T-1773952110096607000 - Extract TUI input handlers into src/input.rs
3. T-1773952110143329000 - Extract ScrollableTableState widget for TUI tables
4. T-1773952110189533000 - Improve TUI table rendering: right-align numerics, conditional row styles, scrollbars
5. T-1773952110237059000 - Add Toast notification pattern to TUI

Call exarp-go task_workflow for each task ID to get their current status. Run all calls in parallel.
