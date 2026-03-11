#!/usr/bin/env bash
# Secure helper script for systemctl service control
# Used by Rust backend to control PWA services via systemctl --user
# This script validates service names and executes systemctl commands safely
set -euo pipefail

# Allowed PWA service names (whitelist for security)
ALLOWED_SERVICES=(
  "pwa-web"
  "pwa-alpaca"
  "pwa-ib-gateway"
  "pwa-ib"
  "pwa-discount-bank"
  "pwa-nats"
  "pwa-rust-backend"
)

# Map service names from API to systemd service names
declare -A SERVICE_MAP
SERVICE_MAP["web"]="pwa-web"
SERVICE_MAP["alpaca"]="pwa-alpaca"
SERVICE_MAP["ib-gateway"]="pwa-ib-gateway"
SERVICE_MAP["ib_gateway"]="pwa-ib-gateway"
SERVICE_MAP["gateway"]="pwa-ib-gateway"
SERVICE_MAP["ib"]="pwa-ib"
SERVICE_MAP["discount-bank"]="pwa-discount-bank"
SERVICE_MAP["discount_bank"]="pwa-discount-bank"
SERVICE_MAP["nats"]="pwa-nats"
SERVICE_MAP["rust-backend"]="pwa-rust-backend"
SERVICE_MAP["rust_backend"]="pwa-rust-backend"

ACTION="${1:-}"
SERVICE_NAME="${2:-}"

if [ -z "${ACTION}" ] || [ -z "${SERVICE_NAME}" ]; then
  echo "Usage: $0 <action> <service_name>" >&2
  echo "Actions: start, stop, restart, status, enable, disable" >&2
  exit 1
fi

# Map service name to systemd service name
SYSTEMD_SERVICE="${SERVICE_MAP[$SERVICE_NAME]:-}"
if [ -z "${SYSTEMD_SERVICE}" ]; then
  # If not in map, check if it's already a valid systemd service name
  if [[ " ${ALLOWED_SERVICES[*]} " =~ " ${SERVICE_NAME} " ]]; then
    SYSTEMD_SERVICE="${SERVICE_NAME}"
  else
    echo "Error: Invalid service name: ${SERVICE_NAME}" >&2
    exit 1
  fi
fi

# Validate action
case "${ACTION}" in
  start|stop|restart|status|enable|disable|is-active|is-enabled)
    ;;
  *)
    echo "Error: Invalid action: ${ACTION}" >&2
    exit 1
    ;;
esac

# Execute systemctl command
case "${ACTION}" in
  status)
    systemctl --user status "${SYSTEMD_SERVICE}.service" --no-pager -l || {
      # Return exit code 3 if service doesn't exist (not found)
      if [ $? -eq 4 ]; then
        exit 3
      fi
      exit $?
    }
    ;;
  is-active)
    systemctl --user is-active "${SYSTEMD_SERVICE}.service" 2>/dev/null || exit 1
    ;;
  is-enabled)
    systemctl --user is-enabled "${SYSTEMD_SERVICE}.service" 2>/dev/null || exit 1
    ;;
  *)
    systemctl --user "${ACTION}" "${SYSTEMD_SERVICE}.service" || exit $?
    ;;
esac
