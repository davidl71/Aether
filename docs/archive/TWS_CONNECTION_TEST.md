# TWS API Connection Test Guide

**Date**: 2025-11-13
**Purpose**: Test and verify TWS/Gateway connectivity
**Status**: Updated for Rust-first stack (native C++ test program removed)

---

## Overview

This guide helps you verify that TWS or IB Gateway is reachable before using the Rust backend or TUI.

## Test Tools

### Shell script (current)

**File**: `scripts/test_tws_connection.sh`

- Checks if TWS/Gateway is listening on the usual ports (4001, 4002, 7496, 7497)
- No build required

```bash
# Check all standard ports
./scripts/test_tws_connection.sh

# Check a specific port (e.g. paper 7497)
./scripts/test_tws_connection.sh 7497
```

### Application connection

Use the **Rust backend or TUI** with your config (host, port, client_id). Start the service and confirm connection in logs; accept the API connection in the TWS/Gateway popup if prompted.

```bash
cd agents/backend && cargo run -p tui_service
# Use config with tws.host, tws.port (e.g. 7497 for paper)
```

## Quick Start

1. Start TWS or IB Gateway and enable API (e.g. port 7497 for paper).
2. Run `./scripts/test_tws_connection.sh 7497` to confirm the port is listening.
3. Start the Rust TUI/backend with config pointing at that host/port.
4. Accept the connection in the TWS/Gateway dialog and check logs.

## Legacy

- A C++ test program (`native/tests/test_tws_connection.cpp`) and building/running it via the script were removed with the native build.
- The main application referred to here was the native `ib_box_spread` binary; use the Rust stack instead.
