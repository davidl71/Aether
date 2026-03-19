#!/usr/bin/env bash
# Generate shell completion scripts for the Rust CLI (aether/cli).
# Usage: ./scripts/generate_completions.sh [bash|zsh|fish|all]
#
# Completions are generated from the Rust binary (agents/backend/bin/cli).
# Set CLI_BINARY=legacy to use the legacy C++ binary from build/bin/ib_box_spread if present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/agents/backend"
COMPLETIONS_DIR="$PROJECT_ROOT/completions"
CLI_BINARY="${CLI_BINARY:-rust}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
SHELLS="${1:-all}"

# Ensure completions directory exists
mkdir -p "$COMPLETIONS_DIR"

# Build Rust CLI and return path to binary (or empty on failure)
ensure_rust_cli() {
  if [ ! -f "$BACKEND_DIR/target/debug/cli" ]; then
    echo -e "${YELLOW}Rust CLI not found. Building...${NC}" >&2
    (cd "$BACKEND_DIR" && cargo build -p cli) >&2 || return 1
  fi
  echo "$BACKEND_DIR/target/debug/cli"
}

# Function to generate bash completion (Rust CLI)
generate_bash_completion() {
  local binary
  binary="$(ensure_rust_cli)" || {
    echo -e "${RED}Error: Could not build Rust CLI${NC}" >&2
    return 1
  }

  echo -e "${GREEN}Generating bash completion (aether)...${NC}"
  "$binary" --generate-completion bash >"$COMPLETIONS_DIR/aether.bash"
  chmod +x "$COMPLETIONS_DIR/aether.bash"
  echo -e "${GREEN}✓ Bash completion generated: $COMPLETIONS_DIR/aether.bash${NC}"
}

# Function to generate zsh completion (Rust CLI)
generate_zsh_completion() {
  local binary
  binary="$(ensure_rust_cli)" || {
    echo -e "${RED}Error: Could not build Rust CLI${NC}" >&2
    return 1
  }

  echo -e "${GREEN}Generating zsh completion (aether)...${NC}"
  "$binary" --generate-completion zsh >"$COMPLETIONS_DIR/_aether"
  chmod +x "$COMPLETIONS_DIR/_aether"
  echo -e "${GREEN}✓ Zsh completion generated: $COMPLETIONS_DIR/_aether${NC}"
}

# Function to generate fish completion (Rust CLI)
generate_fish_completion() {
  local binary
  binary="$(ensure_rust_cli)" || {
    echo -e "${RED}Error: Could not build Rust CLI${NC}" >&2
    return 1
  }

  echo -e "${GREEN}Generating fish completion (aether)...${NC}"
  "$binary" --generate-completion fish >"$COMPLETIONS_DIR/aether.fish"
  chmod +x "$COMPLETIONS_DIR/aether.fish"
  echo -e "${GREEN}✓ Fish completion generated: $COMPLETIONS_DIR/aether.fish${NC}"
}

# Main execution
cd "$PROJECT_ROOT"

case "$SHELLS" in
bash)
  generate_bash_completion
  ;;
zsh)
  generate_zsh_completion
  ;;
fish)
  generate_fish_completion
  ;;
all | *)
  generate_bash_completion
  generate_zsh_completion
  generate_fish_completion
  ;;
esac

echo ""
echo -e "${GREEN}✓ Completion scripts generated in $COMPLETIONS_DIR${NC}"
echo ""
echo "To install completions (Rust CLI binary name: aether):"
echo "  Bash:  source $COMPLETIONS_DIR/aether.bash"
echo "  Zsh:   fpath=($COMPLETIONS_DIR \$fpath) && compinit"
echo "  Fish:  cp $COMPLETIONS_DIR/aether.fish ~/.config/fish/completions/"
echo ""
echo "Or use: ./scripts/install_completions.sh"
