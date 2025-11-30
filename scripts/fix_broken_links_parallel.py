#!/usr/bin/env python3
"""
Parallel broken link fixer - fixes broken documentation links in parallel.
"""

import json
import re
import sys
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
from typing import List, Dict, Optional

def find_file_by_name(target_name: str, docs_dir: Path, base_file: Path) -> Optional[str]:
    """Find a file by name (name-based approach)"""
    base_dir = base_file.parent
    target_stem = Path(target_name).stem.lower()
    
    candidates = []
    for md_file in docs_dir.rglob("*.md"):
        if md_file.stem.lower() == target_stem or md_file.name.lower() == target_name.lower():
            try:
                rel_path = md_file.relative_to(base_dir)
                candidates.append((md_file, str(rel_path)))
            except ValueError:
                rel_path = md_file.relative_to(docs_dir)
                base_parts = base_dir.relative_to(docs_dir).parts
                if base_parts == ():
                    candidates.append((md_file, str(rel_path)))
                else:
                    up_levels = len(base_parts)
                    rel_path_str = '../' * up_levels + str(rel_path)
                    candidates.append((md_file, rel_path_str))
    
    if candidates:
        candidates.sort(key=lambda x: (
            x[0].parent != base_dir,
            len(Path(x[1]).parts)
        ))
        return candidates[0][1]
    
    return None

def fix_links_in_file(args_tuple):
    """Fix broken links in a single file (runs in parallel process)"""
    md_file_str, docs_dir_str = args_tuple
    md_file = Path(md_file_str)
    docs_dir = Path(docs_dir_str)
    
    fixes = []
    
    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')
        
        for line_num, line in enumerate(lines, 1):
            matches = list(re.finditer(r'\[([^\]]+)\]\(([^)]+)\)', line))
            for match in matches:
                link_text, link_path = match.groups()
                
                # Skip external links, anchors, and mailto
                if link_path.startswith('http') or link_path.startswith('#') or link_path.startswith('mailto:'):
                    continue
                
                # Skip code references (looks like function parameters)
                if 'const ' in link_path or '&' in link_path or link_path.startswith('*'):
                    continue
                
                # Check if link is broken
                if link_path.startswith('./') or not link_path.startswith('/'):
                    base_dir = md_file.parent
                    target_path = (base_dir / link_path).resolve()
                    
                    if not target_path.exists():
                        # Try to find alternative
                        alt_path = find_file_by_name(link_path, docs_dir, md_file)
                        
                        if alt_path:
                            old_link = f'[{link_text}]({link_path})'
                            new_link = f'[{link_text}]({alt_path})'
                            
                            # Update the line
                            lines[line_num - 1] = lines[line_num - 1].replace(old_link, new_link)
                            
                            fixes.append({
                                'line': line_num,
                                'old': old_link,
                                'new': new_link,
                                'method': 'name_based'
                            })
                        else:
                            # Can't fix - might be intentional or file doesn't exist
                            # For now, we'll skip these
                            pass
        
        # Write back if fixes were made
        if fixes:
            md_file.write_text('\n'.join(lines), encoding='utf-8')
            
    except Exception as e:
        return {
            'file': str(md_file),
            'success': False,
            'error': str(e),
            'fixes': []
        }
    
    return {
        'file': str(md_file),
        'success': True,
        'fixes': fixes
    }

def main():
    """Main execution - fixes broken links in parallel"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Fix broken documentation links in parallel')
    parser.add_argument('--docs-dir', type=str, default='docs', help='Documentation directory')
    parser.add_argument('--workers', type=int, default=8, help='Number of parallel workers')
    parser.add_argument('--broken-links-file', type=str, default='docs/BROKEN_LINKS_REPORT.json',
                       help='JSON file with broken links report')
    
    args = parser.parse_args()
    
    docs_dir = Path(args.docs_dir).resolve()
    if not docs_dir.exists():
        print(f"❌ Error: Documentation directory not found: {docs_dir}")
        sys.exit(1)
    
    # Load broken links report
    broken_links_file = Path(args.broken_links_file)
    if broken_links_file.exists():
        with open(broken_links_file, 'r') as f:
            broken_links_data = json.load(f)
        
        # Get unique files with broken links
        files_to_fix = set(link['file'] for link in broken_links_data)
        files_to_fix = [docs_dir.parent / f for f in files_to_fix if Path(f).exists()]
    else:
        # If no report, scan all files
        print(f"⚠️  Broken links report not found, scanning all files...")
        files_to_fix = list(docs_dir.rglob('*.md'))
    
    print(f"📋 Processing {len(files_to_fix)} files with broken links...")
    print(f"⚙️  Using {args.workers} parallel workers")
    
    all_results = []
    
    # Fix links in parallel
    with ProcessPoolExecutor(max_workers=args.workers) as executor:
        futures = [
            executor.submit(fix_links_in_file, (str(f), str(docs_dir)))
            for f in files_to_fix
        ]
        
        completed = 0
        for future in as_completed(futures):
            result = future.result()
            all_results.append(result)
            completed += 1
            fixes_count = len(result.get('fixes', []))
            if fixes_count > 0:
                print(f"  ✅ {completed}/{len(files_to_fix)} - Fixed {fixes_count} links in {Path(result['file']).name}")
            else:
                print(f"  ⏳ {completed}/{len(files_to_fix)} - No fixes needed", end='\r')
    
    # Summary
    total_fixes = sum(len(r.get('fixes', [])) for r in all_results)
    files_fixed = sum(1 for r in all_results if len(r.get('fixes', [])) > 0)
    
    print(f"\n{'='*60}")
    print(f"📊 SUMMARY")
    print(f"{'='*60}")
    print(f"Files processed: {len(files_to_fix)}")
    print(f"Files with fixes: {files_fixed}")
    print(f"Total links fixed: {total_fixes}")
    print(f"{'='*60}")
    
    # Save report
    report_path = Path('docs/BROKEN_LINKS_FIX_REPORT.json')
    with open(report_path, 'w') as f:
        json.dump(all_results, f, indent=2)
    print(f"\n✅ Report saved to {report_path}")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
