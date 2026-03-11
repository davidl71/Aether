#!/usr/bin/env bash
# config.sh - Shared configuration loader for shell scripts
# Provides functions to read port assignments and other settings from config.json
# Supports environment variable overrides
#
# AI CONTEXT FOR AGENTS:
# =====================
# This file contains shared configuration functions used by ALL service scripts:
#   - web/scripts/run-*.sh (Alpaca, IB, Discount Bank)
#   - scripts/start_alpaca_service.sh
#
# PURPOSE: Centralized configuration management with env var overrides
# PATTERN: Functions read from config.json, fall back to defaults
# PRIORITY: ENV_VAR > config.json > default_port
# DEPENDENCIES: Requires jq for JSON parsing (optional, falls back gracefully)
# TESTING: See spec/scripts/include/config_spec.sh
#
# CONFIG FILE LOCATIONS (searched in order):
#   1. IB_BOX_SPREAD_CONFIG environment variable
#   2. ~/.config/ib_box_spread/config.json
#   3. ~/Library/Application Support/ib_box_spread/config.json (macOS)
#   4. ${PROJECT_ROOT}/config/config.json
#   5. ${PROJECT_ROOT}/config/config.example.json
#   6. /usr/local/etc/ib_box_spread/config.json
#   7. /etc/ib_box_spread/config.json
#
# USAGE PATTERN IN SERVICE SCRIPTS:
#   source "${SCRIPTS_DIR}/include/config.sh"
#   PORT=$(config_get_port "alpaca" 8000)
#   if ! config_check_port_available "${PORT}"; then
#     echo "Port ${PORT} is in use" >&2
#     exit 1
#   fi
#
# ENVIRONMENT VARIABLE OVERRIDES:
#   Service ports can be overridden via environment variables:
#   - ALPACA_PORT (overrides services.alpaca.port)
#   - IB_PORT (overrides services.ib.port)
#   - DISCOUNT_BANK_PORT (overrides services.discount_bank.port)
#
# ERROR HANDLING: Functions return 0 on success, 1 on failure
#                 Error messages written to stderr
#                 Graceful fallback when jq not available

# Find config file using same logic as Python config_adapter
_find_config_file() {
  local config_path="${1:-}"
  local candidates=()

  # If explicit path provided, use it
  if [ -n "${config_path}" ]; then
    candidates+=("${config_path}")
  fi

  # Check environment variable
  if [ -n "${IB_BOX_SPREAD_CONFIG:-}" ]; then
    candidates+=("${IB_BOX_SPREAD_CONFIG}")
  fi

  # Standard locations
  local home="${HOME:-}"
  if [ -n "${home}" ]; then
    candidates+=("${home}/.config/ib_box_spread/config.json")
    if [[ "$(uname)" == "Darwin" ]]; then
      candidates+=("${home}/Library/Application Support/ib_box_spread/config.json")
    fi
  fi

  # Project root config
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  candidates+=("${script_dir}/config/config.json")
  candidates+=("${script_dir}/config/config.example.json")

  # System locations
  candidates+=("/usr/local/etc/ib_box_spread/config.json")
  candidates+=("/etc/ib_box_spread/config.json")

  # Return first existing file
  for candidate in "${candidates[@]}"; do
    if [ -f "${candidate}" ]; then
      echo "${candidate}"
      return 0
    fi
  done

  return 1
}

# Get service port from config with environment variable override
# Usage: config_get_port <service_name> [default_port]
# Example: PORT=$(config_get_port "ib" 8002)
config_get_port() {
  local service_name="${1:-}"
  local default_port="${2:-}"

  if [ -z "${service_name}" ]; then
    echo "Error: service_name required" >&2
    return 1
  fi

  # Check environment variable first (highest priority)
  local env_var_name
  env_var_name=$(echo "${service_name}" | tr '[:lower:]' '[:upper:]' | tr '_' ' ')
  env_var_name="${env_var_name}_PORT"
  env_var_name=$(echo "${env_var_name}" | tr ' ' '_')

  if [ -n "${!env_var_name:-}" ]; then
    echo "${!env_var_name}"
    return 0
  fi

  # Try config file
  local config_file
  config_file=$(_find_config_file)

  if [ -n "${config_file}" ] && command -v jq >/dev/null 2>&1; then
    local port
    port=$(jq -r ".services.${service_name}.port // empty" "${config_file}" 2>/dev/null)
    if [ -n "${port}" ] && [ "${port}" != "null" ]; then
      echo "${port}"
      return 0
    fi
  fi

  # Fall back to default
  if [ -n "${default_port}" ]; then
    echo "${default_port}"
    return 0
  fi

  # No value found
  return 1
}

# Check whether a service is enabled in config with environment override support.
# Usage: config_is_enabled <service_name> [default_value]
# Returns 0 if enabled, 1 if disabled.
config_is_enabled() {
  local service_name="${1:-}"
  local default_value="${2:-true}"

  if [ -z "${service_name}" ]; then
    echo "Error: service_name required" >&2
    return 1
  fi

  local env_var_name
  env_var_name=$(echo "${service_name}" | tr '[:lower:]' '[:upper:]' | tr '_' ' ')
  env_var_name="${env_var_name}_ENABLED"
  env_var_name=$(echo "${env_var_name}" | tr ' ' '_')

  if [ -n "${!env_var_name:-}" ]; then
    case "${!env_var_name}" in
    1 | true | TRUE | yes | YES | on | ON) return 0 ;;
    0 | false | FALSE | no | NO | off | OFF) return 1 ;;
    esac
  fi

  local config_file
  config_file=$(_find_config_file)
  if [ -n "${config_file}" ] && command -v jq >/dev/null 2>&1; then
    local enabled
    enabled=$(jq -r ".services.${service_name}.enabled // empty" "${config_file}" 2>/dev/null)
    case "${enabled}" in
    true) return 0 ;;
    false) return 1 ;;
    esac
  fi

  case "${default_value}" in
  1 | true | TRUE | yes | YES | on | ON) return 0 ;;
  *) return 1 ;;
  esac
}

# Check if a port is available (not in use)
# Usage: config_check_port_available <port>
# Returns 0 if available, 1 if in use
config_check_port_available() {
  local port="${1:-}"

  if [ -z "${port}" ]; then
    echo "Error: port required" >&2
    return 1
  fi

  if command -v lsof >/dev/null 2>&1; then
    lsof -ti ":${port}" >/dev/null 2>&1 && return 1 || return 0
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN" && return 1 || return 0
  else
    # Fallback: try to connect
    if command -v python3 >/dev/null 2>&1; then
      python3 -c "import socket; s = socket.socket(); s.settimeout(0.1); result = s.connect_ex(('127.0.0.1', ${port})); s.close(); exit(0 if result != 0 else 1)" 2>/dev/null
      return $?
    fi
  fi

  # If we can't check, assume available
  return 0
}

# Get config value (generic)
# Usage: config_get <path> [default]
# Example: VALUE=$(config_get ".tws.port" 7497)
config_get() {
  local json_path="${1:-}"
  local default_value="${2:-}"

  if [ -z "${json_path}" ]; then
    echo "Error: json_path required" >&2
    return 1
  fi

  local config_file
  config_file=$(_find_config_file)

  if [ -n "${config_file}" ] && command -v jq >/dev/null 2>&1; then
    local value
    value=$(jq -r "${json_path} // empty" "${config_file}" 2>/dev/null)
    if [ -n "${value}" ] && [ "${value}" != "null" ]; then
      echo "${value}"
      return 0
    fi
  fi

  # Fall back to default
  if [ -n "${default_value}" ]; then
    echo "${default_value}"
    return 0
  fi

  return 1
}

# Find TOML config file for Rust backend
_find_toml_config_file() {
  local candidates=()
  local root

  # Get project root (scripts/include/config.sh -> scripts -> project root)
  if [ -n "${PROJECT_ROOT:-}" ]; then
    root="${PROJECT_ROOT}"
  elif [ -d "$(dirname "${BASH_SOURCE[0]}")/../../../agents/backend" ]; then
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  else
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
  fi

  # Check BACKEND_CONFIG env var
  if [ -n "${BACKEND_CONFIG:-}" ]; then
    candidates+=("${BACKEND_CONFIG}")
  fi

  # Standard locations for backend TOML config
  candidates+=("${root}/agents/backend/config/default.toml")
  candidates+=("${root}/config/backend.toml")

  for path in "${candidates[@]}"; do
    if [ -f "${path}" ]; then
      echo "${path}"
      return 0
    fi
  done

  return 1
}

# Get rust_backend port from TOML config
# Usage: config_get_rust_backend_port [default_port]
config_get_rust_backend_port() {
  local default_port="${1:-9090}"
  local toml_path

  toml_path=$(_find_toml_config_file)

  if [ -n "${toml_path}" ] && command -v sed >/dev/null 2>&1; then
    local port
    port=$(sed -n 's/^rest_addr.*:\s*"\([^:]*\):\([^"]*\)"/\2/p' "${toml_path}" 2>/dev/null | head -1)
    if [ -n "${port}" ]; then
      echo "${port}"
      return 0
    fi
  fi

  # Fall back to default
  echo "${default_port}"
  return 0
}
