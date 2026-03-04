#!/usr/bin/env bash
# Clean Rust build and optional global caches.
#
# Usage:
#   ./scripts/clean_rust_cache.sh           # workspace only (cargo clean in agents/backend)
#   ./scripts/clean_rust_cache.sh --global  # also clean ~/.cargo/registry/cache and ~/.cargo/git
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

do_global=0
for a in "$@"; do
  case "${a}" in
    --global) do_global=1 ;;
    *) echo "Usage: $0 [--global]" 1>&2; exit 1 ;;
  esac
done

echo "==> Cleaning Rust workspace (agents/backend)"
(cd "${ROOT}/agents/backend" && cargo clean)

if [[ "${do_global}" -eq 1 ]]; then
  echo "==> Cleaning global Cargo registry cache"
  rm -rf "${HOME}/.cargo/registry/cache"
  echo "==> Cleaning global Cargo git checkouts"
  rm -rf "${HOME}/.cargo/git"
  echo "Done. Next cargo build will re-download as needed."
else
  echo "Done. Use --global to also clear ~/.cargo/registry/cache and ~/.cargo/git."
fi
