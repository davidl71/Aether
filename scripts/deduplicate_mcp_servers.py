#!/usr/bin/env python3
"""
Remove duplicate MCP servers from mcp.json configuration.

Detects duplicates based on (command, args) tuple and keeps the first occurrence.
"""
import json
import sys
from collections import OrderedDict


def deduplicate_mcp_servers(config: dict) -> dict:
    """Remove duplicate MCP servers from configuration."""
    servers = config.get("mcpServers", {})
    unique_servers = OrderedDict()
    seen_configs = {}
    duplicates_removed = []

    for name, server_config in servers.items():
        # Create a normalized key based on command and args
        command = server_config.get("command", "")
        args = tuple(server_config.get("args", []))
        config_key = (command, args)

        # If we've seen this exact config before, skip it (duplicate)
        if config_key in seen_configs:
            original_name = seen_configs[config_key]
            duplicates_removed.append(f"{name} (duplicate of {original_name})")
            print(
                f"Removing duplicate: {name} (same as {original_name})", file=sys.stderr
            )
            continue

        # Track this config
        seen_configs[config_key] = name
        unique_servers[name] = server_config

    config["mcpServers"] = unique_servers

    # Return result with metadata
    return {
        "config": config,
        "duplicates_removed": duplicates_removed,
        "count": len(duplicates_removed),
    }


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        # Read from file
        with open(sys.argv[1], "r") as f:
            config = json.load(f)
    else:
        # Read from stdin
        config_str = sys.stdin.read()
        config = json.loads(config_str)

    result = deduplicate_mcp_servers(config)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
