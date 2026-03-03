## Ansible Dev Environment Setup

This repository includes playbooks for a reproducible developer setup split into:

- Global roles (reusable across projects): `common`, `editor`, `langs`
- Project role (this repo): `ib_box_spread`

### Prerequisites

- macOS (Darwin) or Linux (Debian/Ubuntu)
- Ansible **9.x or 10.x** (ansible-core 2.15+ or 2.16+)

### Install Ansible

**macOS (Homebrew):**

```bash
brew install ansible
ansible-galaxy collection install community.general
```

For a minimal install (ansible-core only, install collections as needed):

```bash
brew install ansible-core
ansible-galaxy collection install community.general
```

**Linux (Debian/Ubuntu):**

```bash
# Option A: system package (may be older)
sudo apt update && sudo apt install ansible
ansible-galaxy collection install community.general

# Option B: pip/uv (newer versions)
uv pip install ansible
ansible-galaxy collection install community.general
```

**Optional (any OS):** Pin versions with pip:

```bash
uv pip install "ansible>=9,<11" "ansible-core>=2.15"
ansible-galaxy collection install community.general
```

### Structure

- `playbooks/global.yml` — global setup (brew packages/casks, editor, global kit)
- `playbooks/project.yml` — project setup (IB API paths, CMake configure)
- `playbooks/site.yml` — runs both in sequence
- `roles/common`, `roles/editor`, `roles/langs` — global roles
- `roles/ib_box_spread` — project-specific role
- `ansible/playbooks/setup_devtools.yml` — dev tools (uv, cmake, npm/nvm, **pip-audit**, etc.); role `ansible/roles/devtools`. Run via `./setup_global_tools.sh`.

### Usage

Run global setup (safe across all projects):

```bash
ansible-playbook playbooks/global.yml -K -t "global"
```

Run project setup (requires IB API path):

```bash
export IBJTS_DIR=~/IBJts/source/cppclient
ansible-playbook playbooks/project.yml -K -e ibjts_dir="$IBJTS_DIR" -t "project"
```

Run everything:

```bash
export IBJTS_DIR=~/IBJts/source/cppclient
ansible-playbook playbooks/site.yml -K
```

### Tags

- Global: `global`, `common`, `editor`, `langs`
- Project: `project`, `ib`, `cmake`, `lint`

Examples:

```bash
ansible-playbook playbooks/global.yml -K -t "editor"
ansible-playbook playbooks/project.yml -K -t "cmake"
```

### Notes

- Global kit is installed via `scripts/install_global_kit.sh --mode link`
- No secrets are handled by playbooks; IB credentials must remain out-of-band
- Tasks are idempotent and safe to re-run

### Upgrading Ansible

After upgrading Ansible or collections, run a playbook with `deprecation_warnings = True` in `ansible.cfg` (default in this repo) to see any deprecations. Fix playbook or role references as needed, then you can set `deprecation_warnings = False` again if desired. Check version with:

```bash
ansible --version
ansible-galaxy collection list
```

### MCP / exarp-go

This project uses **exarp-go** (Go MCP server) for project automation in Cursor, not the old Python/uvx Exarp stack. Configure it in `.cursor/mcp.json`; `scripts/run_exarp_go.sh` sets `PROJECT_ROOT` and runs the `exarp-go` binary (from PATH or `~/go/bin`, `~/Projects/exarp-go/bin`, or `/usr/local/bin`). Install exarp-go separately (e.g. build from source or `go install` if you have the module path). See `docs/EXARP_GO_MIGRATION_LEFTOVERS.md` and `docs/MCP_REQUIRED_SERVERS.md`.

**CI / scripts / make:** exarp-go is wired in so you can use it outside Cursor:

- **CMake:** From a configured build dir: `cmake --build build --target lint` (all linters, includes exarp-go when available), `cmake --build build --target exarp-lint`, `cmake --build build --target exarp-list`.
- **Scripts:** `scripts/run_exarp_go_tool.sh <tool_name> [json_args]` (e.g. `./scripts/run_exarp_go_tool.sh lint`). List tools with `--list`.
- **Linters:** `./scripts/run_linters.sh` runs exarp-go lint when the binary is available (optional; no failure if missing).
- **CI:** `.github/workflows/lint.yml` has an optional "Run exarp-go lint" step (`continue-on-error`; install exarp-go in CI to enable).
- **Cursor commands:** `lint:exarp`, `exarp:tool`, `exarp:list-tools` in `.cursor/commands.json`.
- **Make (optional):** Root `Makefile` wraps CMake and scripts: `make build` / `make test` / `make clean` use the default preset (macOS: macos-arm64-debug, Linux: linux-x64-debug; override with `PRESET=...`). `make lint`, `make exarp-lint`, etc. run the script-based targets. Run `make help`.
