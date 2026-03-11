#!/usr/bin/env bash
# Install systemd user units and env file so you can manage backends with systemctl --user.
#
# Usage:
#   ./scripts/install_systemd_user_units.sh
#
# Then:
#   systemctl --user daemon-reload
#   systemctl --user start ib-box-spread-rust_backend
#   systemctl --user enable ib-box-spread-rust_backend   # optional: start at login
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SYSTEMD_USER="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
SRC="${PROJECT_ROOT}/config/systemd/user"

mkdir -p "$SYSTEMD_USER"
cp -f "${SRC}"/*.service "${SRC}"/ib-box-spread.env "$SYSTEMD_USER/"
sed "s|REPLACE_PROJECT_ROOT|${PROJECT_ROOT}|g" "${SYSTEMD_USER}/ib-box-spread.env" > "${SYSTEMD_USER}/ib-box-spread.env.tmp"
mv "${SYSTEMD_USER}/ib-box-spread.env.tmp" "${SYSTEMD_USER}/ib-box-spread.env"
echo "Installed to $SYSTEMD_USER"
echo "Run: systemctl --user daemon-reload"
echo "Then: systemctl --user start ib-box-spread-rust_backend ..."
