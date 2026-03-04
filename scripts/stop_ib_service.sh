#!/usr/bin/env bash
# DEPRECATED: This script is a wrapper for service_manager.sh
# Please use: ./scripts/service_manager.sh <action> <service>

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE="ib"
ACTION="stop"

exec "${SCRIPT_DIR}/service_manager.sh" "$ACTION" "$SERVICE"
