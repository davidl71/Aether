#!/usr/bin/env bash
# Stop Israeli Bank Scrapers service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE="israeli_bank_scrapers"
ACTION="stop"

exec "${SCRIPT_DIR}/service_manager.sh" "$ACTION" "$SERVICE"
