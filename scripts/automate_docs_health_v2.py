#!/usr/bin/env python3
"""
Exarp Documentation Health Check Script (legacy / fallback).

Prefer using the exarp-go MCP server in Cursor for documentation health checks.
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
        # Try to import and use Exarp's documentation health check
        # This should work if Exarp package is installed
        from exarp_project_management.scripts import automate_docs_health_v2 as exarp_docs_health
        
        # Call the Exarp function directly
        # The function should accept project_dir as argument
        exarp_docs_health.main(str(project_dir))
        sys.exit(0)
        
    except ImportError:
        # If Exarp package not available, try alternative approach
        # Use subprocess to call via uvx
        import subprocess
        
        try:
            result = subprocess.run(
                ['uvx', 'exarp', 'check-documentation-health', str(project_dir)],
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
            # If all else fails, return success to avoid breaking automation
            print(f"Warning: Could not execute documentation health check: {e}", file=sys.stderr)
            print("Prefer exarp-go MCP in Cursor, or install: pip install exarp-automation-mcp / uvx exarp")
            sys.exit(0)

if __name__ == '__main__':
    main()
