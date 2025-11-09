#!/usr/bin/env bash
# Convenience wrapper to download the Nautilus Trader wheel only via fetch_third_party.sh.
# Useful for automation to pre-stage the wheel under native/third_party/nautilus.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export FETCH_COMPONENTS="nautilus"

"${SCRIPT_DIR}/fetch_third_party.sh"


