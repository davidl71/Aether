#!/usr/bin/env python3
"""
Fix false positive broken links (Task 1)
Fixes code references, mailto links, and placeholders that were incorrectly flagged
"""

import json
import re
import sys
from pathlib import Path
from datetime import datetime

def fix_false_positive_link(line: str, link_info: dict) -> tuple[str, dict]:
    """Fix a false positive link by removing markdown link syntax or fixing format"""
    link_path = link_info['link_path']
    link_text = link_info['link_text']
    full_line = link_info['full_line']

    fix_info = {
        'line': link_info['line'],
        'old': full_line,
        'method': None,
        'new': None
    }

    # Case 1: mailto links - these are valid, just ensure proper format
    if link_path.startswith('mailto:'):
        # mailto links are fine, but check if markdown syntax is correct
        if f'[{link_text}]({link_path})' in line:
            # Already correct format, no change needed
            return line, None
        else:
            # Fix the link format
            new_line = re.sub(
                r'\[([^\]]+)\]\(([^)]+)\)',
                lambda m: f'[{link_text}]({link_path})' if m.group(2) == link_path else m.group(0),
                line
            )
            fix_info['method'] = 'mailto_format_fixed'
            fix_info['new'] = new_line
            return new_line, fix_info

    # Case 2: Code references (const, &, *, function parameters)
    # These should NOT be markdown links - remove link syntax
    if ('const ' in link_path or
        '&' in link_path or
        link_path.startswith('*') or
        'Event event' in link_path or
        link_path in ['data', 'market_data']):

        # Remove markdown link syntax, keep just the text
        # Pattern: [link_text](link_path) -> link_text or just the code
        old_pattern = f'[{link_text}]({link_path})'

        # If link_text is part of code, keep it as-is
        if link_text in ['&', '&promise', '&promise, &received', 'tickerId', 'order_id', 'id', 'this']:
            # Remove markdown link, keep text
            new_line = line.replace(old_pattern, link_text)
        else:
            # Keep the text but remove link
            new_line = line.replace(old_pattern, link_text)

        fix_info['method'] = 'code_reference_fixed'
        fix_info['new'] = new_line
        return new_line, fix_info

    # Case 3: Placeholder links (like "link" or ".gitignore")
    if link_path == 'link' or link_path == '.gitignore':
        # Remove the link entirely or replace with plain text
        old_pattern = f'[{link_text}]({link_path})'
        new_line = line.replace(old_pattern, link_text)
        fix_info['method'] = 'placeholder_removed'
        fix_info['new'] = new_line
        return new_line, fix_info

    # No fix needed
    return line, None

def fix_false_positives():
    """Fix all false positive broken links"""
    docs_dir = Path('/home/david/ib_box_spread_full_universal/docs')

    # Load broken links
    with open(docs_dir / 'BROKEN_LINKS_REPORT.json', 'r') as f:
        broken_links = json.load(f)

    # Filter false positives
    false_positives = [l for l in broken_links if
        l['link_path'].startswith('mailto:') or
        'const ' in l['link_path'] or
        '&' in l['link_path'] or
        l['link_path'].startswith('*') or
        l['link_path'] == 'link' or
        l['link_path'] == '.gitignore' or
        'Event event' in l['link_path'] or
        l['link_path'] in ['data', 'market_data']]

    # Group by file
    files_to_fix = {}
    for link in false_positives:
        file_path = link['file']
        if file_path not in files_to_fix:
            files_to_fix[file_path] = []
        files_to_fix[file_path].append(link)

    all_fixes = []

    # Process each file
    for file_path, links in files_to_fix.items():
        md_file = Path(file_path)

        if not md_file.exists():
            all_fixes.append({
                'file': str(md_file),
                'success': False,
                'error': 'File not found',
                'fixes': []
            })
            continue

        try:
            content = md_file.read_text(encoding='utf-8')
            lines = content.split('\n')

            file_fixes = []

            # Process each link (sort by line number descending to avoid offset issues)
            for link_info in sorted(links, key=lambda x: x['line'], reverse=True):
                line_num = link_info['line']
                if line_num <= len(lines):
                    line = lines[line_num - 1]
                    new_line, fix_info = fix_false_positive_link(line, link_info)

                    if fix_info:
                        lines[line_num - 1] = new_line
                        file_fixes.append(fix_info)

            # Write back if fixes were made
            if file_fixes:
                md_file.write_text('\n'.join(lines), encoding='utf-8')

            all_fixes.append({
                'file': str(md_file),
                'success': True,
                'fixes': file_fixes
            })

        except Exception as e:
            all_fixes.append({
                'file': str(md_file),
                'success': False,
                'error': str(e),
                'fixes': []
            })

    return {
        'success': True,
        'total_files': len(files_to_fix),
        'total_links': len(false_positives),
        'fixes': all_fixes
    }

if __name__ == '__main__':
    result = fix_false_positives()

    # Write report
    report_file = Path('/home/david/ib_box_spread_full_universal/docs/TASK_1_FIX_REPORT.json')
    report_file.write_text(json.dumps(result, indent=2), encoding='utf-8')

    # Print summary
    print(f"Task 1 (False Positives): {'✅ Success' if result['success'] else '❌ Failed'}")
    print(f"  Files processed: {result['total_files']}")
    print(f"  Links fixed: {result['total_links']}")

    total_fixes = sum(len(f.get('fixes', [])) for f in result['fixes'])
    print(f"  Total fixes applied: {total_fixes}")

    for file_result in result['fixes']:
        if file_result.get('fixes'):
            print(f"  {Path(file_result['file']).name}: {len(file_result['fixes'])} fixes")
        elif file_result.get('error'):
            print(f"  {Path(file_result['file']).name}: ❌ {file_result['error']}")
