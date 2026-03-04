#!/usr/bin/env bash
# Fetch and extract third-party dependencies into the local cache.
# Env hints for automation/AI:
#   FETCH_COMPONENTS       comma-separated subset (default: all)
#   PROTOBUF_URL           override download URL
#   INTEL_DECIMAL_URL      path/URL for Intel decimal math tarball
#   IB_API_ARCHIVE         local path or URL to TWS API archive
#   NAUTILUS_TRADER_RELEASE or NAUTILUS_TRADER_WHEEL_URL to locate Nautilus wheel
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PLAYBOOK="${REPO_ROOT}/ansible/playbooks/fetch_third_party.yml"

# Use system CA bundle on macOS so Ansible/Python HTTPS requests verify (avoids CERTIFICATE_VERIFY_FAILED)
if [[ "$(uname)" == "Darwin" ]]; then
  for f in /etc/ssl/cert.pem /usr/local/etc/openssl/cert.pem "$(brew --prefix 2>/dev/null)/etc/openssl@3/cert.pem"; do
    if [[ -n "${f:-}" ]] && [[ -f "$f" ]]; then
      export SSL_CERT_FILE="$f"
      export REQUESTS_CA_BUNDLE="$f"
      break
    fi
  done
fi

if ! command -v ansible-playbook >/dev/null 2>&1; then
  log_error "ansible-playbook not found. Run setup_global_tools.sh first."
  exit 1
fi

log_note "Delegating third-party fetch to Ansible playbook: ${PLAYBOOK}"

export ANSIBLE_ROLES_PATH="${REPO_ROOT}/ansible/roles:${ANSIBLE_ROLES_PATH:-}"

ANSIBLE_OPTS=("--extra-vars" "repo_root=${REPO_ROOT}")
# Run against localhost without SSH (same as setup_global_tools.sh)
ANSIBLE_OPTS+=("-i" "localhost," "--connection=local")

trim() {
  local trimmed="$1"
  trimmed="${trimmed#"${trimmed%%[![:space:]]*}"}"
  trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
  printf '%s' "$trimmed"
}

if [ -n "${FETCH_COMPONENTS:-}" ]; then
  IFS=',' read -ra components <<< "${FETCH_COMPONENTS}"
  json="["
  for component in "${components[@]}"; do
    component_trimmed="$(trim "$component")"
    if [ -n "$component_trimmed" ]; then
      json="${json}\"${component_trimmed}\","
    fi
  done
  json="${json%,}]"
  [ "$json" = "]" ] && json="[]"
  ANSIBLE_OPTS+=("--extra-vars" "fetch_components=${json}")
fi

if [ -n "${PROTOBUF_URL:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "protobuf_url=${PROTOBUF_URL}")
fi

if [ -n "${INTEL_DECIMAL_URL:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "intel_decimal_url=${INTEL_DECIMAL_URL}")
fi

if [ -n "${IB_API_ARCHIVE:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "ib_api_archive=${IB_API_ARCHIVE}")
fi

if [ -n "${NAUTILUS_TRADER_RELEASE:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "nautilus_trader_release=${NAUTILUS_TRADER_RELEASE}")
fi

if [ -n "${NAUTILUS_TRADER_WHEEL_URL:-}" ]; then
  ANSIBLE_OPTS+=("--extra-vars" "nautilus_trader_wheel_url=${NAUTILUS_TRADER_WHEEL_URL}")
fi

ansible-playbook "$PLAYBOOK" "${ANSIBLE_OPTS[@]}" "$@"

log_info "Third-party fetch complete via Ansible."
