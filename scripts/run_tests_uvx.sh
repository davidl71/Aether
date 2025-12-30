#!/bin/bash
# Run Python tests using uvx (isolated pytest environment)
# Usage: ./scripts/run_tests_uvx.sh [--coverage] [--html]

set -euo pipefail

cd "$(dirname "$0")/.."

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

# Check if uvx is available
if ! command -v uvx &> /dev/null; then
    echo "❌ Error: uvx is not installed (part of uv)"
    echo "   Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

echo "🐍 Using uvx for isolated test execution"
echo ""

# Install dependencies if needed
if [ -f "requirements.txt" ]; then
    echo "📦 Installing test dependencies..."
    uvx pip install -q pytest pytest-cov coverage fastapi
fi

echo "🧪 Running Python tests with uvx..."

# Build pytest arguments
PYTEST_ARGS="python/tests/ python/integration/ -v"

if [[ "$COVERAGE" == "true" ]]; then
    PYTEST_ARGS="$PYTEST_ARGS --cov=python/services --cov=python/tui --cov=python/integration"

    if [[ "$HTML" == "true" ]]; then
        PYTEST_ARGS="$PYTEST_ARGS --cov-report=html --cov-report=term"
        echo "📊 Running tests with coverage (HTML report)..."
    else
        PYTEST_ARGS="$PYTEST_ARGS --cov-report=term"
        echo "📊 Running tests with coverage..."
    fi
else
    echo "Running tests without coverage..."
fi

# Run tests using uvx (creates isolated environment automatically)
uvx pytest "$PYTEST_ARGS"

echo ""
echo "✅ Tests completed!"

if [[ "$HTML" == "true" ]]; then
    echo "📁 Coverage HTML report generated in htmlcov/index.html"
fi
