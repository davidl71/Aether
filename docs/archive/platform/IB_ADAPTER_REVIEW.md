# IB Adapter – Human Review Checklist (Scope, Approach, Security)

**Purpose:** Short review document for human decision before further implementation or live use.  
**Crate:** `agents/backend/crates/ib_adapter`  
**Status:** Core flows wired to `ibapi`; BAG (combo) order and event forwarding still stubbed/partial.

---

## 1. Scope

| Area | Current state | Decision / checklist |
|------|----------------|----------------------|
| **Paper vs live** | `IbConfig::paper_trading` (default `true`); port 7497 = paper. Live typically uses 7496. | ☐ Confirm live trading is gated (e.g. explicit config + env or flag). ☐ Document port convention (7497 paper, 7496 live) in config/docs. |
| **Account selection** | `request_account()` uses first managed account from TWS. | ☐ Decide: single-account only, or support multi-account (config-driven account id filter). |
| **Methods implemented** | `connect`, `disconnect`, `request_market_data`, `request_option_chain`, `place_order` (single-leg), `cancel_order`, `request_positions`, `request_account` — all call ibapi. | ☐ Accept current scope for v1; add more (e.g. historical bars, scanner) only when needed. |
| **BAG (combo) order** | `place_bag_order` is a stub: returns `Ok(0)`, TODO T-1773509396769177000. Required for box spreads. | ☐ Prioritize BAG wiring when box-spread execution is needed. |
| **Event channels** | `market_data_tx`, `position_tx`, `order_tx` are created but ticks/order status are not yet forwarded from ibapi callbacks into these channels. | ☐ Defer streaming events until snapshot-based flows are stable, or scope a follow-up task to wire callbacks → channels. |

**Out of scope for this checklist:** Runtime state placeholders in `api/runtime_state.rs`, TUI REST fallback, NATS topic layout.

---

## 2. Approach

| Topic | Notes | Checklist |
|-------|--------|-----------|
| **ibapi crate** | Async Rust client; used for connection, market data subscription, option chain, orders, positions, account summary. | ☐ Confirm ibapi version and upgrade policy (workspace dependency). |
| **Config** | `IbConfig`: host, port, client_id, paper_trading. No credentials in code; TWS/Gateway handles auth. | ☐ Ensure config is loaded from env/file and not hardcoded for production. |
| **Connection lifecycle** | `connect()` sets state to Connecting → Connected (or Error); `disconnect()` drops client (ibapi closes on drop). | ☐ Decide if explicit keepalive/reconnect policy is required later. |
| **Expiry parsing** | Uses `common::expiry::parse_expiry_yyyy_mm_dd` for option contract expiry. | ☐ No change needed unless format or timezone rules change. |
| **Error handling** | Methods return `Result<_, String>`; ibapi errors mapped to string. | ☐ Optional: introduce typed error (e.g. thiserror) for API boundary. |

---

## 3. Security

| Item | Status / recommendation | Checklist |
|------|-------------------------|-----------|
| **No secrets in repo** | Host/port/client_id only; no TWS password or token in code. | ☐ Confirm no credentials committed; use env or secure config. |
| **Paper default** | `paper_trading: true` and port 7497 in default config. | ☐ Keep default paper-only; require explicit switch for live. |
| **Logging** | `info!` for connect/disconnect and address (no account ids in logs). Account summary values are parsed but not logged in adapter. | ☐ Avoid logging order details, account ids, or PII in production. |
| **Network** | Connects to localhost or configured host; no outbound TLS to IB (TWS/Gateway is local). | ☐ If ever connecting to a remote Gateway, ensure TLS and trusted host. |
| **Input validation** | Symbol, quantity, limit_price come from callers; ibapi may reject invalid requests. | ☐ Validate quantity > 0 and sane limit_price in API layer before calling adapter. |

---

## 4. Follow-up

- **Implement BAG order** (T-1773509396769177000) when box-spread execution is required.
- **Optional:** Wire ibapi streaming callbacks into `market_data_tx` / `position_tx` / `order_tx` for real-time UI or downstream.
- **Optional:** Add structured error type and map ibapi errors for better API responses.
- **Doc:** Update `docs/platform/STUB_CODE_PLANNING.md` — ib_adapter is no longer a “full stub”; only BAG order and event forwarding remain stubbed/partial.

---

*Generated for Todo2 task T-1773514701956034000 (Human review: IB adapter implementation scope and approach).*
