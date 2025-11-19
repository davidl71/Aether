#!/usr/bin/env bash
# install_shellspec.sh - Install ShellSpec testing framework
#
# AI CONTEXT:
# This script installs ShellSpec for testing shell scripts.
# ShellSpec is a BDD testing framework for shell scripts.
#
# Usage:
#   ./scripts/install_shellspec.sh
#
# Options:
#   --local    Install in project directory (bin/shellspec)
#   --global   Install globally (~/.local/bin/shellspec) [default]
#   --help     Show this help message

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

INSTALL_MODE="global"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --local)
      INSTALL_MODE="local"
      shift
      ;;
    --global)
      INSTALL_MODE="global"
      shift
      ;;
    --help)
      cat <<EOF
Install ShellSpec testing framework

Usage: $0 [options]

Options:
  --local     Install in project directory (bin/shellspec)
  --global    Install globally (~/.local/bin/shellspec) [default]
  --help      Show this help message

Examples:
  $0                # Install globally
  $0 --local        # Install in project directory
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Run '$0 --help' for usage information" >&2
      exit 1
      ;;
  esac
done

echo "Installing ShellSpec..."
echo ""

if [ "${INSTALL_MODE}" = "local" ]; then
  echo "Installing ShellSpec locally in project directory..."
  echo ""

  # Create bin directory if it doesn't exist
  mkdir -p "${PROJECT_ROOT}/bin"

  # Install ShellSpec
  curl -fsSL https://git.io/shellspec | sh -s -- --yes

  # Check if it was installed
  if [ -f "${PROJECT_ROOT}/bin/shellspec" ]; then
    echo ""
    echo "✓ ShellSpec installed successfully at ${PROJECT_ROOT}/bin/shellspec"
    echo ""
    echo "Run tests with:"
    echo "  ./bin/shellspec"
    echo "  # or"
    echo "  ./scripts/run_tests.sh"
  else
    echo ""
    echo "✗ ShellSpec installation may have failed"
    echo "  Check the output above for errors"
    exit 1
  fi
else
  echo "Installing ShellSpec globally..."
  echo ""
  echo "This will install ShellSpec to ~/.local/bin/shellspec"
  echo ""

  # Install ShellSpec (requires user confirmation)
  curl -fsSL https://git.io/shellspec | sh

  # Check if it was installed
  if [ -f "${HOME}/.local/bin/shellspec" ]; then
    echo ""
    echo "✓ ShellSpec installed successfully at ${HOME}/.local/bin/shellspec"
    echo ""
    echo "Add to PATH if needed:"
    echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo ""
    echo "Or add to your shell profile (~/.zshrc, ~/.bashrc, etc.):"
    echo "  echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> ~/.zshrc"
    echo ""
    echo "Run tests with:"
    echo "  ./scripts/run_tests.sh"
  else
    echo ""
    echo "✗ ShellSpec installation may have failed"
    echo "  Check the output above for errors"
    exit 1
  fi
fi
