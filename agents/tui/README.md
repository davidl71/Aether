# TUI Agent

Responsibilities:
- Implement and maintain the C++-based terminal dashboard (FTXUI, htop-style).
- Integrate with backend REST/WebSocket feeds for live data.
- Provide quick action workflows (combo buy/sell, detail popovers).

Quick start:
1. Build the C++ TUI: `cmake --build build --target ib_box_spread_tui`
2. Run: `./build/ib_box_spread_tui`
3. Use `cursor-agent.json` here to configure a dedicated Cursor agent.

Note: TUI has been migrated from Go to C++ (FTXUI library). The C++ TUI is built as part of the main CMake build.

Key TODOs:
- TODO #24 (complete live data wiring, controls, popovers).
