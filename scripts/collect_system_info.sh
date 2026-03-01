#!/usr/bin/env bash
# collect_system_info.sh - Collect system information for environment documentation
# Thin wrapper around collect_system_info_python.py for compatibility.
# Usage: ./scripts/collect_system_info.sh > system_info.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

exec "${SCRIPT_DIR}/collect_system_info_python.py" "$@"
