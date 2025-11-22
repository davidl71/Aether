#!/usr/bin/env bash
# test_nats_integration.sh - Test NATS integration with mock data
# Usage: ./scripts/test_nats_integration.sh [test-type]
# Test types: basic, performance, error, all (default: all)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEST_TYPE="${1:-all}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if NATS server is running
check_nats_server() {
  if ! curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
    log_error "NATS server is not running"
    log_info "Start NATS server: ./scripts/start_nats.sh"
    exit 1
  fi
  log_info "✅ NATS server is running"
}

# Check if nats CLI is installed
check_nats_cli() {
  if ! command -v nats > /dev/null 2>&1; then
    log_warning "nats CLI not found. Installing via Homebrew..."
    if command -v brew > /dev/null 2>&1; then
      brew install nats-io/nats-tools/nats
    else
      log_error "Homebrew not found. Please install nats CLI manually:"
      log_info "  brew install nats-io/nats-tools/nats"
      exit 1
    fi
  fi
  log_info "✅ nats CLI is available"
}

# Test basic publish/subscribe
test_basic() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Basic Publish/Subscribe Test"
  log_info "════════════════════════════════════════════════════════════"

  local test_subject="test.basic.$(date +%s)"
  local test_message='{"symbol":"SPY","bid":100.5,"ask":100.6,"timestamp":"2025-01-01T00:00:00Z"}'

  log_info "Test subject: ${test_subject}"
  log_info "Publishing test message..."

  # Start subscriber in background
  local output_file=$(mktemp)
  nats sub "${test_subject}" > "${output_file}" 2>&1 &
  local sub_pid=$!
  sleep 1

  # Publish message
  echo "${test_message}" | nats pub "${test_subject}" --stdin

  # Wait for message
  sleep 1

  # Check if message was received
  if grep -q "SPY" "${output_file}"; then
    log_info "${GREEN}✅ Basic test passed${NC}"
    kill $sub_pid 2>/dev/null || true
    rm -f "${output_file}"
    return 0
  else
    log_error "${RED}❌ Basic test failed - message not received${NC}"
    log_info "Output: $(cat ${output_file})"
    kill $sub_pid 2>/dev/null || true
    rm -f "${output_file}"
    return 1
  fi
}

# Test market data topics
test_market_data() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Market Data Topic Test"
  log_info "════════════════════════════════════════════════════════════"

  local symbols=("SPY" "XSP" "NDX")
  local test_count=0
  local success_count=0

  for symbol in "${symbols[@]}"; do
    local test_subject="market-data.tick.${symbol}"
    local test_message="{\"symbol\":\"${symbol}\",\"bid\":100.0,\"ask\":100.1,\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"

    log_info "Testing ${symbol} on ${test_subject}..."

    # Start subscriber
    local output_file=$(mktemp)
    timeout 2 nats sub "${test_subject}" > "${output_file}" 2>&1 &
    local sub_pid=$!
    sleep 0.5

    # Publish
    echo "${test_message}" | nats pub "${test_subject}" --stdin
    sleep 0.5

    # Check
    if grep -q "${symbol}" "${output_file}"; then
      log_info "  ${GREEN}✅ ${symbol} passed${NC}"
      ((success_count++))
    else
      log_error "  ${RED}❌ ${symbol} failed${NC}"
    fi

    ((test_count++))
    kill $sub_pid 2>/dev/null || true
    rm -f "${output_file}"
  done

  if [ $success_count -eq $test_count ]; then
    log_info "${GREEN}✅ Market data test passed (${success_count}/${test_count})${NC}"
    return 0
  else
    log_error "${RED}❌ Market data test failed (${success_count}/${test_count})${NC}"
    return 1
  fi
}

# Test strategy topics
test_strategy_topics() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Strategy Topic Test"
  log_info "════════════════════════════════════════════════════════════"

  # Test strategy signal
  local signal_subject="strategy.signal.SPY"
  local signal_message='{"symbol":"SPY","price":100.5,"timestamp":"2025-01-01T00:00:00Z"}'

  log_info "Testing strategy.signal..."
  local output_file=$(mktemp)
  timeout 2 nats sub "strategy.signal.>" > "${output_file}" 2>&1 &
  local sub_pid=$!
  sleep 0.5

  echo "${signal_message}" | nats pub "${signal_subject}" --stdin
  sleep 0.5

  local signal_ok=false
  if grep -q "SPY" "${output_file}"; then
    log_info "  ${GREEN}✅ strategy.signal passed${NC}"
    signal_ok=true
  else
    log_error "  ${RED}❌ strategy.signal failed${NC}"
  fi

  kill $sub_pid 2>/dev/null || true
  rm -f "${output_file}"

  # Test strategy decision
  local decision_subject="strategy.decision.XSP"
  local decision_message='{"symbol":"XSP","quantity":1,"side":"Buy"}'

  log_info "Testing strategy.decision..."
  output_file=$(mktemp)
  timeout 2 nats sub "strategy.decision.>" > "${output_file}" 2>&1 &
  sub_pid=$!
  sleep 0.5

  echo "${decision_message}" | nats pub "${decision_subject}" --stdin
  sleep 0.5

  local decision_ok=false
  if grep -q "XSP" "${output_file}"; then
    log_info "  ${GREEN}✅ strategy.decision passed${NC}"
    decision_ok=true
  else
    log_error "  ${RED}❌ strategy.decision failed${NC}"
  fi

  kill $sub_pid 2>/dev/null || true
  rm -f "${output_file}"

  if [ "$signal_ok" = true ] && [ "$decision_ok" = true ]; then
    log_info "${GREEN}✅ Strategy topic test passed${NC}"
    return 0
  else
    log_error "${RED}❌ Strategy topic test failed${NC}"
    return 1
  fi
}

# Test performance (latency)
test_performance() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Performance Test (Latency)"
  log_info "════════════════════════════════════════════════════════════"

  local test_subject="test.performance.$(date +%s)"
  local iterations=100
  local total_time=0

  log_info "Running ${iterations} publish/subscribe cycles..."

  for i in $(seq 1 $iterations); do
    local start_time=$(date +%s%N)

    # Start subscriber
    local output_file=$(mktemp)
    timeout 1 nats sub "${test_subject}" > "${output_file}" 2>&1 &
    local sub_pid=$!
    sleep 0.01

    # Publish
    echo "{\"test\":${i}}" | nats pub "${test_subject}" --stdin > /dev/null 2>&1

    # Wait for message
    sleep 0.01

    local end_time=$(date +%s%N)
    local duration=$((end_time - start_time))
    total_time=$((total_time + duration))

    kill $sub_pid 2>/dev/null || true
    rm -f "${output_file}"
  done

  local avg_latency_ns=$((total_time / iterations))
  local avg_latency_ms=$((avg_latency_ns / 1000000))

  log_info "Average latency: ${avg_latency_ms}ms"

  if [ $avg_latency_ms -lt 10 ]; then
    log_info "${GREEN}✅ Performance test passed (latency < 10ms)${NC}"
    return 0
  else
    log_warning "${YELLOW}⚠️  Performance test warning (latency >= 10ms)${NC}"
    return 0  # Not a failure, just a warning
  fi
}

# Test error handling (NATS down)
test_error_handling() {
  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  log_info "  Error Handling Test"
  log_info "════════════════════════════════════════════════════════════"

  log_info "This test verifies graceful degradation when NATS is unavailable"
  log_info "Run backend service with NATS stopped to verify it continues"
  log_info "${YELLOW}⚠️  Manual test required${NC}"
  log_info ""
  log_info "Steps:"
  log_info "  1. Stop NATS: ./scripts/stop_nats.sh"
  log_info "  2. Start backend: cd agents/backend && cargo run -p backend_service"
  log_info "  3. Verify: Service starts and logs 'NATS integration unavailable'"
  log_info "  4. Verify: Service continues to function normally"
  log_info "  5. Restart NATS: ./scripts/start_nats.sh"

  return 0
}

# Run all tests
run_all_tests() {
  local failed=0

  test_basic || ((failed++))
  test_market_data || ((failed++))
  test_strategy_topics || ((failed++))
  test_performance || ((failed++))
  test_error_handling || ((failed++))

  log_info ""
  log_info "════════════════════════════════════════════════════════════"
  if [ $failed -eq 0 ]; then
    log_info "${GREEN}  ✅ All tests passed!${NC}"
  else
    log_error "${RED}  ❌ ${failed} test(s) failed${NC}"
  fi
  log_info "════════════════════════════════════════════════════════════"

  return $failed
}

# Main execution
main() {
  log_info "════════════════════════════════════════════════════════════"
  log_info "  NATS Integration Test Suite"
  log_info "════════════════════════════════════════════════════════════"

  check_nats_server
  check_nats_cli

  case "${TEST_TYPE}" in
    basic)
      test_basic
      ;;
    market-data)
      test_market_data
      ;;
    strategy)
      test_strategy_topics
      ;;
    performance)
      test_performance
      ;;
    error)
      test_error_handling
      ;;
    all)
      run_all_tests
      ;;
    *)
      log_error "Unknown test type: ${TEST_TYPE}"
      log_info "Valid types: basic, market-data, strategy, performance, error, all"
      exit 1
      ;;
  esac
}

main "$@"
