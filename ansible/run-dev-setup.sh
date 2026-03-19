#!/bin/bash
# Run Ansible development setup (pattern from exarp-go ansible/run-dev-setup.sh).
# Installs Galaxy requirements (if present), validates playbook, lists tasks, then runs it.
# Use system CA bundle on macOS so Ansible/Python SSL works (avoids CERTIFICATE_VERIFY_FAILED).

set -e

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
ANSIBLE_DIR="$REPO_ROOT/ansible"
cd "$ANSIBLE_DIR"

# Use system CA bundle on macOS so Ansible/Python SSL works (avoids CERTIFICATE_VERIFY_FAILED)
if [[ "$(uname)" == "Darwin" ]]; then
  for f in /etc/ssl/cert.pem /usr/local/etc/openssl/cert.pem "$(brew --prefix 2>/dev/null)/etc/openssl@3/cert.pem"; do
    if [[ -n "$f" ]] && [[ -f "$f" ]]; then
      export SSL_CERT_FILE="$f"
      export REQUESTS_CA_BUNDLE="$f"
      break
    fi
  done
fi

# Pin Python interpreter so Ansible does not warn about "discovered" interpreter
if [[ -z "${ANSIBLE_PYTHON_INTERPRETER:-}" ]]; then
  if command -v python3 &>/dev/null; then
    ANSIBLE_PYTHON_INTERPRETER="$(command -v python3)"
    export ANSIBLE_PYTHON_INTERPRETER
  elif [[ "$(uname)" == "Darwin" ]] && [[ -x /opt/homebrew/bin/python3 ]]; then
    ANSIBLE_PYTHON_INTERPRETER="/opt/homebrew/bin/python3"
    export ANSIBLE_PYTHON_INTERPRETER
  else
    ANSIBLE_PYTHON_INTERPRETER="/usr/bin/python3"
    export ANSIBLE_PYTHON_INTERPRETER
  fi
fi

echo "=== Ansible Development Setup ==="
echo ""

# --- Check Ansible ---
if ! command -v ansible-playbook &>/dev/null; then
  echo "❌ Ansible not found!"
  echo ""
  if [[ "$(uname)" == "Darwin" ]]; then
    echo "Install via Homebrew:"
    echo "  brew install ansible"
  else
    echo "Install via package manager:"
    echo "  sudo apt install ansible-core   # Debian/Ubuntu"
    echo "  sudo dnf install ansible-core   # Fedora/RHEL"
  fi
  echo ""
  echo "Or via pip/uv:"
  echo "  uv tool install ansible-core"
  echo "  python3 -m pip install --user ansible-core"
  exit 1
fi

echo "✅ Ansible found: $(ansible-playbook --version | head -1)"
echo ""

# --- Install Galaxy requirements ---
if [[ -f requirements.yml ]]; then
  echo "1. Installing Ansible Galaxy requirements..."
  if ansible-galaxy collection install -r requirements.yml --force-with-deps 2>/dev/null; then
    echo "   ✅ Galaxy requirements installed"
  else
    echo "   ⚠️  Galaxy install had issues (continuing anyway)"
  fi
  echo ""
else
  echo "1. No requirements.yml found, skipping galaxy install"
  echo ""
fi

# --- Syntax check ---
echo "2. Checking playbook syntax..."
if ansible-playbook --syntax-check -i inventories/development playbooks/development.yml; then
  echo "   ✅ Syntax check passed"
else
  echo "   ❌ Syntax check failed"
  exit 1
fi
echo ""

# --- Show tasks ---
echo "3. Tasks that will run:"
ansible-playbook --list-tasks -i inventories/development playbooks/development.yml | grep -E "^(  |    )" | head -40
echo ""

# --- Run playbook ---
echo "4. Running development playbook..."
# Skip sudo prompt when ANSIBLE_NO_BECOME=1 or AI_RUN=1 (e.g. AI/automated run; macOS tasks need no become anyway)
if [[ -n "${ANSIBLE_NO_BECOME:-}" ]] || [[ -n "${AI_RUN:-}" ]]; then
  echo "   (no-become mode: not prompting for sudo; skipping Nix install and other privileged tasks)"
  # Use repo_root so cache/tmp live under the repo (writable in sandbox/CI). Skip tasks that need sudo.
  ansible-playbook -i inventories/development playbooks/development.yml \
    -e "repo_root=${REPO_ROOT}" \
    -e "skip_privileged_tasks=true" \
    -e "ansible_python_interpreter=${ANSIBLE_PYTHON_INTERPRETER:-/usr/bin/python3}"
else
  # On macOS no tasks use become (all become tasks are Debian-only), so do not prompt for password.
  if [[ "$(uname)" == "Darwin" ]]; then
    ansible-playbook -i inventories/development playbooks/development.yml \
      -e "ansible_python_interpreter=${ANSIBLE_PYTHON_INTERPRETER:-/usr/bin/python3}"
  else
    ansible-playbook -i inventories/development playbooks/development.yml \
      -e "ansible_python_interpreter=${ANSIBLE_PYTHON_INTERPRETER:-/usr/bin/python3}" \
      --ask-become-pass
  fi
fi
echo ""
echo "=== Setup Complete ==="
echo ""
echo "Verify: cmake --version, ninja --version, uv --version, ctest --test-dir build --output-on-failure"
