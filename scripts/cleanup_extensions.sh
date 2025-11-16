#!/usr/bin/env bash
# cleanup_extensions.sh - Remove redundant or conflicting extensions
# Usage: ./scripts/cleanup_extensions.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Extensions to remove
CONFLICTING=(
  "llvm-vs-code-extensions.vscode-clangd"  # Conflicts with cpptools
)

REDUNDANT=(
  "anysphere.cursorpyright"  # Redundant with Pylance
)

UNUSED=(
  "ms-vscode.powershell"  # Not used in project
  "amazonwebservices.codewhisperer-for-command-line-companion"  # AWS-specific, not used
)

# Check if extension is installed
is_installed() {
  local ext=$1
  cursor --list-extensions 2>/dev/null | grep -q "^${ext}$" || return 1
}

# Remove extension
remove_extension() {
  local ext=$1
  local reason=$2

  if is_installed "$ext"; then
    if [ "${DRY_RUN:-false}" = "true" ]; then
      log_warn "Would remove: $ext ($reason)"
      return 0
    fi

    log_info "Removing: $ext ($reason)"
    if cursor --uninstall-extension "$ext" 2>/dev/null; then
      log_success "Removed: $ext"
      return 0
    else
      log_error "Failed to remove: $ext"
      return 1
    fi
  else
    log_info "Not installed: $ext"
    return 0
  fi
}

# Main function
main() {
  local dry_run=false

  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case $1 in
      --dry-run)
        dry_run=true
        shift
        ;;
      -h|--help)
        cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --dry-run    Show what would be removed without actually removing
  -h, --help   Show this help message

This script removes:
  - Conflicting extensions (clangd vs cpptools)
  - Redundant extensions (cursorpyright vs Pylance)
  - Unused extensions (powershell, codewhisperer)

EOF
        exit 0
        ;;
      *)
        log_error "Unknown option: $1"
        exit 1
        ;;
    esac
  done

  if [ "$dry_run" = "true" ]; then
    DRY_RUN=true
    log_info "DRY RUN MODE - No extensions will be removed"
  fi

  echo ""
  log_info "Analyzing installed extensions..."
  echo ""

  # Check for conflicting extensions
  log_info "Checking for conflicting extensions..."
  local removed_conflicting=0
  for ext in "${CONFLICTING[@]}"; do
    if remove_extension "$ext" "Conflicts with recommended extension"; then
      removed_conflicting=$((removed_conflicting + 1))
    fi
  done

  # Check for redundant extensions
  log_info "Checking for redundant extensions..."
  local removed_redundant=0
  for ext in "${REDUNDANT[@]}"; do
    if remove_extension "$ext" "Redundant with Pylance"; then
      removed_redundant=$((removed_redundant + 1))
    fi
  done

  # Check for unused extensions
  log_info "Checking for unused extensions..."
  local removed_unused=0
  for ext in "${UNUSED[@]}"; do
    if remove_extension "$ext" "Not used in this project"; then
      removed_unused=$((removed_unused + 1))
    fi
  done

  # Summary
  echo ""
  if [ "$dry_run" = "true" ]; then
    log_info "DRY RUN COMPLETE"
    log_info "Run without --dry-run to actually remove extensions"
  else
    local total=$((removed_conflicting + removed_redundant + removed_unused))
    if [ $total -gt 0 ]; then
      log_success "Removed $total extension(s)"
      log_info "Reload Cursor: Cmd+Shift+P → 'Developer: Reload Window'"
    else
      log_info "No extensions to remove"
    fi
  fi

  echo ""
  log_info "Remaining extensions:"
  cursor --list-extensions 2>/dev/null | wc -l | xargs echo "  Total:"
}

# Run main function
main "$@"
