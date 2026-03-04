#!/usr/bin/env python3
"""
Sync MCP server configuration across agents.

Expects exarp-go (released/installed) as the project automation server. Validates
and optionally updates .cursor/mcp.json to use run_exarp_go.sh; does not emit
or require the legacy Python package (project_management_automation).
"""

import json
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

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

def get_exarp_go_config(project_root: Path) -> Dict[str, Any]:
    """Exarp-go MCP entry using run_exarp_go.sh (prefer installed exarp-go on PATH)."""
    return {
        "command": "{{PROJECT_ROOT}}/scripts/run_exarp_go.sh",
        "args": [],
        "env": {
            "PROJECT_ROOT": "{{PROJECT_ROOT}}",
            "EXARP_WATCH": "0"
        },
        "description": "exarp-go MCP: docs health, task alignment, duplicates, security. Set workingDirectory to project root."
    }

def check_mcp_config(config_file: Path) -> Tuple[bool, Optional[str]]:
    """Check if MCP config has exarp-go (run_exarp_go.sh or exarp-go binary)."""
    if not config_file.exists():
        return False, "not_found"

    try:
        with open(config_file, 'r') as f:
            config = json.load(f)

        if 'mcpServers' not in config:
            return False, "no_mcp_servers"

        # exarp-go key
        if 'exarp-go' in config['mcpServers']:
            return True, "correct"

        # exarp key with exarp-go command
        if 'exarp' in config['mcpServers']:
            server = config['mcpServers']['exarp']
            cmd = server.get('command', '')
            if 'exarp-go' in cmd or 'run_exarp_go.sh' in str(cmd):
                return True, "correct"
            # Legacy Python - treat as needs update
            return False, "needs_update_exarp_go"

        for old in ('automa', 'project-management-automation'):
            if old in config['mcpServers']:
                return False, f"old_name_{old}"

        return False, "not_configured"

    except json.JSONDecodeError as e:
        return False, f"invalid_json: {e}"
    except Exception as e:
        return False, f"error: {e}"

def update_mcp_config(config_file: Path, project_root: Path) -> bool:
    """Write exarp-go entry; remove legacy exarp/automa keys."""
    try:
        if config_file.exists():
            backup = config_file.with_suffix('.json.bak')
            if not backup.exists():
                import shutil
                shutil.copy2(config_file, backup)

        if config_file.exists():
            with open(config_file, 'r') as f:
                config = json.load(f)
        else:
            config = {}

        if 'mcpServers' not in config:
            config['mcpServers'] = {}

        for old in ('automa', 'project-management-automation', 'exarp'):
            config['mcpServers'].pop(old, None)

        config['mcpServers']['exarp-go'] = get_exarp_go_config(project_root)

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

    print(f"{Colors.BLUE}=== MCP configuration (exarp-go) ==={Colors.NC}\n")

    main_config = cursor_config_dir / "mcp.json"
    is_correct, status = check_mcp_config(main_config)

    if is_correct:
        print(f"{Colors.GREEN}  ✅ exarp-go configured{Colors.NC}")
    else:
        print(f"{Colors.YELLOW}  ⚠️  {status}{Colors.NC}")
        if status and status not in ("invalid_json", "error"):
            response = input("  Update to exarp-go? (y/N): ").strip().lower()
            if response == 'y':
                if update_mcp_config(main_config, project_root):
                    print(f"{Colors.GREEN}  ✅ Updated{Colors.NC}")
                else:
                    print(f"{Colors.RED}  ❌ Update failed{Colors.NC}")

    agents_dir = project_root / "agents"
    if agents_dir.exists():
        for agent_dir in sorted(agents_dir.iterdir()):
            if not agent_dir.is_dir():
                continue
            agent_config = agent_dir / ".cursor" / "mcp.json"
            print(f"{Colors.BLUE}  Agent: {agent_dir.name}{Colors.NC}")
            ac_ok, ac_status = check_mcp_config(agent_config)
            if ac_ok:
                print(f"{Colors.GREEN}    ✅ exarp-go configured{Colors.NC}")
            else:
                print(f"{Colors.YELLOW}    ⚠️  {ac_status}{Colors.NC}")
                if ac_status and ac_status not in ("invalid_json", "error"):
                    r = input("    Update? (y/N): ").strip().lower()
                    if r == 'y' and update_mcp_config(agent_config, project_root):
                        print(f"{Colors.GREEN}    ✅ Updated{Colors.NC}")

    docs_dir = project_root / "docs"
    example_file = docs_dir / "MCP_CONFIG_EXAMPLE.json"
    example = {"mcpServers": {"exarp-go": get_exarp_go_config(project_root)}}
    docs_dir.mkdir(exist_ok=True)
    with open(example_file, 'w') as f:
        json.dump(example, f, indent=2)
        f.write('\n')
    print(f"\n{Colors.GREEN}  Example: {example_file}{Colors.NC}")

    print(f"\n{Colors.YELLOW}Next: Restart Cursor; use exarp-go MCP tools with workingDirectory = project root.{Colors.NC}")
    print(f"{Colors.BLUE}=== Done ==={Colors.NC}\n")

if __name__ == "__main__":
    main()
