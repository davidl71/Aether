#!/usr/bin/env bash
# Install Polkit rules for secure PWA service control
# This allows the web app to control systemd services without sudo
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
POLKIT_RULES_DIR="$ROOT_DIR/web/scripts/systemd/polkit-rules"
POLKIT_TARGET_DIR="/etc/polkit-1/rules.d"

# Colors for output
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

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "${RED}✗ This script must be run as root (use sudo)${NC}" >&2
  echo "${YELLOW}  Run: sudo $0${NC}" >&2
  exit 1
fi

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]] && [[ "$OSTYPE" != "linux"* ]]; then
  echo "${YELLOW}⚠ This script is for Linux only.${NC}"
  exit 1
fi

# Check if Polkit is installed
if ! command -v pkaction >/dev/null 2>&1; then
  echo "${RED}✗ Polkit is not installed${NC}" >&2
  echo "${YELLOW}  Install with: sudo apt-get install policykit-1${NC}" >&2
  exit 1
fi

echo "${BLUE}Installing Polkit rules for PWA service control...${NC}"
echo ""

# Create target directory if it doesn't exist
mkdir -p "${POLKIT_TARGET_DIR}"

# Install rules file
RULES_FILE="${POLKIT_RULES_DIR}/10-pwa-services.rules"
TARGET_FILE="${POLKIT_TARGET_DIR}/10-pwa-services.rules"

if [ ! -f "${RULES_FILE}" ]; then
  echo "${RED}✗ Rules file not found: ${RULES_FILE}${NC}" >&2
  exit 1
fi

# Copy rules file
cp "${RULES_FILE}" "${TARGET_FILE}"
chmod 644 "${TARGET_FILE}"

echo "${GREEN}✓ Installed Polkit rules to ${TARGET_FILE}${NC}"
echo ""

# Reload Polkit
echo "${BLUE}Reloading Polkit...${NC}"
systemctl reload polkit 2>/dev/null || {
  echo "${YELLOW}⚠ Could not reload Polkit daemon (may need manual restart)${NC}"
  echo "${YELLOW}  Run: sudo systemctl restart polkit${NC}"
}

echo ""
echo "${GREEN}✓ Polkit rules installed${NC}"
echo ""
echo "The web app can now control PWA services via systemctl --user commands."
echo "No sudo required for service control operations."
