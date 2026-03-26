#!/usr/bin/env bash
# Add recommended entries to .cursorignore (generated code, .dmg, web dist).
# Idempotent: skips if the block is already present.
# Usage: ./scripts/update_cursorignore.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CURSORIGNORE="${PROJECT_ROOT}/.cursorignore"

MARKER="# Generated code (reproducible at build; reduces AI context)"
BLOCK_FILE="$(mktemp)"
trap 'rm -f "${BLOCK_FILE}"' EXIT
cat <<'BLOCK_END' >"${BLOCK_FILE}"
# DMG images (third-party read-only disk images; binary)
.dmg/

# Generated code (reproducible at build; reduces AI context)
# Note: web/ folder archived (React PWA retired), keeping patterns for reference
# web/src/generated/
# web/src/proto/

# Web build output (archived)
# web/dist/
# web/dev-dist/
BLOCK_END

if grep -q "^${MARKER}$" "${CURSORIGNORE}" 2>/dev/null; then
  echo ".cursorignore already contains the block; nothing to do."
  exit 0
fi

# Insert after the first "**/build-*/" line (under Build Artifacts)
if ! grep -q '^\*\*/build-\*/' "${CURSORIGNORE}"; then
  echo "Could not find insertion point (**/build-*/) in .cursorignore" >&2
  exit 1
fi

tmp="$(mktemp)"
trap 'rm -f "${tmp}" "${BLOCK_FILE}"' EXIT
awk -v blockfile="${BLOCK_FILE}" '
  /^\*\*\/build-\*\/$/ && !done {
    print
    print ""
    while ((getline line < blockfile) > 0) print line
    close(blockfile)
    done = 1
    next
  }
  { print }
' "${CURSORIGNORE}" >"${tmp}"
mv "${tmp}" "${CURSORIGNORE}"
echo "Updated .cursorignore: added .dmg/, generated/, web/dist entries."
exit 0
