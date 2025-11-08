#!/usr/bin/env bash
# Fetch and extract third-party dependencies into the local cache.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="${REPO_ROOT}/third_party/cache"
PROTOBUF_VERSION="3.20.3"
PROTOBUF_ARCHIVE="protobuf-${PROTOBUF_VERSION}.tar.gz"
PROTOBUF_SRC_DIR="${REPO_ROOT}/third_party/protobuf-${PROTOBUF_VERSION}"
PROTOBUF_URL_DEFAULT="https://github.com/protocolbuffers/protobuf/archive/refs/tags/v${PROTOBUF_VERSION}.tar.gz"

INTEL_DIR="${REPO_ROOT}/third_party/IntelRDFPMathLib20U2"
IB_STUB_DIR="${REPO_ROOT}/third_party/tws-api"
NAUTILUS_DIR="${REPO_ROOT}/third_party/nautilus"

mkdir -p "${CACHE_DIR}"

log()
{
  printf '[fetch] %s\n' "$*"
}

warn()
{
  printf '\033[33m[warn]\033[0m %s\n' "$*"
}

err()
{
  printf '\033[31m[error]\033[0m %s\n' "$*" >&2
}

fetch_archive()
{
  local url="$1"
  local dest="$2"

  if [ -f "${dest}" ]; then
    log "Using cached archive ${dest}";
    return 0
  fi

  if command -v curl >/dev/null 2>&1; then
    log "Downloading ${url}";
    curl -L "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    log "Downloading ${url}";
    wget -O "$dest" "$url"
  else
    err "Neither curl nor wget is available; cannot download ${url}";
    return 1
  fi
}

extract_archive()
{
  local archive_path="$1"
  local dest_dir="$2"
  local strip_components="${3:-1}"

  rm -rf "$dest_dir"
  mkdir -p "$dest_dir"
  log "Extracting ${archive_path} -> ${dest_dir}";
  tar -xf "$archive_path" -C "$dest_dir" --strip-components "$strip_components"
}

setup_protobuf()
{
  local proto_cache="${CACHE_DIR}/${PROTOBUF_ARCHIVE}"
  local proto_url="${PROTOBUF_URL:-${PROTOBUF_URL_DEFAULT}}"

  fetch_archive "$proto_url" "$proto_cache"
  extract_archive "$proto_cache" "$PROTOBUF_SRC_DIR"
}

setup_intel_decimal()
{
  local intel_url="${INTEL_DECIMAL_URL:-}";
  local intel_archive="${CACHE_DIR}/IntelRDFPMathLib20U2.tar.gz"

  if [ -d "$INTEL_DIR" ]; then
    log "Intel decimal math library already present"
    return 0
  fi

  if [ -n "$intel_url" ]; then
    fetch_archive "$intel_url" "$intel_archive"
  elif [ ! -f "$intel_archive" ]; then
    warn "INTEL_DECIMAL_URL not set and no cached archive found."
    warn "Place IntelRDFPMathLib20U2.tar.gz in ${intel_archive} manually or set INTEL_DECIMAL_URL."
    return 0
  fi

  if [ -f "$intel_archive" ]; then
    extract_archive "$intel_archive" "$INTEL_DIR" 0
    log "Intel decimal math library ready"
  fi
}

setup_ib_api_stub()
{
  if [ -d "$IB_STUB_DIR" ]; then
    log "IB API already present under third_party/tws-api"
    return 0
  fi

  if [ -z "${IB_API_ARCHIVE:-}" ]; then
    warn "IB_API_ARCHIVE not set; skipping automatic IB API extraction."
    warn "Place the unpacked TWS API under third_party/tws-api or provide IB_API_ARCHIVE pointing to the zip."
    return 0
  fi

  local ib_cache
  ib_cache="${CACHE_DIR}/$(basename "${IB_API_ARCHIVE}")"

  if [ -f "$IB_API_ARCHIVE" ]; then
    cp "$IB_API_ARCHIVE" "$ib_cache"
  else
    fetch_archive "$IB_API_ARCHIVE" "$ib_cache"
  fi

  mkdir -p "$IB_STUB_DIR"
  log "Extracting IB API archive into third_party/tws-api"
  unzip -o "$ib_cache" -d "$IB_STUB_DIR"
}

setup_nautilus_trader()
{
  mkdir -p "$NAUTILUS_DIR"

  local wheel_url="${NAUTILUS_TRADER_WHEEL_URL:-}"
  local release_tag="${NAUTILUS_TRADER_RELEASE:-stable}"

  if [ -z "$wheel_url" ]; then
    if ! command -v curl >/dev/null 2>&1; then
      warn "curl is required to discover Nautilus Trader releases; skipping wheel fetch."
      return 0
    fi
    if ! command -v python3 >/dev/null 2>&1; then
      warn "python3 not available; cannot parse GitHub release JSON. Set NAUTILUS_TRADER_WHEEL_URL manually."
      return 0
    fi

    local api_url="${NAUTILUS_TRADER_RELEASE_API:-https://api.github.com/repos/nautechsystems/nautilus_trader/releases/tags/${release_tag}}"
    log "Querying Nautilus Trader release '${release_tag}'"
    local release_json
    if ! release_json="$(curl -sfL "$api_url")"; then
      warn "Unable to fetch Nautilus Trader release metadata from ${api_url}"
      return 0
    fi

    wheel_url="$(printf '%s' "$release_json" | python3 - <<'PY'
import json
import sys

try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

for asset in data.get("assets", []):
    name = asset.get("name", "")
    if name.endswith(".whl") and "py3" in name:
        url = asset.get("browser_download_url")
        if url:
            print(url)
        break
PY
)"
    if [ -z "$wheel_url" ]; then
      warn "No wheel asset found in Nautilus Trader release '${release_tag}'. Set NAUTILUS_TRADER_WHEEL_URL to override."
      return 0
    fi
  fi

  local wheel_dest="${NAUTILUS_DIR}/$(basename "$wheel_url")"
  fetch_archive "$wheel_url" "$wheel_dest"
  log "Nautilus Trader wheel ready at ${wheel_dest}"
}

setup_protobuf
setup_intel_decimal
setup_ib_api_stub
setup_nautilus_trader

log "Third-party fetch complete."
