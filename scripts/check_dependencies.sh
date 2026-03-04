#!/usr/bin/env bash
# Check all dependencies required for the platform
# Usage: ./scripts/check_dependencies.sh [--install]

set -euo pipefail

INSTALL_MODE=false
if [ "${1:-}" = "--install" ]; then
  INSTALL_MODE=true
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Platform Dependency Check ===${NC}"
echo ""

# Track missing dependencies
MISSING_DEPS=()
OPTIONAL_DEPS=()

# Check function
check_dep() {
  local name=$1
  local install_cmd=${2:-}
  local optional=${3:-false}

  printf "%-20s " "$name:"
  if command -v "$name" &>/dev/null; then
    local version=$($name --version 2>&1 | head -1 | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' || echo "installed")
    echo -e "${GREEN}✓${NC} $version"
    return 0
  else
    if [ "$optional" = "true" ]; then
      echo -e "${YELLOW}⚠ Missing (optional)${NC}"
      OPTIONAL_DEPS+=("$name:$install_cmd")
    else
      echo -e "${RED}✗ Missing (required)${NC}"
      MISSING_DEPS+=("$name:$install_cmd")
    fi
    return 1
  fi
}

# Core system tools
echo -e "${BLUE}Core System Tools:${NC}"
check_dep "bash"
check_dep "git"
check_dep "cmake"
check_dep "ninja" "brew install ninja"
check_dep "make"
echo ""

# Compilers
echo -e "${BLUE}Compilers:${NC}"
check_dep "gcc" "brew install gcc" true
check_dep "clang"
check_dep "g++" "brew install gcc" true
echo ""

# Build tools
echo -e "${BLUE}Build & Cache Tools:${NC}"
check_dep "sccache" "brew install sccache" true
check_dep "ccache" "brew install ccache" true
check_dep "distcc" "brew install distcc" true
echo ""

# Python ecosystem
echo -e "${BLUE}Python Ecosystem:${NC}"
check_dep "python3"
check_dep "pip3"
check_dep "uv" "brew install uv" true
echo ""

# Check Python packages
if command -v python3 &>/dev/null; then
  echo -e "${BLUE}Python Packages:${NC}"
  for pkg in uvicorn fastapi textual numpy pandas; do
    printf "%-20s " "$pkg:"
    if python3 -c "import $pkg" 2>/dev/null; then
      version=$(python3 -c "import $pkg; print($pkg.__version__)" 2>/dev/null || echo "installed")
      echo -e "${GREEN}✓${NC} $version"
    else
      echo -e "${YELLOW}⚠ Missing${NC}"
      OPTIONAL_DEPS+=("python:$pkg:pip3 install $pkg")
    fi
  done
  echo ""
fi

# Node.js ecosystem
echo -e "${BLUE}Node.js Ecosystem:${NC}"
check_dep "node" "brew install node"
check_dep "npm" "brew install node"
check_dep "npx" "brew install node"
echo ""

# Rust ecosystem
echo -e "${BLUE}Rust Ecosystem:${NC}"
check_dep "rustc" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_dep "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo ""

# Service dependencies
echo -e "${BLUE}Service Dependencies:${NC}"
check_dep "lsof" # macOS built-in
check_dep "jq" "brew install jq" true
check_dep "nats-server" "brew tap nats-io/nats-tools && brew install nats-server" true
echo ""

# TWS/IB Gateway
echo -e "${BLUE}Trading Platforms:${NC}"
printf "%-20s " "TWS/IB Gateway:"
if [ -d "/Applications/Trader Workstation.app" ] || [ -d "/Applications/IB Gateway.app" ]; then
  echo -e "${GREEN}✓${NC} Installed"
else
  echo -e "${YELLOW}⚠ Not found${NC} (Download from https://www.interactivebrokers.com)"
fi
echo ""

# Optional tools
echo -e "${BLUE}Optional Tools:${NC}"
check_dep "ansible" "brew install ansible" true
check_dep "docker" "brew install --cask docker" true
check_dep "clang-tidy" "brew install llvm" true
check_dep "clang-format" "brew install llvm" true
check_dep "markdownlint" "npm install -g markdownlint-cli" true
echo ""

# Summary
echo ""
echo -e "${BLUE}=== Summary ===${NC}"
echo ""

if [ ${#MISSING_DEPS[@]} -eq 0 ] && [ ${#OPTIONAL_DEPS[@]} -eq 0 ]; then
  echo -e "${GREEN}✓ All dependencies satisfied!${NC}"
  exit 0
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
  echo -e "${RED}Missing Required Dependencies (${#MISSING_DEPS[@]}):${NC}"
  for dep_info in "${MISSING_DEPS[@]}"; do
    dep=$(echo "$dep_info" | cut -d: -f1)
    cmd=$(echo "$dep_info" | cut -d: -f2-)
    echo "  - $dep"
    if [ -n "$cmd" ]; then
      echo "    Install: $cmd"
    fi
  done
  echo ""
fi

if [ ${#OPTIONAL_DEPS[@]} -gt 0 ]; then
  echo -e "${YELLOW}Missing Optional Dependencies (${#OPTIONAL_DEPS[@]}):${NC}"
  for dep_info in "${OPTIONAL_DEPS[@]}"; do
    dep=$(echo "$dep_info" | cut -d: -f1)
    cmd=$(echo "$dep_info" | cut -d: -f2-)
    echo "  - $dep"
    if [ -n "$cmd" ]; then
      echo "    Install: $cmd"
    fi
  done
  echo ""
fi

# Install mode
if [ "$INSTALL_MODE" = true ]; then
  echo -e "${BLUE}=== Installation Mode ===${NC}"
  echo ""

  if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "Installing required dependencies..."
    for dep_info in "${MISSING_DEPS[@]}"; do
      cmd=$(echo "$dep_info" | cut -d: -f2-)
      if [ -n "$cmd" ] && [ "$cmd" != "$dep_info" ]; then
        echo "Running: $cmd"
        eval "$cmd" || echo "Failed to install, please install manually"
      fi
    done
  fi

  echo ""
  echo "Optional dependencies are not installed automatically."
  echo "Run individual install commands above if needed."
fi

# Exit with appropriate code
if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
  exit 1
else
  exit 0
fi
