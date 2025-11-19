#!/usr/bin/env bash
# Automatically download and install IB Client Portal Gateway
# This script downloads, extracts, and sets up the gateway for easy use
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
GATEWAY_DIR="${IB_GATEWAY_DIR:-$ROOT_DIR/ib-gateway}"
GATEWAY_DOWNLOAD_URL="https://download2.interactivebrokers.com/portal/clientportal.gw.zip"
GATEWAY_ZIP="${GATEWAY_DIR}/clientportal.gw.zip"
GATEWAY_EXTRACT_DIR="${GATEWAY_DIR}"  # ZIP extracts directly to gateway directory
GATEWAY_RUN_SCRIPT="${GATEWAY_DIR}/run-gateway.sh"

# Colors for output (only if terminal supports it)
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  RED=$(tput setaf 1)
  GREEN=$(tput setaf 2)
  YELLOW=$(tput setaf 3)
  BLUE=$(tput setaf 4)
  NC=$(tput sgr0)
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  NC=''
fi

echo "${BLUE}IB Client Portal Gateway Installation${NC}"
echo ""

# Check if gateway is already installed
if [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
  echo "${GREEN}✓ IB Client Portal Gateway already installed at: ${GATEWAY_DIR}${NC}"
  echo ""
  echo "To start the gateway:"
  echo "  ${BLUE}${GATEWAY_RUN_SCRIPT}${NC}"
  echo ""
  read -p "Reinstall? (y/N): " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 0
  fi
  echo ""
  echo "${YELLOW}Removing existing installation...${NC}"
  # Remove extracted files but keep the directory structure
  find "${GATEWAY_DIR}" -mindepth 1 -maxdepth 1 ! -name "clientportal.gw.zip" -exec rm -rf {} + 2>/dev/null || true
  rm -f "${GATEWAY_ZIP}" "${GATEWAY_RUN_SCRIPT}" 2>/dev/null || true
fi

# Create gateway directory
mkdir -p "${GATEWAY_DIR}"

# Check for required tools
MISSING_TOOLS=()

if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
  MISSING_TOOLS+=("curl or wget")
fi

if ! command -v unzip >/dev/null 2>&1; then
  MISSING_TOOLS+=("unzip")
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
  echo "${RED}Error: Missing required tools: ${MISSING_TOOLS[*]}${NC}" >&2
  echo "" >&2
  echo "Install missing tools:" >&2
  if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  brew install ${MISSING_TOOLS[*]}" >&2
  elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "  sudo apt-get install ${MISSING_TOOLS[*]}" >&2
    echo "  # or: sudo yum install ${MISSING_TOOLS[*]}" >&2
  fi
  exit 1
fi

# Download the gateway
echo "${BLUE}Downloading IB Client Portal Gateway...${NC}"
echo "  URL: ${GATEWAY_DOWNLOAD_URL}"
echo "  Destination: ${GATEWAY_ZIP}"
echo ""

if command -v curl >/dev/null 2>&1; then
  if curl -L -f --progress-bar -o "${GATEWAY_ZIP}" "${GATEWAY_DOWNLOAD_URL}"; then
    echo "${GREEN}✓ Download complete${NC}"
  else
    echo "${RED}Error: Download failed${NC}" >&2
    exit 1
  fi
elif command -v wget >/dev/null 2>&1; then
  if wget --progress=bar:force -O "${GATEWAY_ZIP}" "${GATEWAY_DOWNLOAD_URL}" 2>&1; then
    echo "${GREEN}✓ Download complete${NC}"
  else
    echo "${RED}Error: Download failed${NC}" >&2
    exit 1
  fi
fi

# Verify ZIP file
if [ ! -f "${GATEWAY_ZIP}" ]; then
  echo "${RED}Error: Downloaded file not found${NC}" >&2
  exit 1
fi

ZIP_SIZE=$(stat -f%z "${GATEWAY_ZIP}" 2>/dev/null || stat -c%s "${GATEWAY_ZIP}" 2>/dev/null || echo "0")
if [ "${ZIP_SIZE}" -lt 1000 ]; then
  echo "${RED}Error: Downloaded file appears to be too small (${ZIP_SIZE} bytes)${NC}" >&2
  echo "  This might indicate a download error or redirect issue" >&2
  rm -f "${GATEWAY_ZIP}"
  exit 1
fi

echo ""

# Extract the gateway
echo "${BLUE}Extracting gateway...${NC}"
if unzip -q -o "${GATEWAY_ZIP}" -d "${GATEWAY_DIR}"; then
  echo "${GREEN}✓ Extraction complete${NC}"
else
  echo "${RED}Error: Extraction failed${NC}" >&2
  exit 1
fi

# Verify extraction
if [ ! -f "${GATEWAY_DIR}/bin/run.sh" ]; then
  echo "${RED}Error: Gateway extraction incomplete - run.sh not found${NC}" >&2
  echo "  Expected: ${GATEWAY_DIR}/bin/run.sh" >&2
  echo "  Found files:" >&2
  find "${GATEWAY_DIR}" -name "*.sh" -type f 2>/dev/null | head -5 >&2
  exit 1
fi

# Make run script executable
chmod +x "${GATEWAY_DIR}/bin/run.sh" 2>/dev/null || true

# Create convenience run script
echo "${BLUE}Creating convenience run script...${NC}"
cat > "${GATEWAY_RUN_SCRIPT}" << 'EOF'
#!/usr/bin/env bash
# Convenience script to run IB Client Portal Gateway
# Handles failures gracefully to avoid killing tmux sessions
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ ! -f "${SCRIPT_DIR}/bin/run.sh" ]; then
  echo "Error: Gateway not found at ${SCRIPT_DIR}" >&2
  echo "Run install-ib-gateway.sh first" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

cd "${SCRIPT_DIR}"

# Find config file (prefer conf.yaml, fallback to conf.tws.yaml)
CONFIG_FILE="${SCRIPT_DIR}/root/conf.yaml"
if [ ! -f "${CONFIG_FILE}" ]; then
  CONFIG_FILE="${SCRIPT_DIR}/root/conf.tws.yaml"
fi

if [ ! -f "${CONFIG_FILE}" ]; then
  echo "Error: No config file found. Expected: root/conf.yaml or root/conf.tws.yaml" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

# Run gateway with config file and handle failures gracefully
if ! ./bin/run.sh "${CONFIG_FILE}" "$@"; then
  EXIT_CODE=$?
  echo "" >&2
  echo "⚠ IB Gateway exited with error code ${EXIT_CODE}" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit "${EXIT_CODE}"
fi
EOF

chmod +x "${GATEWAY_RUN_SCRIPT}"

# Clean up ZIP file
rm -f "${GATEWAY_ZIP}"

echo ""
echo "${GREEN}✓ IB Client Portal Gateway installed successfully!${NC}"
echo ""
echo "Installation location:"
echo "  ${GATEWAY_DIR}"
echo ""
echo "To start the gateway:"
echo "  ${BLUE}${GATEWAY_RUN_SCRIPT}${NC}"
echo ""
echo "Or manually:"
echo "  ${BLUE}cd ${GATEWAY_DIR} && ./bin/run.sh${NC}"
echo ""
echo "The gateway will:"
echo "  1. Start on https://localhost:5000"
echo "  2. Open a browser for authentication"
echo "  3. Provide API access for the IB service"
echo ""
echo "After starting the gateway, you can run:"
echo "  ${BLUE}./web/scripts/run-ib-service.sh${NC}"
