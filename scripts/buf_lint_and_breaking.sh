#!/usr/bin/env bash
# Run buf lint and buf breaking on proto/ from repo root.
# Usage: ./scripts/buf_lint_and_breaking.sh [--lint-only | --breaking-only]
# Requires buf (brew install bufbuild/buf/buf or https://buf.build/docs/cli/installation).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROTO_DIR="$REPO_ROOT/proto"
RUN_LINT=1
RUN_BREAKING=1

for arg in "$@"; do
  case "$arg" in
  --lint-only) RUN_BREAKING=0 ;;
  --breaking-only) RUN_LINT=0 ;;
  -h | --help)
    echo "Usage: $0 [--lint-only | --breaking-only]"
    echo "  Default: run both buf lint and buf breaking (against .git#branch=main)."
    exit 0
    ;;
  esac
done

if ! command -v buf >/dev/null 2>&1; then
  echo "buf not found. Install: brew install bufbuild/buf/buf" >&2
  echo "See https://buf.build/docs/cli/installation" >&2
  exit 1
fi

cd "$REPO_ROOT"
exit_code=0

# Use paths relative to REPO_ROOT (buf expects relative paths for --path)
PROTO_REL="${PROTO_DIR#$REPO_ROOT/}"

if [[ "$RUN_LINT" -eq 1 ]]; then
  echo "=== buf lint (proto/) ==="
  if buf lint --path "$PROTO_REL"; then
    echo "[buf lint] OK"
  else
    exit_code=1
  fi
fi

if [[ "$RUN_BREAKING" -eq 1 ]]; then
  echo "=== buf breaking (proto/ vs main) ==="
  if buf breaking --path "$PROTO_REL" --against ".git#branch=main"; then
    echo "[buf breaking] OK"
  else
    exit_code=1
  fi
fi

exit "$exit_code"
