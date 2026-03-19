#!/usr/bin/env bash
# Install shell completion scripts for CLI tools
# Usage: ./scripts/install_completions.sh [bash|zsh|fish|all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPLETIONS_DIR="$PROJECT_ROOT/completions"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
SHELLS="${1:-all}"

# Detect current shell if not specified
if [ "$SHELLS" = "auto" ]; then
  case "$SHELL" in
  */bash | bash)
    SHELLS="bash"
    ;;
  */zsh | zsh)
    SHELLS="zsh"
    ;;
  */fish | fish)
    SHELLS="fish"
    ;;
  *)
    echo -e "${YELLOW}Could not detect shell. Using 'all'${NC}"
    SHELLS="all"
    ;;
  esac
fi

# Ensure completions exist
if [ ! -d "$COMPLETIONS_DIR" ] || [ -z "$(ls -A "$COMPLETIONS_DIR" 2>/dev/null)" ]; then
  echo -e "${YELLOW}Completions not found. Generating first...${NC}"
  "$SCRIPT_DIR/generate_completions.sh" "$SHELLS"
fi

# Function to install bash completion (Rust CLI: aether)
install_bash_completion() {
  local completion_file="$COMPLETIONS_DIR/aether.bash"

  if [ ! -f "$completion_file" ]; then
    echo -e "${RED}Error: Bash completion not found. Run ./scripts/generate_completions.sh first.${NC}"
    return 1
  fi

  # Determine bash completion directory
  if [ -d "/usr/local/etc/bash_completion.d" ]; then
    COMPLETION_DIR="/usr/local/etc/bash_completion.d"
  elif [ -d "/etc/bash_completion.d" ]; then
    COMPLETION_DIR="/etc/bash_completion.d"
  else
    COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$COMPLETION_DIR"
  fi

  echo -e "${GREEN}Installing bash completions to $COMPLETION_DIR...${NC}"
  sudo cp "$completion_file" "$COMPLETION_DIR/" 2>/dev/null || cp "$completion_file" "$COMPLETION_DIR/"

  echo -e "${GREEN}✓ Bash completions installed (aether)${NC}"
  echo "  Run: source $COMPLETION_DIR/aether.bash"
}

# Function to install zsh completion (Rust CLI: aether)
install_zsh_completion() {
  local completion_file="$COMPLETIONS_DIR/_aether"

  if [ ! -f "$completion_file" ]; then
    echo -e "${RED}Error: Zsh completion not found. Run ./scripts/generate_completions.sh first.${NC}"
    return 1
  fi

  # Determine zsh completion directory
  if [ -d "/usr/local/share/zsh/site-functions" ]; then
    COMPLETION_DIR="/usr/local/share/zsh/site-functions"
  elif [ -d "/usr/share/zsh/site-functions" ]; then
    COMPLETION_DIR="/usr/share/zsh/site-functions"
  else
    COMPLETION_DIR="$HOME/.zsh/completions"
    mkdir -p "$COMPLETION_DIR"
  fi

  echo -e "${GREEN}Installing zsh completions to $COMPLETION_DIR...${NC}"
  sudo cp "$completion_file" "$COMPLETION_DIR/" 2>/dev/null || cp "$completion_file" "$COMPLETION_DIR/"

  echo -e "${GREEN}✓ Zsh completions installed${NC}"
  echo "  Run: rm -f ~/.zcompdump* && compinit"
}

# Function to install fish completion (Rust CLI: aether)
install_fish_completion() {
  local completion_file="$COMPLETIONS_DIR/aether.fish"

  if [ ! -f "$completion_file" ]; then
    echo -e "${RED}Error: Fish completion not found. Run ./scripts/generate_completions.sh first.${NC}"
    return 1
  fi

  # Determine fish completion directory
  if [ -n "$XDG_CONFIG_HOME" ]; then
    COMPLETION_DIR="$XDG_CONFIG_HOME/fish/completions"
  else
    COMPLETION_DIR="$HOME/.config/fish/completions"
  fi

  mkdir -p "$COMPLETION_DIR"

  echo -e "${GREEN}Installing fish completions to $COMPLETION_DIR...${NC}"
  cp "$completion_file" "$COMPLETION_DIR/"

  echo -e "${GREEN}✓ Fish completions installed${NC}"
  echo "  Completions will be available in new fish sessions"
}

# Main execution
cd "$PROJECT_ROOT"

case "$SHELLS" in
bash)
  install_bash_completion
  ;;
zsh)
  install_zsh_completion
  ;;
fish)
  install_fish_completion
  ;;
all | *)
  install_bash_completion
  install_zsh_completion
  install_fish_completion
  ;;
esac

echo ""
echo -e "${GREEN}✓ Completions installed successfully!${NC}"
echo ""
echo "To use completions:"
echo "  Bash:  Restart terminal or run: source ~/.bashrc"
echo "  Zsh:   Restart terminal or run: compinit"
echo "  Fish:  Restart terminal (completions load automatically)"
