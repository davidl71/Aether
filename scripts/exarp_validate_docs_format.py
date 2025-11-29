#!/usr/bin/env python3
"""
Exarp-compatible wrapper for documentation format validation.

This script follows Exarp's script pattern and can be integrated into
Exarp's daily automation or called directly.
"""

import json
import sys
from pathlib import Path

# Add parent directory to path to import validation script
sys.path.insert(0, str(Path(__file__).parent))

# Import the validation script's functions
import re
from typing import List, Tuple

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

# Required fields (minimum)
REQUIRED_FIELDS = [
    r'\*\*Website\*\*:',
    r'\*\*Description\*\*:',
    r'\*\*Relevance to Box Spread Trading\*\*:',
]

# Recommended fields
RECOMMENDED_FIELDS = [
    r'\*\*Key Features\*\*:',
    r'\*\*API Types\*\*:',
    r'\*\*Integration Considerations\*\*:',
    r'\*\*Use Cases\*\*:',
]

# URL pattern
URL_PATTERN = r'<https?://[^>]+>'

# Entry pattern (starts with ###)
ENTRY_PATTERN = r'^### (.+)$'


class ValidationError:
    def __init__(self, file: str, line: int, message: str):
        self.file = file
        self.line = line
        self.message = message

    def to_dict(self):
        return {
            'file': self.file,
            'line': self.line,
            'message': self.message
        }


def find_entries(content: str) -> List[Tuple[int, str]]:
    """Find all API entry headers (### Provider Name), excluding section headers."""
    entries = []
    lines = content.split('\n')

    skip_patterns = [
        'Core Trading APIs', 'Market Data', 'Trading Frameworks',
        'FIX Protocol', 'Trading Simulators', 'Quantitative Finance',
        'Financial Infrastructure', 'Open Data', 'Brokerage API',
        'Market Structure', 'Risk Management', 'Financial Tools',
        'Broker Selection', 'Rust', 'Go', 'TypeScript', 'Swift',
        'How to Use', 'Keeping This Index', 'Quick Reference',
    ]

    for i, line in enumerate(lines, 1):
        match = re.match(ENTRY_PATTERN, line)
        if match:
            entry_name = match.group(1)
            if not any(skip in entry_name for skip in skip_patterns):
                entries.append((i, entry_name))

    return entries


def validate_entry(content: str, entry_line: int, entry_name: str) -> List[ValidationError]:
    """Validate a single entry."""
    errors = []
    lines = content.split('\n')

    entry_start = entry_line - 1
    entry_end = len(lines)

    for i in range(entry_line, len(lines)):
        line = lines[i]
        if re.match(r'^### ', line) or re.match(r'^## ', line):
            entry_end = i
            break

    entry_content = '\n'.join(lines[entry_start:entry_end])

    for field in REQUIRED_FIELDS:
        if not re.search(field, entry_content):
            field_name = field.replace(r'\*\*', '').replace(r'\*', '').replace(':', '').replace('\\', '')
            errors.append(ValidationError(
                "API_DOCUMENTATION_INDEX.md",
                entry_line,
                f"Missing required field: {field_name}"
            ))

    missing_recommended = []
    for field in RECOMMENDED_FIELDS:
        if not re.search(field, entry_content):
            field_name = field.replace(r'\*\*', '').replace(r'\*', '').replace(':', '').replace('\\', '')
            missing_recommended.append(field_name)

    if missing_recommended:
        errors.append(ValidationError(
            "API_DOCUMENTATION_INDEX.md",
            entry_line,
            f"Missing recommended fields: {', '.join(missing_recommended)} (warning)"
        ))

    urls = re.findall(URL_PATTERN, entry_content)
    for url in urls:
        if not url.startswith('<http://') and not url.startswith('<https://'):
            errors.append(ValidationError(
                "API_DOCUMENTATION_INDEX.md",
                entry_line,
                f"Invalid URL format: {url} (should use angle brackets)"
            ))

    return errors


def main():
    """Main entry point following Exarp script pattern"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Validate API documentation format (Exarp-compatible)'
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project root directory (default: current directory)'
    )
    parser.add_argument(
        '--file',
        type=str,
        help='Specific file to validate (default: API_DOCUMENTATION_INDEX.md)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )
    parser.add_argument(
        '--output',
        type=str,
        help='Output JSON report file (optional)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()
    docs_dir = project_dir / 'docs'
    index_file = docs_dir / (args.file or 'API_DOCUMENTATION_INDEX.md')

    if not index_file.exists():
        error_msg = {
            'status': 'error',
            'message': f'File not found: {index_file}',
            'project_dir': str(project_dir)
        }
        if args.json:
            print(json.dumps(error_msg, indent=2))
        else:
            print(f"{RED}Error: {error_msg['message']}{NC}", file=sys.stderr)
        sys.exit(1)

    if not args.json:
        print("🔍 Validating API documentation entry format...")
        print("")

    content = index_file.read_text()
    entries = find_entries(content)

    all_errors = []
    warnings = []

    for entry_line, entry_name in entries:
        errors = validate_entry(content, entry_line, entry_name)
        for error in errors:
            if "warning" in error.message.lower():
                warnings.append(error)
            else:
                all_errors.append(error)

    # Build report
    report = {
        'status': 'success' if not all_errors else 'error',
        'file': str(index_file.relative_to(project_dir)),
        'entries_found': len(entries),
        'errors': [e.to_dict() for e in all_errors],
        'warnings': [w.to_dict() for w in warnings],
        'total_errors': len(all_errors),
        'total_warnings': len(warnings)
    }

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"Found {len(entries)} entries")
        print("")

        if all_errors:
            print(f"{RED}❌ Errors found:{NC}")
            for error in all_errors:
                print(f"  {RED}{error.file}:{error.line}: {error.message}{NC}")
            print("")

        if warnings:
            print(f"{YELLOW}⚠️  Warnings:{NC}")
            for warning in warnings:
                print(f"  {YELLOW}{warning.file}:{warning.line}: {warning.message}{NC}")
            print("")

        if not all_errors and not warnings:
            print(f"{GREEN}✅ All entries follow the standard format{NC}")
        elif not all_errors:
            print(f"{YELLOW}⚠️  Validation passed with warnings{NC}")
        else:
            print(f"{RED}❌ Validation failed{NC}")

    if args.output:
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        if not args.json:
            print(f"\n✅ Report saved to {args.output}")

    # Exit with appropriate code
    if all_errors:
        sys.exit(1)
    elif warnings:
        sys.exit(0)  # Warnings don't fail
    else:
        sys.exit(0)


if __name__ == '__main__':
    main()
