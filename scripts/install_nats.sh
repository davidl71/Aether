#!/usr/bin/env bash
# Install NATS server and CLI tools for macOS
# Supports Homebrew and direct binary installation
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[info] Installing NATS server and tools..."

# Check if already installed
if command -v nats-server >/dev/null 2>&1; then
  echo "[info] NATS server already installed: $(nats-server -v)"
  exit 0
fi

# Try Homebrew first (preferred)
if command -v brew >/dev/null 2>&1; then
  echo "[info] Installing via Homebrew..."

  # Add NATS tap if not already added
  if ! brew tap | grep -q "nats-io/nats-tools"; then
    brew tap nats-io/nats-tools
  fi

  # Install NATS server
  brew install nats-server || {
    echo "[warn] Homebrew installation failed, trying direct binary..."
    install_nats_binary
  }

  # Install NATS CLI tools
  brew install nats-io/nats-tools/nats || echo "[warn] NATS CLI installation failed (optional)"

  echo "[info] NATS server installed via Homebrew"
  echo "[info] Version: $(nats-server -v)"
  exit 0
fi

# Fallback to direct binary installation
install_nats_binary() {
  echo "[info] Installing NATS server binary directly..."

  NATS_VERSION="2.10.18"
  ARCH="amd64"
  if [[ $(uname -m) == "arm64" ]]; then
    ARCH="arm64"
  fi

  DOWNLOAD_URL="https://github.com/nats-io/nats-server/releases/download/v${NATS_VERSION}/nats-server-v${NATS_VERSION}-darwin-${ARCH}.zip"
  INSTALL_DIR="/usr/local/bin"

  # Create temp directory
  TEMP_DIR=$(mktemp -d)
  trap "rm -rf $TEMP_DIR" EXIT

  # Download and extract
  echo "[info] Downloading NATS server v${NATS_VERSION}..."
  curl -L -o "$TEMP_DIR/nats-server.zip" "$DOWNLOAD_URL"
  unzip -q "$TEMP_DIR/nats-server.zip" -d "$TEMP_DIR"

  # Install binary
  if [[ -w "$INSTALL_DIR" ]]; then
    cp "$TEMP_DIR/nats-server-v${NATS_VERSION}-darwin-${ARCH}/nats-server" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/nats-server"
    echo "[info] NATS server installed to $INSTALL_DIR/nats-server"
  else
    echo "[info] Need sudo to install to $INSTALL_DIR"
    sudo cp "$TEMP_DIR/nats-server-v${NATS_VERSION}-darwin-${ARCH}/nats-server" "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/nats-server"
    echo "[info] NATS server installed to $INSTALL_DIR/nats-server"
  fi

  # Verify installation
  if command -v nats-server >/dev/null 2>&1; then
    echo "[info] NATS server installed successfully"
    echo "[info] Version: $(nats-server -v)"
  else
    echo "[error] Installation failed - nats-server not found in PATH"
    exit 1
  fi
}

install_nats_binary

echo "[info] NATS installation complete!"
