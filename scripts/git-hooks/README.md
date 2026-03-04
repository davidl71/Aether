# Versioned Git hooks

Hook scripts in this directory can be used as your Git hooks so hook logic is versioned and shared.

## Setup (optional)

Point Git at this directory (from repo root):

```bash
git config core.hooksPath scripts/git-hooks
```

After that, Git will run `prepare-commit-msg`, `pre-commit`, `pre-push`, etc. from here instead of `.git/hooks`.

**Note:** If you use the **pre-commit framework** (`pre-commit install`), it installs into `.git/hooks`. Setting `core.hooksPath` to `scripts/git-hooks` replaces that; Git will no longer run the framework's hooks unless you invoke it from a script in this directory.

---

## Hooks included

### prepare-commit-msg

Appends a `Co-authored-by:` trailer from config so you don't have to type it every time.

**Config (one of):**

- `git config ib_box_spread.coauthor "Co Author Name <email@example.com>"`
- `git config exarp-go.coauthor "..."` (compatibility with exarp-go config)

Skips merge/squash commits and does nothing if `Co-authored-by:` is already in the message.

### pre-commit

Runs exarp-go **docs health** and **multi-language dependency security scan** (`scan_dependency_security`) for this repo (via `scripts/run_exarp_go.sh`). Blocks commit if either fails. If exarp-go reports "only supported for Go" or tool unknown, the security step is skipped so commit is not blocked.

**Why did I get "security scan is only supported for Go projects"?**  
Git is running the hook from **`.git/hooks/pre-commit`** (installed by exarp-go's `setup_hooks`). That hook calls `-tool security` (Go-only) and has no skip for non-Go repos. This repo is not Go, so it fails. **Fix:** use our versioned hooks so the security step is skipped for this repo:

```bash
git config core.hooksPath scripts/git-hooks
```

Then Git runs `scripts/git-hooks/pre-commit`, which uses `scan_dependency_security` and skips when exarp-go reports Go-only. You do **not** need to run `make install` in sibling exarp-go for commits to succeed; you only need Git to use this directory as the hooks path.

### pre-push

Runs exarp-go **task alignment** (todo2) and **multi-language dependency security scan** (`scan_dependency_security`) for this repo. Blocks push if either fails. If exarp-go reports "only supported for Go" or tool unknown, the security step is skipped so push is not blocked.

---

## exarp-go pre-commit / pre-push

This directory includes **pre-commit** and **pre-push** that call exarp-go (via `scripts/run_exarp_go.sh`) for docs health, **multi-language dependency security** (`scan_dependency_security`: Python/Rust/npm), and task alignment. If you prefer exarp-go to install hooks into `.git/hooks` itself, use **exarp-go's `setup_hooks` tool** from Cursor with `workingDirectory` set to this repo instead of using `core.hooksPath`.

See `docs/EXARP_GO_GIT_HOOKS_LEARNINGS.md` for patterns we took from exarp-go (versioned hooks, Co-authored-by, stdin redirect, graceful skip when exarp-go is missing).

---

## Alternative: only Co-authored-by

If you want the Co-authored-by behavior but keep using `.git/hooks` (e.g. for pre-commit framework), install just this hook:

```bash
cp scripts/git-hooks/prepare-commit-msg .git/hooks/prepare-commit-msg
chmod +x .git/hooks/prepare-commit-msg
git config ib_box_spread.coauthor "Co Author Name <email@example.com>"
```

Do **not** set `core.hooksPath` so the pre-commit framework keeps managing `.git/hooks/pre-commit`.
