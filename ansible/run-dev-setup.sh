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
ansible-playbook -i inventories/development playbooks/development.yml --ask-become-pass
echo ""
echo "=== Setup Complete ==="
echo ""
echo "Verify: cmake --version, ninja --version, uv --version, ctest --test-dir build --output-on-failure"
