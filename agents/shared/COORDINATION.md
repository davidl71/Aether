# Coordination Guidelines

To keep all agents aligned and give the master planner continuous feedback, please follow these steps:

> Backend work is now split between `backend-agent` (core platform), `backend-mock-agent` (mock TWS pipeline), and `backend-data-agent` (combo + liquidity feeds). Coordinate hand-offs explicitly in the shared TODO table.

## 1. Update the Shared TODO Table
- For every task you touch, update `agents/shared/TODO_OVERVIEW.md`:
  - `pending` → `in_progress` when you start work.
  - `in_progress` → `completed` when the change is merged.
  - Optionally add short notes if the task is blocked or needs follow-up.
- This table is the single source of truth for overall status.

## 2. Capture API & Design Changes
- When you modify snapshot payloads or command endpoints, immediately update `agents/shared/API_CONTRACT.md` so all front ends stay compatible.
- If you discover recurring problems or edge cases, record them in `agents/shared/KnownIssues.md`.
- Link to any relevant pull requests or commits in those files so the planner can trace the context quickly.

## 3. CI Monitoring
- Jenkins (or GitHub Actions) runs the multi-stage pipeline defined in `agents/shared/CI.md` and the root `Jenkinsfile`.
- If a stage fails, check the pipeline logs, add an entry to `KnownIssues.md` if it’s a recurring problem, and update the TODO table if the fix requires a new task.
- Configure Jenkins notifications (email/Slack/etc.) as needed so the planner is alerted automatically.

Following these conventions ensures the master planner always has up-to-date information without needing separate status meetings.

