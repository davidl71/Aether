#!/usr/bin/env bash
# Generate machine-local MCP config files.
# Run once after cloning or switching machines: ./scripts/setup_mcp.sh
#
# Generates:
#   .cursor/mcp.json   (gitignored)
#   opencode.json      (gitignored; template: opencode.json.example)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Resolve exarp-go migrations dir
if [[ -d "${HOME}/exarp-go/migrations" ]]; then
  EXARP_MIGRATIONS_DIR="${HOME}/exarp-go/migrations"
elif [[ -d "${PROJECT_ROOT}/../exarp-go/migrations" ]]; then
  EXARP_MIGRATIONS_DIR="$(cd "${PROJECT_ROOT}/../exarp-go" && pwd)/migrations"
elif [[ -d "${PROJECT_ROOT}/../../mcp/exarp-go/migrations" ]]; then
  EXARP_MIGRATIONS_DIR="$(cd "${PROJECT_ROOT}/../../mcp/exarp-go" && pwd)/migrations"
else
  EXARP_MIGRATIONS_DIR="${HOME}/exarp-go/migrations"
fi

# Resolve binary paths (prefer installed, fall back to npx/uvx launchers)
resolve_bin() {
  local name="$1" fallback_cmd="$2"
  local paths=(
    "${HOME}/.npm-global/bin/${name}"
    "${HOME}/.local/bin/${name}"
    "$(which "${name}" 2>/dev/null || true)"
  )
  for p in "${paths[@]}"; do
    [[ -x "${p}" ]] && echo "${p}" && return
  done
  echo "${fallback_cmd}"
}

CONTEXT7_BIN="$(resolve_bin context7-mcp "npx -y @upstash/context7-mcp")"
FILESYSTEM_BIN="$(resolve_bin mcp-server-filesystem "npx -y @modelcontextprotocol/server-filesystem")"
SEQ_THINKING_BIN="$(resolve_bin mcp-server-sequential-thinking "npx -y @modelcontextprotocol/server-sequential-thinking")"
TRACTATUS_BIN="$(resolve_bin tractatus_thinking "npx -y tractatus_thinking")"
MEMORY_BIN="$(resolve_bin mcp-server-memory "npx -y @modelcontextprotocol/server-memory")"
FETCH_BIN="$(resolve_bin mcp-server-fetch "uvx mcp-server-fetch")"
GIT_BIN="$(resolve_bin mcp-server-git "uvx mcp-server-git")"

echo "Project root:      ${PROJECT_ROOT}"
echo "Exarp migrations:  ${EXARP_MIGRATIONS_DIR}"
echo "context7:          ${CONTEXT7_BIN}"
echo "filesystem:        ${FILESYSTEM_BIN}"
echo "fetch:             ${FETCH_BIN}"
echo "git:               ${GIT_BIN}"

# Generate opencode.json
sed \
  -e "s|__PROJECT_ROOT__|${PROJECT_ROOT}|g" \
  -e "s|__EXARP_MIGRATIONS_DIR__|${EXARP_MIGRATIONS_DIR}|g" \
  "${PROJECT_ROOT}/opencode.json.example" > "${PROJECT_ROOT}/opencode.json"
echo "✓ opencode.json"

# Generate .cursor/mcp.json
python3 - << PYEOF
import json, os

project_root = "${PROJECT_ROOT}"
exarp_migrations = "${EXARP_MIGRATIONS_DIR}"

def cmd(bin_path, *extra_args):
    """Return command/args split for a binary path or 'cmd arg1 arg2' fallback string."""
    parts = bin_path.split() if " " in bin_path else [bin_path]
    return parts + list(extra_args)

config = {
  "mcpServers": {
    "context7":          {"command": "${CONTEXT7_BIN}".split()[0], "args": "${CONTEXT7_BIN}".split()[1:]},
    "exarp-go": {
      "disabled": False,
      "command": f"{project_root}/scripts/run_exarp_go.sh",
      "args": [],
      "env": {"PROJECT_ROOT": project_root, "EXARP_MIGRATIONS_DIR": exarp_migrations}
    },
    "filesystem":        {"command": "${FILESYSTEM_BIN}".split()[0], "args": "${FILESYSTEM_BIN}".split()[1:] + [project_root]},
    "sequential_thinking": {"command": "${SEQ_THINKING_BIN}".split()[0], "args": "${SEQ_THINKING_BIN}".split()[1:]},
    "tractatus_thinking":  {"command": "${TRACTATUS_BIN}".split()[0], "args": "${TRACTATUS_BIN}".split()[1:]},
    "fetch":             {"command": "${FETCH_BIN}".split()[0], "args": "${FETCH_BIN}".split()[1:]},
    "memory":            {"command": "${MEMORY_BIN}".split()[0], "args": "${MEMORY_BIN}".split()[1:]},
    "git":               {"command": "${GIT_BIN}".split()[0], "args": "${GIT_BIN}".split()[1:] + ["--repository", project_root]}
  }
}

out = os.path.join(project_root, ".cursor", "mcp.json")
with open(out, "w") as f:
    json.dump(config, f, indent=2)
    f.write("\n")
print("✓ .cursor/mcp.json")
PYEOF

echo ""
echo "Done. Restart Cursor, OpenCode, and Claude Code to pick up the new config."
