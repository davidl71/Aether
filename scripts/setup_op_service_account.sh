#!/usr/bin/env bash
# setup_op_service_account.sh - Set up and connect 1Password CLI with a service account
#
# Use this script to export OP_SERVICE_ACCOUNT_TOKEN and optionally use the op CLI to
# create the service account and placeholder secret items for this project.
#
# Usage:
#   source ./scripts/setup_op_service_account.sh     # Export token into current shell
#   ./scripts/setup_op_service_account.sh export    # Load token and verify
#   ./scripts/setup_op_service_account.sh verify    # Test that op works with the token
#   ./scripts/setup_op_service_account.sh setup-token    # Create service account via op CLI, save token
#   ./scripts/setup_op_service_account.sh setup-secrets   # Create placeholder secret items via op CLI
#   ./scripts/setup_op_service_account.sh setup-full      # Interactive: setup-token + setup-secrets + export
#   ./scripts/setup_op_service_account.sh generate-and-configure  # Create/update items, optional generate, print OP_* exports
#   ./scripts/setup_op_service_account.sh help      # Show help and doc link
#
# Token sources (first wins):
#   1. OP_SERVICE_ACCOUNT_TOKEN already set in environment
#   2. File: ${OP_SERVICE_ACCOUNT_TOKEN_FILE:-$HOME/.config/op/service_account_token}
#   3. When run interactively: prompt to paste token (not when sourced)
#
# Interactive defaults (when set): OP_SETUP_VAULT, OP_SERVICE_ACCOUNT_NAME.
# If OP_SETUP_VAULT is unset, last-used vault is read from VAULT_DEFAULT_FILE (saved after successful setup).
# Optional: OP_CREATE_VAULT_IF_MISSING=1 makes "Create vault?" default to Yes.
#
# See: docs/ONEPASSWORD_INTEGRATION.md (Create and connect a service account using the op CLI)
# CLI reference: https://developer.1password.com/docs/cli/reference/management-commands

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DOC_URL="docs/ONEPASSWORD_INTEGRATION.md"
TOKEN_FILE="${OP_SERVICE_ACCOUNT_TOKEN_FILE:-$HOME/.config/op/service_account_token}"
VAULT_DEFAULT_FILE="${OP_VAULT_DEFAULT_FILE:-$(dirname "${TOKEN_FILE}")/ib_box_spread_vault}"

# Detect interactive session (stdin and stdout are TTYs; avoid prompting when piped or in CI)
is_interactive() {
  [[ -t 0 ]] && [[ -t 1 ]]
}

usage() {
  echo "Usage: source ${SCRIPT_DIR}/setup_op_service_account.sh   # export into shell"
  echo "       ${SCRIPT_DIR}/setup_op_service_account.sh [export|verify|setup-token|setup-secrets|setup-full|generate-and-configure|help]"
  echo ""
  echo "  export        - Load token and export; run 'op vault list' to verify."
  echo "  verify        - Check that op CLI works with the current token."
  echo "  setup-token   - (Optional) Run 'op' CLI to create a service account and save token to file."
  echo "  setup-secrets - (Optional) Run 'op' CLI to create placeholder secret items (Alpaca, Tastytrade, etc.)."
  echo "  setup-full    - Interactive only: setup-token, then setup-secrets, then export (for first-time setup)."
  echo "  generate-and-configure - Create/update backend secret items, optionally generate passwords, print OP_* exports."
  echo "                  Use --all for non-interactive (requires OP_SETUP_VAULT). Use --output-env FILE to write exports."
  echo "  help          - Show this message and doc link."
  echo ""
  echo "Token is read from (in order):"
  echo "  1. Existing OP_SERVICE_ACCOUNT_TOKEN"
  echo "  2. File: ${TOKEN_FILE}"
  echo "  3. Interactive paste (export only, when not sourced)"
  echo ""
  echo "Docs: ${PROJECT_ROOT}/${DOC_URL}"
  echo "Backend secrets providers: ${PROJECT_ROOT}/docs/BACKEND_SECRETS_PROVIDERS.md"
}

# Load token into OP_SERVICE_ACCOUNT_TOKEN (caller can export it)
load_token() {
  if [[ -n "${OP_SERVICE_ACCOUNT_TOKEN:-}" ]]; then
    return 0
  fi
  if [[ -f "${TOKEN_FILE}" ]]; then
    OP_SERVICE_ACCOUNT_TOKEN="$(cat "${TOKEN_FILE}" | tr -d '[:space:]')"
    return 0
  fi
  return 1
}

# Verify op CLI works with current token
verify_op() {
  if ! command -v op &>/dev/null; then
    echo "op CLI not found. Install: https://developer.1password.com/docs/cli/" >&2
    return 1
  fi
  if [[ -z "${OP_SERVICE_ACCOUNT_TOKEN:-}" ]]; then
    echo "OP_SERVICE_ACCOUNT_TOKEN is not set. Run: source ${SCRIPT_DIR}/setup_op_service_account.sh" >&2
    return 1
  fi
  # Unset Connect vars so service account token is used
  unset OP_CONNECT_HOST OP_CONNECT_TOKEN 2>/dev/null || true
  if op vault list &>/dev/null; then
    echo "OK: op CLI is connected (service account)."
    return 0
  fi
  echo "op vault list failed. Check token and vault access." >&2
  return 1
}

# Export token and optionally verify
do_export() {
  if ! load_token; then
    if is_interactive && [[ "${BASH_SOURCE[0]:-}" != "${0:-}" ]]; then
      echo "Paste service account token (ops_...); Ctrl+D when done:"
      OP_SERVICE_ACCOUNT_TOKEN="$(cat | tr -d '[:space:]')"
    else
      echo "Set OP_SERVICE_ACCOUNT_TOKEN or create ${TOKEN_FILE} with the token." >&2
      echo "See ${DOC_URL} for creating a service account." >&2
      return 1
    fi
  fi
  export OP_SERVICE_ACCOUNT_TOKEN
  # Set OP_FRED_API_KEY_SECRET to op reference when vault is known (so TUI and risk-free-rate service get FRED from 1Password)
  if [[ -z "${OP_FRED_API_KEY_SECRET:-}" ]]; then
    FRED_VAULT="${OP_SETUP_VAULT:-$(read_vault_default)}"
    if [[ -n "${FRED_VAULT}" ]]; then
      export OP_FRED_API_KEY_SECRET="op://${FRED_VAULT}/FRED API/credential"
      if [[ "${BASH_SOURCE[0]:-}" != "${0:-}" ]]; then
        echo "Exported OP_FRED_API_KEY_SECRET (op reference)."
      fi
    fi
  fi
  if [[ "${BASH_SOURCE[0]:-}" != "${0:-}" ]]; then
    echo "Exported OP_SERVICE_ACCOUNT_TOKEN into current shell."
  else
    echo "export OP_SERVICE_ACCOUNT_TOKEN='${OP_SERVICE_ACCOUNT_TOKEN:0:20}...'"
    verify_op
  fi
}

# Require op CLI
require_op() {
  if ! command -v op &>/dev/null; then
    echo "op CLI not found. Install: https://developer.1password.com/docs/cli/" >&2
    return 1
  fi
  return 0
}

# Read last-used vault name (for interactive default when OP_SETUP_VAULT is unset)
read_vault_default() {
  if [[ -f "${VAULT_DEFAULT_FILE}" ]]; then
    local v
    v="$(cat "${VAULT_DEFAULT_FILE}" 2>/dev/null | head -1 | tr -d '[:space:]')"
    [[ -n "${v}" ]] && echo "${v}"
  fi
}

# Save vault name as default for next run
save_vault_default() {
  local name="$1"
  [[ -z "${name}" ]] && return
  mkdir -p "$(dirname "${VAULT_DEFAULT_FILE}")"
  printf '%s' "${name}" >"${VAULT_DEFAULT_FILE}"
}

# Ensure vault exists; if missing and interactive, prompt to create it. Returns 0 if vault exists (or was created), 1 otherwise.
ensure_vault_exists() {
  local name="$1"
  if [[ -z "${name}" ]]; then
    return 1
  fi
  if op vault get "${name}" &>/dev/null; then
    return 0
  fi
  if ! is_interactive; then
    echo "Vault \"${name}\" not found. Create it in 1Password or run interactively to create it." >&2
    return 1
  fi
  if [[ -n "${OP_CREATE_VAULT_IF_MISSING:-}" ]]; then
    echo -n "Vault \"${name}\" not found. Create it now? (Y/n): "
  else
    echo -n "Vault \"${name}\" not found. Create it now? (y/n): "
  fi
  read -r ans
  create_it=0
  if [[ "${ans}" =~ ^[yY] ]]; then
    create_it=1
  elif [[ -z "${ans}" && -n "${OP_CREATE_VAULT_IF_MISSING:-}" ]]; then
    create_it=1
  fi
  if [[ "${create_it}" -eq 1 ]]; then
    if op vault create "${name}"; then
      echo "Created vault \"${name}\"."
      return 0
    fi
    echo "Failed to create vault." >&2
    return 1
  fi
  echo "Skipping. Create the vault in 1Password and re-run." >&2
  return 1
}

# Create service account via op CLI and save token to file
do_setup_token() {
  require_op || return 1
  if [[ -f "${TOKEN_FILE}" ]] && [[ -z "${OP_FORCE_SETUP_TOKEN:-}" ]]; then
    echo "Token file already exists: ${TOKEN_FILE}" >&2
    echo "Set OP_FORCE_SETUP_TOKEN=1 to create a new service account anyway." >&2
    return 0
  fi
  echo "You need to be signed in with a personal 1Password account (not the service account)."
  if ! op account get &>/dev/null; then
    echo "Running: op signin"
    op signin || {
      echo "Sign-in failed." >&2
      return 1
    }
  fi
  SA_NAME="${OP_SERVICE_ACCOUNT_NAME:-IB Box Spread Automation}"
  VAULT_NAME="${OP_SETUP_VAULT:-}"
  if is_interactive; then
    VAULT_DEFAULT="${OP_SETUP_VAULT:-$(read_vault_default)}"
    echo -n "Service account name (default: ${SA_NAME}): "
    read -r SA_INPUT
    [[ -n "${SA_INPUT}" ]] && SA_NAME="${SA_INPUT}"
    if [[ -n "${VAULT_DEFAULT}" ]]; then
      echo -n "Vault name to grant read access to the service account (default: ${VAULT_DEFAULT}): "
    else
      echo -n "Vault name to grant read access to the service account (required): "
    fi
    read -r VAULT_NAME
    VAULT_NAME="${VAULT_NAME:-$VAULT_DEFAULT}"
  fi
  if [[ -z "${VAULT_NAME}" ]]; then
    if ! is_interactive; then
      echo "Non-interactive: set OP_SETUP_VAULT. Service accounts need at least one vault." >&2
      return 1
    fi
    echo "Vault name is required." >&2
    return 1
  fi
  ensure_vault_exists "${VAULT_NAME}" || return 1
  save_vault_default "${VAULT_NAME}"
  echo "Creating service account \"${SA_NAME}\" with read access to vault \"${VAULT_NAME}\"..."
  OUT="$(op service-account create "${SA_NAME}" --vault "${VAULT_NAME}:read_items" 2>&1)" || true
  if [[ -z "${OUT}" ]]; then
    echo "op service-account create failed or produced no output." >&2
    return 1
  fi
  TOKEN=""
  while IFS= read -r line; do
    if [[ "$line" =~ ops_[a-zA-Z0-9_]{20,} ]]; then
      TOKEN="${BASH_REMATCH[0]}"
      break
    fi
  done <<<"$OUT"
  if [[ -z "${TOKEN}" ]]; then
    echo "Could not parse service account token from output. Save it manually from:" >&2
    echo "$OUT" >&2
    return 1
  fi
  mkdir -p "$(dirname "${TOKEN_FILE}")"
  printf '%s' "${TOKEN}" >"${TOKEN_FILE}"
  chmod 600 "${TOKEN_FILE}"
  echo "Token saved to ${TOKEN_FILE}"
  echo "Also save this token in 1Password (e.g. Secure Note) — it is shown only once."
  export OP_SERVICE_ACCOUNT_TOKEN="${TOKEN}"
  verify_op
}

# Create placeholder secret items via op CLI
do_setup_secrets() {
  require_op || return 1
  if ! op account get &>/dev/null; then
    echo "Running: op signin (sign in with personal or service account that has write access)"
    op signin || {
      echo "Sign-in failed." >&2
      return 1
    }
  fi
  VAULT_NAME="${OP_SETUP_VAULT:-}"
  if is_interactive; then
    VAULT_DEFAULT="${OP_SETUP_VAULT:-$(read_vault_default)}"
    if [[ -n "${VAULT_DEFAULT}" ]]; then
      echo -n "Vault name to create secret items in (default: ${VAULT_DEFAULT}): "
    else
      echo -n "Vault name to create secret items in: "
    fi
    read -r VAULT_NAME
    VAULT_NAME="${VAULT_NAME:-$VAULT_DEFAULT}"
  fi
  if [[ -z "${VAULT_NAME}" ]]; then
    if ! is_interactive; then
      echo "Non-interactive: set OP_SETUP_VAULT." >&2
      return 1
    fi
    echo "Vault name is required. Set OP_SETUP_VAULT or enter when prompted." >&2
    return 1
  fi
  ensure_vault_exists "${VAULT_NAME}" || return 1
  save_vault_default "${VAULT_NAME}"
  echo "Creating placeholder items in vault \"${VAULT_NAME}\" (fill real values in 1Password app)..."
  for title in "Alpaca API" "Tastytrade" "FRED API" "Alpha Vantage API" "Finnhub API" "Polygon API"; do
    if op item get "${title}" --vault "${VAULT_NAME}" &>/dev/null; then
      echo "Item already exists: ${title}"
    else
      if [[ "${title}" == "FRED API" ]] || [[ "${title}" == "Alpha Vantage API" ]] || [[ "${title}" == "Finnhub API" ]] || [[ "${title}" == "Polygon API" ]]; then
        op item create --category "API Credential" --vault "${VAULT_NAME}" --title "${title}" "credential=replace-me" 2>/dev/null && echo "Created: ${title}" || echo "Failed: ${title}" >&2
      else
        op item create --category "API Credential" --vault "${VAULT_NAME}" --title "${title}" "username=replace-me" "credential=replace-me" 2>/dev/null && echo "Created: ${title}" || echo "Failed: ${title}" >&2
      fi
    fi
  done
  echo ""
  echo "Suggested exports (update Vault/Item names if different):"
  echo "  export OP_ALPACA_API_KEY_ID_SECRET=\"op://${VAULT_NAME}/Alpaca API/username\""
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET=\"op://${VAULT_NAME}/Alpaca API/credential\""
  echo "  export OP_TASTYTRADE_USERNAME_SECRET=\"op://${VAULT_NAME}/Tastytrade/username\""
  echo "  export OP_TASTYTRADE_PASSWORD_SECRET=\"op://${VAULT_NAME}/Tastytrade/credential\""
  echo "  export OP_FRED_API_KEY_SECRET=\"op://${VAULT_NAME}/FRED API/credential\""
  echo "  export OP_POLYGON_API_KEY_SECRET=\"op://${VAULT_NAME}/Polygon API/credential\""
  echo "  export OP_ALPHA_VANTAGE_API_KEY_SECRET=\"op://${VAULT_NAME}/Alpha Vantage API/credential\""
  echo "  export OP_FINNHUB_API_KEY_SECRET=\"op://${VAULT_NAME}/Finnhub API/credential\""
}

# Full interactive setup: token + secrets + export (only when interactive)
do_setup_full() {
  if ! is_interactive; then
    echo "setup-full requires an interactive terminal. Set OP_SETUP_VAULT and run setup-token then setup-secrets." >&2
    return 1
  fi
  echo "=== 1Password full setup (token + placeholder secrets) ==="
  do_setup_token || return 1
  do_setup_secrets || return 1
  do_export || return 1
  echo "Done. Add the suggested OP_*_SECRET exports to your shell profile, then copy config/config.example.json to config/config.json and add op:// refs if needed."
}

# Generate a secure random password (for new items when --generate is used)
safe_gen_password() {
  if command -v openssl &>/dev/null; then
    openssl rand -base64 32 2>/dev/null | tr -d '\n'
  else
    head -c 32 /dev/urandom 2>/dev/null | base64 | tr -d '\n'
  fi
}

# Generate and configure: create/update backend secret items, optionally generate passwords, print OP_* exports
OUTPUT_ENV_FILE=""
GENERATE_ALL_FLAG=""
do_generate_and_configure() {
  require_op || return 1
  if ! op account get &>/dev/null; then
    echo "Run: op signin (or export OP_SERVICE_ACCOUNT_TOKEN)" >&2
    return 1
  fi
  load_token || true
  export OP_SERVICE_ACCOUNT_TOKEN 2>/dev/null || true
  VAULT_NAME="${OP_SETUP_VAULT:-}"
  if is_interactive && [[ -z "${GENERATE_ALL_FLAG}" ]]; then
    VAULT_DEFAULT="${OP_SETUP_VAULT:-$(read_vault_default)}"
    echo -n "Vault name for backend secret items (default: ${VAULT_DEFAULT:-none}): "
    read -r VAULT_NAME
    VAULT_NAME="${VAULT_NAME:-$VAULT_DEFAULT}"
  fi
  if [[ -z "${VAULT_NAME}" ]]; then
    echo "Set OP_SETUP_VAULT or run interactively and enter vault name. See docs/BACKEND_SECRETS_PROVIDERS.md" >&2
    return 1
  fi
  ensure_vault_exists "${VAULT_NAME}" || return 1
  save_vault_default "${VAULT_NAME}"
  echo "Creating or reusing backend secret items in vault \"${VAULT_NAME}\"..."
  for title in "Alpaca API" "Tastytrade" "FRED API" "Alpha Vantage API" "Finnhub API" "Polygon API"; do
    if op item get "${title}" --vault "${VAULT_NAME}" &>/dev/null; then
      echo "Item exists: ${title}"
    else
      if [[ "${title}" == "FRED API" ]] || [[ "${title}" == "Alpha Vantage API" ]] || [[ "${title}" == "Finnhub API" ]] || [[ "${title}" == "Polygon API" ]]; then
        cred="replace-me"
        [[ -n "${GENERATE_ALL_FLAG}" ]] && cred="$(safe_gen_password)"
        op item create --category "API Credential" --vault "${VAULT_NAME}" --title "${title}" "credential=${cred}" 2>/dev/null && echo "Created: ${title}" || echo "Failed: ${title}" >&2
      else
        user="replace-me"
        cred="replace-me"
        [[ -n "${GENERATE_ALL_FLAG}" ]] && cred="$(safe_gen_password)"
        op item create --category "API Credential" --vault "${VAULT_NAME}" --title "${title}" "username=${user}" "credential=${cred}" 2>/dev/null && echo "Created: ${title}" || echo "Failed: ${title}" >&2
      fi
    fi
  done
  echo ""
  echo "Add these to your shell profile or source before running services:"
  echo ""
  EXPORTS=(
    "export OP_ALPACA_API_KEY_ID_SECRET=\"op://${VAULT_NAME}/Alpaca API/username\""
    "export OP_ALPACA_API_SECRET_KEY_SECRET=\"op://${VAULT_NAME}/Alpaca API/credential\""
    "export OP_TASTYTRADE_USERNAME_SECRET=\"op://${VAULT_NAME}/Tastytrade/username\""
    "export OP_TASTYTRADE_PASSWORD_SECRET=\"op://${VAULT_NAME}/Tastytrade/credential\""
    "export OP_FRED_API_KEY_SECRET=\"op://${VAULT_NAME}/FRED API/credential\""
    "export OP_ALPHA_VANTAGE_API_KEY_SECRET=\"op://${VAULT_NAME}/Alpha Vantage API/credential\""
    "export OP_FINNHUB_API_KEY_SECRET=\"op://${VAULT_NAME}/Finnhub API/credential\""
    "export OP_POLYGON_API_KEY_SECRET=\"op://${VAULT_NAME}/Polygon API/credential\""
  )
  for line in "${EXPORTS[@]}"; do
    echo "  ${line}"
  done
  if [[ -n "${OUTPUT_ENV_FILE}" ]]; then
    mkdir -p "$(dirname "${OUTPUT_ENV_FILE}")"
    {
      echo "# Backend secrets (op:// refs) - generated by setup_op_service_account.sh generate-and-configure"
      echo "# Source with: source ${OUTPUT_ENV_FILE}"
      for line in "${EXPORTS[@]}"; do
        echo "${line}"
      done
    } >>"${OUTPUT_ENV_FILE}"
    echo ""
    echo "Exports appended to ${OUTPUT_ENV_FILE}"
  fi
  echo ""
  echo "Fill real broker API keys in the 1Password app (Alpaca, Tastytrade). See docs/BACKEND_SECRETS_PROVIDERS.md"
}

# When script is run (not sourced)
if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]]; then
  OUTPUT_ENV_FILE=""
  GENERATE_ALL_FLAG=""
  CMD="export"
  while [[ $# -gt 0 ]]; do
    case "$1" in
    --output-env=*)
      OUTPUT_ENV_FILE="${1#--output-env=}"
      shift
      ;;
    --output-env)
      if [[ -n "${2:-}" ]]; then
        OUTPUT_ENV_FILE="$2"
        shift 2
      else
        shift
      fi
      ;;
    --all)
      GENERATE_ALL_FLAG=1
      shift
      ;;
    export | verify | setup-token | setup-secrets | setup-full | generate-and-configure | help | --help | -h)
      CMD="$1"
      shift
      ;;
    *)
      CMD="$1"
      shift
      ;;
    esac
  done
  case "${CMD}" in
  export)
    do_export
    ;;
  verify)
    load_token || true
    export OP_SERVICE_ACCOUNT_TOKEN 2>/dev/null || true
    verify_op
    ;;
  setup-token)
    do_setup_token
    ;;
  setup-secrets)
    do_setup_secrets
    ;;
  setup-full)
    do_setup_full
    ;;
  generate-and-configure)
    do_generate_and_configure
    ;;
  help | --help | -h)
    usage
    ;;
  *)
    echo "Unknown option: ${CMD}" >&2
    usage
    exit 1
    ;;
  esac
  exit $?
fi

# When sourced: load and export token
if load_token; then
  export OP_SERVICE_ACCOUNT_TOKEN
fi
