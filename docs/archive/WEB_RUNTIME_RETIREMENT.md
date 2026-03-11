## Web Runtime Retirement

As of March 11, 2026, the React/Vite web app is retired as an active runtime surface.

Current supported frontends:
- Rust TUI
- native CLI

What changed:
- web is no longer part of active service orchestration defaults
- supervisor and systemd defaults no longer start the web dev server
- active product docs no longer describe web as a supported frontend

Why:
- current migration focus is backend simplification and TUI/CLI delivery
- keeping the browser client active increased duplication in routing, health, account, and service status paths

What remains:
- `web/` stays in the repository as archived implementation/reference material
- future reintroduction should happen only after a deliberate UI framework decision
- see `docs/platform/UI_FRAMEWORK_OPTIONS.md` for framework evaluation criteria and options
