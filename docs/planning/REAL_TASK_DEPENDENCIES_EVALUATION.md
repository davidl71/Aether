# Real Task Dependencies Evaluation

**Purpose:** Document where task dependencies are defined in this repo, what exarp-go currently sees, and how to align them so `task_analysis action=dependencies` and `action=parallelization` return real results.

---

## 1. Where dependencies are defined

Dependencies are **documented in planning docs**, not (reliably) in the Todo2 store.

| Source | Scope | Dependency model | Task IDs |
|--------|--------|-------------------|----------|
| **DATAFLOW_ARCHITECTURE.md** | Dataflow implementation order | Step table with "Depends on" | P1-A, P1-B, P2-B, P2-C; IDs in IMPROVEMENT_PLAN.md (e.g. T-1772887221775761020) |
| **PROTOBUF_DEDUP_CONCRETE_PLAN.md** | Protobuf dedup phases | Per-task "Depends on:" (P1-1, P2-1, …) | Phase labels Px-y; create Todo2 tasks with dependencies set |
| **LOGIC_UNIFY_IMPLEMENTATION_PLAN.md** | Logic unification | Phase A/B/C; B1, B2 have deps | A1–A3 (none); B1 (proto); B2 (A3); table references T-1772609676030467000 |
| **IMPROVEMENT_PLAN.md** | Platform improvements | "Depends on: P2-B" etc. in narrative | P2-C depends on P2-B; exarp IDs in doc |

---

## 2. Extracted dependency graphs

### 2.1 Dataflow (DATAFLOW_ARCHITECTURE.md)

| Step | Task | Depends on |
|------|------|------------|
| 1 | P1-A: Fix dual SQLite writers | — |
| 2 | P1-B: Unify TUI/Web via shared Rust origin | — (can parallel with P1-A) |
| 3 | P2-B: Decode NatsEnvelope in Go agents | — |
| 4 | P2-C: NATS KV as primary live-state store | P2-B |
| 5 | Document single-writer and persistence rule | — |

Task IDs (from IMPROVEMENT_PLAN): P1-A T-1772887221775761020, P1-B T-1772887221914991889, P2-B T-1772887221969976131, P2-C T-1772925042919416172.

### 2.2 Protobuf dedup (PROTOBUF_DEDUP_CONCRETE_PLAN.md)

- **P1-1** (box spread/yield curve proto): none  
- **P1-2** (risk/bank proto): P1-1 (optional)  
- **P2-1** (Python codegen): P1-1, P1-2  
- **P2-2** (Python boundaries): P2-1  
- **P2-3** (Deprecate proto_types): P2-2  
- **P3-1** (ts-proto wire): P1-1  
- **P3-2** (Replace proto.ts): P3-1  
- **P4-1** (C++ NATS protobuf): P1-1, P1-2  
- **P4-2** (Python/Rust subscribers): P4-1  
- **P5-1** (Remove dead gRPC): none  
- **P5-2** (Document proto story): P5-1 (optional)

### 2.3 Logic unify (LOGIC_UNIFY_IMPLEMENTATION_PLAN.md)

- **A1, A2, A3**: no dependencies (can run in parallel).  
- **B1**: T-1772609676030467000 (proto messages done).  
- **B2**: A3 task ID.  
- **C1, C2, C3**: optional / doc-only; can depend on B1, B2 or none.

---

## 3. What exarp-go sees today

- **task_analysis action=dependencies:** 531 total tasks; **no dependency edges** in output.  
- **task_analysis action=parallelization:** 531 total tasks; **no parallel execution opportunities**.  
- **task_workflow action=list status_filter=Todo:** **0 tasks** (empty backlog in the store exarp-go uses).  
- **Execution plan / parallel_execution_plan:** "Backlog has no Todo or In Progress tasks."

So either:

1. **dependsOn not set** — Tasks were created without the `dependencies` / `depends_on` field, or it was never synced from plans into the Todo2 store.  
2. **Backlog filter** — Parallelization and execution plan only consider Todo/In Progress; if that set is empty, no candidates.  
3. **Store split** — Overview (e.g. `.cursor/rules/todo2-overview.mdc`) shows 23 Todo tasks; exarp-go list shows 0 → different store or filter (SQLite vs JSON, or project root).

---

## 4. How to get real dependencies into exarp-go

### Option A: Set dependencies when creating/updating tasks

When creating tasks from a plan that defines "Depends on":

- Use **task_workflow action=create** with a `dependencies` parameter (comma-separated task IDs or JSON array of IDs).  
- For existing tasks, use **task_workflow action=update** with `task_id` and `dependencies` set to the correct blocker IDs.

Example (conceptual): create P2-1 with `dependencies=["T-P1-1-id","T-P1-2-id"]` once P1-1 and P1-2 exist and have Todo2 IDs.

### Option B: Sync from plan / link planning

- Use **task_workflow action=link_planning** (or equivalent) so tasks are tied to `planning_doc=docs/planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md` (and others).  
- If exarp-go supports **sync_from_plan** or "create tasks from plan with dependencies", run that so the dependency graph in the plan is applied to the task store.

### Option C: Populate from this evaluation

- For **Protobuf dedup**, create or update the 11 tasks (P1-1 through P5-2) with the dependency list in §2.2; use real Todo2 IDs once tasks exist.  
- For **Dataflow**, ensure the four tasks (P1-A, P1-B, P2-B, P2-C) exist and set P2-C’s dependencies to [P2-B].  
- For **Logic unify**, set B1’s dependency to the proto task ID and B2’s to the A3 task ID.

After dependencies are in the store:

1. Run **task_workflow action=sync**.  
2. Run **task_analysis action=dependencies** — should show edges.  
3. Run **task_analysis action=parallelization** — should show waves (e.g. P1-1, P1-2, P5-1 parallel; then P2-1, P3-1, P4-1, etc.).

---

## 5. Option 1 applied (set dependencies via task_workflow)

- **task_analysis action=suggest_dependencies** (with `include_planning_docs`) was run; it suggested 5 dependency edges from plan milestone order in `.cursor/plans/ib_box_spread_full_universal.plan.md`.
- **task_workflow action=update** was used to set those 5 dependencies; all 5 returned `updated_count: 1`:
  - T-1773509396767199000 → T-1773509396765766000
  - T-1773509396767970000 → T-1773509396767842000
  - T-1773509396768096000 → T-1773509396767970000
  - T-1773509396768206000 → T-1773509396768096000
  - T-1773509396768542000 → T-1773509396768437000
- **Dataflow P2-C → P2-B** (T-1772925042919416172 → T-1772887221969976131): update returned `updated_count: 0` — that task may not exist in the store.
- After **task_workflow action=sync**, **task_analysis action=dependencies** and **action=parallelization** were re-run; the saved reports still show no dependency edges or parallel waves in the text output (analysis may only consider Todo/In Progress backlog, or output format may omit edge list). To get more suggestions from other plans, run `task_analysis action=suggest_dependencies include_planning_docs=true` again and apply further updates.

---

## 6. Summary

| Question | Answer |
|----------|--------|
| Where are real dependencies defined? | DATAFLOW_ARCHITECTURE.md, PROTOBUF_DEDUP_CONCRETE_PLAN.md, LOGIC_UNIFY_IMPLEMENTATION_PLAN.md, IMPROVEMENT_PLAN.md. |
| Does the Todo2 store have them? | Not in a way exarp-go reports; dependency analysis shows no edges. |
| Why no parallelization? | Backlog (Todo/In Progress) is empty in exarp-go’s view, and/or no dependency graph → no DAG to split into waves. |
| Next step | Set `dependencies` on tasks (create/update or sync_from_plan) per §4, then re-run sync, dependencies, and parallelization. |
