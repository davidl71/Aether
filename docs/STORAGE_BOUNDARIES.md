---  
title: Storage Boundaries for Read-Only Exploration Mode  
---  
  
## 1. Canonical Snapshot  
  
- **Type**: `api::SystemSnapshot` / `RuntimeSnapshotDto`.  
- **Owner**: `backend_service` publishes it on `snapshot.{backend_id}` every second.  
- **Responsibility**: captures the authoritative ledger/positions/orders/alerts state that every TUI tab renders; update via `snapshot_publisher` after every market-data event.  
- **Read-only UI contract**: TUI accepts whichever snapshot arrives last (`App::set_snapshot` always overrides), and dashboard widgets treat it as the single source of truth for orders/alerts/positions.  
  
## 2. Market-Data Ingestion and Aggregation  
  
- **Sources**: IB/TWS (`spawn_broker_market_data_loop` with priority 100), Yahoo/FMP/Polygon providers (via `create_provider` and `MarketDataIngestor`).  
- **Aggregator**: `market_data::MarketDataAggregator` selects the winning quote per symbol by comparing each event’s `source_priority` and timestamp, then publishes the best quote to:  
  - `market-data.tick.{symbol}` (proto `MarketDataEvent` with `source`/`source_priority`)  
  - `market-data.candle.{symbol}` (proto `CandleSnapshot` derived from the snapshot’s candle state)  
  - `system.alerts` (alerts emitted on wide spreads or similar conditions)  
- **TUI contract**: the TUI listens on the above NATS subjects and renders only those ticks/candles; it also surfaces the last `source@priority` badge plus the age of the most recent tick so operators can see which provider currently won and whether the feed is fresh.  
  
## 3. Backlog & Task Store

- **Database**: `.todo2/todo2.db` is the SQLite store that tracks tasks; each task has a unique `id` PK and metadata (status, priority, project, etc.).
- **Mirror**: `.todo2/state.todo2.json` reflects the SQLite content and is read by the backlog UI; run the repo’s sync command after any edits so the two stay aligned, otherwise downstream task-list commands break with UNIQUE constraint errors.
- **Contract**: update tasks via the official task workflow tool (status-only updates, then sync), and when the sync reports duplicate IDs delete the offending rows directly from SQLite before running it again. `T-1774365437322588000` / `T-1774365387778319000` document this cleanup workflow.
  
## 4. Tooling Interfaces  
  
- **NATS**: backend publishes on `market-data.*` subjects, while the TUI subscribes via `tui_service/src/nats.rs`; ensure the same NATS URL and subjects are configured for both (look at `TuiConfig::nats_url`).  
**Task tooling**: use the repo’s approved task toolchain (list/update/delete/sync); the JSON cache is merely a snapshot of the SQLite table, so any drift must be resolved on the SQLite side before syncing.
  
## 5. Verification & Monitoring  
  
- Run `/Users/davidl/Projects/mcp/exarp-go/bin/exarp-go task sync` after any manual SQLite change and confirm that the exit value reports only `synced_count` (no errors).  
- Periodically query `sqlite3 .todo2/todo2.db "SELECT id,count(*) FROM tasks GROUP BY id HAVING count(*)>1"`—the result must stay empty.  
- Keep `T-1774362544084047000`’s regression tests running so Charts/Alerts visibly show “waiting for live data” when market feeds or snapshots pause.  
