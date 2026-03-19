#!/usr/bin/env bash
# Test snapshot-write and yield-curve CLI commands; print troubleshooting hints if empty/fail.
# Run from repo root: ./scripts/check_snapshot_yield_curve.sh
# Requires: NATS (with JetStream) and backend_service running for full success.

set -e
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/agents/backend"

echo "=== snapshot-write ==="
if cargo run -p cli -- snapshot-write 2>&1; then
  echo "[OK] snapshot-write succeeded"
else
  r=$?
  echo ""
  echo ">>> Troubleshooting: snapshot-write failed (exit ${r}; no responders?)"
  echo "    1. Rebuild backend: cargo build -p backend_service"
  echo "    2. Restart backend_service so it subscribes to api.snapshot.publish_now"
  echo "    3. Ensure NATS is running (e.g. lsof -i :4222)"
  echo "    See: docs/runbooks/SNAPSHOT_YIELD_CURVE_TROUBLESHOOTING.md"
  echo ""
fi

echo "=== yield-curve --symbol SPX ==="
OUT=$(cargo run -p cli -- yield-curve --symbol SPX 2>&1) || true
echo "$OUT"
if echo "$OUT" | grep -q "No yield curve points"; then
  echo ""
  echo ">>> Troubleshooting: yield curve empty"
  echo "    1. NATS must have JetStream (config/nats-server.conf jetstream { store_dir })"
  echo "    2. Backend config [market_data] must have symbols = [\"SPX\", ...]"
  echo "    3. Restart backend; wait a few seconds for yield curve writer to populate KV"
  echo "    See: docs/runbooks/SNAPSHOT_YIELD_CURVE_TROUBLESHOOTING.md"
  echo ""
elif echo "$OUT" | grep -q "Rate %"; then
  echo "[OK] yield-curve returned points"
fi
