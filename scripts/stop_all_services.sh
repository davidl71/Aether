#!/usr/bin/env bash
# Stop all services in reverse dependency order (delegates to parameterized service.sh)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Use unified service manager; reverse of start order
for svc in web tradestation tastytrade alpaca discount riskfree ib memcached nats rust; do
  echo "==> Stopping $svc..."
  "${SCRIPTS_DIR}/service.sh" stop "$svc" || true
done

echo
echo "==> All services stopped."
