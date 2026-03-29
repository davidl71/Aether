# Python in Aether (remaining footprint)

The product runtime is **Rust** (`agents/backend/`): backend, TUI, CLI, IB adapter. Python remains for an **optional Nautilus ↔ NATS agent**, **repo automation scripts**, **editor skill helpers**, and **Ansible collection modules**.

For the full language map, see **`docs/MULTI_LANGUAGE_CODEBASE.md`**.

---

## 1. Nautilus agent (`agents/nautilus/`)

Packaged with **`pyproject.toml`** and **`uv`**. Bridges IB via NautilusTrader to NATS using the shared protobuf schema.

| Item | Purpose |
|------|---------|
| `src/nautilus_agent/` | Agent entrypoint, NATS bridge, strategy hooks, config |
| `tests/` | `pytest` suite |
| `scripts/generate_proto.py` | Python gRPC/protobuf stubs for this agent (not the root `proto/generate.sh` path) |
| `config/default.toml` | Example agent config |

**Just recipes** (from repo root): `nautilus-sync`, `proto-gen-nautilus`, `nautilus-paper`, `nautilus-start`, `test-python` / `test-nautilus`, `test-python-cov`.

**Docs:** `agents/nautilus/README.md` (may still mention legacy C++ “nautilus mode” toggles that no longer match the Rust-only tree—verify against `Justfile` and `AGENTS.md`).

**Related design notes:** `docs/NAUTILUS_INTEGRATION_RUST.md`, `docs/nautilus_model_research.md`.

---

## 2. Root `scripts/*.py` (automation and tooling)

These are **not** part of the Rust binary; run with `python3` or `uv run` from repo root as noted in each script or in `Justfile`.

| Script | Role |
|--------|------|
| `sync_global_docs.py` | Sync / path maintenance for global docs (also referenced from `.cursor/commands.json`) |
| `parallel_wave_remaining.py` | Wave runner batches (see `.cursor/skills/wave-runner/SKILL.md`) |
| `benchmark_backend_services.py` | Backend benchmarking (`just benchmark`) |
| `update_stale_docs.py` | Stale-doc refresh helper |
| `generate_docs_summary_tables.py` | Doc summary tables |
| `collect_system_info_python.py` | System info collection (replaces older shell variant; see `scripts/SCRIPTS_AUDIT.md`) |
| `analyze_task_execution_modes.py` | Task execution analysis |
| `audit_in_progress_tasks.py` | In-progress task audit |
| `create_notebooklm_resources.py` | NotebookLM-oriented helper |
| `dev_watch_tui_runner.py` | Dev loop for TUI |
| `test_ib_positions.py` | IB positions check script |
| `curl_configured.py` | Curl helper against configured API |
| `swiftness_integration_manual.py` | Manual Swiftness integration |

Broader shell/script inventory: **`scripts/SCRIPTS_AUDIT.md`** (last updated in-file; counts may drift).

---

## 3. Cursor skill helpers

| Path | Role |
|------|------|
| `.cursor/skills/ui-ux-pro-max/scripts/*.py` | Supporting scripts for the UI/UX skill (search, design tokens, etc.) |

Treat as editor tooling, not production runtime.

---

## 4. Ansible

| Path | Role |
|------|------|
| `.ansible/collections/ansible_collections/community/general/plugins/modules/*.py` | Vendored **community.general** modules (e.g. Homebrew). Third-party layout; do not edit for app logic. |

---

## 5. Protocol buffers and Python

| Mechanism | Python output |
|-----------|----------------|
| **`./proto/generate.sh`** | **Does not** emit Python anymore (buf/protoc for other languages only; see script comments). |
| **`agents/nautilus/scripts/generate_proto.py`** | Generates stubs for the Nautilus agent when you run `just proto-gen-nautilus`. |

**`.gitignore`** still includes **`python/generated/`** for any local legacy betterproto output; nothing in-repo depends on that path today.

---

## 6. Recently removed (archaeology)

The following were removed as low-maintenance cleanup; no replacement paths are required for Rust-first workflows:

- **`Main/`** — LEAN-era `test_box_spread_basic.py` and README.
- **`scripts/recreate_python_generated_init.py`** — paired with old root betterproto output under `python/generated/`.
- **`requirements-notebooks.txt`**, **`requirements-jupyterlab.txt`** — root notebook stacks (use project-local or `uv` deps if you add notebooks again).

---

## 7. Package management

Prefer **`uv`** for Python in this repo (`uv sync`, `uv run`), consistent with `Justfile` and `.cursorrules`. The Nautilus agent is the only first-class Python **package** layout; root `scripts/*.py` are mostly standalone modules.
