#!/usr/bin/env bash
# Test script to verify repository installation setup (dry-run), or install if run as root with --install.
# Usage:
#   ./scripts/test_repo_install.sh              # Dry-run: show status and required tools
#   sudo ./scripts/test_repo_install.sh --install   # Install repo (after status checks)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="${PROJECT_ROOT}/deb-repo"

INSTALL_MODE=false
REMAINING_ARGS=()
for arg in "$@"; do
  if [ "$arg" = "--install" ]; then
    INSTALL_MODE=true
  else
    REMAINING_ARGS+=("$arg")
  fi
done

echo "=== Repository Installation Test ==="
echo ""
echo "Repository location: $REPO_DIR"
echo ""

# Check if repo exists
if [ ! -d "$REPO_DIR" ]; then
  echo "⚠ Repository does not exist yet."
  echo "  Run: ./scripts/create_deb_repo.sh"
  echo ""
else
  echo "✓ Repository directory exists"
fi

# Check for GPG key
if [ -f "$REPO_DIR/public.key" ]; then
  echo "✓ GPG public key found"
  echo "  Key location: $REPO_DIR/public.key"
else
  echo "⚠ GPG public key not found"
  echo "  Will be generated when creating repository"
fi

# Check for packages
if [ -d "$REPO_DIR/pool" ] && [ "$(ls -A $REPO_DIR/pool 2>/dev/null)" ]; then
  echo "✓ Packages found in pool/"
  ls -1 "$REPO_DIR/pool" | head -5
else
  echo "⚠ No packages found in pool/"
  echo "  Packages will be created when building repository"
fi

# Check Ubuntu version
if [ -f /etc/os-release ]; then
  source /etc/os-release
  echo ""
  echo "Detected OS: $ID $VERSION_ID"
  if [ "$ID" = "ubuntu" ]; then
    UBUNTU_MAJOR=$(echo "$VERSION_ID" | cut -d'.' -f1)
    if [ "$UBUNTU_MAJOR" -ge 25 ]; then
      echo "✓ Ubuntu 25.04+ detected - will use .sources format"
    else
      echo "⚠ Ubuntu < 25.04 - .sources format will still be used (recommended)"
    fi
  fi
fi

# Check for required tools
echo ""
echo "=== Required Tools ==="
for cmd in gpg apt-get; do
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "✓ $cmd found: $(which $cmd)"
  else
    echo "✗ $cmd not found"
  fi
done

echo ""
echo "=== Installation Command ==="
echo "To install the repository, run:"
echo "  sudo $PROJECT_ROOT/scripts/install_deb_repo.sh"
echo ""
echo "Or use this script with --install (as root):"
echo "  sudo $0 --install"
echo ""
echo "Or if repository is on a remote server:"
echo "  sudo $PROJECT_ROOT/scripts/install_deb_repo.sh --repo-url http://your-server/deb-repo"
echo ""

if [ "$INSTALL_MODE" = true ]; then
  if [ "$EUID" -eq 0 ]; then
    echo "Running as root - proceeding with installation..."
    echo ""
    exec "$SCRIPT_DIR/install_deb_repo.sh" "${REMAINING_ARGS[@]}"
  else
    echo "⚠ This script must be run as root to install. Run: sudo $0 --install"
    exit 1
  fi
fi
