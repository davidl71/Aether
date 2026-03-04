# Exarp Oh My Zsh Plugin
# Uses exarp-go CLI when available. Python MCP server (exarp_project_management) is deprecated.
# See docs/MCP_REQUIRED_SERVERS.md and docs/EXARP_GO_MIGRATION_LEFTOVERS.md.

export EXARP_PLUGIN_VERSION="2.0.0"

# Resolve exarp-go binary: PATH, EXARP_GO_ROOT/bin, or run_exarp_go.sh in project
_exarp_go_cmd() {
  if command -v exarp-go &>/dev/null; then
    echo "exarp-go"
    return 0
  fi
  if [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
    echo "${EXARP_GO_ROOT}/bin/exarp-go"
    return 0
  fi
  local root="${PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null)}"
  if [[ -n "$root" ]] && [[ -x "$root/scripts/run_exarp_go.sh" ]]; then
    echo "$root/scripts/run_exarp_go.sh"
    return 0
  fi
  return 1
}

# Run exarp-go with -tool and optional -args (project dir as env)
_exarp_go_tool() {
  local tool="$1"
  shift
  local exarp="$(_exarp_go_cmd)"
  if [[ -z "$exarp" ]]; then
    echo "❌ exarp-go not found. Install exarp-go (Python MCP server is deprecated)." >&2
    return 1
  fi
  local project_dir="${1:-.}"
  export PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$project_dir" && git rev-parse --show-toplevel 2>/dev/null || echo "$project_dir")}"
  "$exarp" -tool "$tool" "$@" 2>/dev/null
}

# Check if exarp-go is available
exarp_check() {
  if _exarp_go_cmd &>/dev/null; then
    echo "✅ exarp-go is available"
    return 0
  fi
  echo "❌ exarp-go not found. Install it or set EXARP_GO_ROOT. Python MCP server is deprecated." >&2
  return 1
}

# Run exarp MCP server (exarp-go stdio)
exarp_server() {
  local exarp="$(_exarp_go_cmd)"
  if [[ -z "$exarp" ]]; then
    echo "❌ exarp-go not found." >&2
    return 1
  fi
  "$exarp" "$@"
}

# Documentation health
exarp_docs_health() {
  local project_dir="${1:-.}"
  _exarp_go_tool health -args '{"action":"docs"}' "$project_dir" || true
}

# Task alignment
exarp_task_align() {
  local project_dir="${1:-.}"
  _exarp_go_tool task_workflow -args '{"action":"alignment"}' "$project_dir" || true
}

# Duplicate detection
exarp_duplicates() {
  local project_dir="${1:-.}"
  _exarp_go_tool detect_duplicate_tasks -args '{}' "$project_dir" || true
}

# Security scan
exarp_security() {
  local project_dir="${1:-.}"
  _exarp_go_tool security -args '{"action":"scan"}' "$project_dir" || true
}

# Daily automation
exarp_daily() {
  local project_dir="${1:-.}"
  _exarp_go_tool session -args '{"action":"daily"}' "$project_dir" || true
}

# Automation opportunities
exarp_opportunities() {
  local project_dir="${1:-.}"
  _exarp_go_tool report -args '{"action":"opportunities"}' "$project_dir" || true
}

# Show exarp status
exarp_status() {
  echo "📊 Exarp (exarp-go)"
  echo ""
  exarp_check
  echo ""
  echo "Commands (exarp-go):"
  echo "  exarp_docs_health   - Documentation health"
  echo "  exarp_task_align    - Task alignment"
  echo "  exarp_duplicates    - Duplicate task detection"
  echo "  exarp_security      - Dependency security scan"
  echo "  exarp_daily         - Daily automation"
  echo "  exarp_opportunities - Automation opportunities"
  echo "  exarp_tasks         - Task list (if available)"
  echo "  exarp_motd          - MOTD (if available)"
}

# Task list (exarp-go task list when available)
exarp_tasks() {
  local exarp="$(_exarp_go_cmd)"
  if [[ -z "$exarp" ]]; then
    echo "❌ exarp-go not found." >&2
    return 1
  fi
  export PROJECT_ROOT="${PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null)}"
  "$exarp" -tool task_workflow -args '{"action":"sync","sub_action":"list"}' 2>/dev/null || echo "Use exarp-go MCP in Cursor for task list."
}

# MOTD placeholder (exarp-go session prime or similar)
exarp_motd() {
  local exarp="$(_exarp_go_cmd)"
  if [[ -z "$exarp" ]]; then
    return 0
  fi
  export PROJECT_ROOT="${PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null)}"
  "$exarp" -tool session -args '{"action":"prime","compact":true}' 2>/dev/null | head -20 || true
}

exarp_task_summary() {
  exarp_tasks
}

# Aliases
alias exd='exarp_docs_health'
alias exa='exarp_task_align'
alias exdup='exarp_duplicates'
alias exsec='exarp_security'
alias exday='exarp_daily'
alias exopp='exarp_opportunities'
alias exsrv='exarp_server'
alias exstat='exarp_status'
alias excheck='exarp_check'
alias ext='exarp_tasks'
alias exm='exarp_motd'

# Completion
_exarp_completion() {
  local -a commands
  commands=(
    'server:Run exarp-go MCP server'
    'docs-health:Documentation health'
    'task-align:Task alignment'
    'duplicates:Duplicate detection'
    'security:Security scan'
    'daily:Daily automation'
    'opportunities:Automation opportunities'
    'status:Show status'
    'check:Check exarp-go'
  )
  _describe 'exarp commands' commands
}

if command -v compdef &>/dev/null; then
  compdef _exarp_completion exarp
fi
