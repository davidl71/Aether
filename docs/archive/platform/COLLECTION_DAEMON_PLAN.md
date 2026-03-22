# Unified Collection Path — Design Plan

**Epic**: E5 (IMPROVEMENT_PLAN.md) — Replace Python data-collection polling with a single thin collector.  
**Task**: T-1772887222970694620  
**Last updated**: 2026-03-08

## 1. Objective

Replace Python polling loops (TUI RestProvider, file pollers, health pollers) with one **thin collector** that:

- **Subscribes to NATS** for C++ events (market data, signals, decisions)
- **Polls broker REST APIs** on configurable intervals (positions, rates)
- **Writes** to NATS KV (live state) and QuestDB or existing pipeline (history)
- **Exposes** `/metrics` (Prometheus format) for health monitoring

Python services become **read-only analytics** — they query NATS KV or QuestDB. No Python process writes to storage or polls on a timer.

**Priority lens**: Thin daemons + data persistence. Single binary, low memory, no runtime.

## 2. Prerequisites

- **P2-C**: NATS KV as primary live-state store (optional for first slice; daemon can start with stub writers).
- **Existing**: C++ `nats_client.cpp` publishing `NatsEnvelope` to `market-data.tick.*`, `strategy.signal.*`, `strategy.decision.*` (see DATAFLOW_ARCHITECTURE.md §3).

## 3. Components

| Component        | Responsibility                          | Input                    | Output                    |
|-----------------|------------------------------------------|--------------------------|---------------------------|
| **NATS consumer** | Subscribe to C++ events                 | NATS subjects            | Decoded events → writers |
| **Broker poller**| HTTP GET broker REST on interval         | Config (URL, interval)   | Snapshots → writers      |
| **Live writer**  | Write current state to NATS KV           | Events + poll results    | NATS KV buckets          |
| **History writer**| Write time-series to QuestDB/Arrow       | Events                   | ILP or Arrow Flight      |
| **HTTP server**  | Serve `/metrics` (Prometheus)            | —                        | GET /metrics             |

## 4. Configuration (env)

| Variable           | Default              | Description                          |
|--------------------|----------------------|--------------------------------------|
| `NATS_URL`         | `nats://localhost:4222` | NATS server URL                    |
| `NATS_SUBJECTS`    | (see below)         | Comma-separated subjects to subscribe |
| `METRICS_LISTEN`   | `:9090`             | Listen address for /metrics          |
| `BROKER_POLL_INTERVAL` | `30s`           | Interval for broker REST polling (future) |
| `QUESTDB_ILP_ADDR` | unset               | QuestDB ILP sink address; when set, writes `MarketDataEvent` ticks to QuestDB |

Default subjects (aligned with DATAFLOW_ARCHITECTURE.md):

- `market-data.tick.>`
- `strategy.signal.>`
- `strategy.decision.>`

Publishers must publish to concrete symbol-scoped subjects. Wildcards are subscriber-only.

## 5. Message handling

- All C++ messages use **NatsEnvelope** (protobuf): `message_type` + `payload`.
- Daemon decodes with `agents/go/proto/v1` and dispatches by `message_type`:
  - `MarketDataEvent` → history writer (QuestDB ILP) + optional KV
  - `StrategySignal` / `StrategyDecision` → history + KV (live state)
- **Stub writer** (first slice): log events via slog; optional no-op or in-memory buffer. No QuestDB/NATS KV in first slice.

## 6. Phases

| Phase   | Scope                                              | Status  |
|--------|----------------------------------------------------|--------|
| **0**  | Design doc + one slice: NATS subscribe + stub writer + /metrics | Done (this slice) |
| **1**  | Wire real QuestDB ILP writer in the Rust backend collector | Done |
| **2**  | Add broker REST poller (configurable URL + interval) | Backlog |
| **3**  | NATS KV writer for live state (after P2-C)         | Done |
| **4**  | Decommission Python polling (TUI/Web read from NATS KV / QuestDB) | Backlog |

## 7. Files

| Path | Purpose |
|------|--------|
| `agents/backend/services/backend_service/src/collection_aggregation.rs` | Rust collector: NATS subscribe, decode NatsEnvelope, sink pipeline |
| `docs/platform/COLLECTION_DAEMON_PLAN.md` | This plan |
| `docs/platform/DATAFLOW_ARCHITECTURE.md` | NATS contract and data flow (reference) |
| `docs/platform/IMPROVEMENT_PLAN.md` | Epic E5 and priority matrix |

## 8. Metrics (Prometheus)

- `collection_daemon_messages_received_total{subject, message_type}` — counter
- `collection_daemon_errors_total{component}` — counter (decode/write errors)
- `collection_daemon_last_event_timestamp_seconds{subject}` — gauge (optional)

## 9. Related

- **Epic E5**: `docs/platform/IMPROVEMENT_PLAN.md` § Priority 5 — E5  
- **NATS contract**: `docs/platform/DATAFLOW_ARCHITECTURE.md` §3  
- **Go agents**: `agents/go/README.md`  
- **QuestDB fanout**: handled directly by the Rust backend collector via the optional QuestDB sink (`QUESTDB_ILP_ADDR`)
