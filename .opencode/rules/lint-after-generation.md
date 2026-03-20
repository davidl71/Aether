---
description: Run lint only after generation is complete, not during
alwaysApply: false
---

# Lint After Generation

**Rule:** Only run linters after you have finished generating or editing code. Never run lint while the user is still receiving streamed output or while you are still applying edits.

## Rationale

Running linters during streamed output creates confusing interleaved output for the user. The correct sequence is:

1. **Think / plan / implement**
2. **Run lint on the specific files you changed**
3. **Fix any lint errors found**
4. **Repeat until clean**

## Commands

| File type | Lint command |
|---|---|
| Rust (`agents/backend/**/*.rs`) | `cd agents/backend && cargo fmt && cargo clippy` |
| Rust (scoped) | `cargo fmt -p <crate> && cargo clippy -p <crate>` |
| Shell (`scripts/*.sh`) | `shellcheck <path>` |
| Python | `uv run ruff check <path>` |
| All | `./scripts/run_linters.sh` |

## When to Run

- **After completing a multi-file implementation** — scope lint to changed files
- **Before committing** — full lint sweep via `./scripts/run_linters.sh`
- **After `cargo build` fails** — run clippy to catch type errors first
- **Never** while user is still receiving streamed response
