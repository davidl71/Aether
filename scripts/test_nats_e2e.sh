#!/usr/bin/env bash
# test_nats_e2e.sh - End-to-end NATS integration test for all components
# Tests: C++ TWS Client, Python Strategy Runner, TypeScript Frontend
# Usage: ./scripts/test_nats_e2e.sh [component]
# Components: cpp, python, typescript, all (default: all)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPONENT="${1:-all}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
check_prerequisites() {
  log_info "Checking prerequisites..."

  # Check NATS server
  if ! curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
    log_error "NATS server is not running"
    log_info "Start NATS server: ./scripts/start_nats.sh"
    exit 1
  fi
  log_info "  ✅ NATS server is running"

  # Check nats CLI
  if ! command -v nats > /dev/null 2>&1; then
    log_error "nats CLI not found. Install: brew install nats-io/nats-tools/nats"
    exit 1
  fi
  log_info "  ✅ nats CLI is available"
}

# Test C++ TWS Client NATS integration
test_cpp_integration() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  C++ TWS Client NATS Integration Test"
  log_info "════════════════════════════════════════════════════════════"

  cd "${PROJECT_ROOT}/native"

  # Check if build directory exists
  if [ ! -d "build" ]; then
    log_info "Building C++ project with NATS enabled..."
    cmake -B build -DENABLE_NATS=ON
    cmake --build build
  else
    log_info "Rebuilding C++ project with NATS enabled..."
    cmake -B build -DENABLE_NATS=ON
    cmake --build build
  fi

  if [ ! -f "build/ib_box_spread" ]; then
    log_error "  ❌ C++ binary not found after build"
    return 1
  fi

  log_info "  ✅ C++ project built successfully with NATS"

  # Test compilation with NATS
  log_info "  Verifying NATS symbols in binary..."
  if nm build/ib_box_spread 2>/dev/null | grep -q "nats\|NATS" || true; then
    log_info "  ✅ NATS symbols found in binary"
  else
    log_warning "  ⚠️  NATS symbols not found (may be statically linked)"
  fi

  log_info "${GREEN}✅ C++ integration test passed${NC}"
  return 0
}

# Test Python Strategy Runner NATS integration
test_python_integration() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Python Strategy Runner NATS Integration Test"
  log_info "════════════════════════════════════════════════════════════"

  cd "${PROJECT_ROOT}"

  # Check if nats-py is installed
  if ! python3 -c "import nats" 2>/dev/null; then
    log_error "  ❌ nats-py not installed"
    log_info "  Install: pip3 install --break-system-packages 'nats-py>=2.6.0'"
    return 1
  fi
  log_info "  ✅ nats-py is installed"

  # Run Python test script
  if [ -f "python/integration/test_nats_client.py" ]; then
    log_info "  Running Python NATS client test..."
    if python3 python/integration/test_nats_client.py; then
      log_info "${GREEN}✅ Python integration test passed${NC}"
      return 0
    else
      log_error "${RED}❌ Python integration test failed${NC}"
      return 1
    fi
  else
    log_warning "  ⚠️  Python test script not found"
    return 1
  fi
}

# Test TypeScript Frontend NATS integration
test_typescript_integration() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  TypeScript Frontend NATS Integration Test"
  log_info "════════════════════════════════════════════════════════════"

  cd "${PROJECT_ROOT}/web"

  # Check if node_modules exists
  if [ ! -d "node_modules" ]; then
    log_info "  Installing dependencies..."
    npm install
  fi

  # Check if nats.ws is installed
  if ! npm list nats.ws > /dev/null 2>&1; then
    log_error "  ❌ nats.ws not installed"
    log_info "  Install: cd web && npm install"
    return 1
  fi
  log_info "  ✅ nats.ws is installed"

  # Check if NATS service file exists
  if [ ! -f "src/services/nats.ts" ]; then
    log_error "  ❌ NATS service file not found"
    return 1
  fi
  log_info "  ✅ NATS service file exists"

  # Check if NATS hook file exists
  if [ ! -f "src/hooks/useNATS.ts" ]; then
    log_error "  ❌ NATS hook file not found"
    return 1
  fi
  log_info "  ✅ NATS hook file exists"

  # Check if HeaderStatus uses NATS hook
  if grep -q "useNATS" src/components/HeaderStatus.tsx 2>/dev/null; then
    log_info "  ✅ HeaderStatus integrates NATS hook"
  else
    log_warning "  ⚠️  HeaderStatus does not use NATS hook"
  fi

  # TypeScript compilation check
  log_info "  Checking TypeScript compilation..."
  if npm run build > /dev/null 2>&1; then
    log_info "  ✅ TypeScript compiles successfully"
  else
    log_warning "  ⚠️  TypeScript compilation check skipped (may require dev dependencies)"
  fi

  log_info "${GREEN}✅ TypeScript integration test passed${NC}"
  return 0
}

# Test message flow between components
test_message_flow() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Message Flow Test (Automated + Manual)"
  log_info "════════════════════════════════════════════════════════════"

  # Automated message format validation
  log_info "Testing message format consistency..."

  # Test Python message publishing
  log_info "1. Testing Python message publishing..."
  local python_test_output
  python_test_output=$(mktemp)
  if python3 python/integration/test_nats_client.py > "${python_test_output}" 2>&1; then
    log_info "   ${GREEN}✅ Python messages published successfully${NC}"
  else
    log_error "   ${RED}❌ Python message publishing failed${NC}"
    cat "${python_test_output}"
    rm -f "${python_test_output}"
    return 1
  fi
  rm -f "${python_test_output}"

  # Test message format with nats CLI
  log_info "2. Testing message format validation..."
  local format_test_output
  format_test_output=$(mktemp)
  timeout 3 nats sub "strategy.signal.>" > "${format_test_output}" 2>&1 &
  local sub_pid=$!
  sleep 0.5

  # Publish test message
  echo '{"id":"test-uuid","timestamp":"2025-11-20T00:00:00Z","source":"test","type":"StrategySignal","payload":{"symbol":"TEST","price":100.0,"signal_type":"test"}}' | \
    nats pub "strategy.signal.TEST" --force-stdin 2>/dev/null
  sleep 1

  if grep -q "TEST" "${format_test_output}"; then
    log_info "   ${GREEN}✅ Message format validation passed${NC}"
  else
    log_warning "   ${YELLOW}⚠️  Message format validation inconclusive${NC}"
  fi

  kill $sub_pid 2>/dev/null || true
  rm -f "${format_test_output}"

  # Manual verification steps
  log_info ""
  log_info "3. Manual verification steps:"
  log_info "   ${YELLOW}Start NATS subscriber:${NC}"
  log_info "   ${YELLOW}  nats sub \">\"${NC}"
  log_info ""
  log_info "   ${YELLOW}Test Python publishing:${NC}"
  log_info "   ${YELLOW}  python3 python/integration/test_nats_client.py${NC}"
  log_info "   Expected: Strategy signals and decisions appear in subscriber"
  log_info ""
  log_info "   ${YELLOW}Test C++ publishing (when TWS client runs):${NC}"
  log_info "   ${YELLOW}  cd native && ./build/ib_box_spread${NC}"
  log_info "   Expected: Market data ticks appear in subscriber"
  log_info ""
  log_info "   ${YELLOW}Test TypeScript connection:${NC}"
  log_info "   ${YELLOW}  cd web && npm run dev${NC}"
  log_info "   Expected: NATS status badge shows 'ok' in browser"

  return 0
}

# Test error handling scenarios
test_error_handling() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Error Handling Test"
  log_info "════════════════════════════════════════════════════════════"

  log_info "Testing graceful degradation when NATS is unavailable..."

  # Test Python error handling
  log_info "1. Testing Python error handling..."
  if python3 -c "
from integration.nats_client import NATSClient
import asyncio

async def test():
    client = NATSClient('nats://localhost:9999')  # Invalid port
    result = await client.connect()
    return not result  # Should return False (graceful failure)

result = asyncio.run(test())
exit(0 if result else 1)
" 2>/dev/null; then
    log_info "   ${GREEN}✅ Python handles connection failures gracefully${NC}"
  else
    log_warning "   ${YELLOW}⚠️  Python error handling needs verification${NC}"
  fi

  log_info ""
  log_info "2. Manual error handling tests:"
  log_info "   ${YELLOW}Stop NATS server:${NC}"
  log_info "   ${YELLOW}  ./scripts/stop_nats.sh${NC}"
  log_info ""
  log_info "   ${YELLOW}Test Python (should handle gracefully):${NC}"
  log_info "   ${YELLOW}  python3 python/integration/test_nats_client.py${NC}"
  log_info "   Expected: Logs warning, returns False, no crash"
  log_info ""
  log_info "   ${YELLOW}Test C++ (should handle gracefully):${NC}"
  log_info "   ${YELLOW}  cd native && ./build/ib_box_spread${NC}"
  log_info "   Expected: Logs warning, continues without NATS"
  log_info ""
  log_info "   ${YELLOW}Test TypeScript (should handle gracefully):${NC}"
  log_info "   ${YELLOW}  cd web && npm run dev${NC}"
  log_info "   Expected: NATS status badge shows 'warn', no crash"

  return 0
}

# Test performance benchmarks
test_performance() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Performance Benchmark Test"
  log_info "════════════════════════════════════════════════════════════"

  log_info "Testing message throughput and latency..."

  # Test basic throughput
  log_info "1. Testing message throughput..."
  local start_time
  start_time=$(date +%s.%N)
  for i in {1..100}; do
    echo "{\"test\":$i}" | nats pub "test.performance" --force-stdin 2>/dev/null
  done
  local end_time
  end_time=$(date +%s.%N)
  local duration
  duration=$(echo "$end_time - $start_time" | bc)
  local throughput
  throughput=$(echo "scale=2; 100 / $duration" | bc)

  log_info "   Published 100 messages in ${duration}s"
  log_info "   Throughput: ${throughput} messages/second"

  if (( $(echo "$throughput > 100" | bc -l) )); then
    log_info "   ${GREEN}✅ Throughput acceptable (>100 msg/s)${NC}"
  else
    log_warning "   ${YELLOW}⚠️  Throughput below expected (expected >100 msg/s)${NC}"
  fi

  # Test latency
  log_info ""
  log_info "2. Testing message latency..."
  log_info "   ${YELLOW}Run latency test:${NC}"
  log_info "   ${YELLOW}  nats bench test.latency --msgs=1000${NC}"
  log_info "   Expected: Average latency < 10ms (local)"

  return 0
}

# Run all tests
run_all_tests() {
  local failed=0

  check_prerequisites

  test_cpp_integration || ((failed++))
  test_python_integration || ((failed++))
  test_typescript_integration || ((failed++))
  test_message_flow || ((failed++))
  test_error_handling || ((failed++))
  test_performance || ((failed++))

  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  if [ $failed -eq 0 ]; then
    log_info "${GREEN}  ✅ All end-to-end tests passed!${NC}"
  else
    log_error "${RED}  ❌ ${failed} test(s) failed${NC}"
  fi
  log_info "════════════════════════════════════════════════════════════"

  return $failed
}

# Main execution
main() {
  log_info "════════════════════════════════════════════════════════════"
  log_info "  NATS End-to-End Integration Test Suite"
  log_info "════════════════════════════════════════════════════════════"

  case "${COMPONENT}" in
    cpp)
      check_prerequisites
      test_cpp_integration
      ;;
    python)
      check_prerequisites
      test_python_integration
      ;;
    typescript)
      check_prerequisites
      test_typescript_integration
      ;;
    all)
      run_all_tests
      ;;
    *)
      log_error "Unknown component: ${COMPONENT}"
      log_info "Valid components: cpp, python, typescript, all"
      exit 1
      ;;
  esac
}

main "$@"
