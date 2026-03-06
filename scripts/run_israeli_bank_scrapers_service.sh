#!/usr/bin/env bash
# Run Israeli Bank Scrapers service (or CLI scrape) with credentials from 1Password or env.
#
# 1Password: set OP_SCRAPER_*_SECRET to op:// paths (e.g. op://Vault/Item/field).
# Env fallback: SCRAPER_DISCOUNT_ID, SCRAPER_DISCOUNT_PASSWORD, SCRAPER_DISCOUNT_NUM, etc.
#
# Usage:
#   ./scripts/run_israeli_bank_scrapers_service.sh           # start HTTP server
#   ./scripts/run_israeli_bank_scrapers_service.sh scrape     # run once and write to ledger
#
# 1Password env vars (optional):
#   OP_SCRAPER_DISCOUNT_ID_SECRET, OP_SCRAPER_DISCOUNT_PASSWORD_SECRET, OP_SCRAPER_DISCOUNT_NUM_SECRET
#   OP_SCRAPER_LEUMI_USERNAME_SECRET, OP_SCRAPER_LEUMI_PASSWORD_SECRET
#   OP_SCRAPER_HAPOALIM_USER_CODE_SECRET, OP_SCRAPER_HAPOALIM_PASSWORD_SECRET
#   ... or generic: OP_SCRAPER_ID_SECRET, OP_SCRAPER_PASSWORD_SECRET, OP_SCRAPER_USERNAME_SECRET, OP_SCRAPER_NUM_SECRET

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SCRIPTS_DIR="${PROJECT_ROOT}/scripts"
SERVICE_DIR="${PROJECT_ROOT}/services/israeli-bank-scrapers-service"

if [ ! -d "${SERVICE_DIR}" ]; then
  echo "Error: Israeli bank scrapers service not found at ${SERVICE_DIR}" >&2
  exit 1
fi

# shellcheck source=scripts/include/config.sh
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# shellcheck source=scripts/include/onepassword.sh
if [ -f "${SCRIPTS_DIR}/include/onepassword.sh" ]; then
  source "${SCRIPTS_DIR}/include/onepassword.sh"
fi

# --- Discount Bank ---
SCRAPER_DISCOUNT_ID=$(read_credential "${OP_SCRAPER_DISCOUNT_ID_SECRET:-}" "${SCRAPER_DISCOUNT_ID:-${SCRAPER_ID:-}}" 2>/dev/null || echo "")
SCRAPER_DISCOUNT_PASSWORD=$(read_credential "${OP_SCRAPER_DISCOUNT_PASSWORD_SECRET:-}" "${SCRAPER_DISCOUNT_PASSWORD:-${SCRAPER_PASSWORD:-}}" 2>/dev/null || echo "")
SCRAPER_DISCOUNT_NUM=$(read_credential "${OP_SCRAPER_DISCOUNT_NUM_SECRET:-}" "${SCRAPER_DISCOUNT_NUM:-${SCRAPER_NUM:-}}" 2>/dev/null || echo "")

# --- Leumi ---
SCRAPER_LEUMI_USERNAME=$(read_credential "${OP_SCRAPER_LEUMI_USERNAME_SECRET:-}" "${SCRAPER_LEUMI_USERNAME:-${SCRAPER_USERNAME:-}}" 2>/dev/null || echo "")
SCRAPER_LEUMI_PASSWORD=$(read_credential "${OP_SCRAPER_LEUMI_PASSWORD_SECRET:-}" "${SCRAPER_LEUMI_PASSWORD:-${SCRAPER_PASSWORD:-}}" 2>/dev/null || echo "")

# --- Hapoalim ---
SCRAPER_HAPOALIM_USER_CODE=$(read_credential "${OP_SCRAPER_HAPOALIM_USER_CODE_SECRET:-}" "${SCRAPER_HAPOALIM_USER_CODE:-${SCRAPER_USERNAME:-}}" 2>/dev/null || echo "")
SCRAPER_HAPOALIM_PASSWORD=$(read_credential "${OP_SCRAPER_HAPOALIM_PASSWORD_SECRET:-}" "${SCRAPER_HAPOALIM_PASSWORD:-${SCRAPER_PASSWORD:-}}" 2>/dev/null || echo "")

# --- Generic (fallback for other banks) ---
SCRAPER_USERNAME=$(read_credential "${OP_SCRAPER_USERNAME_SECRET:-}" "${SCRAPER_USERNAME:-}" 2>/dev/null || echo "")
SCRAPER_PASSWORD=$(read_credential "${OP_SCRAPER_PASSWORD_SECRET:-}" "${SCRAPER_PASSWORD:-}" 2>/dev/null || echo "")
SCRAPER_ID=$(read_credential "${OP_SCRAPER_ID_SECRET:-}" "${SCRAPER_ID:-}" 2>/dev/null || echo "")
SCRAPER_NUM=$(read_credential "${OP_SCRAPER_NUM_SECRET:-}" "${SCRAPER_NUM:-}" 2>/dev/null || echo "")
SCRAPER_CARD_6_DIGITS=$(read_credential "${OP_SCRAPER_CARD_6_DIGITS_SECRET:-}" "${SCRAPER_CARD_6_DIGITS:-}" 2>/dev/null || echo "")

export SCRAPER_DISCOUNT_ID SCRAPER_DISCOUNT_PASSWORD SCRAPER_DISCOUNT_NUM
export SCRAPER_LEUMI_USERNAME SCRAPER_LEUMI_PASSWORD
export SCRAPER_HAPOALIM_USER_CODE SCRAPER_HAPOALIM_PASSWORD
export SCRAPER_USERNAME SCRAPER_PASSWORD SCRAPER_ID SCRAPER_NUM SCRAPER_CARD_6_DIGITS
export SCRAPER_COMPANY_ID="${SCRAPER_COMPANY_ID:-discount}"
export SCRAPER_START_DATE="${SCRAPER_START_DATE:-}"
export LEDGER_DATABASE_PATH="${LEDGER_DATABASE_PATH:-}"

# Port for HTTP server (default from config or 8010)
if [ -z "${PORT:-}" ] && command -v config_get_port >/dev/null 2>&1; then
  PORT=$(config_get_port "israeli_bank_scrapers" 8010)
fi
export PORT="${PORT:-8010}"

MODE="${1:-start}"
cd "${SERVICE_DIR}"

if [ "${MODE}" = "scrape" ]; then
  exec node src/cli-scrape.js
else
  exec node src/index.js
fi
