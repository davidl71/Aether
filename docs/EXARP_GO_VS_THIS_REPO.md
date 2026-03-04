# exarp-go vs this repo: what we don't duplicate

This repo uses **exarp-go as MCP** (and optionally CLI) for tasks, docs health, security, reports, and session prime. We also ported some **patterns** (Ansible SSL fix, CA certs, run-dev-setup flow, Justfile recipes). This doc clarifies what is exarp-go's job vs what we do locally so we don't duplicate built-in exarp-go functionality.

---

## We use exarp-go for (do not reimplement)

| Need | Use | Notes |
|------|-----|--------|
| Task list / create / update / show | exarp-go `task_workflow` or CLI `exarp-go task list` | Set `workingDirectory` to this repo root |
| Docs health, git/CI status | exarp-go `health` tool | |
| Project overview, scorecard, briefing | exarp-go `report` (overview, scorecard, briefing) | |
| Session context at conversation start | exarp-go `session` with `action=prime` | |
| Security scan (multi-language) | exarp-go `security` with `action=scan` | |
| Duplicate task detection | exarp-go `detect_duplicate_tasks_tool` | |
| Task alignment | exarp-go `analyze_todo2_alignment_tool` | |
| Lint (Go, shell, markdown, C++, etc.) | exarp-go `lint` tool (e.g. `linter=shellcheck`, `linter=auto`, `path=...`) | Via `./scripts/run_exarp_go_tool.sh lint` or MCP |

These stay in exarp-go; we only invoke them (MCP or `run_exarp_go.sh` / `run_exarp_go_tool.sh`).

---

## Overlap: shellcheck

- **exarp-go:** The **lint** tool can run shellcheck when invoked with `linter=shellcheck` and a path (e.g. `path=scripts`). So `./scripts/run_exarp_go_tool.sh lint` with appropriate args can run shellcheck.
- **This repo:** We added **`just lint-shell`**, which runs `shellcheck -x scripts/*.sh ansible/run-dev-setup.sh` **locally** without needing the exarp-go binary.

**Why keep both:**  
`just lint-shell` is a **local shortcut** for CI or when exarp-go isn't installed; exarp-go lint is the unified entry point when you have exarp-go (and can run shellcheck plus other linters). We are **not** reimplementing exarp-go—we're providing a fallback for shellcheck-only runs.

**When to use which:**  
- Prefer **exarp-go lint** (e.g. `run_exarp_go_tool.sh lint`) when exarp-go is available and you want all linters.  
- Use **`just lint-shell`** when you only want shellcheck or exarp-go isn't on PATH.

---

## No overlap: Ansible and CA certs

- **Ansible dev setup:** exarp-go does **not** run Ansible for *this* project. Its own repo has `ansible/run-dev-setup.sh` for setting up the exarp-go dev environment. We **ported the pattern** (SSL fix, galaxy, syntax-check, run playbook) into **our** `ansible/run-dev-setup.sh` and **`just ansible-dev`** for *this* project's dev environment. So no duplication of exarp-go functionality.
- **CA certs in Ansible:** exarp-go's **common** role updates CA trust and verifies the bundle on the machine where exarp-go is developed. We ported that **pattern** into our **devtools** role for the machine where this repo is developed. Again, same idea, different scope; exarp-go doesn't expose "configure CA certs on the client project" as a tool.

---

## Summary

| Area | Duplicating exarp-go? | Notes |
|------|------------------------|--------|
| Tasks, health, report, session, security, alignment, duplicate detection | No | We only call exarp-go |
| Lint (including shellcheck) | Partial | exarp-go lint can run shellcheck; `just lint-shell` is a local shellcheck-only shortcut |
| Ansible run script + SSL | No | exarp-go doesn't run our Ansible; we ported the pattern |
| CA certs in Ansible | No | We ported the pattern into our devtools role; exarp-go has no "setup CA" tool for this repo |

So we are **not** duplicating built-in exarp-go functionality except for offering a **local shellcheck path** (`just lint-shell`) when exarp-go isn't used; that is intentional for CI and environments without exarp-go.
