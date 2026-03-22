# IBKR Connection Testing – Quick Start

**Status:** Updated for Rust-first stack. Native C++ test binaries and some scripts were removed.

## The Problem (Identified)

IB Gateway Live can reject API connections shortly after handshake when security settings block incoming connections.

**Root cause:** "Accept incoming connection requests automatically" is disabled.

## The Fix (Simple)

1. Open IB Gateway (or TWS).
2. **Configure → Settings → API → Settings**
3. **Check:** ☑ Accept incoming connection requests automatically
4. Click **OK**
5. **Restart IB Gateway**
6. Verify ports: `./scripts/test_tws_connection.sh`

## Quick Test Commands

```bash
# Check if TWS/Gateway ports are listening (4001, 4002, 7496, 7497)
./scripts/test_tws_connection.sh

# Test a specific port (e.g. paper 7497)
./scripts/test_tws_connection.sh 7497
```

Then use the **Rust backend or TUI** with your config (paper port 7497, live 7496 / Gateway 4002 live, 4001 paper):

```bash
cd agents/backend && cargo run -p tui_service
# or start backend_service and use REST/CLI with config pointing to TWS
```

## Expected Output (When Fixed)

### Port check script

```
✓ Port 7497 is listening
All tests passed! TWS is ready.
```

### Next steps

1. Start Rust backend/TUI with config that uses the same host/port.
2. Accept the connection in the TWS/Gateway API popup if prompted.
3. Check application logs for connection confirmation.

## Reference

- **Ports:** 7497 = TWS paper, 7496 = TWS live, 4002 = Gateway paper, 4001 = Gateway live.
- **Legacy:** Previous native C++ test programs (`test_positions_live`, `test_simple_connect`, `diagnose_ibkr.sh`, etc.) were removed with the native build. Use `test_tws_connection.sh` for port checks and the Rust stack for full connection and trading flows.
