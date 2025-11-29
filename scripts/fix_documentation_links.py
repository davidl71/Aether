#!/usr/bin/env python3
"""
Fix broken documentation links automatically.

This script finds broken internal markdown links and attempts to fix them
by searching for alternative paths (case variations, subdirectories, etc.).
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

def find_alternative_path(link_path, base_file, docs_dir):
    """Try to find alternative paths for broken links"""
    base_dir = base_file.parent
    link_name = Path(link_path).name
    
    # Try exact match first
    if (base_dir / link_path).exists():
        return base_dir / link_path
    
    # Try common alternatives
    alternatives = [
        link_path.lower(),
        link_path.upper(),
        link_path.replace('_', '-'),
        link_path.replace('-', '_'),
        link_name.lower(),
        link_name.upper(),
    ]
    
    # Search in various locations
    search_dirs = [
        base_dir,
        docs_dir,
        docs_dir / 'research',
        docs_dir / 'research' / 'architecture',
        docs_dir / 'research' / 'integration',
        docs_dir / 'research' / 'external',
        docs_dir / 'platform',
    ]
    
    for alt in alternatives:
        for search_dir in search_dirs:
            target = search_dir / alt
            if target.exists():
                # Calculate relative path from base_file to target
                try:
                    # Use relative_to with docs_dir as base for consistent paths
                    rel_from_docs = target.relative_to(docs_dir)
                    base_from_docs = base_dir.relative_to(docs_dir)
                    
                    # Calculate relative path
                    if base_from_docs == Path('.'):
                        return str(rel_from_docs)
                    else:
                        # Count how many levels up we need to go
                        levels_up = len(base_from_docs.parts)
                        rel_path = Path('../' * levels_up) / rel_from_docs
                        return str(rel_path)
                except ValueError:
                    # If paths don't share common base, use absolute from docs
                    rel_from_docs = target.relative_to(docs_dir)
                    return str(rel_from_docs)
    
    return None

def fix_links_in_file(md_file, docs_dir, dry_run=True):
    """Fix broken links in a markdown file"""
    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')
        fixes = []
        
        for i, line in enumerate(lines):
            # Find all markdown links
            matches = list(re.finditer(r'\[([^\]]+)\]\(([^)]+)\)', line))
            if not matches:
                continue
            
            for match in matches:
                link_text, link_path = match.groups()
                
                # Skip external links and anchors
                if link_path.startswith('http') or link_path.startswith('#'):
                    continue
                
                # Resolve relative path
                if link_path.startswith('./') or not link_path.startswith('/'):
                    base_dir = md_file.parent
                    target_path = (base_dir / link_path).resolve()
                    
                    if not target_path.exists():
                        # Try to find alternative
                        alt_path = find_alternative_path(link_path, md_file, docs_dir)
                        if alt_path:
                            # Fix the link
                            old_link = f'[{link_text}]({link_path})'
                            new_link = f'[{link_text}]({alt_path})'
                            lines[i] = lines[i].replace(old_link, new_link)
                            fixes.append({
                                'line': i + 1,
                                'old': link_path,
                                'new': alt_path
                            })
        
        if fixes and not dry_run:
            md_file.write_text('\n'.join(lines), encoding='utf-8')
        
        return fixes
    except Exception as e:
        print(f"Error processing {md_file}: {e}", file=sys.stderr)
        return []

def main():
    docs_dir = Path("docs")
    dry_run = '--dry-run' in sys.argv
    
    if dry_run:
        print("DRY RUN MODE - No files will be modified\n")
    
    all_fixes = []
    files_fixed = 0
    
    # Process all markdown files
    for md_file in sorted(docs_dir.rglob("*.md")):
        fixes = fix_links_in_file(md_file, docs_dir, dry_run=dry_run)
        if fixes:
            files_fixed += 1
            rel_path = md_file.relative_to(docs_dir)
            print(f"{rel_path}: {len(fixes)} fixes")
            for fix in fixes:
                print(f"  Line {fix['line']}: {fix['old']} -> {fix['new']}")
            all_fixes.extend(fixes)
    
    print(f"\n{'Would fix' if dry_run else 'Fixed'} {len(all_fixes)} broken links in {files_fixed} files")
    
    if dry_run and all_fixes:
        print("\nRun without --dry-run to apply fixes")

if __name__ == '__main__':
    main()
