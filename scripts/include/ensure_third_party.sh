#!/usr/bin/env bash
# ensure_third_party.sh - Ensure TWS API and Intel decimal exist; run fetch_third_party.sh if missing.
# Source from build scripts after cd to project root. Usage:
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   . "${SCRIPT_DIR}/include/ensure_third_party.sh"
#   ensure_third_party   # call when PWD is project root

if [[ -n "${__IB_BOX_SPREAD_ENSURE_THIRD_PARTY_INCLUDED:-}" ]]; then
  return 0 2>/dev/null || :
fi
__IB_BOX_SPREAD_ENSURE_THIRD_PARTY_INCLUDED=1

ensure_third_party() {
  local need_fetch=false
  # CMake expects TWS API at native/third_party/tws-api (include: .../source/cppclient/client or .../IBJts/...)
  if [ ! -d "native/third_party/tws-api/source/cppclient/client" ] && [ ! -d "native/third_party/tws-api/IBJts" ]; then
    need_fetch=true
  fi
  # Intel decimal: CMake may use U2 or U4
  if [ ! -d "native/third_party/IntelRDFPMathLib20U4" ] && [ ! -d "native/third_party/IntelRDFPMathLib20U2" ]; then
    need_fetch=true
  fi
  if [ "$need_fetch" = true ]; then
    if ! command -v ansible-playbook &>/dev/null; then
      echo "Third-party deps missing (TWS API and/or Intel decimal). Install Ansible and run:" >&2
      echo "  ./scripts/fetch_third_party.sh" >&2
      exit 1
    fi
    echo "=== Fetching missing third-party dependencies ==="
    ./scripts/fetch_third_party.sh
  fi
}
