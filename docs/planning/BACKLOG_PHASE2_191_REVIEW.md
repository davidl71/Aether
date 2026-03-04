# Phase 2: 191 Todo2 Tasks Not in Shared TODO – Review

**Date:** 2026-02-27  
**Task:** T-1772111114876005000  
**Goal:** Decide which Todo2-only tasks appear in shared TODO; document count.

## Counts

- **Shared TODO** ([agents/shared/TODO_OVERVIEW.md](../agents/shared/TODO_OVERVIEW.md)): ~55 rows (numeric IDs 4–44, CI-1–CI-5, 6 parallel-exec IDs, plus new backlog section).
- **Todo2** (from `.todo2/state.todo2.json` in workspace): 344 tasks total (per alignment report). Many use long numeric IDs (e.g. T-1772135684202624000); shared table uses short IDs (4, 5, …) or T-20251129… style.
- **Gap:** “191” = Todo2 tasks that have no matching row in the shared TODO table (by ID or description match). These remain Todo2-only; no need to add all 191 to the shared table.

## Actions taken

1. **High-priority backlog added to shared TODO**  
   The 4 dependency-ready “suggested next” tasks from Todo2 (review misaligned, review 191, ts-proto codegen, memcached C++ cache) are now listed in `agents/shared/TODO_OVERVIEW.md` under **Backlog (Todo2 suggested next)** so agents see them.

2. **Review tasks marked completed in shared TODO**  
   T-1772206054086289000 (review misaligned) and T-1772111114876005000 (review 191) are marked **completed** in the shared table upon completion of Phase 1 and Phase 2.

3. **No bulk sync**  
   `scripts/automate_todo_sync.py` was not present in the repo; no automated sync run. Manual update of shared TODO as above.

## Outcome

- **In shared TODO:** Existing rows + 4 new backlog rows (2 completed, 2 pending).
- **Todo2-only:** Remaining ~191+ tasks stay in Todo2; high-priority ones that are dependency-ready are now also in the shared table for visibility.
