#!/usr/bin/env bash
# sync_mcp_config_agents.sh - Sync MCP server configuration across all agents
#
# Ensures all agents have the correct project-management-automation MCP server
# configuration with relative paths that work on different Cursor servers.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MCP_SERVER_DIR="$PROJECT_ROOT/mcp-servers/project-management-automation"
CURSOR_CONFIG_DIR="$PROJECT_ROOT/.cursor"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== MCP Configuration Sync for All Agents ===${NC}\n"

# Check if MCP server directory exists
if [ ! -d "$MCP_SERVER_DIR" ]; then
    echo -e "${RED}Error: MCP server directory not found at $MCP_SERVER_DIR${NC}" >&2
    exit 1
fi

# Check if run_server.sh exists
if [ ! -f "$MCP_SERVER_DIR/run_server.sh" ]; then
    echo -e "${RED}Error: run_server.sh not found at $MCP_SERVER_DIR/run_server.sh${NC}" >&2
    exit 1
fi

# Make run_server.sh executable
chmod +x "$MCP_SERVER_DIR/run_server.sh"

# Function to generate MCP config entry
generate_mcp_config() {
    local server_path="$1"
    local use_relative="${2:-true}"

    if [ "$use_relative" = "true" ]; then
        # Use relative path from project root
        local rel_path="${server_path#$PROJECT_ROOT/}"
        cat <<EOF
    "project-management-automation": {
      "command": "\${PROJECT_ROOT}/${rel_path}/run_server.sh",
      "args": [],
      "env": {
        "PROJECT_ROOT": "${PROJECT_ROOT}"
      },
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
EOF
    else
        # Use absolute path (for documentation/reference)
        cat <<EOF
    "project-management-automation": {
      "command": "${server_path}/run_server.sh",
      "args": [],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
EOF
    fi
}

# Function to check if MCP config exists and has correct entry
check_mcp_config() {
    local config_file="$1"
    local config_name="$2"

    if [ ! -f "$config_file" ]; then
        echo -e "${YELLOW}  ⚠️  Config file not found: $config_file${NC}"
        return 1
    fi

    if grep -q "project-management-automation" "$config_file" 2>/dev/null; then
        # Check if it uses the wrapper script
        if grep -q "run_server.sh" "$config_file" 2>/dev/null; then
            echo -e "${GREEN}  ✅ Correctly configured${NC}"
            return 0
        else
            echo -e "${YELLOW}  ⚠️  Found but needs update (not using run_server.sh)${NC}"
            return 2
        fi
    else
        echo -e "${YELLOW}  ⚠️  Not configured${NC}"
        return 1
    fi
}

# Function to update MCP config
update_mcp_config() {
    local config_file="$1"
    local config_name="$2"

    echo -e "${BLUE}  Updating $config_name...${NC}"

    # Backup existing config
    if [ -f "$config_file" ]; then
        cp "$config_file" "${config_file}.bak.$(date +%Y%m%d_%H%M%S)"
    fi

    # Create or update config
    if [ ! -f "$config_file" ]; then
        # Create new config file
        cat > "$config_file" <<EOF
{
  "mcpServers": {
$(generate_mcp_config "$MCP_SERVER_DIR" "false")
  }
}
EOF
    else
        # Update existing config
        # Check if mcpServers section exists
        if grep -q '"mcpServers"' "$config_file"; then
            # Remove old project-management-automation entry if it exists
            python3 <<PYTHON_SCRIPT
import json
import sys
from pathlib import Path

config_file = Path("$config_file")

try:
    with open(config_file, 'r') as f:
        config = json.load(f)
except Exception as e:
    print(f"Error reading config: {e}", file=sys.stderr)
    sys.exit(1)

# Ensure mcpServers exists
if 'mcpServers' not in config:
    config['mcpServers'] = {}

# Update or add project-management-automation entry
config['mcpServers']['project-management-automation'] = {
    "command": "${MCP_SERVER_DIR}/run_server.sh",
    "args": [],
    "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
}

# Write back
with open(config_file, 'w') as f:
    json.dump(config, f, indent=2)
    f.write('\n')
PYTHON_SCRIPT
        else
            # Create mcpServers section
            python3 <<PYTHON_SCRIPT
import json
import sys
from pathlib import Path

config_file = Path("$config_file")

try:
    with open(config_file, 'r') as f:
        config = json.load(f)
except Exception:
    config = {}

config['mcpServers'] = {
    "project-management-automation": {
        "command": "${MCP_SERVER_DIR}/run_server.sh",
        "args": [],
        "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
}

with open(config_file, 'w') as f:
    json.dump(config, f, indent=2)
    f.write('\n')
PYTHON_SCRIPT
        fi
    fi

    echo -e "${GREEN}  ✅ Updated${NC}"
}

# Check main project MCP config
echo -e "${BLUE}Checking main project MCP config...${NC}"
MAIN_CONFIG="$CURSOR_CONFIG_DIR/mcp.json"
if check_mcp_config "$MAIN_CONFIG" "Main project"; then
    STATUS_MAIN=0
else
    STATUS_MAIN=$?
    if [ "$STATUS_MAIN" -eq 2 ] || [ "$STATUS_MAIN" -eq 1 ]; then
        read -p "Update main project config? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            update_mcp_config "$MAIN_CONFIG" "Main project"
        fi
    fi
fi

# Check agent configs
echo -e "\n${BLUE}Checking agent configurations...${NC}"
AGENTS_DIR="$PROJECT_ROOT/agents"
AGENT_CONFIGS=()

if [ -d "$AGENTS_DIR" ]; then
    for agent_dir in "$AGENTS_DIR"/*; do
        if [ -d "$agent_dir" ]; then
            agent_name=$(basename "$agent_dir")
            agent_config="$agent_dir/.cursor/mcp.json"

            # Create .cursor directory if it doesn't exist
            if [ ! -d "$(dirname "$agent_config")" ]; then
                mkdir -p "$(dirname "$agent_config")"
            fi

            echo -e "${BLUE}  Agent: $agent_name${NC}"
            if check_mcp_config "$agent_config" "$agent_name"; then
                :
            else
                STATUS=$?
                if [ "$STATUS" -eq 2 ] || [ "$STATUS" -eq 1 ]; then
                    read -p "    Update $agent_name config? (y/N) " -n 1 -r
                    echo
                    if [[ $REPLY =~ ^[Yy]$ ]]; then
                        update_mcp_config "$agent_config" "$agent_name"
                    fi
                fi
            fi
        fi
    done
fi

# Generate example config for documentation
echo -e "\n${BLUE}Generating example configuration...${NC}"
EXAMPLE_CONFIG="$MCP_SERVER_DIR/MCP_CONFIG_EXAMPLE.json"
cat > "$EXAMPLE_CONFIG" <<EOF
{
  "mcpServers": {
$(generate_mcp_config "$MCP_SERVER_DIR" "false")
  }
}
EOF
echo -e "${GREEN}  ✅ Example config saved to $EXAMPLE_CONFIG${NC}"

# Summary
echo -e "\n${BLUE}=== Summary ===${NC}"
echo -e "${GREEN}✅ MCP server path: $MCP_SERVER_DIR${NC}"
echo -e "${GREEN}✅ Wrapper script: $MCP_SERVER_DIR/run_server.sh${NC}"
echo -e "\n${YELLOW}⚠️  Next steps:${NC}"
echo -e "  1. Review updated configurations"
echo -e "  2. Restart Cursor completely on each agent/server"
echo -e "  3. Verify MCP server appears in Cursor settings"
echo -e "  4. Test with: Check documentation health tool"

echo -e "\n${BLUE}=== Done ===${NC}"
