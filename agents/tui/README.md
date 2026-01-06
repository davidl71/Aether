# TUI Agent

Responsibilities:
- Implement and maintain the Python-based terminal dashboard (Textual framework).
- Integrate with backend REST/WebSocket feeds for live data.
- Provide quick action workflows (combo buy/sell, detail popovers).

Quick start:
1. Install dependencies: `pip install textual requests`
2. Run: `python -m python.tui`
3. Use `cursor-agent.json` here to configure a dedicated Cursor agent.

Note: TUI has been migrated from C++ (FTXUI) to Python (Textual). The Python TUI provides better maintainability and shared data models with the PWA frontend.

Key TODOs:
- TODO #24 (complete live data wiring, controls, popovers).
