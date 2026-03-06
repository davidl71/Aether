#!/usr/bin/env bash
# check_nats_performance.sh - NATS server performance and config check
# Usage: ./scripts/check_nats_performance.sh
# Requires: NATS server running (e.g. ./scripts/start_nats.sh), curl

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
MONITOR_URL="${NATS_MONITOR_URL:-http://localhost:8222}"
CONFIG_FILE="${ROOT_DIR}/config/nats-server.conf"

echo "NATS performance check"
echo "======================"
echo ""

# 1. Server reachability
echo "1. Server"
if ! curl -sf --connect-timeout 2 "${MONITOR_URL}/healthz" >/dev/null 2>&1; then
  echo "   NATS server not reachable at ${MONITOR_URL}"
  echo "   Start with: ./scripts/start_nats.sh"
  exit 1
fi
echo "   Reachable: ${MONITOR_URL}"

# 2. /varz - server metrics
echo ""
echo "2. Server metrics (varz)"
if VARZ=$(curl -sf --connect-timeout 2 "${MONITOR_URL}/varz" 2>/dev/null); then
  if echo "$VARZ" | grep -q '"port"'; then
    PORT=$(echo "$VARZ" | sed -n 's/.*"port"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    echo "   port: ${PORT:-4222}"
  fi
  if echo "$VARZ" | grep -q '"max_payload"'; then
    MAX_PAYLOAD=$(echo "$VARZ" | sed -n 's/.*"max_payload"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    echo "   max_payload: ${MAX_PAYLOAD:-?} bytes"
  fi
  if echo "$VARZ" | grep -q '"in_msgs"'; then
    IN_MSGS=$(echo "$VARZ" | sed -n 's/.*"in_msgs"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    OUT_MSGS=$(echo "$VARZ" | sed -n 's/.*"out_msgs"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    echo "   in_msgs: ${IN_MSGS:-0}, out_msgs: ${OUT_MSGS:-0}"
  fi
  if echo "$VARZ" | grep -q '"in_bytes"'; then
    IN_BYTES=$(echo "$VARZ" | sed -n 's/.*"in_bytes"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    OUT_BYTES=$(echo "$VARZ" | sed -n 's/.*"out_bytes"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    echo "   in_bytes: ${IN_BYTES:-0}, out_bytes: ${OUT_BYTES:-0}"
  fi
else
  echo "   (could not fetch varz)"
fi

# 3. /connz - connections
echo ""
echo "3. Connections (connz)"
if CONNZ=$(curl -sf --connect-timeout 2 "${MONITOR_URL}/connz" 2>/dev/null); then
  if echo "$CONNZ" | grep -q '"num_connections"'; then
    NUM=$(echo "$CONNZ" | sed -n 's/.*"num_connections"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    echo "   num_connections: ${NUM:-0}"
  fi
else
  echo "   (could not fetch connz)"
fi

# 4. Config file summary
echo ""
echo "4. Config (config/nats-server.conf)"
if [[ -f "$CONFIG_FILE" ]]; then
  grep -E '^(port|http_port|ping_interval|ping_max|write_deadline|max_connections|max_payload|websocket)' "$CONFIG_FILE" 2>/dev/null | sed 's/^/   /' || true
  if grep -q 'jetstream' "$CONFIG_FILE" 2>/dev/null; then
    grep -E 'store_dir|max_mem|max_file' "$CONFIG_FILE" 2>/dev/null | sed 's/^/   /' || true
  fi
else
  echo "   (config file not found)"
fi

# 5. Optional: quick pub latency (if nats CLI available)
echo ""
echo "5. Latency check (optional)"
if command -v nats >/dev/null 2>&1; then
  SUBJECT="perf.check.$(date +%s)"
  (nats sub "$SUBJECT" --count=1 --timeout=3 2>/dev/null &)
  sleep 0.3
  if nats pub "$SUBJECT" "ping" -q 2>/dev/null; then
    echo "   nats pub: OK (message sent)"
  else
    echo "   nats pub: skipped or failed"
  fi
else
  echo "   Install nats CLI for latency test: brew install nats-io/nats-tools/nats"
fi

echo ""
echo "Done. For tuning see docs/NATS_SETUP.md (Performance section)."
