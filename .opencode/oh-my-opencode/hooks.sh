#!/usr/bin/env bash
# Oh My OpenCode hooks for Aether

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../../" && pwd)"

# Colors
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Hook: Check project root
check_project_root() {
  if [[ -z "${PROJECT_ROOT:-}" ]]; then
    echo -e "${YELLOW}⚠ Warning: PROJECT_ROOT not set${NC}"
    echo "Set it with: export PROJECT_ROOT=$PROJECT_ROOT"
  fi
}

# Hook: Refresh task cache
refresh_task_cache() {
  # This would refresh the exarp-go task cache
  # In practice, the plugin handles this automatically
  :
}

# Hook: Show task reminders
show_task_reminders() {
  # Check if there are in-progress tasks
  if command -v exarp-go >/dev/null 2>&1; then
    local in_progress
    in_progress=$(exarp-go task list --status "In Progress" 2>/dev/null | head -5)
    if [[ -n "$in_progress" ]]; then
      echo -e "${CYAN}📋 In Progress:${NC}"
      echo "$in_progress" | while read -r line; do
        echo -e "  ${PURPLE}•${NC} $line"
      done
    fi
  fi
}

# Hook: Prime session
prime_session() {
  # Auto-prime if configured
  if [[ "${OMO_AUTO_PRIME:-1}" == "1" ]]; then
    echo -e "${CYAN}🚀 Auto-priming session...${NC}"
    # The plugin handles actual priming
  fi
}

# Hook: Show welcome
show_welcome() {
  if [[ "${OMO_SHOW_WELCOME:-1}" == "1" ]]; then
    echo -e "${CYAN}"
    echo "  █████╗ ██╗  ██╗███████╗████████╗██╗  ██╗███████╗██████╗ "
    echo " ██╔══██╗╚██╗██╔╝██╔════╝╚══██╔══╝██║  ██║██╔════╝██╔══██╗"
    echo " ███████║ ╚███╔╝ █████╗     ██║   ███████║█████╗  ██████╔╝"
    echo " ██╔══██║ ██╔██╗ ██╔══╝     ██║   ██╔══██║██╔══╝  ██╔══██╗"
    echo " ██║  ██║██╔╝ ██╗███████╗   ██║   ██║  ██║███████╗██║  ██║"
    echo " ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝"
    echo -e "${NC}"
    echo -e "${GREEN}Welcome to Aether!${NC} Run ${YELLOW}/welcome${NC} for help."
    echo ""
  fi
}

# Main hook dispatcher
case "${1:-}" in
pre-command)
  check_project_root
  refresh_task_cache
  ;;
post-command)
  show_task_reminders
  ;;
session-start)
  prime_session
  show_welcome
  ;;
*)
  echo "Usage: $0 {pre-command|post-command|session-start}"
  exit 1
  ;;
esac
