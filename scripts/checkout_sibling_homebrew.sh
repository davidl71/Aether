#!/usr/bin/env bash
# checkout_sibling_homebrew.sh - Check out sibling Homebrew tap in ~/Projects/trading
#
# Clone homebrew-ib-box-spread next to this repo if missing; optionally pull and/or
# add the tap locally. Run from repo root or scripts/.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TRADING_ROOT="${TRADING_ROOT:-$(dirname "${PROJECT_ROOT}")}"
TAP_REPO="${TAP_REPO:-homebrew-ib-box-spread}"
TAP_REPO_URL="${TAP_REPO_URL:-git@github.com:davidl71/homebrew-ib-box-spread.git}"
TAP_NAME="${TAP_NAME:-davidl71/ib-box-spread}"

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Check out the sibling Homebrew tap repository in TRADING_ROOT (default: parent of
this repo, e.g. ~/Projects/trading). Clone if missing; optionally pull and/or
add the tap with brew tap --force-local.

Options:
  --pull              After clone/checkout, run git pull in the tap repo
  --tap               Run: brew tap --force-local TAP_NAME <tap-dir>
  --tap-name NAME     Tap name (default: davidl71/ib-box-spread)
  -h, --help          Show this message

Environment:
  TRADING_ROOT        Parent directory for the tap (default: parent of repo root)
  TAP_REPO            Repository directory name (default: homebrew-ib-box-spread)
  TAP_REPO_URL        Clone URL (default: git@github.com:davidl71/homebrew-ib-box-spread.git)

Examples:
  $0                  # Clone tap sibling if missing
  $0 --pull           # Clone or pull
  $0 --tap            # Clone if missing, then brew tap --force-local
  $0 --pull --tap     # Pull and add local tap
EOF
}

DO_PULL=false
DO_TAP=false

while [[ $# -gt 0 ]]; do
  case "$1" in
  --pull)
    DO_PULL=true
    shift
    ;;
  --tap)
    DO_TAP=true
    shift
    ;;
  --tap-name)
    TAP_NAME="$2"
    shift 2
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    echo "Unknown option: $1" >&2
    usage
    exit 1
    ;;
  esac
done

TAP_DIR="${TRADING_ROOT}/${TAP_REPO}"

if [[ ! -d "${TAP_DIR}" ]]; then
  echo "Cloning ${TAP_REPO} into ${TRADING_ROOT}..."
  mkdir -p "${TRADING_ROOT}"
  git clone "${TAP_REPO_URL}" "${TAP_DIR}"
  echo "Cloned: ${TAP_DIR}"
else
  echo "Tap repo already present: ${TAP_DIR}"
  if [[ "${DO_PULL}" = true ]]; then
    (cd "${TAP_DIR}" && git pull)
  fi
fi

if [[ "${DO_TAP}" = true ]]; then
  if ! command -v brew >/dev/null 2>&1; then
    echo "Error: brew not found, cannot add tap" >&2
    exit 1
  fi
  echo "Adding local tap: ${TAP_NAME} -> ${TAP_DIR}"
  brew tap --force-local "${TAP_NAME}" "${TAP_DIR}"
  echo "Tap added. Install with: brew install ${TAP_NAME}/ib-box-spread"
fi
