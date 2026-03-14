# Phase 1: Misaligned Tasks Review Log

**Date:** 2026-02-27  
**Task:** T-1772206054086289000 (Review misaligned tasks)  
**Source:** TODO2 alignment (TODO2_ALIGNMENT_REPORT.md removed; 68 misaligned tasks at time of review)

## Summary of decisions

| Category | Count | Action |
|----------|-------|--------|
| Already Done in Todo2 | 18 | No change; alignment considers them misaligned because not in shared TODO or different phrasing |
| Still Todo / backlog | 42 | Left in Todo2; high-priority ones added to shared TODO below |
| Meta / duplicate review tasks | 2 | Mark Done when this review is complete (T-1772206054086289000, T-1772130709755428000) |
| Infrastructure / completed work | 6 | Treated as Done (Ansible memcached, Justfile proto-gen, etc.) |

## Per-task classification

### Meta and review tasks

- **T-1772206054086289000** Review misaligned tasks — *This task; mark Done after review.*
- **T-1772111114876005000** Review 191 Todo2 tasks not in shared TODO — *Phase 2; mark Done after that review.*
- **T-1772130709755428000** Review misaligned tasks — *Duplicate of T-1772206054086289000; cancel or mark Done with other.*

### Already Done (per Todo2 overview / conversation)

- T-1772142920963275000 Phase 6: Discount bank Rust canonical
- T-1772142918323013000 Phase 5: Python risk call C++ or document as stub
- T-1772142915869231000 Phase 3: C++ protobuf codegen and boundary adapter
- T-1772142857852296000 Phase 2: Python use generated proto types at NATS/REST
- T-1772142856323561000 Phase 4: Python box spread thin wrapper
- T-1772142854494578000 Phase 1: Extend proto/messages.proto
- T-1772142847778941000 Cross-language dedup: Protobuf + single-source strategy
- T-1772136087097427000 Add memcached to Ansible provisioning — *Done (in ansible/roles/devtools).*
- T-1772135684402673000 Add Justfile recipe for protobuf codegen — *Done (Justfile has proto-gen).*
- T-1772135684202624000 Fix C++ toolchain - Xcode Command Line Tools headers — *Done (committed).*

### Infrastructure / partially done

- T-1772136087069653000 Create CacheClient abstraction with memcached backend — *C++ and Python abstractions exist; Memcached backend in C++ is Phase 4.*
- T-1772136087147162000 Add memcached cache to Python integration layer — *Python cache_client exists; pymemcache integration can be verified separately.*

### Still todo (backlog; no status change)

Remaining 50+ tasks from the misaligned list are left as-is in Todo2. High-priority or agent-visible ones are added to shared TODO in Phase 2 (agents/shared/TODO_OVERVIEW.md removed).

## Outcome

- No bulk status changes applied to Todo2 (no script/MCP available in this run).
- This log serves as the decision record; user or automation can mark T-1772206054086289000 and T-1772130709755428000 Done and align any Done tasks in Todo2 if needed.
- Shared TODO updated in Phase 2 with high-priority backlog items.
