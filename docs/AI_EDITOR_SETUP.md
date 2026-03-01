# AI Editors, Skills & Subagents Setup

This project is structured so **OpenCode**, **Claude Code**, **Cursor**,
**Cursor/plugin skills**, and **subagents** (e.g. Cursor `mcp_task`, exarp-go,
Claude custom agents) all use the same canonical context and commands.

## Canonical context (single source of truth)

| Priority | File | Purpose |
|----------|------|---------|
| 1 | [AGENTS.md](../AGENTS.md) | Full project guidelines (all AI tools) |
| 2 | [CLAUDE.md](../CLAUDE.md) | Claude quick reference; references AGENTS.md |
| 3 | [ARCHITECTURE.md](../ARCHITECTURE.md) | System architecture |
| 4 | [.cursor/rules/](../.cursor/rules/) | Cursor glob-based rules (in Cursor) |

**Rule:** Prefer updating AGENTS.md and CLAUDE.md over duplicating instructions in
tool-specific configs. Tool-specific files should only add *how* to invoke
things (e.g. commands, MCP), not *what* the project rules are.

---

## By tool

### OpenCode

- **Config:** [.opencode.json](../.opencode.json) — LSP (clangd, pyright),
  command directory.
- **Commands:** [.opencode/commands/](../.opencode/commands/) — `prime-context`,
  `build`, `test`, `test-one`, `lint`, `review-file`, `write-tests`,
  `ai-context`.
- **Usage:** Run commands via OpenCode UI; start with `ai-context` or
  `prime-context` to load AGENTS.md and CLAUDE.md.

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
  CLAUDE.md, and (when relevant) .cursor/rules.
- **Discovery:** Project root = workspace root; key files are at repo root and
  under `.cursor/`, `.claude/`, `.opencode/`.
- **Invocation:** Use the Skill tool with the skill path; skills can READ
  AGENTS.md and CLAUDE.md for project rules.

### Subagents (mcp_task, exarp-go, Claude agents)

- **Context:** Subagents receive task descriptions and optional file paths;
  they do not automatically get full workspace context.
- **Best practice:** When launching a subagent (e.g. `mcp_task` with
  explore/shell/code-reviewer), include in the prompt:

  - Project root path.
  - Pointer: "Canonical project guidelines: AGENTS.md and CLAUDE.md in repo
    root."
  - Build/test: "Build: `ninja -C build` or use CMake presets; tests:
    `ctest --test-dir build --output-on-failure`."
- **exarp-go:** Uses PROJECT_ROOT; session prime and other tools can attach
  task context. Same AGENTS.md/CLAUDE.md apply.
- **Claude custom agents:** Each agent in `.claude/agents/` can reference
  AGENTS.md and CLAUDE.md in its instructions.

---

## Command parity (build / test / lint)

Use these so behavior is consistent across OpenCode, Claude, and Cursor:

| Action | OpenCode | Cursor command | Shell |
|--------|----------|----------------|-------|
| Prime context | `ai-context` or `prime-context` | — | — |
| Build | `build` | `build:debug` | `ninja -C build` |
| Test | `test` | `test:run` | `ctest --test-dir build --output-on-failure` |
| Lint | `lint` | `lint:run` | `./scripts/run_linters.sh` |

Cursor also has presets (e.g. `macos-arm64-debug`); see
[.cursor/commands.json](../.cursor/commands.json) and [CLAUDE.md](../CLAUDE.md).

---

## Adding or changing AI context

1. **Project-wide rules:** Update [AGENTS.md](../AGENTS.md) (and
   [CLAUDE.md](../CLAUDE.md) if Claude-specific).
1. **Cursor-only, file-type rules:** Add or edit `.cursor/rules/*.mdc` with
   the right `globs`.
1. **New OpenCode command:** Add a `.md` file under
   [.opencode/commands/](../.opencode/commands/).
1. **New Cursor command:** Add an entry to
   [.cursor/commands.json](../.cursor/commands.json).
1. **New Claude agent:** Add a `.md` under
   [.claude/agents/](../.claude/agents/) and reference AGENTS.md/CLAUDE.md.

See also:
[.cursor/rules/ai-context-standards.mdc](../.cursor/rules/ai-context-standards.mdc).
