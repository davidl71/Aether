#!/usr/bin/env bash
# setup_worktree.sh - Setup a new git worktree with all TWS API dependencies built
# Usage: ./scripts/setup_worktree.sh [worktree-name] [branch]
#
# This script:
# 1. Creates a new git worktree
# 2. Builds Intel Decimal library (libbid.a)
# 3. Builds TWS API library (libtwsapi.dylib)
# 4. Configures and builds the main project
#
# Prerequisites:
# - Git repository with main branch checked out
# - CMake 3.21+
# - C++ compiler (clang/g++)
# - Protocol Buffers installed (brew install protobuf)
# - Abseil libraries installed (brew install abseil)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
source "${SCRIPT_DIR}/include/logging.sh"
# shellcheck source=./include/build_logging.sh
source "${SCRIPT_DIR}/include/build_logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKTREE_NAME="${1:-worktree-$(date +%Y%m%d-%H%M%S)}"
BRANCH="${2:-main}"
WORKTREE_DIR="${PROJECT_ROOT}/../${WORKTREE_NAME}"

log_info "=========================================="
log_info "  Git Worktree Setup Script"
log_info "=========================================="
log_info "Worktree name: ${WORKTREE_NAME}"
log_info "Branch: ${BRANCH}"
log_info "Worktree directory: ${WORKTREE_DIR}"
log_info ""

# Check prerequisites
check_prerequisites() {
  log_info "Checking prerequisites..."
  
  local missing_deps=()
  
  if ! command -v git >/dev/null 2>&1; then
    missing_deps+=("git")
  fi
  
  if ! command -v cmake >/dev/null 2>&1; then
    missing_deps+=("cmake")
  fi
  
  if ! command -v make >/dev/null 2>&1 && ! command -v ninja >/dev/null 2>&1; then
    missing_deps+=("make or ninja")
  fi
  
  if [[ ! -f "/usr/local/lib/libprotobuf.dylib" ]] && [[ ! -f "/opt/homebrew/lib/libprotobuf.dylib" ]]; then
    missing_deps+=("protobuf (brew install protobuf)")
  fi
  
  if [[ ${#missing_deps[@]} -gt 0 ]]; then
    log_error "Missing prerequisites:"
    for dep in "${missing_deps[@]}"; do
      log_error "  - ${dep}"
    done
    exit 1
  fi
  
  log_info "All prerequisites met ✓"
}

# Create git worktree
create_worktree() {
  log_info "Creating git worktree..."
  
  if [[ -d "${WORKTREE_DIR}" ]]; then
    log_error "Worktree directory already exists: ${WORKTREE_DIR}"
    log_error "Please remove it first or choose a different name"
    exit 1
  fi
  
  cd "${PROJECT_ROOT}"
  
  # Check if we're in a git repository
  if ! git rev-parse --git-dir >/dev/null 2>&1; then
    log_error "Not in a git repository. Please run this from the repository root."
    exit 1
  fi
  
  # Create worktree
  log_info "Creating worktree at ${WORKTREE_DIR}..."
  git worktree add "${WORKTREE_DIR}" "${BRANCH}" || {
    log_error "Failed to create worktree"
    exit 1
  }
  
  log_info "Worktree created successfully ✓"
}

# Build Intel Decimal library
build_intel_decimal() {
  log_info "Building Intel Decimal library (libbid.a)..."
  
  local intel_dir="${WORKTREE_DIR}/native/third_party/IntelRDFPMathLib20U2/LIBRARY"
  local build_dir="${intel_dir}/build"
  local lib_path="${intel_dir}/libbid.a"
  
  if [[ -f "${lib_path}" ]]; then
    log_note "Intel Decimal library already exists, skipping build"
    return 0
  fi
  
  if [[ ! -d "${intel_dir}" ]]; then
    log_error "Intel Decimal library source not found at ${intel_dir}"
    log_error "Please ensure the library is extracted to native/third_party/IntelRDFPMathLib20U2/"
    exit 1
  fi
  
  cd "${intel_dir}"
  mkdir -p "${build_dir}"
  
  log_info "Configuring Intel Decimal library build..."
  cmake -S . -B "${build_dir}" -DCMAKE_BUILD_TYPE=Release >/dev/null 2>&1 || {
    log_error "Failed to configure Intel Decimal library"
    exit 1
  }
  
  log_info "Building Intel Decimal library..."
  cmake --build "${build_dir}" >/dev/null 2>&1 || {
    log_error "Failed to build Intel Decimal library"
    exit 1
  }
  
  if [[ ! -f "${lib_path}" ]]; then
    log_error "Intel Decimal library not found after build: ${lib_path}"
    exit 1
  fi
  
  log_info "Intel Decimal library built: ${lib_path} ✓"
}

# Build TWS API library
build_tws_api() {
  log_info "Building TWS API library (libtwsapi.dylib)..."
  
  local tws_client_dir="${WORKTREE_DIR}/native/third_party/tws-api/IBJts/source/cppclient/client"
  local build_dir="${tws_client_dir}/build"
  local lib_path="${build_dir}/lib/libtwsapi.dylib"
  local lib_debug_path="${build_dir}/lib/libtwsapid.dylib"
  
  if [[ -f "${lib_path}" ]] || [[ -f "${lib_debug_path}" ]]; then
    log_note "TWS API library already exists, skipping build"
    # Create symlink if needed
    if [[ -f "${lib_debug_path}" ]] && [[ ! -f "${lib_path}" ]]; then
      cd "${build_dir}/lib"
      ln -sf libtwsapid.dylib libtwsapi.dylib
    fi
    return 0
  fi
  
  if [[ ! -d "${tws_client_dir}" ]]; then
    log_error "TWS API source not found at ${tws_client_dir}"
    log_error "Please ensure TWS API is extracted to native/third_party/tws-api/"
    exit 1
  fi
  
  cd "${tws_client_dir}"
  mkdir -p "${build_dir}"
  
  log_info "Configuring TWS API build..."
  cmake -S . -B "${build_dir}" -DCMAKE_BUILD_TYPE=Debug >/dev/null 2>&1 || {
    log_error "Failed to configure TWS API"
    exit 1
  }
  
  log_info "Building TWS API library..."
  cmake --build "${build_dir}" 2>&1 | grep -E "(Building|Linking|Built|Error)" || true
  
  # Check for debug version and create symlink
  if [[ -f "${lib_debug_path}" ]] && [[ ! -f "${lib_path}" ]]; then
    cd "${build_dir}/lib"
    ln -sf libtwsapid.dylib libtwsapi.dylib
    log_note "Created symlink: libtwsapi.dylib -> libtwsapid.dylib"
  fi
  
  if [[ ! -f "${lib_path}" ]] && [[ ! -f "${lib_debug_path}" ]]; then
    log_error "TWS API library not found after build"
    exit 1
  fi
  
  log_info "TWS API library built: ${lib_path} ✓"
}

# Configure and build main project
build_main_project() {
  log_info "Configuring main project..."
  
  local build_dir="${WORKTREE_DIR}/build"
  cd "${WORKTREE_DIR}"
  
  mkdir -p "${build_dir}"
  
  log_info "Running CMake configuration..."
  cmake -S native -B "${build_dir}" \
    -DCMAKE_BUILD_TYPE=Debug \
    -DENABLE_NATIVE_CLI=ON \
    -DBUILD_TESTING=ON \
    2>&1 | grep -E "(Using TWS|Resolved|Native CLI|Missing|Error)" || true
  
  log_info "Building main project..."
  cmake --build "${build_dir}" --target ib_box_spread 2>&1 | tail -5
  
  local binary="${build_dir}/bin/ib_box_spread"
  if [[ -f "${binary}" ]]; then
    log_info "Main project built successfully ✓"
    log_info "Binary location: ${binary}"
  else
    log_warning "Main project binary not found, but build completed"
  fi
}

# Print summary
print_summary() {
  log_info ""
  log_info "=========================================="
  log_info "  Setup Complete!"
  log_info "=========================================="
  log_info ""
  log_info "Worktree: ${WORKTREE_DIR}"
  log_info "Branch: ${BRANCH}"
  log_info ""
  log_info "Next steps:"
  log_info "  1. cd ${WORKTREE_DIR}"
  log_info "  2. ./build/bin/ib_box_spread --help"
  log_info "  3. cp config/config.example.json config/config.json"
  log_info "  4. Edit config/config.json with your settings"
  log_info ""
  log_info "To remove this worktree:"
  log_info "  git worktree remove ${WORKTREE_DIR}"
  log_info ""
}

# Main execution
main() {
  check_prerequisites
  create_worktree
  build_intel_decimal
  build_tws_api
  build_main_project
  print_summary
}

# Run main function
main "$@"

