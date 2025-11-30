#!/usr/bin/env python3
"""Fix path issues in NOTEBOOKS_WORKFLOW.md (Task 12)"""

import json
from pathlib import Path

def find_file_by_path(target_path: str, base_file: Path, root_dir: Path) -> tuple[Path | None, str | None]:
    """Find file by resolving relative path"""
    base_dir = base_file.parent

    # Try resolving relative path
    if target_path.startswith('../'):
        target_file = (base_dir.parent / target_path[3:]).resolve()
        if target_file.exists():
            try:
                rel_path = target_file.relative_to(base_dir)
                return target_file, str(rel_path)
            except ValueError:
                pass

    # Search by filename
    target_name = Path(target_path).name
    for file in root_dir.rglob(target_name):
        if file.exists():
            try:
                rel_path = file.relative_to(base_dir)
                return file, str(rel_path)
            except ValueError:
                pass

    return None, None

def fix_notebooks_links():
    """Fix broken links in NOTEBOOKS_WORKFLOW.md"""
    docs_dir = Path('/home/david/ib_box_spread_full_universal/docs')
    root_dir = Path('/home/david/ib_box_spread_full_universal')
    md_file = docs_dir / 'NOTEBOOKS_WORKFLOW.md'

    if not md_file.exists():
        return {'success': False, 'error': 'File not found', 'fixes': []}

    fixes = []

    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')

        # Fix ../notebooks/06-dev-workflow/decision_log.ipynb (line 236)
        target_line_num = 236
        if target_line_num <= len(lines):
            line = lines[target_line_num - 1]
            if '../notebooks/06-dev-workflow/decision_log.ipynb' in line:
                found_file, rel_path = find_file_by_path(
                    '../notebooks/06-dev-workflow/decision_log.ipynb',
                    md_file,
                    root_dir
                )

                if found_file and found_file.exists():
                    old_link = '../notebooks/06-dev-workflow/decision_log.ipynb'
                    lines[target_line_num - 1] = line.replace(old_link, rel_path)
                    fixes.append({
                        'line': target_line_num,
                        'old': old_link,
                        'new': rel_path,
                        'method': 'path_fixed',
                        'found_file': str(found_file)
                    })
                else:
                    lines[target_line_num - 1] = f"<!-- {line} - File not found -->"
                    fixes.append({
                        'line': target_line_num,
                        'old': line,
                        'new': lines[target_line_num - 1],
                        'method': 'commented_out'
                    })

        if fixes:
            md_file.write_text('\n'.join(lines), encoding='utf-8')

    except Exception as e:
        return {'success': False, 'error': str(e), 'fixes': []}

    return {'success': True, 'fixes': fixes, 'file': str(md_file)}

if __name__ == '__main__':
    result = fix_notebooks_links()
    report_file = Path('/home/david/ib_box_spread_full_universal/docs/TASK_12_FIX_REPORT.json')
    report_file.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(f"Task 12: {'✅' if result['success'] else '❌'} {len(result.get('fixes', []))} fixes")
