#!/usr/bin/env python3
"""
Fix broken links in PCAP_CAPTURE.md (Task 4)
Runs in background to fix CONFIGURATION.md and TROUBLESHOOTING.md links
"""

import json
import re
import sys
from pathlib import Path
from datetime import datetime

def find_similar_file(target_name: str, docs_dir: Path, base_file: Path) -> tuple[Path | None, str | None]:
    """Find similar file by name"""
    target_stem = Path(target_name).stem.lower()
    base_dir = base_file.parent

    # Search for similar files
    candidates = []
    for md_file in docs_dir.rglob("*.md"):
        file_stem = md_file.stem.lower()
        # Check if stems match or one contains the other
        if target_stem in file_stem or file_stem in target_stem:
            try:
                rel_path = md_file.relative_to(base_dir)
                candidates.append((md_file, str(rel_path)))
            except ValueError:
                rel_path = md_file.relative_to(docs_dir)
                base_parts = base_dir.relative_to(docs_dir).parts if base_dir != docs_dir else ()
                if base_parts == ():
                    candidates.append((md_file, str(rel_path)))
                else:
                    up_levels = len(base_parts)
                    rel_path_str = '../' * up_levels + str(rel_path)
                    candidates.append((md_file, rel_path_str))

    if candidates:
        # Prefer files in same directory, then files with matching name
        candidates.sort(key=lambda x: (
            x[0].parent != base_dir,
            x[0].stem.lower() != target_stem,
            len(Path(x[1]).parts)
        ))
        return candidates[0]

    return None, None

def fix_pcap_capture_links():
    """Fix broken links in PCAP_CAPTURE.md"""
    docs_dir = Path('/home/david/ib_box_spread_full_universal/docs')
    md_file = docs_dir / 'PCAP_CAPTURE.md'

    if not md_file.exists():
        return {
            'success': False,
            'error': f'File not found: {md_file}',
            'fixes': []
        }

    fixes = []

    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')

        # Fix CONFIGURATION.md link (line 278)
        target_line_num = 278
        if target_line_num <= len(lines):
            line = lines[target_line_num - 1]
            if 'CONFIGURATION.md' in line:
                # Search for similar file
                similar_file, rel_path = find_similar_file('CONFIGURATION.md', docs_dir, md_file)

                if similar_file:
                    # Update link
                    old_link = 'CONFIGURATION.md'
                    new_link = rel_path
                    lines[target_line_num - 1] = line.replace(old_link, new_link)
                    fixes.append({
                        'line': target_line_num,
                        'old': old_link,
                        'new': new_link,
                        'method': 'similar_file_found',
                        'found_file': str(similar_file)
                    })
                else:
                    # File doesn't exist - comment it out
                    lines[target_line_num - 1] = f"<!-- {line} - File not found -->"
                    fixes.append({
                        'line': target_line_num,
                        'old': line,
                        'new': lines[target_line_num - 1],
                        'method': 'commented_out'
                    })

        # Fix TROUBLESHOOTING.md link (line 279)
        target_line_num = 279
        if target_line_num <= len(lines):
            line = lines[target_line_num - 1]
            if 'TROUBLESHOOTING.md' in line:
                # Search for similar file
                similar_file, rel_path = find_similar_file('TROUBLESHOOTING.md', docs_dir, md_file)

                if similar_file:
                    # Update link
                    old_link = 'TROUBLESHOOTING.md'
                    new_link = rel_path
                    lines[target_line_num - 1] = line.replace(old_link, new_link)
                    fixes.append({
                        'line': target_line_num,
                        'old': old_link,
                        'new': new_link,
                        'method': 'similar_file_found',
                        'found_file': str(similar_file)
                    })
                else:
                    # File doesn't exist - comment it out
                    lines[target_line_num - 1] = f"<!-- {line} - File not found -->"
                    fixes.append({
                        'line': target_line_num,
                        'old': line,
                        'new': lines[target_line_num - 1],
                        'method': 'commented_out'
                    })

        # Write back if fixes were made
        if fixes:
            md_file.write_text('\n'.join(lines), encoding='utf-8')

    except Exception as e:
        return {
            'success': False,
            'error': str(e),
            'fixes': []
        }

    return {
        'success': True,
        'fixes': fixes,
        'file': str(md_file)
    }

if __name__ == '__main__':
    result = fix_pcap_capture_links()

    # Write report
    report_file = Path('/home/david/ib_box_spread_full_universal/docs/TASK_4_FIX_REPORT.json')
    report_file.write_text(json.dumps(result, indent=2), encoding='utf-8')

    # Print summary
    print(f"Task 4 (PCAP_CAPTURE.md): {'✅ Success' if result['success'] else '❌ Failed'}")
    print(f"  Fixes: {len(result.get('fixes', []))}")
    if result.get('error'):
        print(f"  Error: {result['error']}")
    for fix in result.get('fixes', []):
        print(f"  Line {fix['line']}: {fix['method']}")
        if 'found_file' in fix:
            print(f"    Found: {fix['found_file']}")
