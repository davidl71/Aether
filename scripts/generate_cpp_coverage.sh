#!/bin/bash
# Generate C++ code coverage report
# Usage: ./scripts/generate_cpp_coverage.sh [--rebuild]

set -euo pipefail

cd "$(dirname "$0")/.."

REBUILD=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --rebuild)
            REBUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--rebuild]"
            exit 1
            ;;
    esac
done

# Check if lcov is installed
if ! command -v lcov &> /dev/null; then
    echo "❌ Error: lcov is not installed"
    echo "   Install with: brew install lcov"
    exit 1
fi

# Check if genhtml is available
if ! command -v genhtml &> /dev/null; then
    echo "❌ Error: genhtml is not installed (part of lcov)"
    echo "   Install with: brew install lcov"
    exit 1
fi

echo "📊 Generating C++ coverage report..."
echo ""

# Navigate to native directory
cd native

# Create coverage build directory
COVERAGE_BUILD_DIR="build-coverage"
if [[ "$REBUILD" == "true" ]] || [[ ! -d "$COVERAGE_BUILD_DIR" ]]; then
    echo "🔨 Building with coverage enabled..."
    rm -rf "$COVERAGE_BUILD_DIR"
    mkdir -p "$COVERAGE_BUILD_DIR"
    cd "$COVERAGE_BUILD_DIR"

    # Configure with coverage flags
    cmake .. \
        -DCMAKE_BUILD_TYPE=Debug \
        -DCMAKE_CXX_FLAGS="--coverage -g -O0" \
        -DCMAKE_EXE_LINKER_FLAGS="--coverage" \
        -DBUILD_TESTING=ON

    # Build
    cmake --build .
else
    echo "📦 Using existing coverage build..."
    cd "$COVERAGE_BUILD_DIR"
fi

# Run tests to generate coverage data
echo ""
echo "🧪 Running tests to generate coverage data..."
ctest --output-on-failure || {
    echo "⚠️  Warning: Some tests failed, but continuing with coverage generation..."
}

# Capture coverage data
echo ""
echo "📊 Capturing coverage data..."
lcov --capture --directory . --output-file coverage.info

# Filter out system and third-party code
echo "🔍 Filtering coverage data..."
lcov --remove coverage.info \
    '/usr/*' \
    '/opt/*' \
    '*/third_party/*' \
    '*/build/*' \
    '*/tests/*' \
    --output-file coverage_filtered.info

# Generate HTML report
echo "📁 Generating HTML coverage report..."
genhtml coverage_filtered.info --output-directory coverage_html

# Display summary
echo ""
echo "✅ C++ coverage report generated!"
echo "📁 HTML report available at: native/$COVERAGE_BUILD_DIR/coverage_html/index.html"
echo ""
echo "📊 Coverage summary:"
lcov --summary coverage_filtered.info | tail -5
