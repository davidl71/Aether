#!/usr/bin/env python3
"""
Automatically sync global documentation files for Cursor IDE.

This script:
1. Validates all documentation files exist
2. Generates path lists for easy copy-paste
3. Can detect new documentation files
4. Updates the configuration file

Usage:
    python3 scripts/sync_global_docs.py [--check] [--update-config] [--generate-paths]
"""

import json
import os
import sys
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any

# Colors for terminal output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'  # No Color

def load_config(config_path: Path) -> Dict[str, Any]:
    """Load the global docs configuration file."""
    with open(config_path, 'r') as f:
        return json.load(f)

def save_config(config: Dict[str, Any], config_path: Path) -> None:
    """Save the global docs configuration file."""
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)
        f.write('\n')

def check_file_exists(project_root: Path, file_path: str) -> bool:
    """Check if a documentation file exists."""
    full_path = project_root / file_path
    return full_path.exists() and full_path.is_file()

def validate_files(project_root: Path, config: Dict[str, Any]) -> tuple[List[str], List[str]]:
    """Validate all files in the config exist."""
    all_files = []
    missing_files = []

    for category in ['highPriority', 'external', 'secondary']:
        if category in config:
            for item in config[category]:
                file_path = item['path']
                all_files.append(file_path)
                if not check_file_exists(project_root, file_path):
                    missing_files.append(file_path)

    return all_files, missing_files

def generate_paths_file(project_root: Path, config: Dict[str, Any], output_path: Path) -> None:
    """Generate a file with absolute paths for easy copy-paste."""
    with open(output_path, 'w') as f:
        f.write("# Cursor Global Docs - File Paths\n")
        f.write(f"# Generated: {datetime.now().isoformat()}\n")
        f.write("# Use these paths in Cursor Settings → Features → Docs\n")
        f.write("\n")

        f.write("--- High-Priority Files (Must-Have) ---\n")
        for item in config.get('highPriority', []):
            full_path = project_root / item['path']
            f.write(f"{full_path}\n")

        f.write("\n--- External Documentation (Optional) ---\n")
        for item in config.get('external', []):
            full_path = project_root / item['path']
            f.write(f"{full_path}\n")

        f.write("\n--- Secondary Documentation (Optional) ---\n")
        for item in config.get('secondary', []):
            full_path = project_root / item['path']
            f.write(f"{full_path}\n")

def generate_relative_paths_file(config: Dict[str, Any], output_path: Path) -> None:
    """Generate a file with relative paths."""
    with open(output_path, 'w') as f:
        f.write("# Cursor Global Docs - Relative Paths\n")
        f.write(f"# Generated: {datetime.now().isoformat()}\n")
        f.write("\n")

        f.write("--- High-Priority Files ---\n")
        for item in config.get('highPriority', []):
            f.write(f"{item['path']}\n")

        f.write("\n--- External Documentation ---\n")
        for item in config.get('external', []):
            f.write(f"{item['path']}\n")

        f.write("\n--- Secondary Documentation ---\n")
        for item in config.get('secondary', []):
            f.write(f"{item['path']}\n")

def detect_new_files(project_root: Path, config: Dict[str, Any], docs_dir: Path) -> List[str]:
    """Detect new markdown files in docs/ that aren't in config."""
    all_configured = set()
    for category in ['highPriority', 'external', 'secondary']:
        if category in config:
            for item in config[category]:
                all_configured.add(item['path'])

    new_files = []
    for md_file in docs_dir.rglob('*.md'):
        relative_path = md_file.relative_to(project_root)
        path_str = str(relative_path).replace('\\', '/')
        if path_str not in all_configured:
            new_files.append(path_str)

    return sorted(new_files)

def main():
    parser = argparse.ArgumentParser(
        description='Sync global documentation files for Cursor IDE'
    )
    parser.add_argument('--check', action='store_true',
                       help='Only check files, do not update')
    parser.add_argument('--update-config', action='store_true',
                       help='Update lastUpdated timestamp in config')
    parser.add_argument('--generate-paths', action='store_true',
                       help='Generate path files for copy-paste')
    parser.add_argument('--detect-new', action='store_true',
                       help='Detect new documentation files')
    args = parser.parse_args()

    # Get project root
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    # Load config
    config_path = project_root / '.cursor' / 'global-docs.json'
    if not config_path.exists():
        print(f"{Colors.RED}Error: Config file not found: {config_path}{Colors.NC}")
        sys.exit(1)

    config = load_config(config_path)

    print(f"{Colors.BLUE}========================================{Colors.NC}")
    print(f"{Colors.BLUE}Cursor Global Docs - Sync Script{Colors.NC}")
    print(f"{Colors.BLUE}========================================{Colors.NC}")
    print()

    # Validate files
    print(f"{Colors.BLUE}Validating documentation files...{Colors.NC}")
    all_files, missing_files = validate_files(project_root, config)

    print()
    for file_path in all_files:
        if file_path in missing_files:
            print(f"{Colors.RED}✗{Colors.NC} {file_path} {Colors.RED}(MISSING){Colors.NC}")
        else:
            print(f"{Colors.GREEN}✓{Colors.NC} {file_path}")

    print()
    print(f"{Colors.BLUE}Summary:{Colors.NC}")
    print(f"  Total files: {len(all_files)}")
    print(f"  Found: {len(all_files) - len(missing_files)}")
    print(f"  Missing: {len(missing_files)}")

    if missing_files:
        print()
        print(f"{Colors.RED}Missing files:{Colors.NC}")
        for file_path in missing_files:
            print(f"  - {file_path}")
        if not args.check:
            sys.exit(1)
    else:
        print()
        print(f"{Colors.GREEN}All documentation files are present!{Colors.NC}")

    # Detect new files
    if args.detect_new:
        print()
        print(f"{Colors.BLUE}Detecting new documentation files...{Colors.NC}")
        docs_dir = project_root / 'docs'
        new_files = detect_new_files(project_root, config, docs_dir)
        if new_files:
            print(f"{Colors.YELLOW}New files found (not in config):{Colors.NC}")
            for file_path in new_files:
                print(f"  - {file_path}")
        else:
            print(f"{Colors.GREEN}No new files detected{Colors.NC}")

    # Generate path files
    if args.generate_paths or not args.check:
        print()
        print(f"{Colors.BLUE}Generating path files...{Colors.NC}")

        paths_file = project_root / '.cursor' / 'global-docs-paths.txt'
        generate_paths_file(project_root, config, paths_file)
        print(f"{Colors.GREEN}Absolute paths: {paths_file}{Colors.NC}")

        relative_paths_file = project_root / '.cursor' / 'global-docs-paths-relative.txt'
        generate_relative_paths_file(config, relative_paths_file)
        print(f"{Colors.GREEN}Relative paths: {relative_paths_file}{Colors.NC}")

    # Update config
    if args.update_config and not args.check:
        print()
        print(f"{Colors.BLUE}Updating config file...{Colors.NC}")
        config['lastUpdated'] = datetime.now().strftime('%Y-%m-%d')
        save_config(config, config_path)
        print(f"{Colors.GREEN}Config updated{Colors.NC}")

    print()
    print(f"{Colors.GREEN}Done!{Colors.NC}")

if __name__ == '__main__':
    main()
