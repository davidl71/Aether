---
name: wave-runner
description: Run parallel-execution waves (Wave 0, 1, 2) or next batch of tasks from the plan. Use when the user says "run wave", "run wave 1", "run next batch", "continue waves", or "execute wave 2".
---

# Wave Runner

Execute tasks from the parallel-execution plan in waves. Progress is Todo2-only: remaining = wave task IDs that are not `Done`.

## Wave definition and order

- **Wave IDs:** `.cursor/plans/parallel-execution-waves.json` — keys `wave_0`, `wave_1`, `wave_2` (arrays of task IDs).
- **Order:** Wave 0 → Wave 1 → Wave 2. Do not start the next wave until the current wave has no remaining (non-Done) tasks.
- **Plan context:** `.cursor/plans/parallel-execution-subagents.plan.md` and project plan (e.g. `.cursor/plans/Aether.plan.md`).

## Get remaining for a wave

1. **Load wave IDs** — Read the array for the requested wave from `parallel-execution-waves.json` (e.g. `wave_0`).
2. **Get non-Done tasks** — Remaining = wave IDs ∩ (Todo + In Progress from Todo2).
   - **Option A (MCP/CLI):** `task_workflow` with `action=list`, `status_filter=Todo`, then again `status_filter=In Progress`. Run through the repo wrapper (`./scripts/run_exarp_go.sh -tool task_workflow -args '<json>'` or the equivalent MCP call) so the CLI sees this project’s root. Union the returned task IDs, then intersect with the wave’s IDs.
   - **Option B (script):** From repo root: `python3 scripts/parallel_wave_remaining.py <wave_index> [batch_size]` (wave_index: 0, 1, or 2; batch_size default 15). Prints remaining and next batch.
     - To reduce context switching, you can prefer batching tasks by component tags (repeatable): `--tag tui`, `--tag cli`, `--tag backend`, etc.
     - Example: `python3 scripts/parallel_wave_remaining.py 0 12 --tag cli`
     - If you want a batch that contains *only* matching tasks (may be smaller than batch_size): add `--strict-tag`.
3. If remaining is empty for current wave, advance to next wave (1, then 2) and repeat.

## Run the next batch

- **Batch size:** 10–15 task IDs per run (or fewer if fewer remain).
- **Execute:** For each task ID in the batch, run one subagent (e.g. `mcp_task`) with:
  - **task_id** = that task ID
  - **Context** = plan file + task description from Todo2.
- **After each task:** Ensure the task is marked `Done` in Todo2 (via exarp-go `task_workflow` update run through the repo wrapper or via `task update <id> --new-status Done`, so the tooling always targets this project’s store).

## Quick reference

| Step        | Action |
|------------|--------|
| Progress   | Todo2 only (Done = completed). |
| Wave IDs   | `.cursor/plans/parallel-execution-waves.json` |
| Remaining  | Wave IDs ∩ (Todo + In Progress from exarp-go, queried via `scripts/run_exarp_go.sh` or the MCP `task_workflow` tool). |
| Batch size | 10–15 task IDs per run. |
| Order      | Wave 0 → Wave 1 → Wave 2. |

## Full runbook

See `docs/WAVE_RESUME_RUNBOOK.md` for multi-session resume and detailed options (MCP vs CLI vs helper script).
