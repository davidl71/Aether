# Exarp Oh My Zsh Plugin
# Provides aliases, functions, and completions for Exarp project management automation
#
# LEGACY: These commands use the Python package (exarp_project_management). Prefer
# exarp-go MCP server in Cursor for automation. See docs/EXARP_GO_MIGRATION_LEFTOVERS.md

# Plugin metadata
export EXARP_PLUGIN_VERSION="1.1.0"

# Get plugin directory (for helper scripts)
_exarp_plugin_dir() {
  # Try to find plugin directory
  if [[ -n "${ZSH}" ]] && [[ -d "${ZSH}/custom/plugins/exarp" ]]; then
    # Oh My Zsh installation
    echo "${ZSH}/custom/plugins/exarp"
  elif [[ -n "${ZSH_CUSTOM}" ]] && [[ -d "${ZSH_CUSTOM}/plugins/exarp" ]]; then
    # Oh My Zsh custom directory
    echo "${ZSH_CUSTOM}/plugins/exarp"
  elif [[ -f "${0}" ]]; then
    # Direct sourcing - get directory of this script
    local script_dir
    script_dir="$(cd "$(dirname "${0}")" && pwd)"
    echo "${script_dir}"
  else
    # Fallback: assume script is in expected location
    echo "${HOME}/.oh-my-zsh/custom/plugins/exarp"
  fi
}

# ============================================================================
# Helper Functions
# ============================================================================

# Check if exarp is installed
exarp_check() {
  if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    return 1
  fi

  if ! python3 -c "import exarp_project_management" 2>/dev/null; then
    echo "⚠️  Exarp is not installed. Install with: pip install exarp-automation-mcp"
    return 1
  fi

  echo "✅ Exarp is installed and ready"
  return 0
}

# Run exarp MCP server
exarp_server() {
  python3 -m exarp_project_management.server "$@"
}

# Run exarp documentation health check
exarp_docs_health() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.automate_docs_health_v2 "$project_dir"
}

# Run exarp task alignment analysis
exarp_task_align() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.todo2_alignment "$project_dir"
}

# Run exarp duplicate detection
exarp_duplicates() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.duplicate_detection "$project_dir"
}

# Run exarp security scan
exarp_security() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.dependency_security "$project_dir"
}

# Run exarp daily automation
exarp_daily() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.daily_automation "$project_dir"
}

# Run exarp automation opportunities
exarp_opportunities() {
  local project_dir="${1:-.}"
  python3 -m exarp_project_management.scripts.automation_opportunities "$project_dir"
}

# Show exarp status
exarp_status() {
  echo "📊 Exarp Project Management Automation"
  echo ""
  exarp_check
  echo ""
  echo "Available commands:"
  echo "  exarp_server          - Run Exarp MCP server"
  echo "  exarp_docs_health     - Check documentation health"
  echo "  exarp_task_align      - Analyze task alignment"
  echo "  exarp_duplicates      - Detect duplicate tasks"
  echo "  exarp_security         - Scan dependencies for security issues"
  echo "  exarp_daily            - Run daily automation tasks"
  echo "  exarp_opportunities    - Find automation opportunities"
  echo "  exarp_tasks            - Show context-aware task list"
  echo "  exarp_motd             - Show task summary MOTD"
  echo ""
  echo "Aliases:"
  echo "  exd                    - exarp_docs_health"
  echo "  exa                    - exarp_task_align"
  echo "  exdup                  - exarp_duplicates"
  echo "  exsec                  - exarp_security"
  echo "  exday                  - exarp_daily"
  echo "  exopp                  - exarp_opportunities"
  echo "  ext                    - exarp_tasks"
  echo "  exm                    - exarp_motd"
}

# Show context-aware task list
exarp_tasks() {
  local plugin_dir="$(_exarp_plugin_dir)"
  local script_path="${plugin_dir}/exarp_context_tasks.py"

  # Try alternative locations
  if [[ ! -f "${script_path}" ]]; then
    # Try relative to current file
    script_path="$(dirname "${0}")/exarp_context_tasks.py"
  fi

  if [[ ! -f "${script_path}" ]]; then
    echo "❌ Context tasks script not found. Expected: ${script_path}" >&2
    return 1
  fi

  python3 "${script_path}" list 2>/dev/null | python3 -m json.tool 2>/dev/null || {
    echo "📋 Context-Aware Task List"
    echo ""
    python3 "${script_path}" list
  }
}

# Show task summary MOTD
exarp_motd() {
  local plugin_dir="$(_exarp_plugin_dir)"
  local script_path="${plugin_dir}/exarp_context_tasks.py"

  # Try alternative locations
  if [[ ! -f "${script_path}" ]]; then
    # Try relative to current file
    script_path="$(dirname "${0}")/exarp_context_tasks.py"
  fi

  if [[ ! -f "${script_path}" ]]; then
    echo "❌ Context tasks script not found. Expected: ${script_path}" >&2
    return 1
  fi

  python3 "${script_path}" motd 2>/dev/null || {
    echo "⚠️  Error generating MOTD" >&2
    return 1
  }
}

# Get task summary (JSON)
exarp_task_summary() {
  local plugin_dir="$(_exarp_plugin_dir)"
  local script_path="${plugin_dir}/exarp_context_tasks.py"

  # Try alternative locations
  if [[ ! -f "${script_path}" ]]; then
    script_path="$(dirname "${0}")/exarp_context_tasks.py"
  fi

  if [[ ! -f "${script_path}" ]]; then
    echo "❌ Context tasks script not found" >&2
    return 1
  fi

  python3 "${script_path}" summary 2>/dev/null || return 1
}

# ============================================================================
# Aliases
# ============================================================================

# Short aliases for common commands
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

# ============================================================================
# Completion Support
# ============================================================================

# Zsh completion function for exarp commands
_exarp_completion() {
  local -a commands
  commands=(
    'server:Run Exarp MCP server'
    'docs-health:Check documentation health'
    'task-align:Analyze task alignment'
    'duplicates:Detect duplicate tasks'
    'security:Scan dependencies for security issues'
    'daily:Run daily automation tasks'
    'opportunities:Find automation opportunities'
    'status:Show exarp status and available commands'
    'check:Check if exarp is installed'
  )

  _describe 'exarp commands' commands
}

# Register completion if compdef is available
if command -v compdef &> /dev/null; then
  compdef _exarp_completion exarp
fi

# ============================================================================
# Auto-check on plugin load (optional - can be disabled)
# ============================================================================

# Uncomment the line below to auto-check exarp installation on shell startup
# exarp_check > /dev/null 2>&1 || true

# ============================================================================
# MOTD on Shell Startup (optional - can be enabled)
# ============================================================================

# Uncomment the line below to show MOTD on shell startup
# exarp_motd 2>/dev/null || true
