# AI and build patterns from exarp-go (sibling repo)

Patterns we can use or have ported from the exarp-go repo (`../mcp/exarp-go` or `EXARP_GO_ROOT`). exarp-go is Go + Makefile + Taskfile; this repo is C++/multi-language + CMake + Justfile.

## Summary table

| Pattern | exarp-go | This repo | Port / note |
|--------|----------|-----------|-------------|
| **Shortcuts rule** | `.cursor/rules/make-shortcuts.mdc` — prefer `make b`, `make r`, `make tag-build-ok`, `make p`/`pl`/`st` | `.cursor/rules/just-cmake-shortcuts.mdc` | ✅ Added: prefer `just`/CMake commands, `r` for repo root, `git:pull-safe` |
| **pre-push** | `make pre-push` / `task pre-push` (verify + check + test:fast) | `just pre-push` (format + lint + test + build) | ✅ Already in Justfile |
| **pre-commit** | `make pre-commit` / `task pre-commit` (tidy + fmt + vet + lint) | Optional: add `just pre-commit` (format + lint only, no test) | Optional |
| **tag last good build** | `make tag-build-ok` / `task tag-build-ok` | `just tag-ok` | ✅ Already in Justfile |
| **Git from repo root** | `make pull`/`push`/`status` run in `REPO_ROOT` | `git:pull-safe` (stash→pull→pop); no Just push/pull/status | ✅ pull-safe script; optional: add `just pull-safe`, `just status` |
| **Config / sanity** | `make config` → `.make.config` (detect Go, CGO, etc.) | CMake does config; `just verify-toolchain` | ✅ verify-toolchain exists |
| **Skills layout** | `skills/<name>/SKILL.md` with frontmatter (name, description) | `.cursor/skills/*.md` flat + README | Different: we use flat .md; exarp uses dir per skill with SKILL.md |
| **Help / list** | `make help`, `task --list` | `just` (default), `just --list` | ✅ Justfile default shows list |

## exarp-go patterns worth reusing

### 1. AI shortcut rule (ported)

exarp-go’s `make-shortcuts.mdc` tells the AI to suggest short commands and repo-root semantics. We added **`.cursor/rules/just-cmake-shortcuts.mdc`** with the same idea for this repo: prefer `just build`, `just test`, `just tag-ok`, `just pre-push`, and `git:pull-safe` (or `just pull-safe` if added); use `r` or `cd $(git rev-parse --show-toplevel)` when commands must run from repo root.

### 2. pre-commit vs pre-push

- **exarp-go:** `pre-commit` = tidy + fmt + vet + lint (no test); `pre-push` = verify + check + test:fast.
- **This repo:** `just pre-push` = format + lint + test + build. We could add `just pre-commit` = format + lint only for faster local checks before commit.

### 3. Skills format

exarp-go uses **one directory per skill** with a `SKILL.md` file and YAML frontmatter (`name`, `description`). Cursor discovers skills from `~/.cursor/skills/` and `.cursor/skills/`. We use **flat** `.cursor/skills/*.md` plus a README; both are valid. To match exarp-go’s layout you could add `.cursor/skills/<skill-name>/SKILL.md` and keep the README as the index.

### 4. Taskfile vs Makefile

exarp-go prefers **Taskfile.yml** (go-task) and keeps the Makefile as a thin layer. We use **Justfile** as the main task runner and CMake for build. No need to introduce Make or Task here; the pattern to port is “one canonical task runner + shortcut rule for the AI.”

### 5. Sanity / tool verification

exarp-go has `sanity-check` (binary that verifies tools/resources counts) and `make config` writing `.make.config`. We have `just verify-toolchain` and CMake’s configure; no need for a separate config file. Optional: a “sanity” recipe that runs configure + build + test once (e.g. `just sanity`).

## What we did not port

- **Makefile / Taskfile** — We keep Justfile + CMake.
- **Go-specific targets** (go mod tidy, govulncheck, etc.) — We use uv, scripts, and exarp-go MCP for security/docs.
- **exarp-go’s bundled skills** (use-exarp-tools, task-workflow, report-scorecard) — Those are for projects that use exarp-go as MCP; we already reference exarp-go in `.cursor/rules/project-automation.mdc` and `.cursor/commands/exarpauto.md`. We added our own project skills in `.cursor/skills/`.

## References

- exarp-go: `Makefile`, `Taskfile.yml`, `.cursor/rules/make-shortcuts.mdc`, `skills/README.md`, `skills/*/SKILL.md`
- This repo: `Justfile`, `.cursor/commands.json`, `.cursor/skills/README.md`, `docs/archive/AI_EDITOR_SETUP.md`
- **Shell scripts and patterns:** See `docs/EXARP_GO_SCRIPTS_AND_PATTERNS.md` for dev.sh, run-dev-setup.sh (SSL fix), sanity-check, MCP stdio smoke test, and lint patterns.
