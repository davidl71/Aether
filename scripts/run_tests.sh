#!/usr/bin/env bash
# run_tests.sh - Run ShellSpec tests for shell scripts
#
# AI CONTEXT:
# This script runs ShellSpec tests for shared shell script functions.
# It checks for ShellSpec installation and runs tests with appropriate options.
#
# Usage:
#   ./scripts/run_tests.sh [options]
#
# Options:
#   --format <format>    Output format (documentation, progress, tap, junit)
#   --quick              Run only failed tests
#   --parallel <jobs>   Run tests in parallel (default: 4)
#   --verbose           Verbose output
#   --coverage          Generate coverage report (requires kcov)
#   --help              Show this help message

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./include/workspace_paths.sh
. "${SCRIPT_DIR}/include/workspace_paths.sh"

setup_workspace_paths

# Default options
FORMAT="${FORMAT:-documentation}"
QUICK="${QUICK:-false}"
PARALLEL="${PARALLEL:-4}"
VERBOSE="${VERBOSE:-false}"
COVERAGE="${COVERAGE:-false}"
TIMEOUT="${TIMEOUT:-300}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --format)
      FORMAT="$2"
      shift 2
      ;;
    --quick)
      QUICK="true"
      shift
      ;;
    --parallel)
      PARALLEL="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE="true"
      shift
      ;;
    --coverage)
      COVERAGE="true"
      shift
      ;;
    --timeout)
      TIMEOUT="$2"
      shift 2
      ;;
    --help)
      cat <<EOF
Run ShellSpec tests for shell scripts

Usage: $0 [options]

Options:
  --format <format>    Output format (documentation, progress, tap, junit)
  --quick              Run only failed tests
  --parallel <jobs>    Run tests in parallel (default: 4)
  --verbose            Verbose output
  --coverage           Generate coverage report (requires kcov)
  --timeout <seconds>  Timeout for test execution (default: 300)
  --help               Show this help message

Examples:
  $0                          # Run all tests with documentation format
  $0 --format progress        # Run with progress format
  $0 --quick                  # Run only failed tests
  $0 --parallel 8             # Run with 8 parallel jobs
  $0 --coverage               # Generate coverage report
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Run '$0 --help' for usage information" >&2
      exit 1
      ;;
  esac
done

# Find ShellSpec
SHELLSPEC_CMD=""
if command -v shellspec >/dev/null 2>&1; then
  SHELLSPEC_CMD="shellspec"
elif [ -f "${PROJECT_ROOT}/.local/bin/shellspec" ]; then
  SHELLSPEC_CMD="${PROJECT_ROOT}/.local/bin/shellspec"
elif [ -f "${HOME:-}/.local/bin/shellspec" ]; then
  SHELLSPEC_CMD="${HOME}/.local/bin/shellspec"
elif [ -f "${PROJECT_ROOT}/bin/shellspec" ]; then
  SHELLSPEC_CMD="${PROJECT_ROOT}/bin/shellspec"
else
  echo "Error: ShellSpec not found" >&2
  echo "" >&2
  echo "Install ShellSpec:" >&2
  echo "" >&2
  echo "Option 1: Install globally (recommended):" >&2
  echo "  curl -fsSL https://git.io/shellspec | sh" >&2
  echo "  # Answer 'y' when prompted" >&2
  echo "" >&2
  echo "Option 2: Install locally in project:" >&2
  echo "  curl -fsSL https://git.io/shellspec | sh -s -- --yes" >&2
  echo "" >&2
  echo "After installation, add a local bin dir to PATH if needed:" >&2
  echo "  export PATH=\"${PROJECT_ROOT}/.local/bin:\${PATH}\"" >&2
  echo "" >&2
  echo "Then run tests again:" >&2
  echo "  ./scripts/run_tests.sh" >&2
  exit 1
fi

# Build ShellSpec command
SHELLSPEC_ARGS=()

# Timeout
SHELLSPEC_ARGS+=(--timeout "${TIMEOUT}")

# Format
SHELLSPEC_ARGS+=(--format "${FORMAT}")

# Quick mode
if [ "${QUICK}" = "true" ]; then
  SHELLSPEC_ARGS+=(--quick)
fi

# Parallel execution
if [ "${PARALLEL}" -gt 1 ]; then
  SHELLSPEC_ARGS+=(--jobs "${PARALLEL}")
fi

# Verbose
if [ "${VERBOSE}" = "true" ]; then
  SHELLSPEC_ARGS+=(--verbose)
fi

# Coverage
if [ "${COVERAGE}" = "true" ]; then
  if ! command -v kcov >/dev/null 2>&1; then
    echo "Warning: kcov not found, coverage disabled" >&2
    echo "Install kcov: brew install kcov (macOS) or apt-get install kcov (Linux)" >&2
  else
    SHELLSPEC_ARGS+=(--coverage)
  fi
fi

# Change to project root
cd "${PROJECT_ROOT}"

# Run tests
echo "Running ShellSpec tests..."
echo "Command: ${SHELLSPEC_CMD} ${SHELLSPEC_ARGS[*]}"
echo ""

"${SHELLSPEC_CMD}" "${SHELLSPEC_ARGS[@]}"

EXIT_CODE=$?

if [ ${EXIT_CODE} -eq 0 ]; then
  echo ""
  echo "✓ All tests passed!"
else
  echo ""
  echo "✗ Some tests failed (exit code: ${EXIT_CODE})"
  echo ""
  echo "Run with --quick to rerun only failed tests:"
  echo "  $0 --quick"
fi

exit ${EXIT_CODE}
