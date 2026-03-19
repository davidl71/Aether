#!/usr/bin/env bash
# Replace exarp-go's default "MCP Server" overview in generated plan files with the real project description.
# Exarp-go infers "MCP Server" when run via MCP; this script fixes .cursor/plans/*.plan.md after regeneration.
# Usage: from repo root, ./scripts/fix_exarp_plan_overview.sh

set -e

REPO_ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null)}"
PLANS_DIR="${REPO_ROOT}/.cursor/plans"
OVERVIEW='Multi-asset synthetic financing platform (Aether); Rust backend/TUI, box spreads, IBKR integration'
PURPOSE='Multi-asset synthetic financing platform (Aether). Rust backend, TUI, box spreads, IBKR integration; unified portfolio and cash flow across options, futures, bonds, loans.'
PROJECT_TYPE='Trading/financing platform (Rust-first)'

if [[ ! -d "$PLANS_DIR" ]]; then
  echo "No .cursor/plans directory found." >&2
  exit 0
fi

for f in "$PLANS_DIR"/*.plan.md; do
  [[ -f "$f" ]] || continue
  if grep -q 'overview: "MCP Server"' "$f" || grep -q '\*\*Purpose:\*\* MCP Server' "$f"; then
    tmp=$(mktemp)
    sed \
      -e "s|overview: \"MCP Server\"|overview: \"${OVERVIEW}\"|g" \
      -e "s|\*\*Purpose:\*\* MCP Server|\*\*Purpose:\*\* ${PURPOSE}|g" \
      -e "s|\*\*Project type:\*\* MCP Server|\*\*Project type:\*\* ${PROJECT_TYPE}|g" \
      "$f" >"$tmp" && mv "$tmp" "$f"
    echo "Patched overview in $(basename "$f")"
  fi
done
