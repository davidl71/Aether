#!/bin/bash
# Run Python tests with coverage reporting
# Usage: ./scripts/run_python_tests.sh [--coverage] [--html]
# Use USE_NIX=1 to run inside the Nix dev shell (uv, pytest from flake).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=scripts/with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
run_with_nix_if_requested "$@"

cd "${PROJECT_ROOT}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$PROJECT_ROOT/.cache}"
export UV_CACHE_DIR="${UV_CACHE_DIR:-$XDG_CACHE_HOME/uv}"
export PIP_CACHE_DIR="${PIP_CACHE_DIR:-$XDG_CACHE_HOME/pip}"
PYTHON_ARTIFACT_DIR="${PYTHON_ARTIFACT_DIR:-$PROJECT_ROOT/build/test-artifacts/python}"
mkdir -p "$XDG_CACHE_HOME" "$UV_CACHE_DIR" "$PIP_CACHE_DIR" "$PYTHON_ARTIFACT_DIR"

# Activate virtual environment if it exists
if [ -d "python/.venv" ]; then
  source python/.venv/bin/activate
elif [ -d ".venv" ]; then
  source .venv/bin/activate
fi

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

USE_UV=false
if command -v uv &>/dev/null; then
  USE_UV=true
fi

# Check if pytest is available
if [[ "$USE_UV" != "true" ]] && ! command -v pytest &>/dev/null; then
  echo "❌ Error: pytest is not installed"
  echo "   Install with: uv sync --project python --extra dev"
  exit 1
fi

# Check if coverage tools are installed (if coverage requested)
if [[ "$COVERAGE" == "true" ]]; then
  if [[ "$USE_UV" != "true" ]] && ! python3 -c "import pytest_cov" 2>/dev/null; then
    echo "❌ Error: pytest-cov is not installed"
    echo "   Install with: uv sync --project python --extra dev"
    exit 1
  fi
fi

echo "🧪 Running Python tests..."
echo ""

# Run tests
if [[ "$COVERAGE" == "true" ]]; then
  if [[ "$HTML" == "true" ]]; then
    echo "📊 Running tests with coverage (HTML report)..."
    if [[ "$USE_UV" == "true" ]]; then
      uv run --project python pytest python/tests/ python/integration/ \
        --cov=python/services \
        --cov=python/tui \
        --cov=python/integration \
        --cov-report="html:${PYTHON_ARTIFACT_DIR}/htmlcov" \
        --cov-report=term \
        -v
    else
      pytest python/tests/ python/integration/ \
        --cov=python/services \
        --cov=python/tui \
        --cov=python/integration \
        --cov-report="html:${PYTHON_ARTIFACT_DIR}/htmlcov" \
        --cov-report=term \
        -v
    fi
    echo ""
    echo "✅ Coverage HTML report generated in ${PYTHON_ARTIFACT_DIR}/htmlcov/index.html"
  else
    echo "📊 Running tests with coverage..."
    if [[ "$USE_UV" == "true" ]]; then
      uv run --project python pytest python/tests/ python/integration/ \
        --cov=python/services \
        --cov=python/tui \
        --cov=python/integration \
        --cov-report=term \
        -v
    else
      pytest python/tests/ python/integration/ \
        --cov=python/services \
        --cov=python/tui \
        --cov=python/integration \
        --cov-report=term \
        -v
    fi
  fi
else
  echo "Running tests without coverage..."
  if [[ "$USE_UV" == "true" ]]; then
    uv run --project python pytest python/tests/ python/integration/ -v
  else
    pytest python/tests/ python/integration/ -v
  fi
fi

echo ""
echo "✅ Tests completed!"
