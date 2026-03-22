# Shell scripts and patterns from exarp-go

Patterns and scripts in exarp-go (`../mcp/exarp-go` or `EXARP_GO_ROOT`) that we can learn from or port. exarp-go is Go + Makefile; this repo is C++/multi-language + CMake + Justfile — so we adapt the **patterns**, not copy the exact commands.

---

## Run exarp-go tool

From this project's root you can run a specific exarp-go tool (default: lint):

```bash
# Default: lint
./scripts/run_exarp_go_tool.sh

# Specific tool (e.g. testing, security, task_workflow)
./scripts/run_exarp_go_tool.sh testing
./scripts/run_exarp_go_tool.sh security
```

Or with Just: `just exarp lint`, `just exarp testing`, `just exarp-lint`.

In Cursor chat use the exarp-go MCP tools (e.g. task_workflow, report, session, lint). See `.cursor/commands/exarp-tool.md` for the command reference.

---

## 1. Ansible run script + SSL fix (ported)

**exarp-go:** `ansible/run-dev-setup.sh`:

- **SSL on macOS:** Before running Ansible, sets `SSL_CERT_FILE` and `REQUESTS_CA_BUNDLE` to the system CA bundle so Python/Ansible don't hit `CERTIFICATE_VERIFY_FAILED` when fetching Galaxy or HTTPS modules.
- **Flow:** Check Ansible → install Galaxy requirements → syntax-check playbook → list tasks → detect if sudo needed → run playbook (development.yml or development-user.yml).

**Reuse here:** We added **`ansible/run-dev-setup.sh`** with the same SSL block and flow (no development-user playbook; single playbook). Run from repo root or from `ansible/`: `./ansible/run-dev-setup.sh` or `bash ansible/run-dev-setup.sh`.

### 1a. Ansible linters

- **`just ansible-check`** — Playbook syntax-check only (`ansible-playbook --syntax-check` on `ansible/playbooks/development.yml`). No extra deps.
- **`just ansible-lint`** — Run **ansible-lint** on `ansible/` (playbooks and roles). Requires: `pip install ansible-lint` or `uv tool install ansible-lint`. Config: `.ansible-lint` in repo root (profile: basic; excludes `.git/`, `.github/`).
- **`just lint`** / **`./scripts/run_linters.sh`** — Run ansible-lint when the `ansible-lint` executable is present; otherwise skip with a warning. Shellcheck already covers `ansible/run-dev-setup.sh`.

**SSL block (portable):**

```bash
if [[ "$(uname)" == "Darwin" ]]; then
    for f in /etc/ssl/cert.pem /usr/local/etc/openssl/cert.pem; do
        if [[ -f "$f" ]]; then
            export SSL_CERT_FILE="$f"
            export REQUESTS_CA_BUNDLE="$f"
            break
        fi
    done
fi
```

---

## 2. Dev watch script (pattern only)

**exarp-go:** `dev.sh` — watch Go files, optional auto-build, optional auto-test, start/stop server, use fswatch (macOS) or inotifywait (Linux) or polling fallback.

**Patterns:**

- `set -euo pipefail`; `SCRIPT_DIR`; `PROJECT_ROOT="${PROJECT_ROOT:-$SCRIPT_DIR}"` so agents can override.
- Colored log helper (`log INFO/WARN/ERROR`).
- Parse flags (`--watch`, `--test`, `--coverage`, `--quiet`, `--help`).
- Check deps (e.g. `uv`, optional `fswatch`/`inotifywait`), exit with clear message.
- Watch: prefer fswatch → inotifywait → polling.
- Trap cleanup (stop server, remove PID file).

**Reuse here:** We don't have a single "dev.sh" for the C++ app; we have `just` recipes and CMake. When adding a watch script (e.g. for Python TUI or agents), reuse: PROJECT_ROOT override, log helper, dep check, and cleanup trap.

---

## 3. Sanity-check script (pattern only)

**exarp-go:** `scripts/sanity-check.sh` — build a small Go binary that checks expected counts of tools/resources/prompts; Makefile also has a Go-based sanity-check that writes `.make.config`.

**Patterns:**

- Expect known counts; run binary (or MCP stdio); compare actual vs expected; exit 1 on mismatch.
- Optional: run server in background, send JSON-RPC (initialize, tools/list), parse with `jq`, then kill server.

**Reuse here:** We have `just verify-toolchain` and CMake. For MCP or service checks we could add a small script that runs the binary with stdin JSON and checks stdout (see test-mcp-stdio below).

---

## 4. MCP stdio smoke test (pattern only)

**exarp-go:** `scripts/test-mcp-stdio.sh` — send `initialize` and `tools/list` over stdin to the MCP binary; assert JSON-RPC response and presence of tools.

**Patterns:**

- `BINARY="${1:-bin/exarp-go}"`; `PROJECT_ROOT` for env.
- Temp dir for output; trap cleanup.
- Pipe JSON lines + short sleeps into `timeout 5 "$BINARY"`; grep for `"jsonrpc"` and `"tools"`/tool names.
- Exit 0 on pass, 1 on fail.

**Reuse here:** If we add an MCP server or stdio CLI, add a similar smoke test script that sends minimal JSON-RPC and checks output.

---

## 5. Session-prime hook (reference only)

**exarp-go:** `.cursor/hooks/session-prime.sh` — Cursor sessionStart hook; runs `exarp-go -tool session -args '{"action":"prime",...}'`, parses JSON with `jq`, builds `additional_context` string, outputs Cursor schema `{ "additional_context": "...", "continue": true }`.

**Reuse here:** We don't run exarp-go as our main tool in this repo; we use exarp-go as MCP from Cursor. No need to port the hook here; reference for other projects that embed exarp-go.

---

## 6. Makefile patterns (already covered in EXARP_GO_PATTERNS_AND_PORTS.md)

- **Repo root:** `REPO_ROOT := $(shell git rev-parse --show-toplevel)`; targets that need root do `cd "$(REPO_ROOT)"`.
- **Help:** `grep -hE '^[a-zA-Z_-]+:.*?## .*$$' Makefile | awk ...` for "target → description".
- **Config:** `make config` writes `.make.config` with HAVE_* and version vars; `-include .make.config`.
- **Short targets:** `b`, `r`, `p`, `pl`, `st` for build, root, push, pull, status.

We use Justfile and `just verify-toolchain` instead; the idea (single config step, repo-root, short aliases) is the same.

---

## 7. Lint scripts (reference)

**exarp-go:** `make lint-shellcheck`, `make lint-yaml`, `make lint-ansible` — run shellcheck on `scripts/*.sh` and `ansible/run-dev-setup.sh`, yamllint on `.github` and `ansible`, ansible-lint on playbooks.

**Reuse here:** We have `./scripts/run_linters.sh` for C++/Python/JS/shell/Ansible. We added `just lint-shell` (shellcheck on `scripts/*.sh` and `ansible/run-dev-setup.sh`), `just ansible-dev` (run `ansible/run-dev-setup.sh`), `just ansible-check` (playbook syntax-check only), and `just ansible-lint` (ansible-lint on `ansible/` when installed). `run_linters.sh` runs ansible-lint when the executable is present; optional yamllint not wired yet.

---

## Summary: what we ported vs reference

| Item | Ported here | Notes |
|------|-------------|------|
| Ansible run script + SSL fix | ✅ `ansible/run-dev-setup.sh` | Same SSL block; single playbook |
| Dev watch script | Pattern only | Use PROJECT_ROOT, log helper, trap when adding watch |
| Sanity-check | Pattern only | verify-toolchain + CMake; optional MCP smoke later |
| MCP stdio smoke test | Pattern only | Use when we have stdio/MCP binary |
| Session-prime hook | No | exarp-go-specific |
| Lint (shellcheck/yaml/ansible) | Optional | Add to Justfile if desired |

---

## References

- exarp-go: `ansible/run-dev-setup.sh`, `dev.sh`, `scripts/sanity-check.sh`, `scripts/test-mcp-stdio.sh`, `.cursor/hooks/session-prime.sh`, `Makefile`
- This repo: `ansible/run-dev-setup.sh`, `Justfile`, `scripts/run_linters.sh`, `docs/EXARP_GO_PATTERNS_AND_PORTS.md`, `docs/EXARP_GO_ANSIBLE_PATTERNS.md`
