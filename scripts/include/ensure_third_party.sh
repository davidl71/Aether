#!/usr/bin/env bash
# ensure_third_party.sh - Ensure TWS API, Intel decimal, and system deps (Boost, Protobuf) exist.
# Run fetch_third_party.sh when vendored deps are missing; install system packages when missing (macOS/Linux).
# Source from build scripts after cd to project root. Usage:
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   . "${SCRIPT_DIR}/include/ensure_third_party.sh"
#   ensure_third_party   # call when PWD is project root

if [[ -n "${__IB_BOX_SPREAD_ENSURE_THIRD_PARTY_INCLUDED:-}" ]]; then
  # shellcheck disable=SC2317
  return 0 2>/dev/null || true
fi
__IB_BOX_SPREAD_ENSURE_THIRD_PARTY_INCLUDED=1

# Ensure system packages required by CMake (Boost, Protobuf) are installed.
ensure_system_deps() {
  local need_boost=false need_protobuf=false
  if [[ "$(uname -s)" == "Darwin" ]]; then
    if command -v brew &>/dev/null; then
      brew list boost &>/dev/null    || need_boost=true
      brew list protobuf &>/dev/null || need_protobuf=true
      if [ "$need_boost" = true ] || [ "$need_protobuf" = true ]; then
        echo "=== Installing missing system dependencies (Boost, Protobuf) ==="
        [ "$need_boost" = true ]    && brew install boost
        [ "$need_protobuf" = true ] && brew install protobuf
      fi
    else
      [ ! -d /usr/local/include/boost ] && [ ! -d /opt/homebrew/include/boost ] && echo "Install Boost: brew install boost" >&2
      [ ! -f /usr/local/lib/libprotobuf.dylib ] && [ ! -f /opt/homebrew/lib/libprotobuf.dylib ] && echo "Install Protobuf: brew install protobuf" >&2
    fi
  else
    if ! pkg-config --exists boost 2>/dev/null && [ ! -d /usr/include/boost ]; then need_boost=true; fi
    if ! pkg-config --exists protobuf 2>/dev/null && [ ! -f /usr/lib/x86_64-linux-gnu/libprotobuf.so ] && [ ! -f /usr/lib64/libprotobuf.so ]; then need_protobuf=true; fi
    if [ "$need_boost" = true ] || [ "$need_protobuf" = true ]; then
      echo "Install missing system dependencies (run with sudo):" >&2
      [ "$need_boost" = true ]    && echo "  sudo apt-get install -y libboost-all-dev" >&2
      [ "$need_protobuf" = true ] && echo "  sudo apt-get install -y libprotobuf-dev protobuf-compiler" >&2
    fi
  fi
}

ensure_third_party() {
  ensure_system_deps

  # Optional: use read-only compressed DMG for third-party trees (macOS, reduces disk reads)
  if [[ -n "${USE_THIRD_PARTY_DMG:-}" ]] && [[ "$(uname -s)" == "Darwin" ]]; then
    local dmg_path=".dmg/ThirdParty.dmg"
    if [[ -f "${dmg_path}" ]]; then
      # If symlinks not already active, mount DMG and activate
      if [[ ! -L "native/third_party/tws-api" ]] || [[ "$(readlink native/third_party/tws-api 2>/dev/null)" != /Volumes/IBBoxSpreadThirdParty/tws-api ]]; then
        ./scripts/third_party_dmg.sh mount || true
      fi
    fi
  fi

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
