#!/usr/bin/env python3
"""
Exarp Todo2 Alignment Analysis Script

This script is called by Exarp daily automation.
It uses Exarp's internal functions for Todo2 alignment analysis.
"""

import sys
import json
from pathlib import Path

def main():
    """Main entry point for Exarp daily automation"""
    project_dir = Path(sys.argv[1] if len(sys.argv) > 1 else '.').resolve()
    
    try:
        from exarp_project_management.scripts import todo2_alignment as exarp_alignment
        exarp_alignment.main(str(project_dir))
        sys.exit(0)
    except ImportError:
        import subprocess
        try:
            result = subprocess.run(
                ['uvx', 'exarp', 'analyze-todo2-alignment', str(project_dir)],
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
            print(f"Warning: Could not execute alignment analysis: {e}", file=sys.stderr)
            sys.exit(0)

if __name__ == '__main__':
    main()
