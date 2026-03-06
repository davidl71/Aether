#!/bin/bash
# Run Python tests with coverage reporting
# Usage: ./scripts/run_python_tests.sh [--coverage] [--html]
# Use USE_NIX=1 to run inside the Nix dev shell (uv, pytest from flake).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
run_with_nix_if_requested "$@"

cd "${PROJECT_ROOT}"

COVERAGE=false
HTML=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage)
            COVERAGE=true
            shift
            ;;
        --html)
            COVERAGE=true
            HTML=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--coverage] [--html]"
            exit 1
            ;;
    esac
done

# Check if pytest is available
if ! command -v pytest &> /dev/null; then
    echo "❌ Error: pytest is not installed"
    echo "   Install with: pip install pytest pytest-cov coverage"
    exit 1
fi

# Check if coverage tools are installed (if coverage requested)
if [[ "$COVERAGE" == "true" ]]; then
    if ! python3 -c "import pytest_cov" 2>/dev/null; then
        echo "❌ Error: pytest-cov is not installed"
        echo "   Install with: pip install pytest-cov coverage"
        exit 1
    fi
fi

echo "🧪 Running Python tests..."
echo ""

# Run tests
if [[ "$COVERAGE" == "true" ]]; then
    if [[ "$HTML" == "true" ]]; then
        echo "📊 Running tests with coverage (HTML report)..."
        pytest python/tests/ python/integration/ \
            --cov=python/services \
            --cov=python/tui \
            --cov=python/integration \
            --cov-report=html \
            --cov-report=term \
            -v
        echo ""
        echo "✅ Coverage HTML report generated in htmlcov/index.html"
    else
        echo "📊 Running tests with coverage..."
        pytest python/tests/ python/integration/ \
            --cov=python/services \
            --cov=python/tui \
            --cov=python/integration \
            --cov-report=term \
            -v
    fi
else
    echo "Running tests without coverage..."
    pytest python/tests/ python/integration/ -v
fi

echo ""
echo "✅ Tests completed!"
