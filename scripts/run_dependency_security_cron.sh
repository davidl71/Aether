#!/usr/bin/env bash
# run_dependency_security_cron.sh - Runner script for dependency security cron job

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOG_FILE="${SCRIPT_DIR}/dependency_security_cron.log"
ERROR_LOG="${SCRIPT_DIR}/dependency_security_cron_error.log"

# Activate Python environment if it exists
if [ -d "${PROJECT_ROOT}/python/venv" ]; then
  source "${PROJECT_ROOT}/python/venv/bin/activate"
elif [ -d "${PROJECT_ROOT}/python/venv312" ]; then
  source "${PROJECT_ROOT}/python/venv312/bin/activate"
fi

# Run automation script
cd "${PROJECT_ROOT}"
python3 "${SCRIPT_DIR}/automate_dependency_security.py" \
  --config "${SCRIPT_DIR}/dependency_security_config.json" \
  >> "${LOG_FILE}" 2>> "${ERROR_LOG}"

exit_code=$?

if [ $exit_code -eq 0 ]; then
  echo "$(date '+%Y-%m-%d %H:%M:%S') - Dependency security scan completed successfully" >> "${LOG_FILE}"
else
  echo "$(date '+%Y-%m-%d %H:%M:%S') - Dependency security scan failed with exit code $exit_code" >> "${ERROR_LOG}"
fi

exit $exit_code
