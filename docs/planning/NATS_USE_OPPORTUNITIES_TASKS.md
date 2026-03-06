# NATS Use Opportunities – Exarp Tasks

Create Todo2 tasks from this plan by running exarp-go **task_discovery** with `create_tasks=true` and `action=markdown` (or `action=all`), with **workingDirectory** set to this repo root.

**Source:** `docs/NATS_USE_OPPORTUNITIES.md`

**How to create tasks:**

1. **Cursor chat (exarp-go MCP):** Ask: *"Run task_discovery with create_tasks=true. Use action=markdown and workingDirectory set to this project root."* The exarp-go MCP server will discover tasks from this file and create Todo2 tasks.
2. **CLI:** From repo root, run:
   ```bash
   ./scripts/run_exarp_go_tool.sh task_discovery '{"action":"markdown","create_tasks":true}'
   ```
   If your shell mangles the JSON, use the MCP method above.

---

## Tasks (for task_discovery)

### TUI and snapshot

- [ ] **NATS provider for TUI snapshot** – Add a NATS provider in `python/tui/providers.py` that subscribes to `snapshot.ib` or `snapshot.aggregated` and updates TUI on message instead of polling REST/file. Depends on backends publishing snapshots (see next).
- [ ] **TUI backend health via NATS** – Replace or augment BackendHealthAggregator polling with subscription to `system.health` or `health.>`; backends publish health on a timer.
- [ ] **Backend services publish snapshots to NATS** – When IB, Alpaca, TradeStation services build a new snapshot, publish to `snapshot.ib`, `snapshot.alpaca`, `snapshot.tradestation` respectively.
- [ ] **Unified snapshot aggregator service** – Implement a service that subscribes to `snapshot.ib`, `snapshot.alpaca`, `snapshot.tradestation`, merges payloads, and publishes to `snapshot.aggregated` or serves a single REST `/api/snapshot`.

### Alerts and orders

- [ ] **Publish alerts to NATS** – Have backends and strategy publish to `system.alerts` when events occur (e.g. wide spread, margin warning); TUI and PWA subscribe for real-time alerts.
- [ ] **Orders and positions on NATS** – When order manager or TWS client updates order/position, publish to `orders.status.{id}`, `orders.fill.{id}`, `positions.update.{symbol}`; TUI/PWA subscribe to `orders.status.>`, `orders.fill.>`, `positions.update.>`.

### Strategy and RPC

- [ ] **Strategy control and status over NATS** – Frontends publish start/stop/pause to `strategy.control`, subscribe to `strategy.status`; add request/reply for `rpc.strategy.status`.
- [ ] **RPC snapshot endpoint** – Support NATS request/reply on `rpc.system.snapshot` for on-demand snapshot in addition to GET `/api/snapshot`.

### Risk and config

- [ ] **Risk and config events on NATS** – Risk engine publishes to `risk.limit.{type}`, `risk.violation`; config manager broadcasts changes on `system.config`; UIs and logging subscribe.

### QuestDB

- [ ] **Unify market data ingestion via NATS for QuestDB** – Either add a NATS→QuestDB writer (Rust or Python) subscribing to Core NATS `market-data.tick.>` and writing ILP, or have producers publish to JetStream `market.data.{symbol}` in the Go bridge JSON shape and run nats-questdb-bridge.
- [ ] **Adapter for existing nats-questdb-bridge** – Add a small adapter that subscribes to Core NATS `market-data.tick.>`, converts to bridge JSON format, republishes to JetStream `market.data.{symbol}`; run existing Go bridge.
- [ ] **QuestDB: strategy/order/position events** – Subscriber that writes strategy decisions, order fills, position updates from NATS to QuestDB tables for audit and backtests.
- [ ] **QuestDB: snapshot/health history** – Optional subscriber that writes snapshots or health events from NATS to QuestDB for historical dashboards.

### Memcached

- [ ] **Cache invalidation over NATS** – When cached data changes, publish to `cache.invalidate.{scope}` or `cache.invalidate.snapshot.ib`; all memcached-using processes subscribe and delete/refresh keys.
- [ ] **Snapshot cache in memcached with NATS** – Write snapshot to memcached; publish `snapshot.updated.{backend}`; TUI/aggregator subscribes and reads from memcached or invalidates local view.
- [ ] **Market data cache consistency (NATS + memcached)** – Single writer that subscribes to NATS market data and updates memcached, or have C++/producers publish `cache.warm.market.{symbol}` when they update memcached.

---

*End of task list for exarp task_discovery.*
