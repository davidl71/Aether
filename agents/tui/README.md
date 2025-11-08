# TUI Agent

Responsibilities:
- Implement and maintain the Go-based terminal dashboard (GNU top style).
- Integrate with backend REST/WebSocket feeds for live data.
- Provide quick action workflows (combo buy/sell, detail popovers).

Quick start:
1. `bash agents/tui/scripts/setup.sh` to install dependencies (Go modules).
2. `bash agents/tui/scripts/run-tests.sh` to run `go test ./tui/...`.
3. Use `cursor-agent.json` here to configure a dedicated Cursor agent.

Key TODOs:
- TODO #24 (complete live data wiring, controls, popovers).

