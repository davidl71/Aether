# Exarp: Lint

Run exarp-go lint (Go, shell, etc.). Requires exarp-go on PATH or EXARP_GO_ROOT.

**Default:** With no args, exarp-go runs **Go lint only** (golangci-lint). To run other linters, pass JSON as the second argument.

## Usage

From project root:

```bash
# Go lint only (default)
./scripts/run_exarp_go_tool.sh lint
# or
just exarp-lint

# Shell (shellcheck) on scripts/
./scripts/run_exarp_go_tool.sh lint '{"linter":"shellcheck","path":"scripts"}'
# or
just exarp-lint-shell
```

**Local fallback (no exarp-go):** `just lint-shell` runs shellcheck on `scripts/*.sh` and `ansible/run-dev-setup.sh`.

See `docs/EXARP_GO_VS_THIS_REPO.md` for when to use exarp-go lint vs `just lint-shell`.
