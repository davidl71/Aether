## tickrs real-time chart patterns (research)

### Summary
Tickrs is a ratatui-based real-time ticker with multiple chart styles/timeframes. The current research should evolve into a focused source dive of tickrs’ implementation to extract concrete patterns (chart state, update loop, data fetch model).

### Internet references (2026)
- tickrs crate summary: https://crates.io/crates/tickrs
- Ratatui rendering model: https://ratatui.rs/concepts/rendering/
- Ratatui full async events tutorial: https://ratatui.rs/tutorials/counter-async-app/full-async-events/

### Recommendation
- Follow-up task: “tickrs source dive” scoped to chart state model + refresh loop + event routing.
