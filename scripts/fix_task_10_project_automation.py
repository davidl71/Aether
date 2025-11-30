#!/usr/bin/env python3
"""
Fix path issues in PROJECT_AUTOMATION_MCP_EXTENSIONS.md (Task 10)
"""

import json
import re
from pathlib import Path

def find_file_by_path(target_path: str, base_file: Path, docs_dir: Path) -> tuple[Path | None, str | None]:
    """Find file by resolving relative path or searching"""
    base_dir = base_file.parent
    
    # Try resolving relative path
    if target_path.startswith('../'):
        # Go up from base_dir
        target_file = (base_dir.parent / target_path[3:]).resolve()
        if target_file.exists():
            try:
                rel_path = target_file.relative_to(base_dir)
                return target_file, str(rel_path)
            except ValueError:
                # Use absolute path or find alternative
                pass
    
    # Search by filename
    target_name = Path(target_path).name
    for md_file in docs_dir.rglob(target_name):
        try:
            rel_path = md_file.relative_to(base_dir)
            return md_file, str(rel_path)
        except ValueError:
            rel_path = md_file.relative_to(docs_dir)
            base_parts = base_dir.relative_to(docs_dir).parts if base_dir != docs_dir else ()
            if base_parts == ():
                return md_file, str(rel_path)
            else:
                up_levels = len(base_parts)
                rel_path_str = '../' * up_levels + str(rel_path)
                return md_file, rel_path_str
    
    return None, None

def fix_project_automation_links():
    """Fix broken links in PROJECT_AUTOMATION_MCP_EXTENSIONS.md"""
    docs_dir = Path('/home/david/ib_box_spread_full_universal/docs')
    md_file = docs_dir / 'PROJECT_AUTOMATION_MCP_EXTENSIONS.md'
    
    if not md_file.exists():
        return {'success': False, 'error': 'File not found', 'fixes': []}
    
    fixes = []
    
    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')
        
        # Fix ../mcp-servers/project-management-automation/TOOLS_STATUS.md (line 544)
        target_line_num = 544
        if target_line_num <= len(lines):
            line = lines[target_line_num - 1]
            if '../mcp-servers/project-management-automation/TOOLS_STATUS.md' in line:
                found_file, rel_path = find_file_by_path(
                    '../mcp-servers/project-management-automation/TOOLS_STATUS.md',
                    md_file,
                    Path('/home/david/ib_box_spread_full_universal')
                )
                
                if found_file and found_file.exists():
                    old_link = '../mcp-servers/project-management-automation/TOOLS_STATUS.md'
                    lines[target_line_num - 1] = line.replace(old_link, rel_path)
                    fixes.append({
                        'line': target_line_num,
                        'old': old_link,
                        'new': rel_path,
                        'method': 'path_fixed',
                        'found_file': str(found_file)
                    })
                else:
                    # Comment out if not found
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
    result = fix_project_automation_links()
    report_file = Path('/home/david/ib_box_spread_full_universal/docs/TASK_10_FIX_REPORT.json')
    report_file.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(f"Task 10: {'✅' if result['success'] else '❌'} {len(result.get('fixes', []))} fixes")
