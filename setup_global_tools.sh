#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v ansible >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "darwin"* ]] && command -v brew >/dev/null 2>&1; then
    echo "[info] Ansible not detected; installing via Homebrew."
    brew install ansible
  else
    echo "[info] Ansible not detected; installing via pip (requires python3 + pip)."
    if command -v python3 >/dev/null 2>&1 && command -v pip3 >/dev/null 2>&1; then
      pip3 install --user ansible
      export PATH="$HOME/.local/bin:$PATH"
    else
      echo "[error] Python3/pip3 not available. Install them manually, then rerun this script." >&2
      exit 1
    fi
  fi
fi

echo "[info] Running Ansible playbook to configure global developer tooling..."
export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${ANSIBLE_ROLES_PATH:-}"
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

