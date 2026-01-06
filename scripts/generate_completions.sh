#!/usr/bin/env bash
# Generate shell completion scripts for CLI tools
# Usage: ./scripts/generate_completions.sh [bash|zsh|fish|all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$PROJECT_ROOT/build}"
COMPLETIONS_DIR="$PROJECT_ROOT/completions"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
SHELLS="${1:-all}"

# Ensure build directory exists
if [ ! -d "$BUILD_DIR" ]; then
  echo -e "${YELLOW}Build directory not found. Building first...${NC}"
  cd "$PROJECT_ROOT"
  ./scripts/build_universal.sh
fi

# Ensure completions directory exists
mkdir -p "$COMPLETIONS_DIR"

# Function to generate bash completion
generate_bash_completion() {
  local binary="$BUILD_DIR/bin/ib_box_spread"

  if [ ! -f "$binary" ]; then
    echo -e "${RED}Error: Binary not found at $binary${NC}"
    return 1
  fi

  echo -e "${GREEN}Generating bash completion...${NC}"

  # CLI11 supports bash completion generation
  "$binary" --generate-completion bash > "$COMPLETIONS_DIR/ib_box_spread.bash" || {
    # Fallback: create manual completion
    cat > "$COMPLETIONS_DIR/ib_box_spread.bash" << 'EOF'
# Bash completion for ib_box_spread
_ib_box_spread() {
  local cur prev words cword
  _init_completion || return

  case "$prev" in
    -c|--config)
      _filedir json
      return
      ;;
    --log-level)
      COMPREPLY=($(compgen -W "trace debug info warn error" -- "$cur"))
      return
      ;;
  esac

  if [[ "$cur" == -* ]]; then
    COMPREPLY=($(compgen -W "-c --config --dry-run --validate --use-nautilus --log-level -v --version -h --help" -- "$cur"))
  fi
}

complete -F _ib_box_spread ib_box_spread
EOF
  }

  chmod +x "$COMPLETIONS_DIR/ib_box_spread.bash"
  echo -e "${GREEN}✓ Bash completion generated: $COMPLETIONS_DIR/ib_box_spread.bash${NC}"
}

# Function to generate zsh completion
generate_zsh_completion() {
  local binary="$BUILD_DIR/bin/ib_box_spread"

  if [ ! -f "$binary" ]; then
    echo -e "${RED}Error: Binary not found at $binary${NC}"
    return 1
  fi

  echo -e "${GREEN}Generating zsh completion...${NC}"

  # CLI11 supports zsh completion generation
  "$binary" --generate-completion zsh > "$COMPLETIONS_DIR/_ib_box_spread" || {
    # Fallback: create manual completion
    cat > "$COMPLETIONS_DIR/_ib_box_spread" << 'EOF'
#compdef ib_box_spread

_ib_box_spread() {
  local context state line
  local -a options configs log_levels

  options=(
    '(-c --config)'{-c,--config}'[Configuration file path]:config file:_files -g "*.json"'
    '(--dry-run)'--dry-run'[Simulate trading without executing orders]'
    '(--validate)'--validate'[Validate configuration and exit]'
    '(--use-nautilus)'--use-nautilus'[Use nautilus_trader for market data and execution]'
    '(--log-level)'--log-level'[Override log level]:log level:(trace debug info warn error)'
    '(-v --version)'{-v,--version}'[Show version information]'
    '(-h --help)'{-h,--help}'[Show help message]'
  )

  _arguments -s -S $options
}

_ib_box_spread "$@"
EOF
  }

  chmod +x "$COMPLETIONS_DIR/_ib_box_spread"
  echo -e "${GREEN}✓ Zsh completion generated: $COMPLETIONS_DIR/_ib_box_spread${NC}"
}

# Function to generate fish completion
generate_fish_completion() {
  local binary="$BUILD_DIR/bin/ib_box_spread"

  if [ ! -f "$binary" ]; then
    echo -e "${RED}Error: Binary not found at $binary${NC}"
    return 1
  fi

  echo -e "${GREEN}Generating fish completion...${NC}"

  # CLI11 supports fish completion generation
  "$binary" --generate-completion fish > "$COMPLETIONS_DIR/ib_box_spread.fish" || {
    # Fallback: create manual completion
    cat > "$COMPLETIONS_DIR/ib_box_spread.fish" << 'EOF'
# Fish completion for ib_box_spread

complete -c ib_box_spread -s c -l config -r -d "Configuration file path"
complete -c ib_box_spread -l dry-run -d "Simulate trading without executing orders"
complete -c ib_box_spread -l validate -d "Validate configuration and exit"
complete -c ib_box_spread -l use-nautilus -d "Use nautilus_trader for market data and execution"
complete -c ib_box_spread -l log-level -x -a "trace debug info warn error" -d "Override log level"
complete -c ib_box_spread -s v -l version -d "Show version information"
complete -c ib_box_spread -s h -l help -d "Show help message"
EOF
  }

  chmod +x "$COMPLETIONS_DIR/ib_box_spread.fish"
  echo -e "${GREEN}✓ Fish completion generated: $COMPLETIONS_DIR/ib_box_spread.fish${NC}"
}

# Function to generate TUI completion (Python TUI)
generate_tui_completions() {
  echo -e "${GREEN}Generating TUI completions...${NC}"

  # Note: Python TUI uses environment variables and config files
  # Run with: python -m python.tui

  # Bash completion for TUI
  cat > "$COMPLETIONS_DIR/ib_box_spread_tui.bash" << 'EOF'
# Bash completion for Python TUI
# Run with: python -m python.tui
# Uses environment variables: TUI_BACKEND, TUI_API_URL, TUI_SNAPSHOT_FILE
_ib_box_spread_tui() {
  local cur prev words cword
  _init_completion || return

  # Python TUI module completion
  if [[ "$prev" == "-m" ]]; then
    COMPREPLY=($(compgen -W "python.tui" -- "$cur"))
  fi
}

complete -F _ib_box_spread_tui python
EOF

  # Zsh completion for TUI
  cat > "$COMPLETIONS_DIR/_ib_box_spread_tui" << 'EOF'
#compdef python

# Python TUI uses module execution: python -m python.tui
# Config: ~/.config/ib_box_spread/tui_config.json
# Env vars: TUI_BACKEND, TUI_API_URL

_ib_box_spread_tui() {
  local context state line
  local -a options

  options=(
    '(-h --help)'{-h,--help}'[Show help message]'
  )

  _arguments -s -S $options
}

_ib_box_spread_tui "$@"
EOF

  # Fish completion for TUI
  cat > "$COMPLETIONS_DIR/ib_box_spread_tui.fish" << 'EOF'
# Fish completion for Python TUI
# Run with: python -m python.tui
# Uses environment variables: TUI_BACKEND, TUI_API_URL, TUI_SNAPSHOT_FILE

complete -c python -n '__fish_seen_subcommand_from -m' -a "python.tui" -d "Run TUI application"
EOF

  chmod +x "$COMPLETIONS_DIR"/*tui* 2>/dev/null || true
  echo -e "${GREEN}✓ TUI completions generated${NC}"
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
  all|*)
    generate_bash_completion
    generate_zsh_completion
    generate_fish_completion
    generate_tui_completions
    ;;
esac

echo ""
echo -e "${GREEN}✓ Completion scripts generated in $COMPLETIONS_DIR${NC}"
echo ""
echo "To install completions:"
echo "  Bash:  source $COMPLETIONS_DIR/ib_box_spread.bash"
echo "  Zsh:   fpath=($COMPLETIONS_DIR \$fpath) && compinit"
echo "  Fish:  cp $COMPLETIONS_DIR/*.fish ~/.config/fish/completions/"
echo ""
echo "Or use: ./scripts/install_completions.sh"
