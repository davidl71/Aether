# Lint and automation: where things run

Single source of truth for *what* runs *where*. We avoid duplicating **implementation**; multiple **entry points** (Make, CMake, pre-commit, CI) delegate to the same scripts or binaries.

## Implementation (no duplication)

| What | Implemented in | Invoked by |
|------|----------------|------------|
| Full project lint (C++, Python, JS, exarp-go, etc.) | `scripts/run_linters.sh` | Make `lint`, CMake target `lint`, CLI `./scripts/run_linters.sh` |
| exarp-go tools (lint, testing, security, …) | `exarp-go` binary | `scripts/run_exarp_go_tool.sh`, Make `exarp-*`, CMake `exarp-lint` / `exarp-list`, MCP in Cursor |
| Markdown style | markdownlint (npm / config) | `npm run lint:docs`; CI `docs-validation.yml` (not pre-commit) |
| Shell format | shfmt | Pre-commit only |
| Shell lint | shellcheck | Pre-commit only |

So: **one** script for “all linters” (`run_linters.sh`), **one** wrapper for exarp-go CLI (`run_exarp_go_tool.sh`), **one** binary (exarp-go). Make and CMake only call those scripts; they do not reimplement any checks.

## Pre-commit vs run_linters.sh

Different scope; no overlap of tools:

- **Pre-commit**: fast, file-scoped — trailing-whitespace, end-of-file-fixer, check-yaml, check-toml, mixed-line-ending, **shfmt**, **shellcheck**. (Markdownlint is disabled in pre-commit so doc edits don’t block commits.)
- **run_linters.sh**: full project — cppcheck, clang --analyze, infer, swiftlint, exarp-go lint, bandit, eslint, stylelint, TypeScript check, JS syntax. Does **not** run shfmt or shellcheck.

So pre-commit does not duplicate run_linters.sh; it runs a separate set of quick checks. If you want “one command that runs everything”, you could add shellcheck (and optionally shfmt) to `run_linters.sh`; pre-commit would still run them on commit for speed.

## CI

- **lint.yml**: Black, Ruff, MyPy (Python), optional exarp-go lint. Does not run `run_linters.sh` or cppcheck.
- **docs-validation.yml**: markdownlint (`npm run lint:docs:ci`).
- **security-scan.yml**: Bandit and other security checks (overlaps with run_linters.sh’s bandit step by design: CI can focus on security reporting).
- **go-tui.yml**: golangci-lint (Go only).

So CI jobs are not duplicates of Make/CMake; they run in different environments and can focus on Python, docs, or security. The only shared “action” is exarp-go lint (run_linters.sh and lint.yml both can run it).

## Summary

- **CMake / Make**: Same actions; both call the same scripts. Two entry points, one implementation.
- **exarp-go**: One binary; invoked via script, Make, CMake, or MCP. No duplicated logic.
- **Pre-commit**: Different checks (shfmt, shellcheck, format) from run_linters.sh; complementary.
- **CI**: Separate jobs (Python lint, docs, security, Go); optional exarp-go in lint.yml. No unnecessary duplication.

To add a new check: add it in one place (e.g. `run_linters.sh` or pre-commit) and, if needed, wire that into Make/CMake by calling the same script or tool.
