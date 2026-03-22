# Git hooks: what we learned from exarp-go

Patterns from exarp-go’s git hooks and `setup_hooks` tool that we reuse or support in this repo.

---

## 1. Versioned hooks via `core.hooksPath`

**exarp-go:** Keeps hook scripts under `scripts/git-hooks/` (e.g. `prepare-commit-msg`) and documents:

```bash
git config core.hooksPath scripts/git-hooks
```

Then Git runs hooks from the repo instead of `.git/hooks`, so hook logic is versioned and shared.

**Here:** We added `scripts/git-hooks/` with `prepare-commit-msg` (Co-authored-by) and a README. Optional: set `core.hooksPath` to use them. If you use the **pre-commit framework** (`pre-commit install`), it owns `.git/hooks/pre-commit`; using `core.hooksPath` points Git at `scripts/git-hooks` and replaces that. So choose one: either pre-commit framework (current default) or versioned hooks in `scripts/git-hooks` (and optionally chain `pre-commit run` from a script there if you want both).

---

## 2. Co-authored-by trailer (`prepare-commit-msg`)

**exarp-go:** `scripts/git-hooks/prepare-commit-msg` reads `git config exarp-go.coauthor` and appends `Co-authored-by: Name <email>` to the commit message. Skips merge/squash and when the trailer is already present.

**Here:** We added `scripts/git-hooks/prepare-commit-msg` that reads `git config ib_box_spread.coauthor` (or `exarp-go.coauthor` for compatibility). Same behavior: skip merge/squash, don’t duplicate.

---

## 3. Resolve runner and skip gracefully

**exarp-go:** In hooks that call exarp-go, a preamble resolves the binary (repo `bin/exarp-go` or PATH) and **exits 0** if not found so the hook never blocks when exarp-go isn’t installed.

**Pattern:**

```sh
EXARP_GO=""
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
[ -n "$ROOT" ] && [ -x "$ROOT/bin/exarp-go" ] && EXARP_GO="$ROOT/bin/exarp-go"
[ -z "$EXARP_GO" ] && command -v exarp-go >/dev/null 2>&1 && EXARP_GO="exarp-go"
if [ -z "$EXARP_GO" ]; then
  echo "exarp-go not found, skipping hook" >&2
  exit 0
fi
```

**Here:** For hooks that call exarp-go we use the same idea: resolve `scripts/run_exarp_go.sh` (or exarp-go on PATH) and exit 0 if missing so commits/pushes still succeed.

---

## 4. Redirect stdin in hooks

**exarp-go:** Hooks run with stdin from Git (e.g. pre-push receives refs). exarp-go can misinterpret that as MCP/JSON. So hooks redirect stdin:

```sh
"$EXARP_GO" -tool health -args '{"action":"docs"}' </dev/null || exit 1
```

**Here:** Any hook that invokes exarp-go (or our runner) should use `</dev/null` so the tool doesn’t read git’s stdin.

---

## 5. Explicit JSON args and quiet logs

**exarp-go:** Uses explicit `-args '{"action":"..."}'` in hooks to avoid key=value parsing issues. Sets `export GIT_HOOK=1` so exarp-go can suppress noisy logs in hook context.

**Here:** When we run exarp-go from hooks we pass JSON args and can set `GIT_HOOK=1` for the same reason.

---

## 6. Pre-commit / pre-push content (exarp-go)

**exarp-go’s `setup_hooks` (action=git)** installs:

- **pre-commit:** In exarp-go repo: `make silent` then health(docs) + security(scan). In other repos: health(docs) + security(scan). All blocking.
- **pre-push:** analyze_alignment(todo2) + security(scan). Blocking.
- **post-commit:** No-op.
- **post-merge:** task_analysis(duplicates) + task_workflow(sync). Non-blocking (`|| true`).

**Here:** We don’t duplicate that logic. Use **exarp-go’s `setup_hooks` tool** from Cursor (with `workingDirectory` = this repo) to install those hooks into `.git/hooks`. When you commit/push in this repo, the installed hooks run exarp-go with this repo as the project root (Git’s cwd is the repo). So we “learn” the pattern but delegate to exarp-go for the actual checks.

---

## Summary

| Pattern | exarp-go | This repo |
|--------|----------|-----------|
| Versioned hooks | `scripts/git-hooks/`, `core.hooksPath` | `scripts/git-hooks/` + README |
| Co-authored-by | `prepare-commit-msg` + `exarp-go.coauthor` | `prepare-commit-msg` + `ib_box_spread.coauthor` |
| Resolve runner, exit 0 if missing | Preamble in each hook | Use same when we add hooks that call exarp-go |
| Stdin </dev/null | All exarp-go invocations in hooks | Same |
| GIT_HOOK=1, explicit JSON args | Yes | Same when calling exarp-go from hooks |
| pre-commit / pre-push content | health + security; alignment + security | Use exarp-go `setup_hooks` to install (don’t reimplement) |

**References:** exarp-go `internal/tools/hooks_setup.go`, `scripts/git-hooks/README.md`, `scripts/git-hooks/prepare-commit-msg`. This repo: `scripts/git-hooks/`, `.cursor/rules/hooks.mdc`, `.pre-commit-config.yaml`.
