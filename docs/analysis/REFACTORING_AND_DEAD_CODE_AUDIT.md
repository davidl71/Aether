# Refactoring Opportunities and Dead Code Audit

**Date:** 2026-02-26
**Scope:** Full codebase (C++, Python, Rust, TypeScript, Shell)

---

## 1. Dead/Legacy Code Inventory

### 1.1 Python Dead Code (~3,300 lines)

| File | Lines | Status | Reason |
|------|-------|--------|--------|
| `python/integration/linear_client.py` | 539 | **Dead** | Zero importers. Project uses exarp/todo2, not Linear.app |
| `python/integration/excel_dde_client.py` | 253 | **Dead** | Windows-only DDE client. Project runs on macOS |
| `python/integration/excel_rtd_client.py` | 284 | **Dead** | Windows-only RTD client. pywin32/xlwings not installable |
| `python/integration/massive_client.py` | 449 | **Dead** | Massive.com API not used. Only in `__init__.py` try/except |
| `python/integration/massive_websocket.py` | ~200 | **Dead** | Companion to massive_client |
| `python/integration/israeli_broker_scraper.py` | ~300 | **Dead** | Only in `__init__.py` try/except, no service uses it |
| `python/integration/excel_file_importer.py` | ~200 | **Dead** | Only used by israeli_broker_scraper |
| `python/integration/relationship_graph_example.py` | 224 | **Dead** | Demo/example script, not integration code |

**Test files for dead code (also removable):**
- `python/tests/test_excel_dde_client.py`
- `python/tests/test_excel_rtd_client.py`
- `python/tests/test_israeli_broker_scraper.py`
- `python/tests/test_israeli_broker_models.py`
- `python/tests/test_excel_file_importer.py`

### 1.2 C++/WASM Dead Code (~350 lines)

| File | Lines | Status | Reason |
|------|-------|--------|--------|
| `native/wasm/src/wasm_bindings.cpp` | 177 | **Dead** | Emscripten bindings never compiled |
| `native/wasm/src/wasm_exports.cpp` | ~80 | **Dead** | Never compiled |
| `native/wasm/include/wasm_types.h` | ~50 | **Dead** | Never compiled |
| `native/wasm/CMakeLists.txt` | ~100 | **Dead** | Never invoked |
| `web/src/hooks/useWasm.ts` | 80 | **Dead** | Never imported by any component |
| `web/src/wasm/loader.ts` | ~40 | **Dead** | Never imported by any component |
| `web/public/wasm/` | 0 | **Empty** | WASM output dir exists but is empty |

Rust backend now handles all server-side calculations. WASM was an early approach to in-browser pricing that was superseded.

### 1.3 One-Shot Scripts (~4,000 lines)

| Category | Files | Lines | Status |
|----------|-------|-------|--------|
| `scripts/fix_task_*.py` | 10 | ~2,200 | **Done** - tasks already fixed |
| `scripts/add_*_todos.py` | 4 | ~1,800 | **Done** - todos already created |
| `scripts/livevol_api_explorer.py` | 1 | 420 | **Done** - research completed |
| `scripts/cboe_reference_data_parser.py` | 1 | 290 | **Done** - research completed |

### 1.4 Script Directory Bloat

- **117 shell scripts** (14,964 lines)
- **53 Python scripts** (10,196 lines)
- **Total: 25,160 lines** in `scripts/`

Many are duplicated or superseded by exarp automation. Consolidation targets:
- NATS lifecycle (7 scripts → 1 or Go supervisor)
- Doc-fixing (6 scripts → 1 or exarp health tool)
- Todo2 automation (6 scripts → exarp task_workflow)

### 1.5 Protobuf/gRPC Dead Code (~330 lines + build deps)

| Item | Lines | Status | Reason |
|------|-------|--------|--------|
| `agents/backend/crates/api/src/grpc.rs` | 68 | **Dead** | gRPC server on :50051, zero clients |
| `agents/backend/proto/ib_backend.proto` | 19 | **Dead** | Only used by dead grpc.rs |
| `agents/backend/crates/api/build.rs` | 8 | **Dead** | tonic_build for dead proto |
| `agents/backend-market-data/` (entire service) | 149 | **Dead** | gRPC service on :50061, zero clients |
| `agents/backend-market-data/proto/market_data.proto` | 20 | **Dead** | Only used by dead service |
| C++ `PROTOBUF_LIB` in CMakeLists.txt | ~20 | **Phantom** | Links libprotobuf.dylib but no .pb.h included |
| tonic/prost Cargo deps | -- | **Unused** | ~30s added to clean builds |

Both `.proto` files include `option go_package` for Go codegen that was never set up.
The project has converged on REST + NATS + WebSocket; gRPC adds build complexity with zero consumers.

### 1.7 Misplaced Files

5 test files in `python/integration/` instead of `python/tests/`:
- `test_swiftness_integration_simple.py`
- `test_swiftness_integration.py`
- `test_swiftness_import.py`
- `test_relationship_graph.py`
- `test_nats_client.py`

### 1.8 Docs Archive

- `docs/archive/` contains **161 markdown files** with deeply nested recursive paths
- Example: `docs/archive/status-reports/docs/archive/status-reports/docs/...`
- Should be flattened or compressed

---

## 2. Refactoring Opportunities

### 2.1 Message Queuing (NATS)

#### A. Enable JetStream + KV Store
- **Current:** NATS core pub/sub only, JetStream commented out in config
- **Problem:** Fire-and-forget delivery; state lives in `Arc<RwLock<>>` inside Rust process
- **Solution:** Enable JetStream, use KV bucket for SystemSnapshot
- **Impact:** Eliminates C++ snapshot.json writer, enables replay on restart
- **Priority:** High (foundation for all MQ refactoring)

#### B. NATS Heartbeat Health Aggregation
- **Current:** PWA polls 8 HTTP endpoints every 10s (340 lines TypeScript)
- **Solution:** Each service publishes heartbeat to `system.health.{service}`, Rust backend aggregates
- **Impact:** 340 lines → single `/api/v1/health/all` endpoint
- **Priority:** Medium

#### C. NATS Request-Reply for Broker Services
- **Current:** 7 separate HTTP servers on ports 8000-8006
- **Solution:** Rust backend sends `rpc.broker.{name}.{method}`, Python replies via NATS
- **Impact:** Eliminates 7 HTTP server processes and port management
- **Priority:** Low (large scope)

### 2.2 Database

#### A. QuestDB Proper Client + Query Layer
- **Current:** Raw TCP socket ILP writer (98 lines), write-only, no queries
- **Solution:** Official `questdb` package + HTTP SQL query endpoint for real chart data
- **Impact:** Chart endpoint serves real OHLCV instead of synthetic data
- **Priority:** Medium

#### B. Redis State Cache
- **Current:** `Arc<RwLock<SystemSnapshot>>` in single Rust process
- **Solution:** Redis `SET snapshot:{account_id}` with TTL for multi-instance support
- **Alternative:** NATS KV achieves the same without new dependency
- **Priority:** Low

#### C. MongoDB Trade Blotter
- **Current:** SQLite ledger for double-entry journal
- **Solution:** MongoDB for flexible trade documents with aggregation pipeline
- **Impact:** Better compliance/audit, schema-free order types
- **Priority:** Low

### 2.3 Go Opportunities

**No Go code exists today.** Four candidates:

| Component | Lines | Dependencies | Replaces |
|-----------|-------|-------------|----------|
| NATS-to-QuestDB bridge | ~50 | `nats.go` | Python questdb_client.py (98 lines) |
| API Gateway | ~150 | stdlib only | PWA multi-port polling, ports.ts |
| Service Supervisor | ~200 | stdlib only | 7+ shell scripts (~500 lines) |
| Snapshot Materializer | ~80 | `nats.go` | C++ write_snapshot_json (100 lines) |

### 2.4 Code Deduplication

#### A. Python BrokerClientBase
- **17 broker clients**, ~6,000 lines total
- All repeat: env-var loading, session setup, sandbox toggle, HTTP helpers
- Extract to `base_client.py`, estimated reduction: 1,500-2,000 lines

#### B. Rust Swiftness Proxy Helper
- 5 nearly identical ~60-line handlers in `rest.rs`
- Extract `proxy_get`/`proxy_json` helper, reduction: ~250 lines

#### C. TypeScript useFetchJSON Hook
- 4 hooks repeat fetch → parse → setState with loading/error
- Generic `useFetchJSON<T>` hook, reduction: ~200 lines

#### D. C++ Snapshot Writer Extraction
- 100-line function embedded in 1000-line main entry point
- Move to `native/src/snapshot_writer.cpp/.h`
- May be superseded by NATS KV approach

---

## 3. Summary Metrics

| Category | Dead Lines | Refactor Savings | New Lines (Go/helpers) |
|----------|-----------|-----------------|----------------------|
| Python dead code | ~3,300 | — | — |
| Protobuf/gRPC dead code | ~330 | — | — |
| WASM dead code | ~350 | — | — |
| One-shot scripts | ~4,000 | — | — |
| Python deduplication | — | ~1,800 | ~200 |
| Rust deduplication | — | ~250 | ~30 |
| TypeScript deduplication | — | ~540 | ~100 |
| Go new components | — | ~700 | ~480 |
| **Totals** | **~7,980** | **~3,290** | **~810** |

**Net reduction potential: ~10,460 lines** (dead code removal + deduplication - new code)

---

## 4. Recommended Execution Order

1. **Dead code removal** (zero risk, immediate cleanup)
2. **Python BrokerClientBase** (highest deduplication value)
3. **Enable NATS JetStream** (unlocks MQ refactoring)
4. **Go NATS-to-QuestDB bridge** (first Go component, self-contained)
5. **NATS heartbeat health** (depends on JetStream)
6. **QuestDB query layer** (depends on bridge populating data)
7. **Go API Gateway** (depends on stable service topology)
8. **Remaining low-priority items**
