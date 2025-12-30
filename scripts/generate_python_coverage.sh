#!/bin/bash
# Generate Python code coverage report
# Usage: ./scripts/generate_python_coverage.sh [--html] [--term] [--xml]

set -euo pipefail

cd "$(dirname "$0")/.."

HTML=false
TERM=true
XML=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --html)
            HTML=true
            shift
            ;;
        --term)
            TERM=true
            shift
            ;;
        --xml)
            XML=true
            shift
            ;;
        --all)
            HTML=true
            TERM=true
            XML=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--html] [--term] [--xml] [--all]"
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

# Check if coverage tools are installed
if ! python3 -c "import pytest_cov" 2>/dev/null; then
    echo "❌ Error: pytest-cov is not installed"
    echo "   Install with: pip install pytest-cov coverage"
    exit 1
fi

echo "📊 Generating Python coverage report..."
echo ""

# Build coverage report arguments
COV_ARGS="--cov=python/services --cov=python/tui --cov=python/integration"
REPORT_ARGS=""

if [[ "$TERM" == "true" ]]; then
    REPORT_ARGS="$REPORT_ARGS --cov-report=term"
fi

if [[ "$HTML" == "true" ]]; then
    REPORT_ARGS="$REPORT_ARGS --cov-report=html"
    echo "📁 HTML report will be generated in htmlcov/"
fi

if [[ "$XML" == "true" ]]; then
    REPORT_ARGS="$REPORT_ARGS --cov-report=xml"
    echo "📄 XML report will be generated in coverage.xml"
fi

# Run tests with coverage
pytest python/tests/ python/integration/ \
    $COV_ARGS \
    $REPORT_ARGS \
    -v

echo ""
echo "✅ Python coverage report generated!"

if [[ "$HTML" == "true" ]]; then
    echo "📁 Open htmlcov/index.html in your browser to view the HTML report"
fi

if [[ "$XML" == "true" ]]; then
    echo "📄 XML report available at coverage.xml"
fi
