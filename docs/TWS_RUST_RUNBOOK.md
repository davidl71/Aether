# TWS / IBKR operations (Rust stack)

**Purpose:** Single active-doc entry point for connecting Interactive Brokers to Aether’s **Rust** services. Archived C++ TWS notes live under `docs/archive/` (not indexed by default); prefer this runbook and the code paths below for day-to-day work.

---

## Components

| Piece | Location / command |
|-------|---------------------|
| IBKR adapter | `agents/backend/crates/ib_adapter/` (implements broker engine against TWS API) |
| TWS API sources | Sibling checkout `../tws-api/` (see `AGENTS.md`) or vendor path documented there |
| Backend service | `cd agents/backend && cargo run -p backend_service` |
| Yield curve daemon (optional) | `cargo run -p tws_yield_curve_daemon` (when using standalone curve fetch) |
| Product direction | Read-only / data exploration: `docs/DATA_EXPLORATION_MODE.md` |

---

## Connectivity checklist

1. **Run IB Gateway or TWS** with API sockets enabled (Configure → API → Settings).
2. **Paper trading:** use port **7497** for testing (see `AGENTS.md` safety rules).
3. **Live:** only with explicit product/config decisions; never commit credentials.
4. Ensure **`ib_adapter`** can reach the host/port in your config (env / config file as used by `backend_service`).

---

## Design references

- [TWS_BACKEND_PROVIDER_DECISION.md](./TWS_BACKEND_PROVIDER_DECISION.md) — provider choice and deprecation of duplicate adapters in active docs.
- [ARCHITECTURE.md](../ARCHITECTURE.md) — where IBKR fits in the Rust workspace.
- [AGENTS.md](../AGENTS.md), [CLAUDE.md](../CLAUDE.md) — Cargo workspace path (`agents/backend/`), commands, and safety defaults.

---

## Promoting content from archive

If you need legacy operational detail (screenshots, Gateway-specific steps) that still apply to the Rust stack:

1. Verify steps against current Gateway/TWS builds.
2. Copy the **minimal** procedure into this file or `docs/QUICKSTART_RUST.md`.
3. Link back to the archive path in git history or `docs/archive/` for full context.

This replaces keeping duplicate TWS guides only in archive search results.
