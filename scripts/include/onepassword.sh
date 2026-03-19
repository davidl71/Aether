#!/usr/bin/env bash
# onepassword.sh - Shared 1Password credential management functions
# Provides functions for reading credentials from 1Password CLI
#
# AI CONTEXT FOR AGENTS:
# =====================
# This file contains shared 1Password credential management functions:
#   - Used by service scripts that require API credentials
#   - Supports both 1Password CLI and environment variable fallback
#   - Auto-detects field names from 1Password item UUIDs
#
# PURPOSE: Secure credential management with 1Password integration
# PATTERN: Try 1Password first, fall back to environment variables
# DEPENDENCIES: Requires 1Password CLI (op) for 1Password integration
#               Requires Python 3 for JSON parsing
# TESTING: See spec/scripts/include/onepassword_spec.sh
#
# USAGE PATTERN IN SERVICE SCRIPTS:
#   source "${SCRIPTS_DIR}/include/onepassword.sh"
#   OP_API_KEY_SECRET="${OP_ALPACA_API_KEY_ID_SECRET:-}"
#   OP_API_SECRET_SECRET="${OP_ALPACA_API_SECRET_KEY_SECRET:-}"
#   if [ -n "${OP_ALPACA_ITEM_UUID:-}" ]; then
#     op_detect_fields "${OP_ALPACA_ITEM_UUID}" "KEY_FIELD" "SECRET_FIELD"
#     op_build_secret_paths "${OP_ALPACA_ITEM_UUID}" "${KEY_FIELD}" "${SECRET_FIELD}" "OP_API_KEY_SECRET" "OP_API_SECRET_SECRET"
#   fi
#   API_KEY=$(read_credential "${OP_API_KEY_SECRET}" "${API_KEY:-}" || echo "")
#
# 1PASSWORD INTEGRATION:
#   - Supports op:// secret references (op://Vault/Item/Field)
#   - Supports item UUID with auto-detected field names
#   - Works with both personal accounts (op signin) and service accounts (OP_SERVICE_ACCOUNT_TOKEN)
#
# FIELD AUTO-DETECTION:
#   Key fields: API Key ID, api_key_id, API Key, api_key, username, key_id, Key ID, Client ID, client_id
#   Secret fields: API Secret Key, api_secret_key, API Secret, api_secret, credential, secret_key, Secret Key, password, Client Secret, client_secret
#
# ERROR HANDLING: Functions return 0 on success, 1 on failure
#                 Gracefully falls back to environment variables when 1Password unavailable

# Read credential from 1Password or environment variable
# Usage: read_credential <op_secret_path> <env_var_name>
# Returns credential value via stdout, returns 1 if not found
read_credential() {
  local op_secret="${1:-}"
  local env_var="${2:-}"
  local result=""

  # Try 1Password first if secret path is provided
  if [ -n "${op_secret:-}" ] && command -v op >/dev/null 2>&1; then
    result=$(op read "${op_secret}" 2>/dev/null | tr -d '[:space:]' || echo "")
    if [ -n "${result}" ]; then
      echo -n "${result}"
      return 0
    fi
  fi

  # Fall back to environment variable
  if [ -n "${env_var:-}" ]; then
    echo -n "${env_var}"
    return 0
  fi

  return 1
}

# Auto-detect 1Password item fields from UUID
# Usage: op_detect_fields <item_uuid> <key_field_var> <secret_field_var>
# Sets variables with detected field names
op_detect_fields() {
  local item_uuid="${1:-}"
  local key_field_var="${2:-KEY_FIELD}"
  local secret_field_var="${3:-SECRET_FIELD}"

  if [ -z "${item_uuid}" ] || ! command -v op >/dev/null 2>&1; then
    return 1
  fi

  local item_json
  item_json=$(op item get "${item_uuid}" --format json 2>/dev/null || echo "")

  if [ -z "${item_json}" ]; then
    return 1
  fi

  # Detect key field
  local key_field
  key_field=$(echo "${item_json}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
fields = data.get('fields', [])
for name in ['API Key ID', 'api_key_id', 'API Key', 'api_key', 'username', 'key_id', 'Key ID', 'Client ID', 'client_id']:
    for field in fields:
        if field.get('label', '').lower() == name.lower():
            print(name)
            sys.exit(0)
" 2>/dev/null || echo "")

  # Detect secret field
  local secret_field
  secret_field=$(echo "${item_json}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
fields = data.get('fields', [])
for name in ['API Secret Key', 'api_secret_key', 'API Secret', 'api_secret', 'credential', 'secret_key', 'Secret Key', 'password', 'Client Secret', 'client_secret']:
    for field in fields:
        if field.get('label', '').lower() == name.lower() and field.get('type') == 'CONCEALED':
            print(name)
            sys.exit(0)
" 2>/dev/null || echo "")

  # Set variables
  eval "${key_field_var}=\"${key_field}\""
  eval "${secret_field_var}=\"${secret_field}\""

  return 0
}

# Build 1Password secret paths from item UUID
# Usage: op_build_secret_paths <item_uuid> <key_field> <secret_field> <key_path_var> <secret_path_var>
# Sets variables with full op:// paths
op_build_secret_paths() {
  local item_uuid="${1:-}"
  local key_field="${2:-}"
  local secret_field="${3:-}"
  local key_path_var="${4:-OP_KEY_SECRET}"
  local secret_path_var="${5:-OP_SECRET_SECRET}"

  if [ -z "${item_uuid}" ]; then
    return 1
  fi

  local item_json
  item_json=$(op item get "${item_uuid}" --format json 2>/dev/null || echo "")

  if [ -z "${item_json}" ]; then
    return 1
  fi

  local vault_id
  vault_id=$(echo "${item_json}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('id', ''))" 2>/dev/null || echo "")

  local vault_name
  vault_name=$(echo "${item_json}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('name', ''))" 2>/dev/null || echo "")

  local key_path=""
  local secret_path=""

  if [ -n "${vault_id}" ]; then
    key_path="op://${vault_id}/${item_uuid}/${key_field}"
    secret_path="op://${vault_id}/${item_uuid}/${secret_field}"
  elif [ -n "${vault_name}" ]; then
    key_path="op://${vault_name}/${item_uuid}/${key_field}"
    secret_path="op://${vault_name}/${item_uuid}/${secret_field}"
  else
    key_path="op://Private/${item_uuid}/${key_field}"
    secret_path="op://Private/${item_uuid}/${secret_field}"
  fi

  eval "${key_path_var}=\"${key_path}\""
  eval "${secret_path_var}=\"${secret_path}\""

  return 0
}

# Resolve OP_*_SECRET refs and export plain env vars for Rust backend/TUI.
# Call after sourcing this file; safe when op CLI or token is missing (no-op or keeps existing env).
# Usage: source .../onepassword.sh && export_op_secrets_for_rust
export_op_secrets_for_rust() {
  local val
  if val=$(read_credential "${OP_FRED_API_KEY_SECRET:-}" "${FRED_API_KEY:-}" 2>/dev/null) && [[ -n "${val}" ]]; then
    export FRED_API_KEY="${val}"
  fi
  if val=$(read_credential "${OP_FMP_API_KEY_SECRET:-}" "${FMP_API_KEY:-}" 2>/dev/null) && [[ -n "${val}" ]]; then
    export FMP_API_KEY="${val}"
  fi
}
