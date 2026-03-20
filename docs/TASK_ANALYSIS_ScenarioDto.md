# T-1773939573972364000 Findings: ScenarioDto / BoxSpreadScenario Dataflow

## What Was Found

### 1. ScenarioDto was a phantom type
- Referenced in 4 files: `snapshot_proto.rs`, `mock_data.rs`, `lib.rs`, `runtime_state.rs`
- **Never defined** — no struct with this name exists anywhere
- The `scenario_to_proto` and `scenario_from_proto` functions in `snapshot_proto.rs` referenced it but have been stubbed
- `mock_scenarios()` in `mock_data.rs` previously returned `Vec<ScenarioDto>` — now returns `Vec<String>` (stubbed)

### 2. RuntimePositionDto lacks combo detection fields
`RuntimePositionDto` (8 fields: id, symbol, quantity, cost_basis, mark, unrealized_pnl, market_value, account_id) is missing fields needed for combo grouping:
- `position_type` — OPT vs BAG vs STOCK
- `strategy` — broker-provided strategy label ("Box", "Vertical", etc.)
- `derived_strategy_type` — inferred type (Box, VerticalSpread, Unknown)
- `combo_net_bid` / `combo_net_ask` — bid/ask for the whole combo
- `combo_quote_source` — where the quote came from

`PositionSnapshot` has 7 fields (no market_value).

### 3. proto BoxSpreadScenario exists but is not wired
`proto/messages.proto` defines `BoxSpreadScenario` (symbol, strike_width, theoretical_value, estimated_net_debit, implied_apr, scenario_type). This is the canonical proto type for box spread opportunity data.

### 4. TUI Scenarios tab is dead code
- `ui.rs` (live) only renders 5 tabs: Dashboard, Positions, Orders, Alerts, Logs
- Other 4 tabs (Yield, Loans, Scenarios, Settings) are in `ui/mod.rs` (dead) but `ui/mod.rs` itself is also dead since `mod ui` resolves to `ui.rs`
- The Scenarios tab was never implemented in the live `ui.rs`

## Recommended Actions

1. **Remove `ScenarioDto` references entirely** — it was never real. The proto `BoxSpreadScenario` is the canonical type. If TUI needs scenario data, convert proto → domain directly.

2. **Add combo fields to RuntimePositionDto** for the box spread grouping workflow:
   ```rust
   pub struct RuntimePositionDto {
       // existing 8 fields...
       pub position_type: Option<String>,         // "OPT" | "BAG" | "STOCK"
       pub strategy: Option<String>,              // broker label e.g. "Box"
       pub derived_strategy_type: Option<String>, // inferred "Box" | "Vertical"
       pub combo_net_bid: Option<f64>,
       pub combo_net_ask: Option<f64>,
       pub combo_quote_source: Option<String>,
   }
   ```

3. **Wire BoxSpreadScenario proto to TUI** — if the TUI needs to show box spread scenarios, use `nats_adapter::proto::v1::BoxSpreadScenario` directly rather than creating a domain `ScenarioDto`.

4. **Implement Scenarios tab in ui.rs** or confirm it's out of scope.

## Files to Change
- `agents/backend/crates/api/src/runtime_state.rs` — add fields to RuntimePositionDto
- `agents/backend/crates/api/src/state.rs` — add fields to PositionSnapshot  
- `agents/backend/crates/api/src/combo_strategy.rs` — unstub `apply_derived_strategy_types` once fields exist
- `agents/backend/services/tui_service/src/ui.rs` — implement Scenarios tab (or remove dead ui/mod.rs)

## Status: Research Complete
This is a **dataflow gap analysis**. Implementation requires decisions on:
- Whether to add fields to RuntimePositionDto (backward compat impact)
- Whether to wire proto BoxSpreadScenario to TUI directly
- Whether to implement or remove the Scenarios tab
