#!/bin/bash
# build_universal.sh - Universal binary builder for IBKR Box Spread Generator
# Usage: ./build_universal.sh [clean|test|install]

set -euo pipefail
IFS=$'\n\t'

# ============================================================================
# Configuration
# ============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

BUILD_DIR="${BUILD_DIR:-${PROJECT_ROOT}/build}"
BUILD_TYPE="${BUILD_TYPE:-Release}"
CMAKE_GENERATOR="${CMAKE_GENERATOR:-Unix Makefiles}"
PARALLEL_JOBS="${PARALLEL_JOBS:-$(sysctl -n hw.ncpu 2>/dev/null || echo 4)}"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # No Color

# ============================================================================
# Functions
# ============================================================================
log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

cleanup() {
    local exit_code=$?
    if [[ ${exit_code} -ne 0 ]]; then
        log_error "Build failed with exit code ${exit_code}"
    fi
    exit "${exit_code}"
}

check_dependencies() {
    local missing_deps=()

    for cmd in cmake; do
        if ! command -v "${cmd}" &> /dev/null; then
            missing_deps+=("${cmd}")
        fi
    done

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Install with: brew install ${missing_deps[*]}"
        return 1
    fi

    # Check macOS version for universal binary support
    local macos_version
    macos_version=$(sw_vers -productVersion 2>/dev/null | cut -d. -f1)
    if [[ -n "${macos_version}" ]] && [[ ${macos_version} -lt 11 ]]; then
        log_warn "Universal binaries require macOS 11.0 or later"
    fi
}

validate_environment() {
    if [[ ! -f "${PROJECT_ROOT}/CMakeLists.txt" ]]; then
        log_error "CMakeLists.txt not found in ${PROJECT_ROOT}"
        return 1
    fi

    # Sanitize build directory path
    if [[ "${BUILD_DIR}" =~ [^a-zA-Z0-9/_.-] ]]; then
        log_error "Invalid build directory path: ${BUILD_DIR}"
        return 1
    fi
}

clean_build() {
    log_info "Cleaning build directory: ${BUILD_DIR}"
    rm -rf "${BUILD_DIR}"
}

configure_cmake() {
    log_info "Configuring CMake project..."

    cmake -S "${PROJECT_ROOT}" \
          -B "${BUILD_DIR}" \
          -G "${CMAKE_GENERATOR}" \
          -DCMAKE_BUILD_TYPE="${BUILD_TYPE}" \
          -DCMAKE_OSX_ARCHITECTURES="x86_64;arm64" \
          -DCMAKE_OSX_DEPLOYMENT_TARGET="11.0" \
          -DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
          -DCMAKE_COLOR_DIAGNOSTICS=ON
}

build_project() {
    log_info "Building project (${BUILD_TYPE})..."

    cmake --build "${BUILD_DIR}" \
          --config "${BUILD_TYPE}" \
          --parallel "${PARALLEL_JOBS}"
}

verify_binary() {
    local binary="${BUILD_DIR}/bin/ib_box_spread"

    if [[ ! -f "${binary}" ]]; then
        log_error "Binary not found: ${binary}"
        return 1
    fi

    log_info "Verifying universal binary..."

    # Check architectures
    local archs
    archs=$(lipo -archs "${binary}" 2>/dev/null || echo "")

    if [[ "${archs}" == *"x86_64"* ]] && [[ "${archs}" == *"arm64"* ]]; then
        log_info "Binary verified: ${archs}"
    else
        log_warn "Binary may not be universal. Architectures: ${archs:-unknown}"
    fi

    # Show dependencies
    log_info "Library dependencies:"
    otool -L "${binary}" 2>/dev/null || true
}

run_tests() {
    if [[ -d "${BUILD_DIR}" ]]; then
        log_info "Running tests..."
        cd "${BUILD_DIR}"
        ctest --output-on-failure
        cd - > /dev/null
    else
        log_warn "Build directory not found. Run build first."
        return 1
    fi
}

install_binary() {
    local install_dir="${HOME}/.local/bin"
    mkdir -p "${install_dir}"

    log_info "Installing to ${install_dir}"
    cp "${BUILD_DIR}/bin/ib_box_spread" "${install_dir}/"
    chmod +x "${install_dir}/ib_box_spread"

    log_info "Installation complete. Add ${install_dir} to PATH if needed."
}

show_usage() {
    cat << EOF
Usage: $0 [COMMAND]

Commands:
    (default)   Configure and build the project
    clean       Clean build directory
    test        Run tests
    install     Install binary to ~/.local/bin

Environment Variables:
    BUILD_DIR       Build directory (default: ./build)
    BUILD_TYPE      Build type: Release|Debug (default: Release)
    CMAKE_GENERATOR CMake generator (default: Unix Makefiles)
    PARALLEL_JOBS   Number of parallel jobs (default: CPU count)

Example:
    BUILD_TYPE=Debug $0
    $0 clean
    $0 test

EOF
}

# ============================================================================
# Main
# ============================================================================
trap cleanup EXIT

main() {
    local command="${1:-build}"

    if [[ "${command}" == "-h" ]] || [[ "${command}" == "--help" ]]; then
        show_usage
        exit 0
    fi

    log_info "IBKR Box Spread Generator - Universal Binary Builder"
    log_info "Project: ${PROJECT_ROOT}"
    log_info "Build Type: ${BUILD_TYPE}"

    check_dependencies || exit 1
    validate_environment || exit 1

    case "${command}" in
        clean)
            clean_build
            ;;
        test)
            run_tests
            ;;
        install)
            verify_binary && install_binary
            ;;
        build|*)
            configure_cmake
            build_project
            verify_binary
            log_info "Build successful!"
            log_info "Binary location: ${BUILD_DIR}/bin/ib_box_spread"
            ;;
    esac
}

main "$@"
