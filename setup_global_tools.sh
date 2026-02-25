#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- Ensure uv is available (primary Python tool) ---
if ! command -v uv >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "darwin"* ]] && command -v brew >/dev/null 2>&1; then
    echo "[info] uv not detected; installing via Homebrew."
    brew install uv
  else
    echo "[info] uv not detected; installing via official installer."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
  fi
fi
echo "[ok] uv $(uv --version) available."

# --- Ensure Ansible is available ---
if ! command -v ansible >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "darwin"* ]] && command -v brew >/dev/null 2>&1; then
    echo "[info] Ansible not detected; installing via Homebrew."
    brew install ansible
  else
    echo "[info] Ansible not detected; installing via uv tool."
    uv tool install ansible-core
    export PATH="$HOME/.local/bin:$PATH"
  fi
fi

# --- Ensure community.general collection is present ---
if ! ansible-galaxy collection list 2>/dev/null | grep -q community.general; then
  echo "[info] Installing Ansible community.general collection..."
  ansible-galaxy collection install community.general
fi

echo "[info] Running Ansible playbook to configure global developer tooling..."
export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${REPO_ROOT}/roles:${ANSIBLE_ROLES_PATH:-}"
ANSIBLE_COMMON_FLAGS=("-i" "localhost," "--connection=local")
if [[ "$OSTYPE" == "darwin"* ]]; then
  ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" ansible/playbooks/setup_devtools.yml
else
  if [[ -n "${CI:-}" ]]; then
    ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" --become ansible/playbooks/setup_devtools.yml
  else
    ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" --become -K ansible/playbooks/setup_devtools.yml
  fi
fi
echo "[ok] Global toolchain configured via Ansible."

if [ -s "$HOME/.nvm/nvm.sh" ]; then
  set +u
  # shellcheck disable=SC1090
  . "$HOME/.nvm/nvm.sh"
  echo "[info] Ensuring Node.js LTS via nvm..."
  nvm install --lts >/dev/null
  nvm use --lts >/dev/null
  current_node="$(nvm current)"
  set -u
  echo "[ok] nvm environment ready (Node ${current_node})."
else
  echo "[warn] nvm not detected; skip Node.js LTS bootstrap." >&2
fi
