#!/usr/bin/env python3
"""
Fix markdown format errors in documentation.

Common issues:
- Missing blank line before headers
- Missing blank line before lists
- Missing blank line after headers
- Inconsistent spacing
"""

import re
from pathlib import Path
from typing import List, Tuple


def fix_markdown_format(content: str) -> Tuple[str, List[str]]:
    """Fix common markdown format issues."""
    lines = content.split('\n')
    fixed_lines = []
    fixes = []
    in_code_block = False
    code_block_marker = None

    for i, line in enumerate(lines):
        prev_line = lines[i - 1] if i > 0 else ''
        next_line = lines[i + 1] if i < len(lines) - 1 else ''

        # Track code blocks
        if re.match(r'^```', line):
            if not in_code_block:
                in_code_block = True
                code_block_marker = line
            else:
                in_code_block = False
                code_block_marker = None

        # Skip format fixes inside code blocks
        if in_code_block:
            fixed_lines.append(line)
            continue

        # Remove trailing spaces
        original_line = line
        line = line.rstrip()
        if original_line != line:
            fixes.append(f'Line {i+1}: Removed trailing spaces')

        # Remove multiple consecutive blank lines (keep max 1)
        if not line.strip():
            if fixed_lines and not fixed_lines[-1].strip():
                # Skip this blank line (already have one)
                continue

        # Check for header without blank line before
        if re.match(r'^#+', line) and i > 0:
            if prev_line.strip() and not prev_line.startswith('#'):
                # Add blank line before header
                if fixed_lines and fixed_lines[-1].strip():
                    fixed_lines.append('')
                    fixes.append(f'Line {i+1}: Added blank line before header')

        # Check for list item without blank line before
        if re.match(r'^[-*+]\s+', line) and i > 0:
            if prev_line.strip() and not re.match(r'^[-*+]\s+', prev_line) and not prev_line.startswith('#'):
                # Add blank line before list
                if fixed_lines and fixed_lines[-1].strip():
                    fixed_lines.append('')
                    fixes.append(f'Line {i+1}: Added blank line before list')

        # Check for header without blank line after (if next line is not blank and not a header)
        if re.match(r'^#+', line) and next_line.strip() and not re.match(r'^#+', next_line):
            # This will be handled when we process next_line, so we add blank line after header
            fixed_lines.append(line)
            if next_line and not re.match(r'^[-*+]', next_line) and not re.match(r'^```', next_line):
                fixed_lines.append('')
                fixes.append(f'Line {i+1}: Added blank line after header')
            continue

        fixed_lines.append(line)

    return '\n'.join(fixed_lines), fixes


def process_file(file_path: Path) -> Tuple[bool, List[str]]:
    """Process a single markdown file."""
    try:
        original_content = file_path.read_text(encoding='utf-8')
        fixed_content, fixes = fix_markdown_format(original_content)

        if fixes:
            file_path.write_text(fixed_content, encoding='utf-8')
            return True, fixes
        return False, []
    except Exception as e:
        return False, [f'Error: {e}']


def main():
    """Main function."""
    docs_dir = Path('docs')
    if not docs_dir.exists():
        print(f"Error: {docs_dir} does not exist")
        return

    total_fixed = 0
    total_files = 0
    all_fixes = []

    # Process all markdown files
    for file_path in sorted(docs_dir.rglob('*.md')):
        total_files += 1
        was_fixed, fixes = process_file(file_path)

        if was_fixed:
            total_fixed += 1
            all_fixes.append((str(file_path), fixes))
            print(f"✅ Fixed {len(fixes)} issues in {file_path}")

    print(f"\n📊 Summary:")
    print(f"   Files processed: {total_files}")
    print(f"   Files fixed: {total_fixed}")
    print(f"   Total fixes: {sum(len(fixes) for _, fixes in all_fixes)}")

    # Write summary
    summary_path = Path('docs/FORMAT_ERRORS_FIX_SUMMARY.md')
    with summary_path.open('w', encoding='utf-8') as f:
        f.write("# Markdown Format Errors Fix Summary\n\n")
        f.write(f"**Date**: 2025-11-30\n")
        f.write(f"**Files Fixed**: {total_fixed}\n")
        f.write(f"**Total Fixes**: {sum(len(fixes) for _, fixes in all_fixes)}\n\n")
        f.write("## Files Modified\n\n")
        for file_path, fixes in all_fixes:
            f.write(f"### {file_path}\n")
            for fix in fixes:
                f.write(f"- {fix}\n")
            f.write("\n")

    print(f"\n📝 Summary written to {summary_path}")


if __name__ == '__main__':
    main()
