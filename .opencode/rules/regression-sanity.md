---
description: Regression and sanity checks before push or after significant edits
alwaysApply: false
---

# Regression & Sanity Checks

## Before Committing Checklist

1. **Lint:** `cd agents/backend && cargo fmt && cargo clippy`
2. **Test:** `cd agents/backend && cargo test`
3. **Build:** `cd agents/backend && cargo build`
4. **Secrets:** No credentials/API keys in changed files
5. **Verify:** `git diff --stat` looks reasonable

## Before Pushing (full gate)

```bash
./scripts/run_linters.sh  # full project lint
cd agents/backend && cargo build && cargo test
git push
```

## Quick Sanity (after small edits)

```bash
cd agents/backend && cargo build -p <crate> && cargo test -p <crate>
```

## When to Run

| Scenario | Command |
|---|---|
| After any Rust implementation | `cargo build && cargo test` in `agents/backend/` |
| After market data or broker changes | Full build + test |
| Before `git push` | `./scripts/run_linters.sh` |
| After TUI changes | Build the backend that feeds the TUI |
| After doc-only changes | Skip build, just lint |
