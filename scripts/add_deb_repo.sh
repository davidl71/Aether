#!/usr/bin/env bash
# Add the local Debian repository to apt sources
# DEPRECATED: Use install_deb_repo.sh instead for Ubuntu 25.04+ support
# Usage: ./scripts/add_deb_repo.sh [--repo-dir REPO_DIR]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="${REPO_DIR:-${PROJECT_ROOT}/deb-repo}"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --repo-dir)
      REPO_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "WARNING: This script uses the old sources.list format."
echo "For Ubuntu 25.04+ with GPG signing, use: ./scripts/install_deb_repo.sh"
echo ""

if [ ! -d "$REPO_DIR" ]; then
  echo "Error: Repository directory not found: $REPO_DIR"
  echo "Run ./scripts/create_deb_repo.sh first to create the repository"
  exit 1
fi

# Create sources.list.d entry (old format, no GPG)
SOURCES_FILE="/etc/apt/sources.list.d/ib-box-spread.list"

if [ -f "$SOURCES_FILE" ]; then
  echo "Repository already added to apt sources"
  echo "File: $SOURCES_FILE"
else
  echo "Adding repository to apt sources (old format, no GPG)..."
  echo "deb [trusted=yes] file://$REPO_DIR stable main" | sudo tee "$SOURCES_FILE" > /dev/null
  echo "Repository added successfully"
fi

# Update apt cache
echo "Updating apt cache..."
sudo apt-get update

echo "Done! You can now install packages with:"
echo "  sudo apt-get install ib-box-spread-native"
echo "  sudo apt-get install synthetic-financing-platform"
echo "  sudo apt-get install ib-box-spread-web"
echo "  sudo apt-get install ib-box-spread-backend"
