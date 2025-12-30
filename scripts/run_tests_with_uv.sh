#!/bin/bash
# Run Python tests using uv (project-managed environment)
# This uses uv's project management features for better dependency handling
# Usage: ./scripts/run_tests_with_uv.sh [--coverage] [--html]

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

echo "🐍 Using uv for project-managed Python environment"
echo ""

# Navigate to python directory (where pyproject.toml is)
cd python

# Sync project dependencies (installs dev dependencies including pytest)
echo "📦 Syncing project dependencies with uv..."
uv sync --dev

echo ""
echo "🧪 Running Python tests..."

# Build pytest arguments
PYTEST_ARGS="tests/ ../python/integration/ -v"

if [[ "$COVERAGE" == "true" ]]; then
    PYTEST_ARGS="$PYTEST_ARGS --cov=services --cov=tui --cov=integration"

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

# Run tests using uv run (uses project-managed environment)
uv run pytest "$PYTEST_ARGS"

echo ""
echo "✅ Tests completed!"

if [[ "$HTML" == "true" ]]; then
    echo "📁 Coverage HTML report generated in htmlcov/index.html"
fi
