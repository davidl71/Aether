# T-1773939577056599000 Findings: ui.rs vs ui/mod.rs Dead Code

## What Was Found

### Module Resolution
In Rust, when both `ui.rs` and `ui/mod.rs` exist in the same directory:
- `mod ui;` in `main.rs` resolves to **`ui.rs`** (takes precedence)
- `ui/mod.rs` is **completely ignored** and never compiled

### Files
- **`ui.rs`** (454 lines) — **LIVE** — inline ratatui render functions for 5 tabs: Dashboard, Positions, Orders, Alerts, Logs
- **`ui/mod.rs`** (507 lines) — **DEAD** — submodule imports + delegation for all 9 tabs

### ui.rs (live) — 5 tabs
```
render_main match:
  Tab::Dashboard → inline dashboard
  Tab::Positions → inline positions  
  Tab::Orders → inline orders
  Tab::Alerts → inline alerts
  Tab::Logs → inline logs
  _ → unreachable (Yield, Loans, Scenarios, Settings)
```

### ui/mod.rs (dead) — 9 tabs
```
mod alerts, dashboard, logs, loans, orders, positions, scenarios, settings, yield_curve

render_main match:
  Tab::Dashboard → dashboard::render_dashboard
  Tab::Positions → positions::render_positions
  Tab::Orders → orders::render_orders
  Tab::Alerts → alerts::render_alerts
  Tab::Yield → yield_curve::render_yield_curve_tab
  Tab::Loans → loans::render_loans
  Tab::Scenarios → scenarios::render_scenarios
  Tab::Logs → logs::render_logs
  Tab::Settings → settings::render_settings
```

### ui/mod.rs references missing domain fields
When we swapped to test ui/mod.rs as live, it revealed the same missing fields from T-1773939573972364000:
- `metrics.tws_address` — doesn't exist in Metrics
- `position_type` — doesn't exist in RuntimePositionDto
- `strategy` — doesn't exist
- `apr_pct` — doesn't exist  
- `source` — doesn't exist

## Solution

**Two-step fix:**

### Step 1: Add missing fields to domain types (from T-1773939573972364000)
```rust
// In runtime_state.rs - RuntimePositionDto
pub struct RuntimePositionDto {
    // existing 8 fields...
    pub position_type: Option<String>,
    pub strategy: Option<String>,
    pub apr_pct: Option<f64>,
    pub source: Option<String>,
}
```

### Step 2: Merge ui/mod.rs into ui.rs
After fields exist, merge the full 9-tab implementation from `ui/mod.rs` into `ui.rs`:
1. Add submodule imports to `ui.rs` (`mod alerts;`, `mod dashboard;`, etc.)
2. Replace inline render functions with delegation to submodules  
3. Add the 4 missing tab cases (Yield, Loans, Scenarios, Settings)
4. Delete `ui/mod.rs` and all submodule files in `ui/` (dashboard.rs, etc.)

**OR** if keeping submodules: consolidate all submodules inline in `ui.rs` (no submodule files needed).

## Status: Analysis Complete
This is a **two-step refactor** blocked by T-1773939573972364000 (add missing domain fields).
