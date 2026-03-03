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

# Ensure tools installed by uv are on PATH (e.g. ansible-core from uv tool install)
export PATH="${HOME}/.local/bin:${PATH}"

# --- Ensure Ansible is available ---
if ! command -v ansible >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "darwin"* ]] && command -v brew >/dev/null 2>&1; then
    echo "[info] Ansible not detected; installing via Homebrew."
    brew install ansible
  elif command -v uv >/dev/null 2>&1; then
    echo "[info] Ansible not detected; installing via uv tool."
    uv tool install ansible
    export PATH="${HOME}/.local/bin:${PATH}"
  else
    echo "[error] Could not install Ansible: need Homebrew (macOS) or uv." >&2
    exit 1
  fi
fi
if ! command -v ansible >/dev/null 2>&1; then
  echo "[error] Ansible still not in PATH. Add ${HOME}/.local/bin to PATH and re-run." >&2
  exit 1
fi
echo "[ok] Ansible $(ansible --version | head -1) available."

# --- Ensure community.general collection is present ---
if ! ansible-galaxy collection list 2>/dev/null | grep -q community.general; then
  echo "[info] Installing Ansible community.general collection..."
  ansible-galaxy collection install community.general
fi
echo "[ok] Ansible and required collections ready."

echo "[info] Running Ansible playbook to configure global developer tooling..."
export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${REPO_ROOT}/roles:${ANSIBLE_ROLES_PATH:-}"
ANSIBLE_COMMON_FLAGS=("-i" "localhost," "--connection=local")
if [[ "$OSTYPE" == "darwin"* ]]; then
  ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" ansible/playbooks/setup_devtools.yml
else
  if [[ -n "${CI:-}" ]]; then
    ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" --become ansible/playbooks/setup_devtools.yml
  elif [[ "$(id -u)" -eq 0 ]]; then
    # Already root (e.g. sudo ./setup_global_tools.sh): no become, avoids sudo-rs/IDE password timeout
    echo "[info] Running as root; privilege escalation disabled."
    if [[ -n "${SUDO_USER:-}" ]]; then
      export HOME="$(getent passwd "${SUDO_USER}" | cut -d: -f6)"
      echo "[info] Using home directory of ${SUDO_USER}: ${HOME}"
    fi
    ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" -e ansible_become=false ansible/playbooks/setup_devtools.yml
  else
    if [[ ! -t 0 ]] && [[ -z "${ANSIBLE_BECOME_PASSWORD:-}" ]]; then
      echo "[warn] Not running in an interactive terminal (no TTY). sudo cannot prompt for a password." >&2
      echo "[warn] Either run this script from a real terminal, or: sudo $0" >&2
      echo "[warn] Or set ANSIBLE_BECOME_PASSWORD and re-run." >&2
      exit 1
    fi
    if [[ -z "${ANSIBLE_BECOME_PASSWORD:-}" ]]; then
      echo "[info] You will be prompted for your sudo password."
      echo "[info] If the sudo prompt times out, run instead: sudo $0"
    fi
    # Use /dev/tty for become password so IDE terminals (e.g. Cursor) get the prompt
    if [[ -e /dev/tty ]] && [[ -r /dev/tty ]]; then
      ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" --become -K ansible/playbooks/setup_devtools.yml < /dev/tty
    else
      ansible-playbook "${ANSIBLE_COMMON_FLAGS[@]}" --become -K ansible/playbooks/setup_devtools.yml
    fi
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

# --- Ensure Cargo (Rust) is available ---
export PATH="${HOME}/.cargo/bin:${PATH}"
if ! command -v cargo >/dev/null 2>&1; then
  echo "[info] Cargo not detected; installing Rust via rustup."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  export PATH="${HOME}/.cargo/bin:${PATH}"
fi
if command -v cargo >/dev/null 2>&1; then
  echo "[ok] Cargo $(cargo --version) available."
else
  echo "[warn] Cargo not in PATH. Add ${HOME}/.cargo/bin to PATH and re-run." >&2
fi
