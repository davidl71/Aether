# TUI Refactoring Plan

**Status**: In progress  
**Scope**: `python/tui/` (Textual app, components, providers)

## Goals

1. **Smaller app.py** – Move tab UI into `components/` so `app.py` focuses on composition, providers, and refresh loops.
2. **Consistent tab API** – All snapshot-based tabs use the same `update_snapshot(snapshot, **kwargs)` pattern; box-spread-only tab uses `update_data(box_spread_data)`.
3. **Less duplication** – Base class or mixin for “snapshot tab” (optional snapshot, `_update_data()`).
4. **Clear data flow** – Extract box-spread fetch logic so app doesn’t mix HTTP, file read, and UI update.

## Current State

- **app.py** (~420 lines): Defines `SnapshotDisplay`, `DashboardTab`, `PositionsTab`, `OrdersTab`, `AlertsTab`, `ScenariosTab`, `TUIApp`, provider factory, and `main()`. Some tabs already live in `components/` (UnifiedPositionsTab, CashFlowTab, OpportunitySimulationTab, RelationshipVisualizationTab, LoanListTab).
- **components/**: Five files; each tab is a `Container` with `compose()`, `on_mount()`, `update_snapshot()` (or `update_data()` for ScenariosTab), and `_update_data()`.

## Refactoring Steps

### Phase 1: Move inline tabs to components (done in code)

| Class            | From     | To                          |
|------------------|----------|-----------------------------|
| SnapshotDisplay  | app.py   | components/snapshot_display.py (optional) |
| DashboardTab     | app.py   | components/dashboard.py     |
| PositionsTab     | app.py   | components/positions.py     |
| OrdersTab        | app.py   | components/orders.py        |
| AlertsTab        | app.py   | components/alerts.py        |
| ScenariosTab     | app.py   | components/scenarios.py    |

Keep `SnapshotDisplay` in app.py or move to a tiny `components/snapshot_display.py` to keep app.py readable.

### Phase 2: Optional base class for snapshot tabs

- Add `components/base.py` with `SnapshotTabBase(Container)`:
  - `snapshot: Optional[SnapshotPayload] = None`
  - `update_snapshot(self, snapshot: SnapshotPayload, **kwargs) -> None` → sets `self.snapshot`, calls `_update_data()`
  - Subclasses implement `_update_data()` and `compose()`.
- Migrate DashboardTab, PositionsTab, OrdersTab, AlertsTab to inherit from `SnapshotTabBase` (optional; reduces boilerplate).

### Phase 3: App cleanup

- **Tab actions**: `action_next_tab` / `action_prev_tab` are no-ops (Textual handles Tab binding). Remove them or implement via `TabbedContent` focus/switch so they do something visible.
- **Box spread data**: Extract `_update_box_spread_data()` into a small helper (e.g. `get_box_spread_payload(config, file_path) -> Optional[BoxSpreadPayload]`) or a dedicated “box spread data loader” used by the app; keep polling and UI update in app.
- **Historic tab**: Replace placeholder `Label("Historic Positions (coming soon)")` with a minimal `HistoricTab(Container)` that implements `update_snapshot` and shows “No historic data” (or remove tab until feature exists).

### Phase 4: Documentation and tests

- ~~Update `python/tui/README.md` and `MIGRATION_SUMMARY.md` to describe the components layout.~~ ✅ Done.
- Ensure `python/tests/test_tui_app.py` and any component tests still pass; add tests for new components if needed.

## File Layout After Refactor

```
python/tui/
├── __init__.py
├── app.py                 # TUIApp, compose, refresh, create_provider_from_config, main
├── config.py
├── models.py
├── providers.py
├── __main__.py
├── box_spread_loader.py    # get_box_spread_payload() for REST/file
├── components/
│   ├── __init__.py        # Re-export tab classes for app
│   ├── base.py            # (Optional) SnapshotTabBase
│   ├── snapshot_display.py
│   ├── dashboard.py
│   ├── positions.py
│   ├── orders.py
│   ├── alerts.py
│   ├── scenarios.py
│   ├── historic.py
│   ├── unified_positions.py
│   ├── cash_flow.py
│   ├── opportunity_simulation.py
│   ├── relationship_visualization.py
│   └── loan_entry.py
└── tests/
```

## Non-Goals (for this refactor)

- Changing Textual to another framework.
- Adding new features (e.g. IBKR REST provider, setup screen).
- C++ / pybind11 migration (documented elsewhere).

## Verification

- `uv run pytest python/tests/ python/tui/tests/ -v` passes.
- `python -m python.tui` runs with mock provider; all tabs switch and update.
- No new linter errors in `python/tui/`.
