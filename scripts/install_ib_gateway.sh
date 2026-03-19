#!/bin/bash
# IB Gateway Installation Script for macOS
# Automatically downloads and installs IB Gateway

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}IB Gateway Installer${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Detect architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
  echo -e "${GREEN}✓ Detected Apple Silicon (ARM64)${NC}"
  DOWNLOAD_URL="https://download2.interactivebrokers.com/installers/ibgateway/latest-standalone/ibgateway-latest-standalone-macos-arm64.dmg"
  DMG_NAME="ibgateway-macos-arm64.dmg"
elif [[ "$ARCH" == "x86_64" ]]; then
  echo -e "${GREEN}✓ Detected Intel (x86_64)${NC}"
  DOWNLOAD_URL="https://download2.interactivebrokers.com/installers/ibgateway/latest-standalone/ibgateway-latest-standalone-macos-x64.dmg"
  DMG_NAME="ibgateway-macos-x64.dmg"
else
  echo -e "${RED}✗ Unsupported architecture: $ARCH${NC}"
  exit 1
fi

# Create temp directory
TEMP_DIR=$(mktemp -d)
DMG_PATH="$TEMP_DIR/$DMG_NAME"

echo ""
echo -e "${YELLOW}Step 1: Downloading IB Gateway...${NC}"
echo "URL: $DOWNLOAD_URL"
echo "Destination: $DMG_PATH"
echo ""

# Download with progress
if command -v wget &>/dev/null; then
  wget --progress=bar:force -O "$DMG_PATH" "$DOWNLOAD_URL" 2>&1
elif command -v curl &>/dev/null; then
  curl -# -L -o "$DMG_PATH" "$DOWNLOAD_URL"
else
  echo -e "${RED}✗ Neither curl nor wget found. Please install one.${NC}"
  exit 1
fi

if [ ! -f "$DMG_PATH" ]; then
  echo -e "${RED}✗ Download failed - file not found${NC}"
  exit 1
fi

FILE_SIZE=$(du -h "$DMG_PATH" | cut -f1)
echo -e "${GREEN}✓ Downloaded successfully ($FILE_SIZE)${NC}"
echo ""

# Mount the DMG
echo -e "${YELLOW}Step 2: Mounting DMG...${NC}"
MOUNT_POINT=$(hdiutil attach "$DMG_PATH" -nobrowse -noautoopen | grep Volumes | awk '{print $3}')

if [ -z "$MOUNT_POINT" ]; then
  echo -e "${RED}✗ Failed to mount DMG${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Mounted at: $MOUNT_POINT${NC}"
echo ""

# Find the .app bundle
echo -e "${YELLOW}Step 3: Finding IB Gateway.app...${NC}"
APP_SOURCE=$(find "$MOUNT_POINT" -name "*.app" -maxdepth 2 -type d | head -1)

if [ -z "$APP_SOURCE" ]; then
  echo -e "${RED}✗ Could not find .app bundle in DMG${NC}"
  hdiutil detach "$MOUNT_POINT" -quiet
  exit 1
fi

APP_NAME=$(basename "$APP_SOURCE")
echo -e "${GREEN}✓ Found: $APP_NAME${NC}"
echo ""

# Copy to Applications
echo -e "${YELLOW}Step 4: Installing to /Applications...${NC}"
DEST_PATH="/Applications/$APP_NAME"

# Check if already exists
if [ -d "$DEST_PATH" ]; then
  echo -e "${YELLOW}⚠ IB Gateway already exists at $DEST_PATH${NC}"
  read -p "Do you want to replace it? (y/n) " -n 1 -r
  echo ""
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing old version..."
    rm -rf "$DEST_PATH"
  else
    echo -e "${BLUE}Installation cancelled${NC}"
    hdiutil detach "$MOUNT_POINT" -quiet
    rm -rf "$TEMP_DIR"
    exit 0
  fi
fi

echo "Copying $APP_NAME to /Applications..."
cp -R "$APP_SOURCE" /Applications/

if [ ! -d "$DEST_PATH" ]; then
  echo -e "${RED}✗ Installation failed${NC}"
  hdiutil detach "$MOUNT_POINT" -quiet
  exit 1
fi

echo -e "${GREEN}✓ Installed successfully${NC}"
echo ""

# Cleanup
echo -e "${YELLOW}Step 5: Cleaning up...${NC}"
hdiutil detach "$MOUNT_POINT" -quiet
rm -rf "$TEMP_DIR"
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

# Verify installation
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ Installation Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "IB Gateway installed at:"
echo "  $DEST_PATH"
echo ""
echo "Next steps:"
echo "  1. Launch IB Gateway:"
echo "     open \"/Applications/$APP_NAME\""
echo ""
echo "  2. Login with your IBKR credentials"
echo "     Trading Mode: Paper Trading (safe)"
echo ""
echo "  3. Configure API settings:"
echo "     Configure → Settings → API → Settings"
echo "     ✓ Enable ActiveX and Socket Clients"
echo "     ✓ Socket Port: 7497"
echo "     ✓ Trusted IPs: 127.0.0.1"
echo ""
echo "  4. Test connection:"
echo "     ./scripts/test_tws_connection.sh"
echo ""
echo "  5. Run your application:"
echo "     python -m python.tui"
echo ""

# Offer to launch
read -p "Do you want to launch IB Gateway now? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
  echo -e "${GREEN}Launching IB Gateway...${NC}"
  open "/Applications/$APP_NAME"
  echo ""
  echo -e "${YELLOW}⚠ IMPORTANT:${NC}"
  echo "  1. Login with Paper Trading mode"
  echo "  2. Go to Configure → Settings → API → Settings"
  echo "  3. Enable API and set port to 7497"
  echo "  4. Run: ./scripts/test_tws_connection.sh"
fi

echo ""
echo -e "${GREEN}Done!${NC}"
