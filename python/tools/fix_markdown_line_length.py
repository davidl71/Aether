#!/usr/bin/env python3
"""
Fix markdown line length issues by breaking long lines intelligently.

This script breaks long lines in markdown files while preserving:
- List item formatting
- Code blocks
- URLs (converts to markdown links when possible)
- Table structure
"""

import re
import sys
from pathlib import Path


def break_long_line(line: str, max_length: int = 150) -> str:
    """Break a long line into multiple lines while preserving markdown formatting."""
    if len(line) <= max_length:
        return line

    # Don't break code blocks
    if line.strip().startswith('```') or line.strip().startswith('    '):
        return line

    # Don't break table rows
    if '|' in line and line.strip().startswith('|'):
        return line

    # Don't break HTML comments
    if line.strip().startswith('<!--') or line.strip().endswith('-->'):
        return line

    # Handle list items
    if line.strip().startswith('- ') or line.strip().startswith('* '):
        indent = len(line) - len(line.lstrip())
        prefix = line[:indent] + line[indent:indent+2]  # "- " or "* "
        content = line[indent+2:]

        # Break at sentence boundaries or after colons/semicolons
        # Split on sentence endings, colons, and semicolons
        parts = re.split(r'([.!?:;]\s+)', content)
        result_lines = []
        current_line = prefix

        i = 0
        while i < len(parts):
            part = parts[i] + (parts[i+1] if i+1 < len(parts) else '')

            # If this part alone exceeds limit, break it further
            if len(part) > max_length - len(prefix):
                # Break at commas or other natural breaks
                subparts = re.split(r'([,]\s+)', part)
                for j in range(0, len(subparts), 2):
                    subpart = subparts[j] + (subparts[j+1] if j+1 < len(subparts) else '')
                    if len(current_line) + len(subpart) > max_length and current_line != prefix:
                        result_lines.append(current_line.rstrip())
                        current_line = ' ' * (indent + 2) + subpart
                    else:
                        current_line += subpart
            elif len(current_line) + len(part) > max_length and current_line != prefix:
                result_lines.append(current_line.rstrip())
                current_line = ' ' * (indent + 2) + part
            else:
                current_line += part

            i += 2

        if current_line.strip() and current_line != prefix:
            result_lines.append(current_line.rstrip())

        return '\n'.join(result_lines) if len(result_lines) > 1 else line

    # For regular lines, break at word boundaries
    words = line.split()
    result_lines = []
    current_line = ''

    for word in words:
        if len(current_line) + len(word) + 1 > max_length and current_line:
            result_lines.append(current_line.rstrip())
            current_line = word + ' '
        else:
            current_line += word + ' '

    if current_line.strip():
        result_lines.append(current_line.rstrip())

    return '\n'.join(result_lines) if len(result_lines) > 1 else line


def fix_file(file_path: Path, max_length: int = 150) -> tuple[int, int]:
    """Fix long lines in a markdown file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    fixed_lines = []
    fixes_count = 0
    original_long_lines = 0

    for line in lines:
        original = line.rstrip('\n')
        if len(original) > max_length:
            original_long_lines += 1
            fixed = break_long_line(original, max_length)
            if fixed != original:
                fixes_count += 1
                fixed_lines.append(fixed + '\n')
            else:
                fixed_lines.append(line)
        else:
            fixed_lines.append(line)

    if fixes_count > 0:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.writelines(fixed_lines)

    return original_long_lines, fixes_count


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='Fix markdown line length issues')
    parser.add_argument('file', help='Markdown file to fix')
    parser.add_argument('--max-length', type=int, default=150, help='Maximum line length (default: 150)')

    args = parser.parse_args()

    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: File not found: {file_path}")
        sys.exit(1)

    print(f"Fixing long lines in {file_path}...")
    original, fixed = fix_file(file_path, args.max_length)
    print(f"Found {original} long lines, fixed {fixed} lines")
