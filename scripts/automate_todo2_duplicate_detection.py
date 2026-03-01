#!/usr/bin/env python3
"""
Exarp Todo2 Duplicate Detection Script (legacy / fallback).

Prefer using the exarp-go MCP server in Cursor for duplicate detection.
This script is an optional fallback when the Python package is installed or
when `uvx exarp` is available. See docs/EXARP_GO_MIGRATION_LEFTOVERS.md.
"""

import sys
import json
from pathlib import Path

def main():
    """Main entry point for Exarp daily automation"""
    project_dir = Path(sys.argv[1] if len(sys.argv) > 1 else '.').resolve()

    try:
        from exarp_project_management.scripts import duplicate_detection as exarp_duplicates
        exarp_duplicates.main(str(project_dir))
        sys.exit(0)
    except ImportError:
        import subprocess
        try:
            result = subprocess.run(
                ['uvx', 'exarp', 'detect-duplicate-tasks', str(project_dir)],
                capture_output=True,
                text=True,
                timeout=300
            )
            if result.returncode == 0:
                print(result.stdout)
                sys.exit(0)
            else:
                print(f"Error: {result.stderr}", file=sys.stderr)
                sys.exit(result.returncode)
        except (subprocess.TimeoutExpired, FileNotFoundError) as e:
            print(f"Warning: Could not execute duplicate detection: {e}", file=sys.stderr)
            print("Prefer exarp-go MCP in Cursor, or install: pip install exarp-automation-mcp / uvx exarp", file=sys.stderr)
            sys.exit(0)

if __name__ == '__main__':
    main()
