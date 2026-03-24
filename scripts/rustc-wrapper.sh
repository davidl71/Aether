#!/usr/bin/env bash
set -euo pipefail

# Use sccache when it is installed, but keep plain cargo invocations working
# on machines that do not have it.
if command -v sccache >/dev/null 2>&1; then
  exec sccache "$@"
fi

exec "$@"
