---
description: Run TUI wave tasks in parallel via exarp-go
agent: task
model: anthropic/claude-sonnet-4-20250514
---

Run the following TUI tasks in parallel using exarp-go task_workflow run_with_ai:

1. T-1773952110044212000 - Implement dirty flags render optimization in tui_service
2. T-1773952110096607000 - Extract TUI input handlers into src/input.rs
3. T-1773952110143329000 - Extract ScrollableTableState widget for TUI tables
4. T-1773952110189533000 - Improve TUI table rendering: right-align numerics, conditional row styles, scrollbars
5. T-1773952110237059000 - Add Toast notification pattern to TUI

For each task, call exarp-go task_workflow with action=run_with_ai and the task_id. Run all 5 calls in parallel.
