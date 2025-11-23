#!/usr/bin/env bash
# Install PWA services as systemd user services (Linux only)
# Usage: ./install-systemd-services.sh [--enable] [--start]
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SCRIPTS_DIR="$ROOT_DIR/web/scripts"
SYSTEMD_DIR="$SCRIPTS_DIR/systemd"
USER_SYSTEMD_DIR="${HOME}/.config/systemd/user"

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

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]] && [[ "$OSTYPE" != "linux"* ]]; then
  echo "${YELLOW}⚠ This script is for Linux only.${NC}"
  echo "${YELLOW}  On macOS, use brew services instead.${NC}"
  exit 1
fi

# Check if systemctl is available
if ! command -v systemctl >/dev/null 2>&1; then
  echo "${RED}✗ systemctl not found. This script requires systemd.${NC}" >&2
  exit 1
fi

# Check if systemd user services are supported
if ! systemctl --user --version >/dev/null 2>&1; then
  echo "${RED}✗ systemd user services not available.${NC}" >&2
  echo "${YELLOW}  Ensure systemd user session is enabled.${NC}" >&2
  exit 1
fi

# Parse arguments
ENABLE_SERVICES=false
START_SERVICES=false
while [[ $# -gt 0 ]]; do
  case $1 in
    --enable)
      ENABLE_SERVICES=true
      shift
      ;;
    --start)
      START_SERVICES=true
      shift
      ;;
    *)
      echo "${YELLOW}Unknown option: $1${NC}" >&2
      echo "Usage: $0 [--enable] [--start]"
      exit 1
      ;;
  esac
done

# Create user systemd directory
echo "${BLUE}Creating systemd user directory...${NC}"
mkdir -p "${USER_SYSTEMD_DIR}"

# List of services to install
SERVICES=(
  "pwa-web"
  "pwa-alpaca"
  "pwa-tradestation"
  "pwa-ib-gateway"
  "pwa-ib"
  "pwa-discount-bank"
  "pwa-risk-free-rate"
  "pwa-jupyterlab"
  "pwa-nats"
  "pwa-rust-backend"
)

# Install each service file
echo "${BLUE}Installing systemd service files...${NC}"
INSTALLED_COUNT=0
for service in "${SERVICES[@]}"; do
  SERVICE_FILE="${SYSTEMD_DIR}/${service}.service"
  DEST_FILE="${USER_SYSTEMD_DIR}/${service}.service"

  if [ ! -f "${SERVICE_FILE}" ]; then
    echo "${YELLOW}  ⚠ Service file not found: ${SERVICE_FILE}${NC}"
    continue
  fi

  # Replace %h and %i placeholders with actual values
  USER_HOME="${HOME}"
  USER_NAME=$(whoami)

  # Create service file with replaced values
  sed -e "s|%h|${USER_HOME}|g" \
      -e "s|%i|${USER_NAME}|g" \
      "${SERVICE_FILE}" > "${DEST_FILE}"

  echo "${GREEN}  ✓ Installed ${service}.service${NC}"
  INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
done

if [ $INSTALLED_COUNT -eq 0 ]; then
  echo "${RED}✗ No services installed${NC}" >&2
  exit 1
fi

echo "${GREEN}✓ Installed ${INSTALLED_COUNT} service file(s)${NC}"
echo ""

# Reload systemd daemon
echo "${BLUE}Reloading systemd daemon...${NC}"
systemctl --user daemon-reload
echo "${GREEN}✓ Daemon reloaded${NC}"
echo ""

# Enable services if requested
if [ "$ENABLE_SERVICES" = true ]; then
  echo "${BLUE}Enabling services...${NC}"
  for service in "${SERVICES[@]}"; do
    if [ -f "${USER_SYSTEMD_DIR}/${service}.service" ]; then
      if systemctl --user enable "${service}.service" >/dev/null 2>&1; then
        echo "${GREEN}  ✓ Enabled ${service}.service${NC}"
      else
        echo "${YELLOW}  ⚠ Failed to enable ${service}.service${NC}"
      fi
    fi
  done
  echo ""
fi

# Start services if requested
if [ "$START_SERVICES" = true ]; then
  echo "${BLUE}Starting services...${NC}"
  for service in "${SERVICES[@]}"; do
    if [ -f "${USER_SYSTEMD_DIR}/${service}.service" ]; then
      if systemctl --user start "${service}.service" >/dev/null 2>&1; then
        echo "${GREEN}  ✓ Started ${service}.service${NC}"
      else
        echo "${YELLOW}  ⚠ Failed to start ${service}.service${NC}"
        echo "${YELLOW}    Check status: systemctl --user status ${service}.service${NC}"
      fi
    fi
  done
  echo ""
fi

echo "${GREEN}✓ Installation complete${NC}"
echo ""
echo "Commands:"
echo "  ${BLUE}systemctl --user status <service>${NC}  # Check service status"
echo "  ${BLUE}systemctl --user start <service>${NC}  # Start a service"
echo "  ${BLUE}systemctl --user stop <service>${NC}   # Stop a service"
echo "  ${BLUE}systemctl --user restart <service>${NC}  # Restart a service"
echo "  ${BLUE}journalctl --user -u <service>${NC}    # View service logs"
echo ""
echo "To enable all services to start on login:"
echo "  ${BLUE}$0 --enable${NC}"
echo ""
echo "To start all services now:"
echo "  ${BLUE}$0 --start${NC}"
echo ""
echo "To enable and start all services:"
echo "  ${BLUE}$0 --enable --start${NC}"
