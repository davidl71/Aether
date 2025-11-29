#!/usr/bin/env python3
"""
Fix remaining broken documentation links by finding files with similar names.
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

def find_file_by_name(target_name, docs_dir, base_file):
    """Find a file by name, trying various matching strategies"""
    base_dir = base_file.parent
    target_stem = Path(target_name).stem.lower()
    
    # Search all markdown files
    candidates = []
    for md_file in docs_dir.rglob("*.md"):
        if md_file.stem.lower() == target_stem or md_file.name.lower() == target_name.lower():
            # Calculate relative path from base_file
            try:
                rel_path = md_file.relative_to(base_dir)
                candidates.append((md_file, str(rel_path)))
            except ValueError:
                # Files don't share common path, use absolute from docs
                rel_path = md_file.relative_to(docs_dir)
                # Calculate relative path manually
                base_parts = base_dir.relative_to(docs_dir).parts
                if base_parts == ():
                    candidates.append((md_file, str(rel_path)))
                else:
                    up_levels = len(base_parts)
                    rel_path_str = '../' * up_levels + str(rel_path)
                    candidates.append((md_file, rel_path_str))
    
    # Prefer files in same directory or nearby
    if candidates:
        # Sort by path similarity
        candidates.sort(key=lambda x: (
            x[0].parent != base_dir,  # Same directory first
            len(Path(x[1]).parts)  # Shorter paths first
        ))
        return candidates[0][1]
    
    return None

def fix_links_in_file(md_file, docs_dir, dry_run=True):
    """Fix broken links in a markdown file"""
    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')
        fixes = []
        
        for i, line in enumerate(lines):
            matches = list(re.finditer(r'\[([^\]]+)\]\(([^)]+)\)', line))
            if not matches:
                continue
            
            for match in matches:
                link_text, link_path = match.groups()
                
                if link_path.startswith('http') or link_path.startswith('#'):
                    continue
                
                if link_path.startswith('./') or not link_path.startswith('/'):
                    base_dir = md_file.parent
                    target_path = (base_dir / link_path).resolve()
                    
                    if not target_path.exists():
                        # Try to find by name
                        link_name = Path(link_path).name
                        alt_path = find_file_by_name(link_name, docs_dir, md_file)
                        
                        if alt_path:
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
    
    for md_file in sorted(docs_dir.rglob("*.md")):
        fixes = fix_links_in_file(md_file, docs_dir, dry_run=dry_run)
        if fixes:
            files_fixed += 1
            rel_path = md_file.relative_to(docs_dir)
            print(f"{rel_path}: {len(fixes)} fixes")
            for fix in fixes[:3]:  # Show first 3
                print(f"  Line {fix['line']}: {fix['old']} -> {fix['new']}")
            if len(fixes) > 3:
                print(f"  ... and {len(fixes) - 3} more")
            all_fixes.extend(fixes)
    
    print(f"\n{'Would fix' if dry_run else 'Fixed'} {len(all_fixes)} broken links in {files_fixed} files")
    
    if dry_run and all_fixes:
        print("\nRun without --dry-run to apply fixes")

if __name__ == '__main__':
    main()
