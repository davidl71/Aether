# NATS API and topology verification checklist

Use this runbook to verify NATS API (Discount Bank, Loans, FMP, finance rates, strategy) from TUI/CLI and to spot-check the topology table against the running backend/tui.

## Prerequisites

- NATS server running (e.g. port 4222).
- `backend_service` and `tui_service` (or CLI) built and runnable.

## 1. Verify topology table vs code

- **Backend publishes:** `snapshot.{backend_id}`, `market-data.tick.{symbol}`, `strategy.signal/decision.{symbol}`, `system.dlq.backend.*`  
  Code: `backend_service` snapshot_publisher, rest_snapshot; nats_integration; api_handlers (strategy); collection_aggregation (DLQ). See [NATS_API.md](../platform/NATS_API.md) and [CURRENT_TOPOLOGY.md](../platform/CURRENT_TOPOLOGY.md).
- **Backend subscribes:** `system.health`, `NATS_SUBJECTS` (collection), `api.discount_bank.*`, `api.loans.*`, `api.fmp.*`, `api.finance_rates.*`, `api.strategy.start` / `stop` / `cancel_all`.  
  Code: `api_handlers.rs` (exact subject constants), `collection_aggregation.rs` (NATS_SUBJECTS).
- **TUI subscribes:** `snapshot.{backend_id}`. **TUI requests:** `api.finance_rates.build_curve`, `.benchmarks`, `api.strategy.start` / `stop` / `cancel_all`.  
  Code: `tui_service` main.rs, ui/yield_curve.rs.

Spot-check: grep for subject constants in `api_handlers.rs` and compare to NATS_API.md § Request/reply (API) — implemented.

## 2. Verify NATS API from TUI/CLI (manual)

With backend and TUI running:

| Area | How to verify |
|------|----------------|
| **Discount Bank** | TUI: ensure Discount Bank / banking view loads (mock or live). Or use NATS request: `nats req api.discount_bank.balance '{}'` (with reply inbox). |
| **Loans** | TUI: open Loans or equivalent; list/create/update/delete if UI exposed. Or `nats req api.loans.list '{}'`. |
| **FMP** | Only when `FMP_API_KEY` set. TUI or request: `nats req api.fmp.quote '{"symbol":"AAPL"}'`. |
| **Finance rates** | TUI: Yield curve tab; request curve or benchmarks. Or `nats req api.finance_rates.benchmarks '{}'`. |
| **Strategy** | TUI: start/stop/cancel-all (e.g. K for cancel all). Or `nats req api.strategy.cancel_all '{}'`. |

CLI: if CLI has NATS client and requests to api.*, run the same flows from CLI and confirm replies.

## 3. Test NATS_SUBJECTS changes

1. Set `NATS_SUBJECTS` to the new list (e.g. `export NATS_SUBJECTS="market-data.tick.>,strategy.signal.>"`).
2. Start backend: `cargo run -p backend_service` (with `NATS_URL` set).
3. Check logs for collection subscription or errors (no "invalid subject" or subscription failures).
4. Optional: publish a message on a subject in the list and confirm backend receives it (or check LIVE_STATE KV if enabled).

See [NATS_API.md](../platform/NATS_API.md) § Operator guidelines for NATS_SUBJECTS.

## 4. Quick automated checks (optional)

- **Backend subscribes to api.*:** Start backend with NATS_URL; check logs for subscription or handler spawn messages (e.g. "NATS API handlers spawned").
- **Subject list:** See §3 above. Set `NATS_SUBJECTS=market-data.tick.>,strategy.signal.>`; start backend; confirm no subscription errors in logs.
- **Snapshot:** Start backend and TUI; TUI should show "Updated Xs ago" or dashboard data when snapshot is received.

## References

- [NATS_API.md](../platform/NATS_API.md) — subject list, request/reply, scope
- [CURRENT_TOPOLOGY.md](../platform/CURRENT_TOPOLOGY.md) — topology table and runtime shape
