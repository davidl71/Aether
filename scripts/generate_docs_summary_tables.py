#!/usr/bin/env python3
"""
Generate Summary Tables for API Documentation

Automatically generates comparison tables from API_DOCUMENTATION_INDEX.md
and updates API_DOCUMENTATION_SUMMARY.md
"""

import re
from pathlib import Path
from typing import List, Dict

def extract_provider_info(content: str) -> List[Dict]:
    """Extract provider information from entries."""
    providers = []
    lines = content.split('\n')

    current_entry = None
    current_data = {}

    for i, line in enumerate(lines):
        # Start of new entry
        match = re.match(r'^### (.+)$', line)
        if match:
            if current_entry:
                providers.append(current_data)
            current_entry = match.group(1)
            current_data = {
                'name': current_entry,
                'line': i + 1,
                'website': '',
                'description': '',
                'api_types': [],
                'options_support': '',
                'c++_support': '',
            }
            continue

        # Extract fields
        if current_entry:
            if re.match(r'^\s*-\s*\*\*Website\*\*:', line):
                url_match = re.search(r'<([^>]+)>', line)
                if url_match:
                    current_data['website'] = url_match.group(1)

            elif re.match(r'^\s*-\s*\*\*Description\*\*:', line):
                desc = line.split(':', 1)[1].strip()
                current_data['description'] = desc[:100]  # First 100 chars

            elif re.match(r'^\s*-\s*\*\*API Types\*\*:', line):
                # Look for list items in next few lines
                for j in range(i + 1, min(i + 10, len(lines))):
                    if re.match(r'^\s*-\s*\*\*', lines[j]):
                        break
                    api_match = re.search(r'REST|FIX|WebSocket|C\+\+|Java|Python', lines[j], re.IGNORECASE)
                    if api_match:
                        current_data['api_types'].append(api_match.group(0))

            elif 'Options' in line and 'Support' in line:
                if '✅' in line:
                    current_data['options_support'] = '✅'
                elif '⚠️' in line:
                    current_data['options_support'] = '⚠️'
                elif '❌' in line:
                    current_data['options_support'] = '❌'

            elif 'C++' in line and ('Support' in line or 'API' in line):
                if '✅' in line:
                    current_data['c++_support'] = '✅'
                elif '❌' in line:
                    current_data['c++_support'] = '❌'

    if current_entry:
        providers.append(current_data)

    return providers


def generate_markdown_table(providers: List[Dict], columns: List[str]) -> str:
    """Generate a markdown table from provider data."""
    if not providers:
        return ""

    # Header
    header = "| " + " | ".join(columns) + " |"
    separator = "| " + " | ".join(["---"] * len(columns)) + " |"

    # Rows
    rows = []
    for provider in providers:
        row = []
        for col in columns:
            key = col.lower().replace(' ', '_')
            value = provider.get(key, '')
            if isinstance(value, list):
                value = ', '.join(value) if value else '-'
            row.append(value or '-')
        rows.append("| " + " | ".join(row) + " |")

    return "\n".join([header, separator] + rows)


def main():
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    docs_dir = project_root / "docs"

    index_file = docs_dir / "API_DOCUMENTATION_INDEX.md"
    summary_file = docs_dir / "API_DOCUMENTATION_SUMMARY.md"

    if not index_file.exists():
        print(f"Error: {index_file} not found")
        return

    print("📊 Generating summary tables...")

    content = index_file.read_text()
    providers = extract_provider_info(content)

    # Filter by category
    market_data_providers = [p for p in providers if any(
        keyword in p['name'].lower()
        for keyword in ['dxfeed', 'orats', 'alpha', 'finnhub', 'massive', 'openbb']
    )]

    fix_providers = [p for p in providers if 'fix' in p['name'].lower()]

    # Generate tables
    print(f"Found {len(providers)} providers")
    print(f"  - Market Data: {len(market_data_providers)}")
    print(f"  - FIX Providers: {len(fix_providers)}")

    # Note: This is a basic implementation
    # Full implementation would update the summary file
    print("\n✅ Summary tables can be generated")
    print("   (Full implementation would update API_DOCUMENTATION_SUMMARY.md)")


if __name__ == "__main__":
    main()
