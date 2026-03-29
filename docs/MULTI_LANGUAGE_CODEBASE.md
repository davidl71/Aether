# Multi-language map (Aether)

**Primary runtime:** **Rust** in `agents/backend/` (backend API, TUI, CLI, IB adapter, NATS ingestion). See **`AGENTS.md`** and **`CLAUDE.md`** for commands and layout.

Other languages are supporting tooling, optional agents, or archived UI—not co-equal application tiers.

---

## Language overview

| Language | Role | Location | Details |
|----------|------|----------|---------|
| **Rust** | Production services and libraries | `agents/backend/` | `cargo build` / `cargo test` from `agents/backend/`; `just build`, `just test` from repo root |
| **Python** | Optional Nautilus NATS agent; repo scripts; Cursor skill helpers; Ansible modules | `agents/nautilus/`, `scripts/*.py`, `.cursor/skills/**/scripts/`, `.ansible/collections/...` | **`docs/PYTHON_INVENTORY.md`** |
| **Shell** | Build, lint, deploy, NATS/IB test harnesses | `scripts/*.sh` | **`scripts/SCRIPTS_AUDIT.md`** |
| **Protobuf** | Shared messages | `proto/` | Rust: prost via `nats_adapter` `build.rs`; Go/buf: `./proto/generate.sh`; Nautilus Python: `just proto-gen-nautilus` |
| **TypeScript / JS** | TUI E2E tests; archived web app | `tui-e2e/`, `web/` (archived) | `just test-tui-e2e`; web not an active runtime |
| **Swift** | iOS/iPad/desktop apps (out of tree or separate targets) | Not under `agents/backend/` | See product docs if you open those workspaces |

---

## Quick commands

```bash
# Rust (primary)
cd agents/backend && cargo build && cargo test
# or from repo root:
just build
just test

# Python (Nautilus agent only)
just nautilus-sync
just test-python

# Proto (no root-level Python emit; see PYTHON_INVENTORY.md)
./proto/generate.sh
```

---

## See also

- **`docs/PYTHON_INVENTORY.md`** — every remaining Python path and what was removed
- **`docs/TRACKING_AND_GITIGNORE.md`** — generated dirs (e.g. `python/generated/`)
- **`docs/NAUTILUS_INTEGRATION_RUST.md`** — Nautilus vs Rust backend
