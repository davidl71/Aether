#!/usr/bin/env bash
# Start all services in dependency order (delegates to parameterized service.sh)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Use unified service manager; services in dependency order
for svc in nats memcached ib riskfree discount alpaca tastytrade tradestation web; do
  echo "==> Starting $svc..."
  "${SCRIPTS_DIR}/service.sh" start "$svc" || true
done

echo
echo "==> All services started. Logs in: ${ROOT_DIR}/logs/"
echo "    To stop all: ${SCRIPTS_DIR}/stop_all_services.sh"
