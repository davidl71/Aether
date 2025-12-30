#!/bin/bash
# Run Python tests using uv/uvx for isolated environment
# Usage: ./scripts/run_tests_uv.sh [--coverage] [--html]

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

# Check if uv is available
if ! command -v uv &> /dev/null; then
    echo "❌ Error: uv is not installed"
    echo "   Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

echo "🐍 Using uv for Python environment management"
echo ""

# Sync project dependencies using uv
echo "📦 Syncing project dependencies..."
if [ -f "pyproject.toml" ]; then
    uv sync --dev
elif [ -f "requirements.txt" ]; then
    # Create a temporary pyproject.toml or use uv pip
    uv pip install -r requirements.txt
else
    echo "⚠️  Warning: No pyproject.toml or requirements.txt found"
fi

echo ""
echo "🧪 Running Python tests with uv..."

# Build coverage arguments
COV_ARGS=""
REPORT_ARGS=""

if [[ "$COVERAGE" == "true" ]]; then
    COV_ARGS="--cov=python/services --cov=python/tui --cov=python/integration"

    if [[ "$HTML" == "true" ]]; then
        REPORT_ARGS="--cov-report=html --cov-report=term"
        echo "📊 Running tests with coverage (HTML report)..."
    else
        REPORT_ARGS="--cov-report=term"
        echo "📊 Running tests with coverage..."
    fi
else
    echo "Running tests without coverage..."
fi

# Run tests using uv run (ensures correct environment)
uv run pytest python/tests/ python/integration/ \
    "$COV_ARGS" \
    "$REPORT_ARGS" \
    -v

echo ""
echo "✅ Tests completed!"

if [[ "$HTML" == "true" ]]; then
    echo "📁 Coverage HTML report generated in htmlcov/index.html"
fi
