#!/usr/bin/env bash
# setup_dependency_security_cron.sh - Set up cron job for dependency security scanning
# Usage: ./scripts/setup_dependency_security_cron.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
CRON_SCHEDULE="0 6 * * *"  # 6 AM daily
CRON_SCRIPT="${SCRIPT_DIR}/run_dependency_security_cron.sh"
LOG_FILE="${SCRIPT_DIR}/dependency_security_cron.log"
ERROR_LOG="${SCRIPT_DIR}/dependency_security_cron_error.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Create runner script
info "Creating cron runner script: ${CRON_SCRIPT}"

cat > "${CRON_SCRIPT}" << 'CRON_SCRIPT_EOF'
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
CRON_SCRIPT_EOF

chmod +x "${CRON_SCRIPT}"
info "Created and made executable: ${CRON_SCRIPT}"

# Check if cron job already exists
CRON_JOB="${CRON_SCHEDULE} ${CRON_SCRIPT}"
if crontab -l 2>/dev/null | grep -q "${CRON_SCRIPT}"; then
  warn "Cron job already exists for dependency security scan"
  echo ""
  echo "Current crontab entries:"
  crontab -l 2>/dev/null | grep "${CRON_SCRIPT}" || true
  echo ""
  read -p "Do you want to remove the existing entry and add a new one? (y/N): " -n 1 -r
  echo ""
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Remove existing entry
    crontab -l 2>/dev/null | grep -v "${CRON_SCRIPT}" | crontab -
    info "Removed existing cron job"
  else
    info "Keeping existing cron job"
    exit 0
  fi
fi

# Add cron job
info "Adding cron job: ${CRON_SCHEDULE} ${CRON_SCRIPT}"
(crontab -l 2>/dev/null; echo "${CRON_JOB}") | crontab -

info "✅ Cron job installed successfully!"
echo ""
echo "Schedule: ${CRON_SCHEDULE} (6 AM daily)"
echo "Script: ${CRON_SCRIPT}"
echo "Log file: ${LOG_FILE}"
echo "Error log: ${ERROR_LOG}"
echo ""
echo "To view logs:"
echo "  tail -f ${LOG_FILE}"
echo "  tail -f ${ERROR_LOG}"
echo ""
echo "To remove the cron job:"
echo "  crontab -l | grep -v '${CRON_SCRIPT}' | crontab -"
echo ""
echo "To test manually:"
echo "  ${CRON_SCRIPT}"
