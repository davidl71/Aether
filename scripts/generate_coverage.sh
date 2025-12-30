#!/bin/bash
# Generate combined C++ and Python code coverage reports
# Usage: ./scripts/generate_coverage.sh [--cpp] [--python] [--html] [--rebuild]

set -euo pipefail

cd "$(dirname "$0")/.."

RUN_CPP=true
RUN_PYTHON=true
HTML=false
REBUILD=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cpp-only)
            RUN_CPP=true
            RUN_PYTHON=false
            shift
            ;;
        --python-only)
            RUN_CPP=false
            RUN_PYTHON=true
            shift
            ;;
        --html)
            HTML=true
            shift
            ;;
        --rebuild)
            REBUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--cpp-only] [--python-only] [--html] [--rebuild]"
            exit 1
            ;;
    esac
done

echo "📊 Generating Coverage Reports"
echo "================================"
echo ""

TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
echo "Started: $TIMESTAMP"
echo ""

# Generate Python coverage
if [[ "$RUN_PYTHON" == "true" ]]; then
    echo "🐍 Python Coverage"
    echo "-----------------"
    if [[ "$HTML" == "true" ]]; then
        ./scripts/generate_python_coverage.sh --html --term
    else
        ./scripts/generate_python_coverage.sh --term
    fi
    echo ""
fi

# Generate C++ coverage
if [[ "$RUN_CPP" == "true" ]]; then
    echo "⚙️  C++ Coverage"
    echo "----------------"
    if [[ "$REBUILD" == "true" ]]; then
        ./scripts/generate_cpp_coverage.sh --rebuild
    else
        ./scripts/generate_cpp_coverage.sh
    fi
    echo ""
fi

# Summary
echo "================================"
echo "✅ Coverage Reports Generated"
echo ""

if [[ "$RUN_PYTHON" == "true" ]]; then
    if [[ "$HTML" == "true" ]]; then
        echo "🐍 Python: htmlcov/index.html"
    fi
    echo "🐍 Python: Terminal report above"
fi

if [[ "$RUN_CPP" == "true" ]]; then
    echo "⚙️  C++: native/build-coverage/coverage_html/index.html"
fi

echo ""
echo "📊 Next Steps:"
echo "   - Review coverage reports"
echo "   - Identify uncovered code paths"
echo "   - Add tests for critical uncovered areas"
echo "   - Target: 30%+ overall coverage"
