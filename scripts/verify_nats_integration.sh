#!/usr/bin/env bash
# verify_nats_integration.sh - Quick verification script for NATS integration
# Usage: ./scripts/verify_nats_integration.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

log_info "════════════════════════════════════════════════════════════"
log_info "  NATS Integration Verification"
log_info "════════════════════════════════════════════════════════════"

# Check NATS server
log_info ""
log_info "1. Checking NATS server..."
if curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
  log_info "   ✅ NATS server is running"
else
  log_error "   ❌ NATS server is not running"
  log_info "   Start with: ./scripts/start_nats.sh"
  exit 1
fi

# Check backend compilation
log_info ""
log_info "2. Checking backend service compilation..."
cd "${PROJECT_ROOT}/agents/backend"
if [ -f .venv/bin/activate ]; then
  source .venv/bin/activate
  export PYO3_PYTHON="$(which python)"
fi

if cargo check -p backend_service > /dev/null 2>&1; then
  log_info "   ✅ Backend service compiles successfully"
else
  log_error "   ❌ Backend service compilation failed"
  log_info "   Run: cd agents/backend && cargo check -p backend_service"
  exit 1
fi

# Check nats_adapter compilation
log_info ""
log_info "3. Checking nats_adapter crate..."
if cargo check -p nats_adapter > /dev/null 2>&1; then
  log_info "   ✅ nats_adapter compiles successfully"
else
  log_error "   ❌ nats_adapter compilation failed"
  exit 1
fi

# Check integration files exist
log_info ""
log_info "4. Checking integration files..."
FILES=(
  "agents/backend/services/backend_service/src/nats_integration.rs"
  "agents/backend/crates/nats_adapter/src/topics.rs"
  "agents/backend/crates/nats_adapter/src/bridge.rs"
  "scripts/test_nats_integration.sh"
  "docs/NATS_TOPICS_REGISTRY.md"
  "docs/NATS_TESTING_GUIDE.md"
)

ALL_EXIST=true
for file in "${FILES[@]}"; do
  if [ -f "${PROJECT_ROOT}/${file}" ]; then
    log_info "   ✅ ${file}"
  else
    log_error "   ❌ ${file} - MISSING"
    ALL_EXIST=false
  fi
done

if [ "$ALL_EXIST" = false ]; then
  exit 1
fi

# Summary
log_info ""
log_info "════════════════════════════════════════════════════════════"
log_info "  ✅ All checks passed!"
log_info "════════════════════════════════════════════════════════════"
log_info ""
log_info "Next steps:"
log_info "  1. Run tests: ./scripts/test_nats_integration.sh"
log_info "  2. Start backend: cd agents/backend && cargo run -p backend_service"
log_info "  3. Subscribe to topics: nats sub 'market-data.tick.>'"
log_info ""
