#!/usr/bin/env bash
# TUI vs CLI feature parity check — prints doc path and short summary.
# Source of truth: docs/platform/TUI_CLI_FEATURE_PARITY.md
# Usage: ./scripts/check_feature_parity.sh

set -e

SCRIPT_DIR="${SCRIPT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
DOC="$REPO_ROOT/docs/platform/TUI_CLI_FEATURE_PARITY.md"

echo "TUI / CLI feature parity"
echo "Source of truth: docs/platform/TUI_CLI_FEATURE_PARITY.md"
echo ""
if [ -f "$DOC" ]; then
  echo "Summary: TUI uses shared JSON config + NATS; CLI uses TOML + stub loop. See doc for gaps and recommendations."
  echo "Doc path: $DOC"
else
  echo "Warning: doc not found at $DOC" >&2
  exit 1
fi
