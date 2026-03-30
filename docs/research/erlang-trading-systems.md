# Erlang / Elixir open-source trading and banking systems

**Date:** 2026-03-30  
**Todo2:** T-1774476836614010000  
**Audience:** Aether architecture (Rust-first backend); this note is comparative, not a migration proposal.

## Executive summary

Open-source BEAM trading stacks cluster into four roles: **matching engines and order books** (often libraries), **full demos or exchanges** (Phoenix + LiveView or RabbitMQ pipelines), **L2/L3 book maintenance** (Erlang processes + ETS), and **gateways** (ZeroMQ/protobuf to external venues). Aether already uses NATS, protobuf, and a Rust service mesh; the main transferable ideas are **supervision boundaries**, **per-instrument or per-book process isolation**, and **explicit fan-out** of market data—not a wholesale move to OTP.

---

## Projects reviewed

### [realyarilabs/exchange](https://github.com/realyarilabs/exchange)

- **Role:** Elixir library: matching engine, order book, limit/market orders ([Hex package `exchange`](https://hex.pm/packages/exchange), [docs: `Exchange.MatchingEngine`](https://hexdocs.pm/exchange/Exchange.MatchingEngine.html)).
- **Architecture:** Functional domain modules; integrates with **RabbitMQ** and **InfluxDB** for messaging and time-series style persistence (see Hex metadata / README patterns).
- **Takeaway for Aether:** Clear separation between **matching** and **I/O adapters** (queues, metrics). Comparable to keeping Rust core pure and pushing I/O to `nats_adapter` / `ib_adapter`.

### [realyarilabs/exchange_api](https://github.com/realyarilabs/exchange_api)

- **Role:** Phoenix REST API and dashboard over the exchange core.
- **Takeaway:** Operator-facing HTTP + real-time UI is analogous to Aether’s **backend_service + TUI/Web** split; LiveView maps to WS-heavy clients, not to the TUI directly.

### [myotp/trade-hub](https://github.com/myotp/trade-hub)

- **Role:** Full-stack sample **stock exchange**: gateway, matching, clearing, market data, **Phoenix LiveView** UI ([Hex `tradehub` docs](https://hexdocs.pm/tradehub/readme.html)).
- **Takeaway:** End-to-end pipeline visibility for teaching; process boundaries per subsystem mirror how we separate **ingest, snapshot, and UI read models** in Rust.

### [arentrue/order_book](https://github.com/arentrue/order_book)

- **Role:** Erlang **aggregated order book** (per-price-level quantities) driven by `{new_order, ...}`, `{delete_order, ...}`, `{modify_order, ...}` events; exchange assumed to match externally.
- **Data structures:** Process + **ETS**-oriented design; README discusses sharding bid/ask or depth ranges to reduce a single `order_book_instrument` bottleneck.
- **Takeaway:** L2 maintenance as a **single-writer service** with event replay; sharding story parallels **per-symbol NATS consumers** or partitioned in-memory books in Rust.

### [andrediogo92/Exchanger](https://github.com/andrediogo92/Exchanger)

- **Role:** Erlang **gateway**: auth, order routing to external exchanges, **ZeroMQ** via Chumak, **protobuf** messages, rebar3 build.
- **Takeaway:** “Thin edge, fat venue” pattern—close to **broker adapters** that normalize wire protocols behind a small API surface.

---

## Cross-cutting patterns

| Theme | BEAM idioms | Aether analogue |
|--------|-------------|-----------------|
| Fault tolerance | Supervision trees, restarts | Kubernetes / process supervisors, health checks in TUI |
| Concurrency | Per-book or per-instrument processes | Tasks, dedicated threads, or async consumers per feed |
| Shared state | ETS / pg | `Arc`, channels, snapshot caches in `backend_service` |
| Message fan-out | Pub/sub, Phoenix channels, RabbitMQ | NATS `api.*`, WS to clients |
| Serialization | Protobuf, Erlang terms | Prost, NATS protobuf |

---

## Order book and market data

- **L3 / event-sourced books** often use ordered structures per side; **L2** aggregates at price level (`arentrue/order_book` style).
- **Matching** is either in-library (`exchange`) or assumed external with book fed by outcomes—important when comparing to **IBKR / multi-broker** reality where matching is off-box.

---

## Hybrid Rust + Erlang / Elixir

Reasonable hybrids (only if a concrete bottleneck appears):

1. **Rust execution path, BEAM for orchestration or DSL** — e.g. strategy config or workflow in Elixir calling Rust via NIF/Port (high integration cost).
2. **Independent services** — Elixir for a dedicated real-time fan-out or admin UI, Rust for trading/risk; communicate over **NATS or gRPC** (aligns with current stack).
3. **Avoid** duplicating risk and ledger logic across runtimes; keep **one source of truth** for positions and compliance-sensitive code (Rust today).

---

## Recommendations for Aether

1. **Stay Rust-centric** for broker adapters, ledger, and risk; borrow **supervision thinking** in how we scope restart boundaries (daemons, NATS collectors).
2. When scaling market-data fan-out, prefer **more NATS consumers / partitions** over introducing BEAM solely for concurrency.
3. Use these OSS repos as **reference designs** for order-book APIs and gateway layering, not as dependencies.
4. If evaluating BEAM for a **greenfield** subsystem, prototype only behind a **narrow IPC contract** (protobuf + NATS) shared with the existing backend.

---

## References

- [realyarilabs/exchange](https://github.com/realyarilabs/exchange)
- [realyarilabs/exchange_api](https://github.com/realyarilabs/exchange_api)
- [myotp/trade-hub](https://github.com/myotp/trade-hub)
- [arentrue/order_book](https://github.com/arentrue/order_book)
- [andrediogo92/Exchanger](https://github.com/andrediogo92/Exchanger)
