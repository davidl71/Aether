#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# shellcheck source=./include/logging.sh
source "${SCRIPT_DIR}/include/logging.sh"

SCANCODE_VERSION="${SCANCODE_VERSION:-32.2.0}"
VENV_DIR="${REPO_ROOT}/.venv/scancode"
ACTIVATE_PATH="${VENV_DIR}/bin/activate"

ensure_python() {
  if command -v python3 >/dev/null 2>&1; then
    return 0
  fi
  log_error "python3 is required to create the virtual environment. Install Python 3 and retry."
  exit 1
}

create_venv() {
  if [[ -f "${ACTIVATE_PATH}" ]]; then
    log_note "Reusing existing virtual environment at ${VENV_DIR}"
  else
    log_info "Creating Python virtual environment at ${VENV_DIR}"
    python3 -m venv "${VENV_DIR}"
  fi
}

install_scancode() {
  # shellcheck disable=SC1090
  source "${ACTIVATE_PATH}"
  python -m pip install --upgrade pip wheel >/dev/null
  log_info "Installing scancode-toolkit ${SCANCODE_VERSION}"
  python -m pip install --upgrade "scancode-toolkit==${SCANCODE_VERSION}"
  log_info "Installed $(scancode --version 2>&1 | head -n 1)"
}

emit_summary() {
  cat <<EOF

scancode-toolkit is available in the virtual environment:
  source "${ACTIVATE_PATH}"
  scancode --help

To run the standard compliance scan:
  source "${ACTIVATE_PATH}"
  scancode --license --copyright --info -clp --json-pp build/scancode.json .

EOF
}

main() {
  ensure_python
  create_venv
  install_scancode
  emit_summary
}

main "$@"

