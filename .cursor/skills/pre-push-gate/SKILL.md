---
name: pre-push-gate
description: Run the project's full pre-push gate (format, lint, test, build) before push or merge. Use when the user asks "ready to push?", "can I merge?", "pre-push check", or after significant edits before commit/push.
---

# Pre-push Gate

Standardize "is this safe to push?" for this repo using the project's actual gate commands.

## Primary gate (recommended)

From repo root:

```bash
just pre-push
```

Runs: format → lint → test → build-rust. On success: "All pre-push checks passed — safe to push".

## Alternative: partial gates

| Need              | Command |
|-------------------|--------|
| Full gate         | `just pre-push` |
| Quick regression  | `just sanity` (Python tests + TUI QA screenshot; no C++ build) |
| Tag last good     | `just tag-ok` (build + test, then tag current commit) |
| C++ build only    | `just build` or `./scripts/build_ai_friendly.sh --json-only` |
| Rust build only   | `./scripts/build_rust_ai_friendly.sh --json-only` |
| Tests only        | `just test` |
| Lint only         | `just lint` |

## Checklist before suggesting "safe to push"

- [ ] User has run `just pre-push` (or equivalent) and it passed.
- [ ] Remind: no credentials/secrets in code or logs; use paper port 7497 for testing; live trading gated by config.

## Cursor commands

- **Build (default):** Cursor command `build` (AI-friendly JSON).
- **Test:** `test:run`
- **Lint:** `lint:run`

Repo root: from a subdir use `cd $(git rev-parse --show-toplevel)` or alias `r` before running just/scripts.
