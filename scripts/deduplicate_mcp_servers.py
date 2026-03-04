#!/usr/bin/env python3
"""
Remove duplicate MCP servers from mcp.json configuration.

Detects duplicates by:
1. Exact (command, args) tuple - same config under two keys.
2. Same server identity - e.g. two keys both running
   @modelcontextprotocol/server-sequential-thinking (keeps first key).
"""
import json
import re
import sys
from collections import OrderedDict


def server_identity(server_config: dict) -> tuple:
    """Return a tuple that identifies the server for deduplication."""
    command = server_config.get("command", "")
    args = tuple(server_config.get("args", []))
    # Same npm package with no extra args => same server (e.g. sequential_thinking)
    if (
        command == "npx"
        and len(args) == 2
        and args[0] == "-y"
        and isinstance(args[1], str)
        and args[1].startswith("@")
    ):
        return ("npx_pkg", args[1])
    if (
        command == "npx"
        and len(args) == 2
        and args[0] == "-y"
        and isinstance(args[1], str)
        and re.match(r"^[a-z][a-z0-9_-]*$", args[1])
    ):
        return ("npx_pkg", args[1])
    return ("config", command, args)


def deduplicate_mcp_servers(config: dict) -> dict:
    """Remove duplicate MCP servers from configuration."""
    servers = config.get("mcpServers", {})
    unique_servers = OrderedDict()
    seen_configs = {}
    seen_identities = {}
    duplicates_removed = []

    for name, server_config in servers.items():
        ident = server_identity(server_config)
        config_key = (server_config.get("command", ""), tuple(server_config.get("args", [])))

        # Duplicate by exact (command, args)
        if config_key in seen_configs:
            original_name = seen_configs[config_key]
            duplicates_removed.append(f"{name} (duplicate of {original_name})")
            print(
                f"Removing duplicate: {name} (same config as {original_name})",
                file=sys.stderr,
            )
            continue

        # Duplicate by server identity (e.g. same npm package, different key)
        if ident in seen_identities:
            original_name = seen_identities[ident]
            duplicates_removed.append(f"{name} (same server as {original_name})")
            print(
                f"Removing duplicate: {name} (same server as {original_name})",
                file=sys.stderr,
            )
            continue

        seen_configs[config_key] = name
        seen_identities[ident] = name
        unique_servers[name] = server_config

    config["mcpServers"] = unique_servers

    return {
        "config": config,
        "duplicates_removed": duplicates_removed,
        "count": len(duplicates_removed),
    }


def main():
    """Main entry point. Usage: deduplicate_mcp_servers.py [path/to/mcp.json]"""
    if len(sys.argv) > 1:
        path = sys.argv[1]
        with open(path, "r") as f:
            config = json.load(f)
    else:
        path = None
        config_str = sys.stdin.read()
        config = json.loads(config_str)

    result = deduplicate_mcp_servers(config)
    if path and result["count"] > 0:
        with open(path, "w") as f:
            json.dump(result["config"], f, indent=2)
        print(f"Removed {result['count']} duplicate(s); wrote {path}", file=sys.stderr)
    elif path:
        with open(path, "w") as f:
            json.dump(result["config"], f, indent=2)
        print("No duplicates found; config unchanged.", file=sys.stderr)
    else:
        print(json.dumps(result))


if __name__ == "__main__":
    main()
