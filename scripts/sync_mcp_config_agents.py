#!/usr/bin/env python3
"""
Sync MCP server configuration across all agents.

Ensures all agents have the correct project-management-automation MCP server
configuration with paths that work on different Cursor servers.
"""

import json
import sys
from pathlib import Path
from typing import Dict, Any, Optional

# Colors for output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

def find_project_root(start_path: Path) -> Path:
    """Find project root by looking for markers."""
    current = start_path.resolve()
    for _ in range(5):
        if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
            return current
        if current.parent == current:
            break
        current = current.parent
    return start_path.resolve()

def get_mcp_server_config(project_root: Path, mcp_server_dir: Path) -> Dict[str, Any]:
    """Generate MCP server configuration entry."""
    # Use absolute path for command (works across different servers)
    # The run_server.sh script uses relative paths internally
    # Use absolute path for command (required by Cursor MCP config)
    # The run_server.sh script handles virtual environment and path detection
    # Each server will have a different absolute path, which is correct
    run_server_path = mcp_server_dir / "run_server.sh"

    return {
        "command": str(run_server_path.resolve()),
        "args": [],
        "description": (
            "Project management automation tools - documentation health, task alignment, "
            "duplicate detection, security scanning, and automation opportunities"
        )
    }

def check_mcp_config(config_file: Path) -> tuple[bool, Optional[str]]:
    """Check if MCP config exists and has correct entry."""
    if not config_file.exists():
        return False, "not_found"

    try:
        with open(config_file, 'r') as f:
            config = json.load(f)

        if 'mcpServers' not in config:
            return False, "no_mcp_servers"

        # Check for both old name and new name
        server_name = None
        if 'automa' in config['mcpServers']:
            server_name = 'automa'
        elif 'project-management-automation' in config['mcpServers']:
            server_name = 'project-management-automation'
        else:
            return False, "not_configured"

        server_config = config['mcpServers'][server_name]

        # Check if it uses the wrapper script
        command = server_config.get('command', '')
        if 'run_server.sh' in command:
            return True, "correct"
        else:
            return False, "needs_update"

    except json.JSONDecodeError as e:
        return False, f"invalid_json: {e}"
    except Exception as e:
        return False, f"error: {e}"

def update_mcp_config(config_file: Path, mcp_server_dir: Path) -> bool:
    """Update MCP config with correct entry."""
    try:
        # Backup existing config
        if config_file.exists():
            backup_file = config_file.with_suffix('.json.bak')
            if not backup_file.exists():
                config_file.rename(backup_file)

        # Load or create config
        if config_file.exists():
            with open(config_file, 'r') as f:
                config = json.load(f)
        else:
            config = {}

        # Ensure mcpServers exists
        if 'mcpServers' not in config:
            config['mcpServers'] = {}

        # Remove old name if it exists
        if 'project-management-automation' in config['mcpServers']:
            del config['mcpServers']['project-management-automation']

        # Update automa entry (use new name)
        config['mcpServers']['automa'] = get_mcp_server_config(
            config_file.parent.parent, mcp_server_dir
        )

        # Write back
        config_file.parent.mkdir(parents=True, exist_ok=True)
        with open(config_file, 'w') as f:
            json.dump(config, f, indent=2)
            f.write('\n')

        return True

    except Exception as e:
        print(f"{Colors.RED}Error updating {config_file}: {e}{Colors.NC}", file=sys.stderr)
        return False

def main():
    script_dir = Path(__file__).parent.resolve()
    project_root = find_project_root(script_dir)
    mcp_server_dir = project_root / "mcp-servers" / "project-management-automation"
    cursor_config_dir = project_root / ".cursor"

    print(f"{Colors.BLUE}=== MCP Configuration Sync for All Agents ==={Colors.NC}\n")

    # Verify MCP server exists
    if not mcp_server_dir.exists():
        print(f"{Colors.RED}Error: MCP server directory not found at {mcp_server_dir}{Colors.NC}", file=sys.stderr)
        sys.exit(1)

    run_server_sh = mcp_server_dir / "run_server.sh"
    if not run_server_sh.exists():
        print(f"{Colors.RED}Error: run_server.sh not found at {run_server_sh}{Colors.NC}", file=sys.stderr)
        sys.exit(1)

    # Make run_server.sh executable
    run_server_sh.chmod(0o755)

    # Check main project MCP config
    print(f"{Colors.BLUE}Checking main project MCP config...{Colors.NC}")
    main_config = cursor_config_dir / "mcp.json"
    is_correct, status = check_mcp_config(main_config)

    if is_correct:
        print(f"{Colors.GREEN}  ✅ Correctly configured{Colors.NC}")
    else:
        print(f"{Colors.YELLOW}  ⚠️  {status}{Colors.NC}")
        if status in ["not_found", "not_configured", "needs_update", "no_mcp_servers"]:
            response = input("  Update main project config? (y/N): ").strip().lower()
            if response == 'y':
                if update_mcp_config(main_config, mcp_server_dir):
                    print(f"{Colors.GREEN}  ✅ Updated{Colors.NC}")
                else:
                    print(f"{Colors.RED}  ❌ Update failed{Colors.NC}")

    # Check agent configs
    print(f"\n{Colors.BLUE}Checking agent configurations...{Colors.NC}")
    agents_dir = project_root / "agents"

    if agents_dir.exists():
        for agent_dir in sorted(agents_dir.iterdir()):
            if agent_dir.is_dir():
                agent_name = agent_dir.name
                agent_config = agent_dir / ".cursor" / "mcp.json"

                print(f"{Colors.BLUE}  Agent: {agent_name}{Colors.NC}")
                is_correct, status = check_mcp_config(agent_config)

                if is_correct:
                    print(f"{Colors.GREEN}    ✅ Correctly configured{Colors.NC}")
                else:
                    print(f"{Colors.YELLOW}    ⚠️  {status}{Colors.NC}")
                    if status in ["not_found", "not_configured", "needs_update", "no_mcp_servers"]:
                        response = input(f"    Update {agent_name} config? (y/N): ").strip().lower()
                        if response == 'y':
                            if update_mcp_config(agent_config, mcp_server_dir):
                                print(f"{Colors.GREEN}    ✅ Updated{Colors.NC}")
                            else:
                                print(f"{Colors.RED}    ❌ Update failed{Colors.NC}")

    # Generate example config
    print(f"\n{Colors.BLUE}Generating example configuration...{Colors.NC}")
    example_config = mcp_server_dir / "MCP_CONFIG_EXAMPLE.json"
    example_config_data = {
        "mcpServers": {
            "project-management-automation": get_mcp_server_config(project_root, mcp_server_dir)
        }
    }

    with open(example_config, 'w') as f:
        json.dump(example_config_data, f, indent=2)
        f.write('\n')

    print(f"{Colors.GREEN}  ✅ Example config saved to {example_config}{Colors.NC}")

    # Summary
    print(f"\n{Colors.BLUE}=== Summary ==={Colors.NC}")
    print(f"{Colors.GREEN}✅ MCP server path: {mcp_server_dir}{Colors.NC}")
    print(f"{Colors.GREEN}✅ Wrapper script: {run_server_sh}{Colors.NC}")
    print(f"\n{Colors.YELLOW}⚠️  Next steps:{Colors.NC}")
    print("  1. Review updated configurations")
    print("  2. Restart Cursor completely on each agent/server")
    print("  3. Verify MCP server appears in Cursor settings")
    print("  4. Test with: Check documentation health tool")

    print(f"\n{Colors.BLUE}=== Done ==={Colors.NC}")

if __name__ == "__main__":
    main()
