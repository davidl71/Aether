# TUI Order detail overlay – manual verification

**Purpose:** Manually verify that opening an order from the Orders tab shows the detail overlay with correct data (ID, Symbol, Side, Qty, Status, Submitted). No automated test; human QA only.

**Reference:** Order overlay is rendered in `tui_service/src/ui/mod.rs` (`render_detail_overlay`, `DetailPopupContent::Order`). Orders tab and Enter key handling in `tui_service/src/app.rs`.

---

## Prerequisites

1. **NATS** running (e.g. `nats-server -js -DV` or `just services-active-start`).
2. **backend_service** running and publishing snapshot to NATS (`snapshot.{BACKEND_ID}`).
3. Snapshot must contain at least one order. With default/mock backend, `api/mock_data` seeds orders when snapshot is empty; ensure backend has started and published at least one snapshot so TUI receives orders.

---

## Verification steps

1. **Start backend and TUI**
   - From repo root: `just services-active-start` (or start NATS and backend in separate terminals).
   - Start TUI: `cargo run -p tui_service` from `agents/backend` (or `just run-tui` if configured).

2. **Ensure orders in snapshot**
   - If using backend with mock data, orders are seeded automatically.
   - Confirm status bar shows a data source (e.g. `[NATS]`) and “Updated Xs ago” so snapshot is flowing.

3. **Open Orders tab**
   - Press `3` or use Tab / arrow keys to select the **Orders** tab.

4. **Select an order**
   - Use **↑ / ↓** (or **PgUp / PgDn**) to move the selection highlight over an order row.
   - Optional: press **/** to focus the filter, type to filter by symbol/status/side, **Esc** to clear.

5. **Open detail overlay**
   - Press **Enter** on the selected order.

6. **Confirm overlay**
   - A centered overlay titled **Order details** should appear with:
     - **ID**, **Symbol**, **Side** (green for BUY, red for SELL), **Qty**, **Status**, **Submitted** (UTC timestamp).
   - Values must match the selected row in the table.

7. **Close overlay**
   - Press **Esc** to close the overlay and return to the Orders table.

---

## Pass criteria

- Overlay opens on Enter when an order is selected.
- Overlay shows the same order data as the selected row (ID, symbol, side, qty, status, submitted time).
- Esc closes the overlay.

---

## If no orders appear

- Backend may not be seeding mock orders (e.g. `mock_data::seed_snapshot_if_empty`). Check backend logs and snapshot content.
- Ensure TUI is subscribed to the same NATS subject as the backend publishes (`snapshot.{BACKEND_ID}`, default `BACKEND_ID=ib`).

---

## QA / review note

- **Runbook reviewed:** Runbook was cross-checked against `tui_service/src/ui/mod.rs` (`render_detail_overlay`, `DetailPopupContent::Order`) and `tui_service/src/app.rs` (Orders tab, `detail_popup`, Enter/Esc handling). Steps and pass criteria match the implementation. Manual execution is left to the operator per environment (NATS + backend + TUI).
