#!/usr/bin/env bash
# setup_platform_settings.sh - Auto-detect and configure platform-specific settings
# Usage: ./scripts/setup_platform_settings.sh [--force] [--cmake-configure]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SETTINGS_USER="${PROJECT_ROOT}/.vscode/settings.json.user"
SETTINGS_EXAMPLE="${PROJECT_ROOT}/.vscode/settings.json.user.example"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Detect platform
detect_platform() {
  local os_name
  local arch

  os_name=$(uname -s 2>/dev/null || echo "Unknown")
  arch=$(uname -m 2>/dev/null || echo "Unknown")

  case "$os_name" in
    Darwin)
      if [ "$arch" = "arm64" ] || [ "$arch" = "aarch64" ]; then
        echo "macos-arm64"
      elif [ "$arch" = "x86_64" ]; then
        echo "macos-x86_64"
      else
        echo "macos-unknown"
      fi
      ;;
    Linux)
      if [ "$arch" = "x86_64" ] || [ "$arch" = "amd64" ]; then
        echo "linux-x64"
      else
        echo "linux-unknown"
      fi
      ;;
    MINGW*|MSYS*|CYGWIN*)
      echo "windows-x64"
      ;;
    *)
      log_error "Unknown platform: $os_name ($arch)"
      exit 1
      ;;
  esac
}

# Detect C++ compiler
detect_compiler() {
  local platform=$1
  local compiler_path=""

  case "$platform" in
    macos-*)
      # Try Apple Silicon Homebrew first, then Intel Homebrew, then system
      if [ -f "/opt/homebrew/bin/clang++" ]; then
        compiler_path="/opt/homebrew/bin/clang++"
      elif [ -f "/usr/local/bin/clang++" ]; then
        compiler_path="/usr/local/bin/clang++"
      elif command -v clang++ >/dev/null 2>&1; then
        compiler_path=$(command -v clang++)
      fi
      ;;
    linux-*)
      if command -v g++ >/dev/null 2>&1; then
        compiler_path=$(command -v g++)
      fi
      ;;
    windows-*)
      # Try to find MSVC (this is tricky on Windows, may need manual config)
      if command -v cl.exe >/dev/null 2>&1; then
        compiler_path=$(command -v cl.exe)
      elif [ -f "/c/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC" ]; then
        # Find latest MSVC version
        local msvc_base="/c/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC"
        local latest_version
        latest_version=$(ls -1 "$msvc_base" 2>/dev/null | sort -V | tail -1)
        if [ -n "$latest_version" ]; then
          compiler_path="$msvc_base/$latest_version/bin/Hostx64/x64/cl.exe"
        fi
      fi
      ;;
  esac

  echo "$compiler_path"
}

# Detect IntelliSense mode
detect_intellisense_mode() {
  local platform=$1
  local compiler_path=$2

  case "$platform" in
    macos-arm64)
      echo "macos-clang-arm64"
      ;;
    macos-x86_64)
      echo "macos-clang-x64"
      ;;
    linux-*)
      if echo "$compiler_path" | grep -q "g++"; then
        echo "linux-gcc-x64"
      else
        echo "linux-clang-x64"
      fi
      ;;
    windows-*)
      if echo "$compiler_path" | grep -q "cl.exe"; then
        echo "windows-msvc-x64"
      else
        echo "windows-gcc-x64"
      fi
      ;;
    *)
      echo "unknown"
      ;;
  esac
}

# Detect include paths
detect_include_paths() {
  local platform=$1
  local paths=()

  # Workspace-relative paths (always included)
  paths+=("\"\${workspaceFolder}/native/include\"")
  paths+=("\"\${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client\"")

  case "$platform" in
    macos-arm64)
      [ -d "/opt/homebrew/include" ] && paths+=("\"/opt/homebrew/include\"")
      [ -d "/usr/local/include" ] && paths+=("\"/usr/local/include\"")
      ;;
    macos-x86_64)
      [ -d "/usr/local/include" ] && paths+=("\"/usr/local/include\"")
      ;;
    linux-*)
      [ -d "/usr/include" ] && paths+=("\"/usr/include\"")
      [ -d "/usr/local/include" ] && paths+=("\"/usr/local/include\"")
      # Try to find GCC C++ headers
      if [ -d "/usr/include/c++" ]; then
        local gcc_version
        gcc_version=$(ls -1 /usr/include/c++ 2>/dev/null | sort -V | tail -1)
        if [ -n "$gcc_version" ]; then
          paths+=("\"/usr/include/c++/$gcc_version\"")
        fi
      fi
      ;;
    windows-*)
      # Windows paths are complex, user may need to configure manually
      log_warn "Windows include paths may need manual configuration"
      ;;
  esac

  # Format paths for JSON (one per line with proper indentation)
  local formatted_paths=""
  local first=true
  for path in "${paths[@]}"; do
    if [ "$first" = true ]; then
      formatted_paths="    $path"
      first=false
    else
      formatted_paths="$formatted_paths,\n    $path"
    fi
  done
  echo -e "$formatted_paths"
}

# Detect Python interpreter
detect_python() {
  local platform=$1
  local python_path=""

  case "$platform" in
    macos-arm64)
      if [ -f "/opt/homebrew/bin/python3" ]; then
        python_path="/opt/homebrew/bin/python3"
      elif command -v python3 >/dev/null 2>&1; then
        python_path=$(command -v python3)
      fi
      ;;
    macos-x86_64)
      if [ -f "/usr/local/bin/python3" ]; then
        python_path="/usr/local/bin/python3"
      elif command -v python3 >/dev/null 2>&1; then
        python_path=$(command -v python3)
      fi
      ;;
    linux-*|windows-*)
      if command -v python3 >/dev/null 2>&1; then
        python_path=$(command -v python3)
      fi
      ;;
  esac

  echo "$python_path"
}

# Generate settings.json.user
generate_settings_user() {
  local platform=$1
  local intellisense_mode=$2
  local compiler_path=$3
  local python_path=$4
  local include_paths=$5

  log_info "Generating .vscode/settings.json.user for platform: $platform"

  cat > "$SETTINGS_USER" <<EOF
{
  // ==========================================
  // Platform-Specific Settings (Auto-Generated)
  // ==========================================
  // Generated: $(date -Iseconds)
  // Platform: $platform
  // ==========================================

  // C++ IntelliSense Configuration
  "C_Cpp.default.intelliSenseMode": "$intellisense_mode",
  "C_Cpp.default.compilerPath": "$compiler_path",
  "C_Cpp.default.includePath": [
    $include_paths
  ],

  // Python Configuration
EOF

  if [ -n "$python_path" ]; then
    cat >> "$SETTINGS_USER" <<EOF
  "python.defaultInterpreterPath": "$python_path",
EOF
  else
    cat >> "$SETTINGS_USER" <<EOF
  // Python interpreter not detected - using PATH
  // "python.defaultInterpreterPath": "",
EOF
  fi

  cat >> "$SETTINGS_USER" <<EOF

  // ==========================================
  // Personal Preferences (Add your own below)
  // ==========================================
  // See ${SETTINGS_EXAMPLE} for examples
}
EOF

  log_success "Generated $SETTINGS_USER"
}

# Configure CMake preset
configure_cmake() {
  local platform=$1
  local preset=""

  case "$platform" in
    macos-arm64)
      preset="macos-arm64-debug"
      ;;
    macos-x86_64)
      preset="macos-x86_64-debug"
      ;;
    linux-*)
      preset="linux-x64-debug"
      ;;
    windows-*)
      preset="windows-x64-debug"
      ;;
    *)
      log_error "Unknown platform for CMake preset: $platform"
      return 1
      ;;
  esac

  log_info "Configuring CMake with preset: $preset"

  cd "$PROJECT_ROOT"
  if cmake --preset "$preset" >/dev/null 2>&1; then
    log_success "CMake configured successfully"
    log_info "compile_commands.json generated - VS Code will auto-detect settings"
    return 0
  else
    log_warn "CMake configuration failed - you may need to install dependencies"
    log_info "Run manually: cmake --preset $preset"
    return 1
  fi
}

# Main function
main() {
  local force=false
  local cmake_configure=false

  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case $1 in
      --force)
        force=true
        shift
        ;;
      --cmake-configure)
        cmake_configure=true
        shift
        ;;
      -h|--help)
        cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --force              Overwrite existing settings.json.user
  --cmake-configure    Also configure CMake preset
  -h, --help           Show this help message

This script:
  1. Detects your platform (macOS ARM64/x86_64, Windows, Linux)
  2. Detects compiler, include paths, and Python interpreter
  3. Generates .vscode/settings.json.user with platform-specific settings
  4. Optionally configures CMake preset

EOF
        exit 0
        ;;
      *)
        log_error "Unknown option: $1"
        exit 1
        ;;
    esac
  done

  # Check if settings.json.user already exists
  if [ -f "$SETTINGS_USER" ] && [ "$force" != "true" ]; then
    log_warn "$SETTINGS_USER already exists"
    log_info "Use --force to overwrite, or edit manually"
    exit 0
  fi

  # Detect platform
  log_info "Detecting platform..."
  local platform
  platform=$(detect_platform)
  log_success "Detected platform: $platform"

  # Detect compiler
  log_info "Detecting C++ compiler..."
  local compiler_path
  compiler_path=$(detect_compiler "$platform")
  if [ -z "$compiler_path" ]; then
    log_error "No C++ compiler detected!"
    log_info "Please install:"
    case "$platform" in
      macos-*)
        log_info "  macOS: Xcode Command Line Tools (xcode-select --install)"
        ;;
      linux-*)
        log_info "  Linux: sudo apt-get install build-essential"
        ;;
      windows-*)
        log_info "  Windows: Install Visual Studio or MinGW-w64"
        ;;
    esac
    exit 1
  fi
  log_success "Found compiler: $compiler_path"

  # Detect IntelliSense mode
  local intellisense_mode
  intellisense_mode=$(detect_intellisense_mode "$platform" "$compiler_path")
  log_info "IntelliSense mode: $intellisense_mode"

  # Detect include paths
  log_info "Detecting include paths..."
  local include_paths
  include_paths=$(detect_include_paths "$platform")
  log_success "Found include paths"

  # Detect Python
  log_info "Detecting Python interpreter..."
  local python_path
  python_path=$(detect_python "$platform")
  if [ -n "$python_path" ]; then
    log_success "Found Python: $python_path"
  else
    log_warn "Python not detected - will use PATH"
  fi

  # Generate settings.json.user
  generate_settings_user "$platform" "$intellisense_mode" "$compiler_path" "$python_path" "$include_paths"

  # Configure CMake if requested
  if [ "$cmake_configure" = "true" ]; then
    configure_cmake "$platform"
  else
    log_info "To configure CMake, run: $0 --cmake-configure"
  fi

  # Summary
  echo ""
  log_success "Platform settings configured!"
  echo ""
  log_info "Next steps:"
  echo "  1. Review: $SETTINGS_USER"
  echo "  2. Configure CMake: cmake --preset <platform-preset>"
  echo "  3. Reload VS Code: Cmd+Shift+P → 'Developer: Reload Window'"
  echo ""
  log_info "Platform-specific preset:"
  case "$platform" in
    macos-arm64)
      echo "  cmake --preset macos-arm64-debug"
      ;;
    macos-x86_64)
      echo "  cmake --preset macos-x86_64-debug"
      ;;
    linux-*)
      echo "  cmake --preset linux-x64-debug"
      ;;
    windows-*)
      echo "  cmake --preset windows-x64-debug"
      ;;
  esac
}

# Run main function
main "$@"
