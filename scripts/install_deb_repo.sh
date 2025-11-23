#!/usr/bin/env bash
# Install script for IB Box Spread Debian Repository
# Supports Ubuntu 25.04+ with .sources format and GPG signing
# Usage: ./scripts/install_deb_repo.sh [--repo-url URL] [--key-url URL]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
REPO_URL="${REPO_URL:-file://$PROJECT_ROOT/deb-repo}"
KEY_URL="${KEY_URL:-}"
REPO_NAME="ib-box-spread"
SOURCES_FILE="/etc/apt/sources.list.d/${REPO_NAME}.sources"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --repo-url)
      REPO_URL="$2"
      shift 2
      ;;
    --key-url)
      KEY_URL="$2"
      shift 2
      ;;
    *)
      log_error "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if running as root
check_root() {
  if [ "$EUID" -ne 0 ]; then
    log_error "This script must be run as root (use sudo)"
    exit 1
  fi
}

# Detect Ubuntu version
detect_ubuntu_version() {
  if [ ! -f /etc/os-release ]; then
    log_error "Cannot detect OS version. /etc/os-release not found."
    exit 1
  fi

  source /etc/os-release

  if [ "$ID" != "ubuntu" ]; then
    log_warn "This script is designed for Ubuntu. Detected: $ID"
  fi

  # Extract version number (e.g., "25.04" from "25.04")
  UBUNTU_VERSION=$(echo "$VERSION_ID" | cut -d'.' -f1,2)
  UBUNTU_MAJOR=$(echo "$VERSION_ID" | cut -d'.' -f1)
  UBUNTU_MINOR=$(echo "$VERSION_ID" | cut -d'.' -f2)

  log_info "Detected Ubuntu version: $UBUNTU_VERSION"

  # Check if version is >= 25.04
  if [ "$UBUNTU_MAJOR" -lt 25 ] || ([ "$UBUNTU_MAJOR" -eq 25 ] && [ "${UBUNTU_MINOR:-0}" -lt 4 ]); then
    log_warn "Ubuntu version $UBUNTU_VERSION is less than 25.04"
    log_warn "Using .sources format anyway (recommended for all modern Ubuntu versions)"
  fi
}

# Download and import GPG key
import_gpg_key() {
  log_info "Importing GPG key for repository..."

  local key_file="/tmp/${REPO_NAME}-gpg.key"
  local key_fingerprint=""

  # Determine key URL
  if [ -z "$KEY_URL" ]; then
    # Try to get key from repo URL
    if [[ "$REPO_URL" == file://* ]]; then
      local repo_path="${REPO_URL#file://}"
      if [ -f "$repo_path/public.key" ]; then
        KEY_URL="file://$repo_path/public.key"
      fi
    elif [[ "$REPO_URL" == http* ]]; then
      KEY_URL="${REPO_URL%/}/public.key"
    fi
  fi

  # Download or copy key
  if [[ "$KEY_URL" == file://* ]]; then
    local key_path="${KEY_URL#file://}"
    if [ -f "$key_path" ]; then
      cp "$key_path" "$key_file"
      log_info "Copied GPG key from $key_path"
    else
      log_error "GPG key not found at $key_path"
      return 1
    fi
  elif [[ "$KEY_URL" == http* ]]; then
    log_info "Downloading GPG key from $KEY_URL"
    if command -v curl >/dev/null 2>&1; then
      curl -fsSL "$KEY_URL" -o "$key_file" || {
        log_error "Failed to download GPG key"
        return 1
      }
    elif command -v wget >/dev/null 2>&1; then
      wget -q "$KEY_URL" -O "$key_file" || {
        log_error "Failed to download GPG key"
        return 1
      }
    else
      log_error "Neither curl nor wget found. Cannot download GPG key."
      return 1
    fi
  else
    log_error "Invalid KEY_URL: $KEY_URL"
    return 1
  fi

  # Import key
  if ! gpg --no-default-keyring --keyring /usr/share/keyrings/${REPO_NAME}-archive-keyring.gpg --import "$key_file" 2>/dev/null; then
    log_error "Failed to import GPG key"
    rm -f "$key_file"
    return 1
  fi

  # Get key fingerprint
  key_fingerprint=$(gpg --no-default-keyring --keyring /usr/share/keyrings/${REPO_NAME}-archive-keyring.gpg --list-keys --with-colons 2>/dev/null | grep -E "^fpr:" | head -1 | cut -d: -f10)

  if [ -z "$key_fingerprint" ]; then
    log_warn "Could not determine key fingerprint, using keyring file path"
    export GPG_KEY_FINGERPRINT="/usr/share/keyrings/${REPO_NAME}-archive-keyring.gpg"
  else
    export GPG_KEY_FINGERPRINT="$key_fingerprint"
    log_success "GPG key imported. Fingerprint: $key_fingerprint"
  fi

  rm -f "$key_file"
}

# Create .sources file (Ubuntu 25.04+ format)
create_sources_file() {
  log_info "Creating .sources file for Ubuntu 25.04+..."

  # Determine if repo URL is file:// or http(s)://
  local repo_type=""
  local repo_uri=""

  if [[ "$REPO_URL" == file://* ]]; then
    repo_type="file"
    repo_uri="$REPO_URL"
  elif [[ "$REPO_URL" == http* ]]; then
    repo_type="http"
    repo_uri="$REPO_URL"
  else
    log_error "Invalid REPO_URL format: $REPO_URL"
    log_error "Must be file:// or http(s)://"
    exit 1
  fi

  # Create .sources file with Signed-By field
  cat > "$SOURCES_FILE" <<EOF
Types: deb
URIs: $repo_uri
Suites: stable
Components: main
Signed-By: /usr/share/keyrings/${REPO_NAME}-archive-keyring.gpg
EOF

  chmod 644 "$SOURCES_FILE"
  log_success "Created $SOURCES_FILE"

  # Show contents
  log_info "Repository configuration:"
  cat "$SOURCES_FILE" | sed 's/^/  /'
}

# Update apt cache
update_apt_cache() {
  log_info "Updating apt cache..."

  if apt-get update; then
    log_success "Apt cache updated successfully"
  else
    log_error "Failed to update apt cache"
    exit 1
  fi
}

# Verify repository
verify_repository() {
  log_info "Verifying repository..."

  if apt-cache policy | grep -q "$REPO_NAME"; then
    log_success "Repository verified and available"

    # Show available packages
    log_info "Available packages:"
    apt-cache search "$REPO_NAME" 2>/dev/null | grep "^ib-box-spread\|^synthetic-financing-platform\|^project-management\|^trading-mcp" | sed 's/^/  /' || true
  else
    log_warn "Repository not found in apt cache. Packages may not be available yet."
  fi
}

# Main installation
main() {
  log_info "Installing IB Box Spread Debian Repository..."

  check_root
  detect_ubuntu_version
  import_gpg_key
  create_sources_file
  update_apt_cache
  verify_repository

  log_success "Repository installation complete!"
  log_info ""
  log_info "You can now install packages with:"
  log_info "  sudo apt-get install ib-box-spread-native"
  log_info "  sudo apt-get install synthetic-financing-platform"
  log_info "  sudo apt-get install ib-box-spread-web"
  log_info "  sudo apt-get install ib-box-spread-backend"
  log_info "  sudo apt-get install project-management-automation-mcp"
  log_info "  sudo apt-get install trading-mcp-server"
  log_info "  sudo apt-get install ib-box-spread-build-tools"
  log_info "  sudo apt-get install ib-box-spread-automation-tools"
  log_info ""
  log_info "Or install all packages:"
  log_info "  sudo apt-get install ib-box-spread-*"
}

main "$@"
