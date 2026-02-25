#!/usr/bin/env bash
# Backend bootstrap: creates venv via uv, installs Python deps, and fetches Cargo crates.
# Env hints: PYTHON_BIN (preferred interpreter), NAUTILUS_TRADER_WHEEL (optional prebuilt wheel path).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Prefer uv-native setup if ansible is not available
if command -v uv >/dev/null 2>&1 && ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "[info] Using uv directly (ansible-playbook not found)."
  BACKEND_DIR="$SCRIPT_DIR/.."
  PYTHON="${PYTHON_BIN:-python3}"

  uv venv "$BACKEND_DIR/.venv" --python "$PYTHON" 2>/dev/null || true
  uv pip install --python "$BACKEND_DIR/.venv/bin/python" -e "$BACKEND_DIR/python"

  if [ -n "${NAUTILUS_TRADER_WHEEL:-}" ]; then
    uv pip install --python "$BACKEND_DIR/.venv/bin/python" "$NAUTILUS_TRADER_WHEEL"
  fi

  if command -v cargo >/dev/null 2>&1; then
    cargo fetch --manifest-path "$BACKEND_DIR/Cargo.toml"
  fi

  echo "[ok] Backend environment ready (via uv)."
  exit 0
fi

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "[error] Neither uv nor ansible-playbook found. Run setup_global_tools.sh first." >&2
  exit 1
fi

PLAYBOOK="${REPO_ROOT}/ansible/playbooks/backend_setup.yml"
export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${ANSIBLE_ROLES_PATH:-}"

ANSIBLE_OPTS=("--extra-vars" "repo_root=${REPO_ROOT}")

if [ -n "${PYTHON_BIN:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "python_bin=${PYTHON_BIN}")
fi

if [ -n "${NAUTILUS_TRADER_WHEEL:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "nautilus_trader_wheel=${NAUTILUS_TRADER_WHEEL}")
fi

ansible-playbook -i localhost, --connection=local "$PLAYBOOK" "${ANSIBLE_OPTS[@]}" "$@"
