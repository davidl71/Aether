#!/bin/bash
# test_tui.sh - Run TUI tests with various options

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TUI_DIR="${PROJECT_ROOT}/tui"

cd "${TUI_DIR}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
SHORT=false
UPDATE_SNAPSHOTS=false
COVERAGE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --short)
            SHORT=true
            shift
            ;;
        --update-snapshots)
            UPDATE_SNAPSHOTS=true
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--short] [--update-snapshots] [--coverage] [--verbose]"
            exit 1
            ;;
    esac
done

echo "════════════════════════════════════════════════════════════"
echo "  TUI Test Suite"
echo "════════════════════════════════════════════════════════════"
echo ""

# Build test flags
TEST_FLAGS=""
if [ "$VERBOSE" = true ]; then
    TEST_FLAGS="-v"
fi

if [ "$SHORT" = true ]; then
    TEST_FLAGS="${TEST_FLAGS} -short"
fi

# Run unit tests
echo "📋 Running unit tests..."
if ! go test ${TEST_FLAGS} -short ./...; then
    echo -e "${RED}❌ Unit tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Unit tests passed${NC}"
echo ""

# Run integration tests (if not short mode)
if [ "$SHORT" = false ]; then
    echo "🔗 Running integration tests..."
    if ! go test ${TEST_FLAGS} -run TestTUIHelpAndQuit; then
        echo -e "${RED}❌ Integration tests failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Integration tests passed${NC}"
    echo ""
fi

# Run snapshot tests (if not short mode and update flag set)
if [ "$SHORT" = false ] && [ "$UPDATE_SNAPSHOTS" = true ]; then
    echo "📸 Updating snapshots..."
    if ! go test ${TEST_FLAGS} -update-snapshots -run TestTUISnapshot; then
        echo -e "${RED}❌ Snapshot update failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Snapshots updated${NC}"
    echo ""
elif [ "$SHORT" = false ]; then
    echo "📸 Running snapshot tests..."
    if ! go test ${TEST_FLAGS} -run TestTUISnapshot; then
        echo -e "${YELLOW}⚠️  Snapshot tests failed${NC}"
        echo "Run with --update-snapshots to update snapshots if changes are intentional"
        exit 1
    fi
    echo -e "${GREEN}✅ Snapshot tests passed${NC}"
    echo ""
fi

# Generate coverage report
if [ "$COVERAGE" = true ]; then
    echo "📊 Generating coverage report..."
    go test -coverprofile=coverage.out ./...
    COVERAGE_PCT=$(go tool cover -func=coverage.out | tail -1 | awk '{print $3}')
    echo -e "${GREEN}Coverage: ${COVERAGE_PCT}${NC}"

    # Generate HTML report
    go tool cover -html=coverage.out -o coverage.html
    echo "HTML report: ${TUI_DIR}/coverage.html"
    echo ""
fi

echo "════════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo "════════════════════════════════════════════════════════════"
