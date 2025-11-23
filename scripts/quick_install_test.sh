#!/usr/bin/env bash
# Quick test of repository installation (shows what would happen)
set -e

REPO_DIR="/home/david/ib_box_spread_full_universal/deb-repo"

echo "=== Repository Installation Test ==="
echo ""

if [ ! -d "$REPO_DIR" ]; then
  echo "❌ Repository does not exist at: $REPO_DIR"
  echo ""
  echo "Please create it first with:"
  echo "  ./scripts/create_deb_repo.sh"
  echo ""
  exit 1
fi

if [ ! -f "$REPO_DIR/public.key" ]; then
  echo "⚠️  GPG key not found. Repository may not be fully set up."
  echo "   Run: ./scripts/create_deb_repo.sh"
  echo ""
fi

echo "✓ Repository found at: $REPO_DIR"
echo ""

if [ "$EUID" -eq 0 ]; then
  echo "Running as root - proceeding with installation..."
  echo ""
  exec "$(dirname "$0")/install_deb_repo.sh" "$@"
else
  echo "⚠️  This script needs to run as root (sudo)"
  echo ""
  echo "To install the repository, run:"
  echo "  sudo $0"
  echo ""
  echo "Or directly:"
  echo "  sudo $(dirname "$0")/install_deb_repo.sh"
  echo ""
  exit 1
fi
