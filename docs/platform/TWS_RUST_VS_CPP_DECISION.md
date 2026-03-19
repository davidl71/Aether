# TWS: Rust vs C++ client — decision

**Status:** Decision record (2026-03).  
**Related:** [IB_ADAPTER_REVIEW.md](IB_ADAPTER_REVIEW.md), [TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md), [AGENTS.md](../../AGENTS.md).

## Decision

**This repo uses a Rust TWS/IBKR stack only.** There is no C++ TWS client in the active build.

- **Rust:** `agents/backend/crates/ib_adapter` (and `tws_yield_curve`, etc.) use the Rust `ibapi` (or equivalent) client to talk to TWS/IB Gateway. All live order placement, market data, and yield-curve fetching from TWS are implemented in Rust.
- **C++:** The legacy C++ native engine and TWS API under `native/third_party/tws-api/` are **out of scope** for the current build (see root `CMakeLists.txt` and [MULTI_LANGUAGE_CODEBASE.md](../MULTI_LANGUAGE_CODEBASE.md)). C++ is not used for TWS connectivity in this repo.
- **Protobuf:** Our `proto/messages.proto` defines **platform messages** (NATS, snapshot, strategy, etc.). TWS API has its **own** protobuf support (min server version, message IDs) documented in the sibling repo and in [TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md). So “Rust TWS proto” here means: we use Rust to talk to TWS; our proto is for our wire format (NATS/KV), not for replacing the TWS wire protocol.

## Implications

- No “Rust TWS proto vs C++ client” migration in this codebase: there is no C++ TWS client to replace.
- New TWS features (e.g. order placement, market data) are implemented in Rust (`ib_adapter`, `tws_yield_curve`).
- Proto work is about unifying **our** message schema (snapshot, strategy, KV) and optional use of TWS API protobuf where the sibling repo documents it.
