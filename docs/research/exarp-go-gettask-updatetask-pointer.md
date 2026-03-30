# GetTask / UpdateTask research (pointer)

**Todo2:** T-1774888049604959000  

Canonical design note and §4 alignment live in the **exarp-go** repository (SQLite Todo2 implementation):

- `exarp-go` clone: `docs/research/gettask-updatetask-churn.md` — full design note  
- `exarp-go` clone: `docs/OPTIMIZATION_RESULTS.md` §4 — benchmark context + link (updated 2026-03-30)  

If this repo sits next to `exarp-go` under `Projects/` (e.g. `Projects/Trading/Aether` and `Projects/mcp/exarp-go`), a relative path from Aether root is `../../mcp/exarp-go/docs/research/gettask-updatetask-churn.md`.

**Profiling:** From exarp-go root, run database benchmarks listed in §4 (`BenchmarkGetTask`, `BenchmarkUpdateTask`, etc.).
