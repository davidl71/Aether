# AI Editors, Skills & Subagents Setup

## First-time setup (required after cloning or switching machines)

MCP configs are machine-local (paths differ between macOS and Linux). Generate them once:

```bash
./scripts/setup_mcp.sh
```

This writes `.cursor/mcp.json` and `opencode.json` for your machine. Both files are gitignored; `opencode.json.example` is the committed template.

---

This project is structured so **Codex**, **OpenCode**, **Claude Code**,
**Cursor**, **Cursor/plugin skills**, and **subagents** (e.g. Cursor
`mcp_task`, exarp-go, Claude custom agents) all use the same canonical context
and commands.

## Product use case framing

AI tools should treat Aether as a **relative-value financing console** for
comparing and managing multiple financing instruments, not as a narrow box
spread bot and not as a generic ticker/watchlist app.

Primary operator workflow:

- Explore opportunities across **bonds, T-bills, synthetic boxes, bond ETFs,
  and bank loans**
- Express one instrument in terms of another on a common financing basis
- Explore specific synthetic instruments through instrument-level views and
  **OHLCV candle bars** where a synthetic time series is meaningful
- Inspect and manage configuration plus daemon/service health from the TUI
- Support portfolio allocation and execution decisions using cross-instrument
  comparison rather than isolated per-instrument screens

This framing should shape product suggestions, TUI/CLI workflows, analytics,
and naming. The "Ticker vs Console" distinction matters here: compact terminal
patterns from tracking apps are useful, but Aether should be modeled as an
operator console for financing alternatives, relative-value discovery, and
synthetic instrument exploration. User-facing configuration and routine daemon
health should be treated as first-class TUI workflows.

## Canonical context (single source of truth)

| Priority | File | Purpose |
|----------|------|---------|
| 1 | [AGENTS.md](../AGENTS.md) | Full project guidelines (all AI tools) |
| 2 | [CLAUDE.md](../CLAUDE.md) | Claude quick reference; references AGENTS.md |
| 3 | [CODEX.md](../CODEX.md) | Codex quick reference; references AGENTS.md |
| 4 | [ARCHITECTURE.md](../ARCHITECTURE.md) | System architecture |
| 5 | [.cursor/rules/](../.cursor/rules/) | Cursor glob-based rules (in Cursor) |

**Rule:** Prefer updating AGENTS.md for shared rules, plus `CLAUDE.md` or
`CODEX.md` for tool-specific quick references, over duplicating instructions in
tool-specific configs. Tool-specific files should only add *how* to invoke
things (e.g. commands, MCP), not *what* the project rules are.

---

## By tool

### OpenCode

- **Config:** [opencode.json](../opencode.json) — MCP configuration.
- **Commands:** [.opencode/commands/](../.opencode/commands/) — `prime-context`,
  `build`, `test`, `test-one`, `lint`, `review-file`, `write-tests`,
  `ai-context`.
- **Usage:** Run commands via OpenCode UI; start with `ai-context` or
  `prime-context` to load the canonical context files.

### Codex

- **Instructions:** [CODEX.md](../CODEX.md) for Codex-specific quick reference;
  [AGENTS.md](../AGENTS.md) remains the canonical source.
- **Workflow:** Codex should use the same shell commands, docs, and exarp-go
  patterns as the other editors instead of maintaining a separate workflow.
- **Best practice:** If shared AI guidance is wrong or incomplete, fix
  `AGENTS.md`, `CODEX.md`, or these docs rather than inventing a Codex-only rule.

### Claude Code

- **Instructions:** [CLAUDE.md](../CLAUDE.md) (and [AGENTS.md](../AGENTS.md) for
  full guidelines).
- **Custom agents:** [.claude/agents/](../.claude/agents/) — e.g. `code-reviewer`,
  `test-writer`, `trading-reviewer`, `refactor`.
- **Permissions:** [.claude/settings.json](../.claude/settings.json).

### Cursor

- **Rules:** [.cursorrules](../.cursorrules) (main) and
  [.cursor/rules/*.mdc](../.cursor/rules/) (glob-based).
- **Commands:** [.cursor/commands.json](../.cursor/commands.json) — build,
  test, lint, exarp, etc.
- **MCP:** [.cursor/mcp.json](../.cursor/mcp.json) — exarp-go, Context7,
  Ollama, etc.
- **Docs:** [.cursor/rules/ai-context-standards.mdc](../.cursor/rules/ai-context-standards.mdc)
  maps all context files.

### Cursor / plugin skills

- **Context:** Skills should rely on the same canonical context: AGENTS.md,
  the relevant tool quick reference, and (when relevant) `.cursor/rules/`.
- **Project skills:** This repo defines reusable workflows in
  [.cursor/skills/](../.cursor/skills/). Use them by opening or @-mentioning the
  skill file when asking the AI to perform the task (e.g. pull with WIP, add
  native module, before-commit checklist, trading safety, build from clean). See
  [.cursor/skills/README.md](../.cursor/skills/README.md) for the index.
  **UI/UX Pro Max** (design systems for web/PWA) is installed under
  `.cursor/skills/ui-ux-pro-max/` (UI_UX_PRO_MAX_SKILL.md removed; see `.cursor/skills/` for UI/UX skills).
- **Discovery:** Project root = workspace root; key files are at repo root and
  under `.cursor/`, `.claude/`, `.opencode/`.
- **Invocation:** Use the Skill tool with the skill path; skills can read
  AGENTS.md plus the relevant quick reference for project rules.

### Subagents (mcp_task, exarp-go, Claude agents)

- **Context:** Subagents receive task descriptions and optional file paths;
  they do not automatically get full workspace context.
- **Best practice:** When launching a subagent (e.g. `mcp_task` with
  explore/shell/code-reviewer), include in the prompt:

  - Project root path.
  - Pointer: "Canonical project guidelines: AGENTS.md in repo root; use
    CLAUDE.md or CODEX.md as the relevant quick reference."
  - Build/test: "Build: `ninja -C build` or use CMake presets; tests:
    `ctest --test-dir build --output-on-failure`."
- **Reference:** Full list of subagents and when to use each:
  [docs/archive/SUBAGENTS_REFERENCE.md](SUBAGENTS_REFERENCE.md). Project skill:
  [.cursor/skills/when-to-use-subagents.md](../.cursor/skills/when-to-use-subagents.md).
- **exarp-go:** Uses PROJECT_ROOT; session prime and other tools can attach
  task context. Same canonical context applies.
- **Claude custom agents:** Each agent in `.claude/agents/` can reference
  AGENTS.md and CLAUDE.md in its instructions.

---

## Command parity (build / test / lint)

Use these so behavior is consistent across Codex, OpenCode, Claude, and Cursor:

| Action | Codex | OpenCode | Cursor command | Shell |
|--------|-------|----------|----------------|-------|
| Prime context | Read `AGENTS.md`, `CODEX.md`, key docs | `ai-context` or `prime-context` | — | — |
| **exarp-go: prime / handoff / tasks / scorecard** | **See [docs/archive/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](EXARP_GO_CURSOR_CLAUDE_OPENCODE.md)** — same MCP tools in all three; OpenCode: handoff, tasks, scorecard; Claude: prime, handoff, tasks, scorecard; Cursor: session-prime hook + commands. |
| Build | `cmake --build --preset <preset>` | `build` | `build:debug` | `ninja -C build` |
| Test | `ctest --preset <preset> --output-on-failure` | `test` | `test:run` | `ctest --test-dir build --output-on-failure` |
| Lint | `./scripts/run_linters.sh` | `lint` | `lint:run` | `./scripts/run_linters.sh` |

Codex and Cursor should prefer CMake presets (e.g. `macos-arm64-debug`) when
available; see [.cursor/commands.json](../.cursor/commands.json),
[CLAUDE.md](../CLAUDE.md), and [CODEX.md](../CODEX.md).

---

## Hooks

- **Pre-commit (framework):** `.pre-commit-config.yaml` — install with `pre-commit install`. Runs trailing-whitespace, end-of-file-fixer, check-yaml, check-toml, mixed-line-ending, shfmt, shellcheck. Markdownlint is not in pre-commit (see [LINT_AND_AUTOMATION_MATRIX.md](LINT_AND_AUTOMATION_MATRIX.md)).
- **Optional docs validation:** `scripts/setup_pre_commit_hook.sh` — installs a custom pre-commit hook that validates docs when `docs/API_DOCUMENTATION_INDEX.md` is staged.
- **exarp-go:** Use the `setup_hooks` MCP tool (action: git or patterns) to install git hooks or automation patterns; pass `workingDirectory` = this project’s root.

See [.cursor/rules/hooks.mdc](../.cursor/rules/hooks.mdc) and [LINT_AND_AUTOMATION_MATRIX.md](LINT_AND_AUTOMATION_MATRIX.md) for details.

---

## Adding or changing AI context

1. **Project-wide rules:** Update [AGENTS.md](../AGENTS.md) (and
   [CLAUDE.md](../CLAUDE.md) or [CODEX.md](../CODEX.md) if tool-specific).
1. **Cursor-only, file-type rules:** Add or edit `.cursor/rules/*.mdc` with
   the right `globs`.
1. **New OpenCode command:** Add a `.md` file under
   [.opencode/commands/](../.opencode/commands/).
1. **New Cursor command:** Add an entry to
   [.cursor/commands.json](../.cursor/commands.json).
1. **New Claude agent:** Add a `.md` under
   [.claude/agents/](../.claude/agents/) and reference AGENTS.md/CLAUDE.md.
1. **New project skill:** Add a `.md` under
   [.cursor/skills/](../.cursor/skills/) and list it in
   [.cursor/skills/README.md](../.cursor/skills/README.md).

See also:
[.cursor/rules/ai-context-standards.mdc](../.cursor/rules/ai-context-standards.mdc).
