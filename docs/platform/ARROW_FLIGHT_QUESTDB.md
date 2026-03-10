# Apache Arrow Flight for QuestDB — Research & Read Path

**Epic**: T-1772887222569465548 — Replace QuestDB HTTP polling with Arrow Flight SQL for columnar bulk reads.

**References**: [IMPROVEMENT_PLAN.md](IMPROVEMENT_PLAN.md) (E2), [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md).

---

## 1. Current state

- **QuestDB write path**: Go `collection-daemon` (`agents/go/cmd/collection-daemon`) subscribes to NATS (Core or JetStream), decodes `NatsEnvelope`/`MarketDataEvent`, and writes to QuestDB via **ILP** (Influx Line Protocol) on port **9009** when `QUESTDB_ILP_ADDR` is set.
- **QuestDB read path**: Python `questdb_client.py` uses **HTTP** `GET /exec?query=...` (port 9000) and returns JSON rows. Used for `query()`, `get_ohlcv()`, `get_latest_quotes()`.

**Gap**: Bulk/historical reads are row-wise over HTTP; no columnar zero-copy path.

---

## 2. QuestDB and Arrow Flight SQL

QuestDB supports **Arrow Flight SQL** for high-throughput, columnar query results. Typical setup:

- **ILP**: `localhost:9009` (unchanged; bridge keeps using this).
- **HTTP**: `localhost:9000` (unchanged; optional fallback).
- **Arrow Flight SQL**: usually exposed on a dedicated port (e.g. **8812** for gRPC). Check QuestDB server config/docs for the exact port and TLS options.

Clients can use the **Flight SQL** protocol: execute a SQL query, receive one or more Arrow record batches (columnar), with minimal serialization overhead and optional zero-copy in-process.

---

## 3. Go collection-daemon QuestDB sink as Arrow Flight writer (research)

**Scope of E2**: The Go bridge should become an **Arrow Flight writer** *alongside* existing ILP.

### 3.1 Current writer behavior

- Single TCP connection to QuestDB ILP (`QUESTDB_ILP_ADDR`, default `localhost:9009`).
- Each NATS message → one ILP line (`market_data,symbol=... bid=...,ask=...,...`).
- No batching; no Arrow.

### 3.2 Option A: Keep ILP for real-time; add Flight for bulk backfill

- **ILP**: Remain the primary path for live ticks (low latency, line-by-line).
- **Arrow Flight (writer)**: Use when the bridge needs to **bulk insert** (e.g. backfill from JetStream replay). Options:
  - **ADBC Go driver (Flight SQL)**  
    [github.com/apache/arrow-adbc/go/adbc/driver/flightsql](https://github.com/apache/arrow-adbc) — connect to QuestDB’s Flight SQL endpoint, then use **StatementExecutePartitions** / **ExecuteSchema** and **DoPut** with `CommandStatementIngest` (or equivalent) to push Arrow record batches. Requires QuestDB to expose a Flight SQL endpoint that accepts ingest (some implementations support this; verify with QuestDB docs).
  - **Arrow Flight Go client**  
    [github.com/apache/arrow-go](https://github.com/apache/arrow-go) — build Flight client and send batches via DoPut to a custom or standard Flight SQL endpoint.

### 3.3 Option B: Batch in bridge; write via Flight SQL

- Buffer N ticks or T seconds in the bridge.
- Build an Arrow record batch (schema: symbol, bid, ask, last, volume, timestamp).
- Use ADBC Flight SQL driver to **ingest** (e.g. `CommandStatementIngest` or DoPut stream) into QuestDB.

**Recommendation**: Implement **Option A** first (ILP for live, Flight for bulk/backfill) so the bridge stays simple for the hot path. Add batching + Flight writer (Option B) in a follow-up if needed for throughput.

### 3.4 Go ADBC Flight SQL (read) example

For **read** path from Go (e.g. gateway or another service):

```go
// Example: connect to QuestDB Arrow Flight SQL and run a query (read path).
driver := flightsql.NewDriver(memory.NewGoAllocator())
db, _ := driver.NewDatabase(map[string]string{
    adbc.OptionKeyURI: "grpc://localhost:8812", // QuestDB Flight SQL port
})
conn, _ := db.Open(ctx)
stmt, _ := conn.NewStatement()
_ = stmt.SetSqlQuery("SELECT symbol, bid, ask, last, volume, timestamp FROM market_data LIMIT 10000")
reader, _, _ := stmt.ExecuteQuery(ctx)
for reader.Next() {
    batch := reader.RecordBatch()
    // Process columnar batch
}
```

**Dependencies**: `github.com/apache/arrow-adbc/go/adbc`, `go/adbc/driver/flightsql`, `github.com/apache/arrow-go/v18/arrow/memory`.

---

## 4. Python read path: one implemented path

**Scope**: Replace or complement HTTP polling for one read path with Arrow Flight SQL.

### 4.1 Implementation

- In **`python/integration/questdb_client.py`**:
  - **`query_flight_sql(sql, flight_port=8812)`** (or equivalent) uses **pyarrow.flight** to connect to QuestDB’s Flight SQL endpoint, execute the given SQL, and return results as **PyArrow Table** (or list of record batches). If Flight is unavailable (e.g. connection error or pyarrow not installed), fall back to existing HTTP `query()`.
  - **`get_ohlcv()`** can be wired to use `query_flight_sql()` when available, so one concrete read path (OHLCV bars) uses Arrow.
  - Implemented: `get_ohlcv(..., use_flight=True)` (default) calls `query_flight_sql()`; set `use_flight=False` to force HTTP.

### 4.2 Benefits

- **Columnar**: Zero-copy into pandas/analytics if needed (`table.to_pandas()`).
- **Bulk**: 10–100× faster for large result sets than JSON over HTTP.
- **Notebooks**: Same client works in Jupyter for ad-hoc SQL and OHLCV.

### 4.3 Dependencies

- **pyarrow** (with Flight support): already in `requirements-notebooks.txt` (pyarrow>=14.0.0). For `python/integration/` consider adding optional dependency or documenting that Flight path requires `pyarrow` with Flight (e.g. `pip install 'pyarrow[flight]'` or full pyarrow).
- **adbc-driver-flightsql**: optional Python dependency for `query_flight_sql()` and for `get_ohlcv(..., use_flight=True)`. Install with `pip install adbc-driver-flightsql`. When not installed, the client falls back to HTTP `/exec`.

---

## 5. Web / optional Arrow IPC

Epic E2 also mentions: *“Web: optional Arrow IPC for bulk option chain / position history queries.”*

- **Approach**: Backend (Rust or Go) queries QuestDB via Arrow Flight SQL (or ADBC), then streams Arrow IPC to the browser, or the PWA calls an endpoint that returns Arrow-over-HTTP (e.g. application/vnd.apache.arrow.stream).
- **Deferred**: Document as follow-up; not required for “one read path” in this task.

---

## 6. Summary

| Component | Current | After E2 (this scope) |
|----------|---------|------------------------|
| Go collection-daemon QuestDB sink | ILP writer only | **Researched**: add Arrow Flight writer for bulk/backfill (Option A); ILP unchanged for live ticks. |
| Python QuestDB reads | HTTP `/exec` only | **Implemented**: one read path via Arrow Flight SQL (`query_flight_sql` + optional `get_ohlcv` over Flight). Fallback to HTTP. |
| Web bulk reads | — | Documented as optional follow-up. |

**Next steps** (for later tasks):

- Confirm QuestDB Arrow Flight SQL port and ingest API (DoPut / CommandStatementIngest).
- Add ADBC Flight SQL or Arrow Flight Go deps to `agents/go` and implement a bulk writer path in `collection-daemon`.
- Add integration test: Python `query_flight_sql` against local QuestDB with Flight enabled.
