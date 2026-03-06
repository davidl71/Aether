#!/usr/bin/env bash
# Start Israeli Bank Scrapers service (port 8010)
# Use: ./scripts/service_manager.sh start israeli_bank_scrapers

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE="israeli_bank_scrapers"
ACTION="start"

exec "${SCRIPT_DIR}/service_manager.sh" "$ACTION" "$SERVICE"
