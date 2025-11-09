#!/usr/bin/env bash
# Backend bootstrap: creates venv, installs Python deps, and fetches Cargo crates.
# Env hints: PYTHON_BIN (preferred interpreter), NAUTILUS_TRADER_WHEEL (optional prebuilt wheel path).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PLAYBOOK="${REPO_ROOT}/ansible/playbooks/backend_setup.yml"

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "[error] ansible-playbook not found. Run setup_global_tools.sh first." >&2
  exit 1
fi

export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${ANSIBLE_ROLES_PATH:-}"

ANSIBLE_OPTS=("--extra-vars" "repo_root=${REPO_ROOT}")

if [ -n "${PYTHON_BIN:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "python_bin=${PYTHON_BIN}")
fi

if [ -n "${NAUTILUS_TRADER_WHEEL:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "nautilus_trader_wheel=${NAUTILUS_TRADER_WHEEL}")
fi

ansible-playbook "$PLAYBOOK" "${ANSIBLE_OPTS[@]}" "$@"
