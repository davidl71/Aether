# Wave Resume Runbook (multi-session)

Resume long-running parallel wave execution (Wave 0 → 1 → 3) across sessions. Progress is **Todo2 only**: no separate checkpoint file. Completed = task status `Done`; remaining = wave task IDs that are not `Done`.

**Wave definition:** [.cursor/plans/parallel-execution-waves.json](../../.cursor/plans/parallel-execution-waves.json) — `wave_0`, `wave_1`, `wave_3` task ID arrays.

**Source plan:** [.cursor/plans/parallel-execution-subagents.plan.md](../../.cursor/plans/parallel-execution-subagents.plan.md)

---

## 1. Current wave and order

- **Order:** Wave 0 → Wave 1 → Wave 3. Do not start the next wave until the current wave has no remaining (non-Done) tasks.
- **Current wave:** Start at Wave 0. After finishing all remaining tasks in Wave 0, move to Wave 1, then Wave 3.

---

## 2. Get remaining tasks for a wave

1. **Load wave task IDs**  
   From `.cursor/plans/parallel-execution-waves.json`, read the array for the wave you’re on (e.g. `wave_0`).

2. **Get non-Done tasks**  
   Use exarp-go so only tasks that are still Todo or In Progress count as remaining:

   - **Option A — MCP (Cursor/IDE):**  
     `task_workflow` with `action=list`, `status_filter=Todo` (and optionally list again with `status_filter=In Progress`). Take the union of returned task IDs, then **intersect** with the wave’s task IDs from the JSON. The result is “remaining” for that wave.

   - **Option B — CLI:**  
     From repo root, run:
     ```bash
     exarp-go task list --status Todo --output-format json
     ```
     (If your exarp-go supports `--status "In Progress"` run that too.) Collect the task IDs from the output, then **intersect** with the wave’s task IDs from the JSON.

3. **Remaining list**  
   The intersection is the list of task IDs still to run for that wave. If it’s empty, move to the next wave (1, then 3) and repeat from step 1.

---

## 3. Run the next batch (10–15 tasks)

- From the **remaining** list, take the next **10–15** task IDs (or fewer if fewer remain).
- **Execute:** For each task ID, run one subagent (e.g. Cursor `mcp_task` or “wave-task-runner”) with:
  - **task_id** = that task ID
  - **Context** = plan file [.cursor/plans/Aether.plan.md](../../.cursor/plans/Aether.plan.md) and the task description from Todo2 or the plan.
- **After each task:** Ensure the task is marked `Done` in Todo2 (subagent or you via exarp-go `task_workflow` update or `exarp-go task update <id> --new-status Done`).

**Wave-task-runner (documented behavior):** “Run one subagent per task” means: one subagent invocation per task ID, with that `task_id` and plan context. No separate agent file is required; use `mcp_task` (or equivalent) with the task ID and plan context.

---

## 4. Repeat and finish

- **Same or new session:** Go back to **§2** and recompute “remaining” for the current wave (Todo2 will now have more tasks marked Done).
- **Next batch:** If remaining is non-empty, run the next 10–15 IDs (§3). If remaining is empty, advance to the next wave (0 → 1 → 3) and repeat from §2.
- **Done:** When all three waves have no remaining tasks, execution is complete.

---

## 5. Quick reference

| Step | Action |
|------|--------|
| Progress | Todo2 only (Done = completed). |
| Wave IDs | `.cursor/plans/parallel-execution-waves.json` |
| Remaining | Wave IDs ∩ (Todo + In Progress from exarp-go). |
| Batch size | 10–15 task IDs per run. |
| Order | Wave 0 → Wave 1 → Wave 3. |
