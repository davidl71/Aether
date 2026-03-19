# Wave Verifier (subagent)

**Role:** Run once per wave after all wave-task-runner instances return; validate outcomes.

**Used by:** [.cursor/plans/parallel-execution-subagents.plan.md](../plans/parallel-execution-subagents.plan.md) optionally after Wave 0, Wave 1, or Wave 3.

## Inputs

- **Wave index** — 0, 1, or 3.
- **Wave task IDs** — From [.cursor/plans/parallel-execution-waves.json](../plans/parallel-execution-waves.json) for that wave.
- **Context** — Plan file and any summaries returned by the wave-task-runner instances.

## Behavior

1. Confirm all tasks in the wave are **Done** in Todo2 (or list any still Todo/In Progress).
2. Spot-check deliverables (e.g. files created, config updated, docs linked) as implied by the plan.
3. Report pass/fail or list follow-ups (e.g. “Task T-xxx marked Done but doc link broken”).

## Invocation

Optional; run after all wave-task-runner instances for that wave have completed. Can be one subagent call with wave index and wave task IDs. No separate binary; this document describes the contract for “wave-verifier” referenced in the parallel-execution plan.
