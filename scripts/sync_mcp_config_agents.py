#!/usr/bin/env python3
"""
Sync MCP server configuration across all agents.

Supports both exarp-go (Go binary) and legacy Python package. Prefers exarp-go
when present in .cursor/mcp.json; does not overwrite exarp-go with Python config.
"""

import json
import sys
import subprocess
from pathlib import Path
from typing import Dict, Any, Optional, Tuple

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

def check_package_installed() -> Tuple[bool, str]:
    """Check if project_management_automation package is installed."""
    try:
        result = subprocess.run(
            [sys.executable, '-m', 'project_management_automation.server', '--help'],
            capture_output=True,
            text=True,
            timeout=5
        )
        return True, "installed"
    except subprocess.TimeoutExpired:
        return False, "timeout"
    except FileNotFoundError:
        return False, "not_found"
    except Exception as e:
        # Try importing to see if package exists
        try:
            import project_management_automation.server
            return True, "installed"
        except ImportError:
            return False, f"not_installed: {str(e)}"

def get_mcp_server_config() -> Dict[str, Any]:
    """Generate MCP server configuration entry for Python package."""
    return {
        "command": "python3",
        "args": [
            "-m",
            "project_management_automation.server"
        ],
        "description": (
            "Exarp - Project management automation tools (Enochian: Spirit of Air - Communication) - "
            "documentation health, task alignment, duplicate detection, security scanning, and automation opportunities"
        )
    }

def check_mcp_config(config_file: Path) -> Tuple[bool, Optional[str]]:
    """Check if MCP config exists and has correct entry."""
    if not config_file.exists():
        return False, "not_found"

    try:
        with open(config_file, 'r') as f:
            config = json.load(f)

        if 'mcpServers' not in config:
            return False, "no_mcp_servers"

        # Check for exarp server (exarp-go or legacy Python)
        if 'exarp' in config['mcpServers']:
            server_config = config['mcpServers']['exarp']
            command = server_config.get('command', '')
            args = server_config.get('args', [])
            # exarp-go: command is path to binary (e.g. .../exarp-go/bin/exarp-go)
            if 'exarp-go' in command or (args and any('exarp-go' in str(a) for a in args)):
                return True, "correct"
            # Legacy Python module
            if command == 'python3' and '-m' in args and 'project_management_automation.server' in args:
                return True, "correct"
            return False, "needs_update"
        # exarp-go may be registered under "exarp-go" key
        if 'exarp-go' in config['mcpServers']:
            return True, "correct"

        # Check for old names (automa, project-management-automation)
        old_names = ['automa', 'project-management-automation']
        for old_name in old_names:
            if old_name in config['mcpServers']:
                return False, f"old_name_{old_name}"

        return False, "not_configured"

    except json.JSONDecodeError as e:
        return False, f"invalid_json: {e}"
    except Exception as e:
        return False, f"error: {e}"

def update_mcp_config(config_file: Path) -> bool:
    """Update MCP config with correct entry."""
    try:
        # Backup existing config
        if config_file.exists():
            backup_file = config_file.with_suffix('.json.bak')
            if not backup_file.exists():
                import shutil
                shutil.copy2(config_file, backup_file)

        # Load or create config
        if config_file.exists():
            with open(config_file, 'r') as f:
                config = json.load(f)
        else:
            config = {}

        # Ensure mcpServers exists
        if 'mcpServers' not in config:
            config['mcpServers'] = {}

        # Remove old name entries if they exist
        old_names = ['automa', 'project-management-automation']
        for old_name in old_names:
            if old_name in config['mcpServers']:
                del config['mcpServers'][old_name]

        # Update exarp entry (current name)
        config['mcpServers']['exarp'] = get_mcp_server_config()

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
    cursor_config_dir = project_root / ".cursor"

    print(f"{Colors.BLUE}=== MCP Configuration Sync for All Agents ==={Colors.NC}\n")

    # Check if package is installed
    print(f"{Colors.BLUE}Checking package installation...{Colors.NC}")
    is_installed, status = check_package_installed()

    if is_installed:
        print(f"{Colors.GREEN}  ✅ Package installed{Colors.NC}")
    else:
        print(f"{Colors.YELLOW}  ⚠️  Package not installed: {status}{Colors.NC}")
        print(f"\n{Colors.YELLOW}Install the package with:{Colors.NC}")
        print(f"  pip install -e /path/to/project-management-automation")
        print(f"  # or")
        print(f"  pip install git+ssh://git@github.com/davidl71/project-management-automation.git@main")

        response = input(f"\nContinue anyway? (y/N): ").strip().lower()
        if response != 'y':
            print(f"{Colors.RED}Aborted.{Colors.NC}")
            sys.exit(1)

    # Check main project MCP config
    print(f"\n{Colors.BLUE}Checking main project MCP config...{Colors.NC}")
    main_config = cursor_config_dir / "mcp.json"
    is_correct, status = check_mcp_config(main_config)

    if is_correct:
        print(f"{Colors.GREEN}  ✅ Correctly configured{Colors.NC}")
    else:
        print(f"{Colors.YELLOW}  ⚠️  {status}{Colors.NC}")
        if status and status not in ["invalid_json", "error"]:
            response = input("  Update main project config? (y/N): ").strip().lower()
            if response == 'y':
                if update_mcp_config(main_config):
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
                    if status and status not in ["invalid_json", "error"]:
                        response = input(f"    Update {agent_name} config? (y/N): ").strip().lower()
                        if response == 'y':
                            if update_mcp_config(agent_config):
                                print(f"{Colors.GREEN}    ✅ Updated{Colors.NC}")
                            else:
                                print(f"{Colors.RED}    ❌ Update failed{Colors.NC}")
    else:
        print(f"{Colors.YELLOW}  No agents directory found at {agents_dir}{Colors.NC}")

    # Generate example config in docs
    print(f"\n{Colors.BLUE}Generating example configuration...{Colors.NC}")
    docs_dir = project_root / "docs"
    example_config = docs_dir / "MCP_CONFIG_EXAMPLE.json"
    example_config_data = {
        "mcpServers": {
            "exarp": get_mcp_server_config()
        }
    }

    docs_dir.mkdir(exist_ok=True)
    with open(example_config, 'w') as f:
        json.dump(example_config_data, f, indent=2)
        f.write('\n')

    print(f"{Colors.GREEN}  ✅ Example config saved to {example_config}{Colors.NC}")

    # Summary
    print(f"\n{Colors.BLUE}=== Summary ==={Colors.NC}")
    if main_config.exists():
        try:
            with open(main_config) as f:
                c = json.load(f)
            servers = c.get('mcpServers', {})
            if 'exarp-go' in servers or ('exarp' in servers and 'exarp-go' in str(servers.get('exarp', {}).get('command', ''))):
                print(f"{Colors.GREEN}✅ Configuration uses exarp-go (Go MCP server){Colors.NC}")
            else:
                print(f"{Colors.GREEN}✅ Configuration uses exarp (Python or exarp-go){Colors.NC}")
        except Exception:
            print(f"{Colors.GREEN}✅ Configuration check complete{Colors.NC}")
    else:
        print(f"{Colors.GREEN}✅ Configuration check complete{Colors.NC}")
    if not is_installed:
        print(f"{Colors.YELLOW}⚠️  Package installation status: {status}{Colors.NC}")
    print(f"\n{Colors.YELLOW}⚠️  Next steps:{Colors.NC}")
    if not is_installed:
        print("  1. Install the package: pip install -e /path/to/project-management-automation")
        print("  2. Review updated configurations")
    else:
        print("  1. Review updated configurations")
    print("  2. Restart Cursor completely on each agent/server")
    print("  3. Verify MCP server appears in Cursor settings")
    print("  4. Test with: Check documentation health tool")

    print(f"\n{Colors.BLUE}=== Done ==={Colors.NC}")

if __name__ == "__main__":
    main()
