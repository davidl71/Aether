## tui-logger vs custom Logs tab (Aether)

### Summary
Aether `tui_service` is already using `tui-logger` for the Logs tab and a log panel overlay. The remaining question is whether to expand usage (target selector/filter UX) and whether to keep both the Logs tab and overlay.

### Local findings
- Logs widget: `agents/backend/services/tui_service/src/ui/logs.rs`
- State: `agents/backend/services/tui_service/src/app.rs` (`log_state`, `log_display_level`, `show_log_panel`)
- Render sites: `agents/backend/services/tui_service/src/ui/mod.rs` (Operations combined view + optional overlay)

### Internet references (2026)
- `tui-logger` overview: https://lib.rs/crates/tui-logger
- `tui-logger` API docs: https://docs.rs/tui-logger

### Recommendation
- Keep `tui-logger` as the core logs UI (already in place).
- Optional follow-up tasks:
  - Decide on a single logs UX (tab vs overlay vs both).
  - Expose target selector / filtering if logs become noisy.
