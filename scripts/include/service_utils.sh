#!/usr/bin/env bash
# service_utils.sh - Shared service utility functions
# Provides common functions for service health checks, port validation, etc.
#
# AI CONTEXT FOR AGENTS:
# =====================
# This file contains shared service utility functions used by service scripts:
#   - web/scripts/run-*.sh (Alpaca, IB, TradeStation, Discount Bank)
#   - scripts/start_alpaca_service.sh
#
# PURPOSE: Service health checking and port conflict detection
# PATTERN: Functions verify service identity before reporting conflicts
# DEPENDENCIES: Requires Python for health endpoint checking
#               Requires config.sh for port availability checking
# TESTING: See spec/scripts/include/service_utils_spec.sh
#
# USAGE PATTERN IN SERVICE SCRIPTS:
#   source "${SCRIPTS_DIR}/include/service_utils.sh"
#   if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${PORT}" "ALPACA_SERVICE" "Alpaca"; then
#     exit 1
#   fi
#
# HEALTH CHECK LOGIC:
#   - Checks if port is in use (via config_check_port_available)
#   - If port is in use, checks health endpoint to verify service identity
#   - If same service is running, exits gracefully (no need to start new instance)
#   - If different service is running, reports error with helpful message
#
# ERROR HANDLING: Functions return 0 on success, 1 on failure
#                 Provides helpful error messages with resolution steps

# Check service health endpoint
# Usage: check_service_health <python_cmd> <host> <port> <service_name> [expected_key]
# Returns 0 if service is healthy, 1 otherwise
# Sets SERVICE_HEALTH_CHECK variable to service name or "OTHER_SERVICE"
check_service_health() {
  local python_cmd="${1:-}"
  local host="${2:-127.0.0.1}"
  local port="${3:-}"
  local service_name="${4:-}"
  local expected_key="${5:-status}"

  if [ -z "${python_cmd}" ] || [ -z "${port}" ] || [ -z "${service_name}" ]; then
    echo "Error: python_cmd, port, and service_name required" >&2
    return 1
  fi

  SERVICE_HEALTH_CHECK=$("${python_cmd}" -c "
import urllib.request
import json
import sys
try:
    with urllib.request.urlopen('http://${host}:${port}/api/health', timeout=2) as response:
        data = json.loads(response.read().decode())
        val = data.get('${expected_key}')
        # Same service if status is ok or disabled (broker running without credentials)
        if val in ('ok', 'disabled') or (val is None and '${expected_key}' in data):
            print('${service_name}')
        else:
            print('OTHER_SERVICE')
except Exception:
    print('OTHER_SERVICE')
" 2>/dev/null || echo "OTHER_SERVICE")

  if [ "${SERVICE_HEALTH_CHECK}" = "${service_name}" ]; then
    return 0
  else
    return 1
  fi
}

# Check if port is in use and verify service identity
# Usage: check_port_with_service <python_cmd> <host> <port> <service_name> <service_display_name>
# Returns 0 if port available or service already running, 1 if conflict
# Sets SERVICE_ALREADY_RUNNING=1 when same service is already running (caller should exit 0)
check_port_with_service() {
  local python_cmd="${1:-}"
  local host="${2:-127.0.0.1}"
  local port="${3:-}"
  local service_name="${4:-}"
  local service_display_name="${5:-${service_name}}"

  SERVICE_ALREADY_RUNNING=

  if [ -z "${python_cmd}" ] || [ -z "${port}" ] || [ -z "${service_name}" ]; then
    echo "Error: python_cmd, port, and service_name required" >&2
    return 1
  fi

  # Load config functions if available
  if ! command -v config_check_port_available >/dev/null 2>&1; then
    echo "Warning: config_check_port_available not available" >&2
    return 1
  fi

  if ! config_check_port_available "${port}"; then
    echo "Port ${port} is already in use. Checking if it's the ${service_display_name} service..." >&2

    if check_service_health "${python_cmd}" "${host}" "${port}" "${service_name}"; then
      echo "✓ ${service_display_name} service is already running on http://${host}:${port}" >&2
      echo "  Using existing service. No need to start a new one." >&2
      echo "  Set VITE_API_URL=http://${host}:${port}/api/snapshot in your web app" >&2
      echo "" >&2
      SERVICE_ALREADY_RUNNING=1
      return 0
    else
      echo "Error: Port ${port} is in use by another service (not ${service_display_name} service)" >&2
      echo "  Please stop the service on port ${port} or use a different port:" >&2
      echo "  export $(echo "${service_name}" | tr '[:lower:]' '[:upper:]' | tr '_' ' ' | tr ' ' '_')_PORT=<different_port>" >&2
      echo "  Or update config/config.json: services.${service_name}.port" >&2
      return 1
    fi
  fi

  return 0
}
