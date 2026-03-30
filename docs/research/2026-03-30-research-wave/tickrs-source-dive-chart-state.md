## tickrs source dive: chart state + update loop (tarkah/tickrs)

References (verified, 2026): [tickrs repo](https://github.com/tarkah/tickrs), [src/main.rs on master](https://raw.githubusercontent.com/tarkah/tickrs/master/src/main.rs).

### Process model (not Tokio select on one thread)
- **UI events**: `setup_ui_events()` spawns a thread that blocks on `crossterm::event::read()` and sends every `Event` on an **unbounded** crossbeam channel (`ui_events`).
- **Redraw**: Global `REDRAW_REQUEST` is a **bounded(1)** channel; key handlers end with `request_redraw.try_send(())` so coalesced wakeups redraw after state changes.
- **Data refresh**: `DATA_RECEIVED` is bounded(1). Background fetch tasks notify when new API data is available; the main `select!` arm calls `app.update()` then `stock.update()` for each `StockState`.

### Redraw / animation loop
- A **dedicated thread** runs `select!`:
  - On `redraw_requested`: `draw::draw(&mut terminal, &mut app)`.
  - On **default 500ms**: still draws, and calls `stock.loading_tick()` on each stock to animate the loading indicator (continuous repaint without new data).

### App + chart ownership
- `app::App` holds `mode: Mode`, `stocks: Vec<StockState>`, `current_tab`, `chart_type`, `summary_scroll_state`, etc. Modes (`DisplayStock`, `ConfigureChart`, `DisplayOptions`, …) separate navigation from chart configuration UI.
- Per-symbol state lives in **`widget::stock::StockState`**: `prices` (per timeframe buckets), `time_frame`, `chart_type`, `chart_configuration`, `chart_state: Option<ChartState>`, `stock_service`, `loading_tick`, `cache_state`. Timeframe changes call `set_time_frame` → updates service → `set_chart_type` resets derived chart state.

### Event routing (keyboard)
- `event::handle_key_bindings(mode, key_event, app, request_redraw)` dispatches by `Mode` to focused handlers (`handle_keys_display_stock`, `handle_keys_configure_chart`, etc.).
- **Chart pan**: global bindings (not mode-gated) call `stock.chart_state_mut()?.scroll_left()` / `scroll_right()` on Shift+Left/Right or `<` / `>` ([event.rs](https://raw.githubusercontent.com/tarkah/tickrs/master/src/event.rs)).

### Patterns applicable to Aether
1. **Separate redraw from network**: bounded redraw channel + periodic tick for loading animation avoids coupling repaint rate to poll interval.
2. **Per-tab / per-symbol `StockState`**: chart data and `ChartState` live on the symbol widget, not only in global app state.
3. **Explicit modes** for chart config vs viewing vs options — keeps keymaps small and composable.
4. **Data-received pulse**: notify UI thread when async fetch completes; central `update()` applies new domain data before draw.

### Contrast with Aether
- tickrs uses **crossbeam + std::thread** and a mutex-wrapped `App`; Aether uses **tokio** + `mpsc` for NATS/results. The **notification + redraw** and **per-chart state struct** ideas still transfer.
