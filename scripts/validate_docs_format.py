#!/usr/bin/env python3
"""
Validate API Documentation Entry Format

Checks that entries in API_DOCUMENTATION_INDEX.md follow the standard format
defined in API_DOCUMENTATION_ENTRY_TEMPLATE.md
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict

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

    def __str__(self):
        return f"{self.file}:{self.line}: {self.message}"


def find_entries(content: str) -> List[Tuple[int, str]]:
    """Find all API entry headers (### Provider Name), excluding section headers."""
    entries = []
    lines = content.split('\n')

    # Skip entries that are clearly section headers (not API providers)
    skip_patterns = [
        'Core Trading APIs',
        'Market Data',
        'Trading Frameworks',
        'FIX Protocol',
        'Trading Simulators',
        'Quantitative Finance',
        'Financial Infrastructure',
        'Open Data',
        'Brokerage API',
        'Market Structure',
        'Risk Management',
        'Financial Tools',
        'Broker Selection',
        'Rust',
        'Go',
        'TypeScript',
        'Swift',
        'How to Use',
        'Keeping This Index',
        'Quick Reference',
    ]

    for i, line in enumerate(lines, 1):
        match = re.match(ENTRY_PATTERN, line)
        if match:
            entry_name = match.group(1)
            # Skip section headers
            if not any(skip in entry_name for skip in skip_patterns):
                entries.append((i, entry_name))

    return entries


def validate_entry(content: str, entry_line: int, entry_name: str) -> List[ValidationError]:
    """Validate a single entry."""
    errors = []
    lines = content.split('\n')

    # Extract entry content (from ### to next ### or ##)
    entry_start = entry_line - 1
    entry_end = len(lines)

    for i in range(entry_line, len(lines)):
        line = lines[i]
        if re.match(r'^### ', line) or re.match(r'^## ', line):
            entry_end = i
            break

    entry_content = '\n'.join(lines[entry_start:entry_end])

    # Check required fields
    for field in REQUIRED_FIELDS:
        if not re.search(field, entry_content):
            errors.append(ValidationError(
                "API_DOCUMENTATION_INDEX.md",
                entry_line,
                f"Missing required field: {field.replace('*', '').replace(':', '')}"
            ))

    # Check recommended fields (warnings)
    missing_recommended = []
    for field in RECOMMENDED_FIELDS:
        if not re.search(field, entry_content):
            missing_recommended.append(field.replace('*', '').replace(':', ''))

    if missing_recommended:
        errors.append(ValidationError(
            "API_DOCUMENTATION_INDEX.md",
            entry_line,
            f"Missing recommended fields: {', '.join(missing_recommended)} (warning)"
        ))

    # Check URL format
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
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    docs_dir = project_root / "docs"
    index_file = docs_dir / "API_DOCUMENTATION_INDEX.md"

    if not index_file.exists():
        print(f"{RED}Error: {index_file} not found{NC}")
        sys.exit(1)

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

    # Report results
    print(f"Found {len(entries)} entries")
    print("")

    if all_errors:
        print(f"{RED}❌ Errors found:{NC}")
        for error in all_errors:
            print(f"  {RED}{error}{NC}")
        print("")

    if warnings:
        print(f"{YELLOW}⚠️  Warnings:{NC}")
        for warning in warnings:
            print(f"  {YELLOW}{warning}{NC}")
        print("")

    if not all_errors and not warnings:
        print(f"{GREEN}✅ All entries follow the standard format{NC}")
        sys.exit(0)
    elif not all_errors:
        print(f"{YELLOW}⚠️  Validation passed with warnings{NC}")
        sys.exit(0)
    else:
        print(f"{RED}❌ Validation failed{NC}")
        sys.exit(1)


if __name__ == "__main__":
    main()
