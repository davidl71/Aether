# OpenCode and exarp-go AI Context Interaction Patterns

Patterns learned from the **exarp-go** sibling repository and used in this repo for OpenCode, Cursor, and exarp-go MCP integration. Use this doc to align AI context behavior across editors and when cloning or syncing with exarp-go.

---

## 1. OpenCode patterns

### 1.1 Config and discovery

| Item | Location | Purpose |
|------|----------|---------|
| Project config | `.opencode.json` | Description, LSP (clangd, pyright), `data.directory` for commands |
| Commands | `.opencode/commands/*.md` | Slash commands; each file = one command |
| Canonical context | Referenced in commands | AGENTS.md, CLAUDE.md, ARCHITECTURE.md |

**.opencode.json** should reference canonical guidelines and point OpenCode at the command directory so commands are discoverable.

### 1.2 Command format (OpenCode)

Commands are markdown files. Lines define what the assistant does:

- **READ &lt;path&gt;** — Load file into context
- **RUN &lt;shell command&gt;** — Run and use output

**Prime / ai-context:**

- `prime-context.md`: READ AGENTS.md, ARCHITECTURE.md, key source files; RUN git log, ls
- `ai-context.md`: READ AGENTS.md, CLAUDE.md, ARCHITECTURE.md (minimal canonical context)

**Exarp integration:**

- `handoff.md`: Run `./scripts/run_exarp_go.sh -tool session` with `action=handoff`, optional summary/args
- `tasks.md`: Run `./scripts/run_exarp_go.sh -tool task_workflow` with `action=sync`, `sub_action=list`
- `scorecard.md`: Run `./scripts/run_exarp_go.sh -tool report` with `action=overview`; add local metrics

Pattern: OpenCode commands that need exarp-go should use the local CLI wrapper with `RUN` lines. This avoids OpenCode / Anthropic JSON Schema validation failures when exarp-go MCP tool schemas are rejected during startup.

### 1.3 Command parity

Keep behavior consistent across OpenCode, Cursor, and shell:

| Action | OpenCode command | Cursor | Shell |
|--------|------------------|--------|-------|
| Prime context | `ai-context`, `prime-context` | (session-prime hook) | — |
| Build | `build` | `build:debug` | `ninja -C build` |
| Test | `test` | `test:run` | `ctest --test-dir build --output-on-failure` |
| Lint | `lint` | `lint:run` | `./scripts/run_linters.sh` |

See [docs/AI_EDITOR_SETUP.md](AI_EDITOR_SETUP.md) for the full table.

---

## 2. exarp-go AI context patterns

### 2.1 workingDirectory / PROJECT_ROOT

- **MCP:** Every exarp-go tool call must use **`workingDirectory`** set to **this project’s root** (Cursor workspace root).
- **CLI/scripts:** Use **`PROJECT_ROOT`** (or `EXARP_GO_ROOT` for the exarp-go binary). Set via `scripts/run_exarp_go.sh` so exarp-go sees the correct repo (tasks, `.todo2`, etc.).
- **Session prime:** `.cursor/hooks/session-prime.sh` sets `PROJECT_ROOT` and calls `scripts/run_exarp_go.sh` so prime runs in this repo.

### 2.2 Session prime (sessionStart)

- **Trigger:** Cursor sessionStart (new Composer conversation).
- **Script:** `.cursor/hooks/session-prime.sh`
- **Flow:**
  1. Consume stdin (Cursor payload).
  2. Resolve `PROJECT_ROOT`; run `run_exarp_go.sh -tool session -args '{"action":"prime",...}' -json -quiet`.
  3. Parse JSON with `jq`: `status_context`, `status_label`, `suggested_next`, `handoff_alert`, `cursor_cli_suggestion`.
  4. Build `additional_context` string and output Cursor schema: `{ "additional_context": "...", "continue": true }`.
- **Fallback:** If exarp-go or jq missing, return a short message so the session still starts.

Pattern from exarp-go: **session-prime** returns structured context (tasks, suggested next, handoff) so the AI has project state without the user asking.

### 2.3 Handoff

- **Purpose:** Pass context to the next developer or session.
- **OpenCode:** `.opencode/commands/handoff.md` — run `./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff",...}' -json -quiet`.
- **exarp-go:** Saves handoff; session prime can set `handoff_alert` so the next session sees “Review handoff from previous developer.”

### 2.4 Invocation conventions

- **JSON args:** Use explicit `-args '{"action":"...", ...}'` to avoid key=value parsing issues.
- **Stdin in hooks:** Git hooks run with stdin from Git. Redirect with **`</dev/null`** when calling exarp-go so it doesn’t treat refs as JSON-RPC.
- **Quiet in hooks:** Set **`GIT_HOOK=1`** (or equivalent) so exarp-go suppresses noisy logs in hook context.
- **Graceful skip:** If exarp-go isn’t installed or the tool is unknown, **exit 0** so commit/push isn’t blocked.

See [docs/EXARP_GO_GIT_HOOKS_LEARNINGS.md](EXARP_GO_GIT_HOOKS_LEARNINGS.md).

### 2.5 Auto-reprime on project root change

When the user switches project (e.g. different repo):

- Detect change (e.g. different `.git`, `.todo2`, `CMakeLists.txt`, `go.mod`).
- Suggest repriming: “ask exarp to prime my session” or use exarp-go auto_prime / session prime.
- Confirm project root and load project-specific context.

See [.cursor/rules/auto-reprime.mdc](../.cursor/rules/auto-reprime.mdc).

---

## 3. Context hierarchy (single source of truth)

1. **AGENTS.md** — Authoritative project guidelines (all tools).
2. **CLAUDE.md** — Quick reference; references AGENTS.md.
3. **Tool-specific** — Cursor rules, OpenCode commands, exarp-go; reference AGENTS.md, don’t duplicate.
4. **Subagents:** Pass project root + “Canonical guidelines: AGENTS.md, CLAUDE.md in repo root”; they don’t auto-load workspace.

See [.cursor/rules/ai-context-standards.mdc](../.cursor/rules/ai-context-standards.mdc) and [docs/design/AI_AGENT_CONTEXT_STANDARDS.md](design/AI_AGENT_CONTEXT_STANDARDS.md).

---

## 4. Where to look in exarp-go (sibling repo)

When you have the exarp-go repo (e.g. sibling or `EXARP_GO_ROOT`), use it to learn or sync:

| Topic | exarp-go location | This repo |
|-------|-------------------|-----------|
| Session prime | `.cursor/hooks/session-prime.sh` | `.cursor/hooks/session-prime.sh` |
| Handoff / session | MCP session tool, `action=handoff` / `action=prime` | `.opencode/commands/handoff.md`, session-prime.sh |
| Git hooks | `scripts/git-hooks/`, `internal/tools/hooks_setup.go` | `scripts/git-hooks/`, [EXARP_GO_GIT_HOOKS_LEARNINGS.md](EXARP_GO_GIT_HOOKS_LEARNINGS.md) |
| Co-authored-by | `scripts/git-hooks/prepare-commit-msg`, `exarp-go.coauthor` | `scripts/git-hooks/prepare-commit-msg`, `ib_box_spread.coauthor` |
| Ansible + SSL | `ansible/run-dev-setup.sh` | `ansible/run-dev-setup.sh` |
| Dev watch / sanity / MCP stdio | `dev.sh`, `scripts/sanity-check.sh`, `scripts/test-mcp-stdio.sh` | Patterns only — [EXARP_GO_SCRIPTS_AND_PATTERNS.md](EXARP_GO_SCRIPTS_AND_PATTERNS.md) |
| Lint (shell, yaml, ansible) | Makefile lint-* targets | `just lint-shell`, `just ansible-check` |

---

## 5. Checklist: aligning with exarp-go and OpenCode

- [ ] **Canonical context:** AGENTS.md and CLAUDE.md at repo root; tool configs reference them.
- [ ] **OpenCode:** `.opencode.json` + `.opencode/commands/` with `ai-context`, `prime-context`, `handoff`, `tasks`, `scorecard` as needed.
- [ ] **Cursor session prime:** `.cursor/hooks/session-prime.sh` calls exarp-go session prime, returns `additional_context` + `continue`.
- [ ] **exarp-go MCP:** All tools called with `workingDirectory` = this project root.
- [ ] **Handoff:** Handoff command and session prime surface `handoff_alert` in next session.
- [ ] **Git hooks:** Versioned under `scripts/git-hooks/`; exarp-go invocations use `</dev/null`, JSON args, graceful skip when exarp-go missing.
- [ ] **Auto-reprime:** When project root changes, suggest exarp prime and confirm root.
- [ ] **Subagents:** Prompt includes project root and “Canonical guidelines: AGENTS.md, CLAUDE.md”.

---

## Related docs

- [AI_EDITOR_SETUP.md](AI_EDITOR_SETUP.md) — OpenCode, Claude, Cursor, skills, subagents
- [EXARP_GO_GIT_HOOKS_LEARNINGS.md](EXARP_GO_GIT_HOOKS_LEARNINGS.md) — Git hook patterns from exarp-go
- [EXARP_GO_SCRIPTS_AND_PATTERNS.md](EXARP_GO_SCRIPTS_AND_PATTERNS.md) — Scripts and patterns from exarp-go
- exarp-go vs this repo: see [EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](EXARP_GO_CURSOR_CLAUDE_OPENCODE.md) and [project-automation.mdc](../.cursor/rules/project-automation.mdc) (EXARP_GO_VS_THIS_REPO.md removed).
- [.cursor/rules/ai-context-standards.mdc](../.cursor/rules/ai-context-standards.mdc) — Context files and which rule when
- [.cursor/rules/hooks.mdc](../.cursor/rules/hooks.mdc) — Session and git hooks
