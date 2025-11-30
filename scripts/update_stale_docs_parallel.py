#!/usr/bin/env python3
"""
Parallel stale documentation updater - updates stale documentation files in parallel.
"""

import json
import re
import sys
from pathlib import Path
from datetime import datetime, timedelta
from concurrent.futures import ProcessPoolExecutor, as_completed
from typing import List, Dict

def update_stale_file(args_tuple):
    """Update a stale documentation file (runs in parallel process)"""
    md_file_str, update_date = args_tuple
    md_file = Path(md_file_str)

    try:
        content = md_file.read_text(encoding='utf-8')
        lines = content.split('\n')

        updated = False

        # Update date fields in the file
        for i, line in enumerate(lines):
            # Look for date patterns like "**Date**: 2025-01-27" or "Date: 2025-01-27"
            if re.match(r'^\*\*Date\*\*:\s*\d{4}-\d{2}-\d{2}', line):
                old_date = re.search(r'\d{4}-\d{2}-\d{2}', line).group()
                if old_date != update_date:
                    lines[i] = line.replace(old_date, update_date)
                    updated = True
            elif re.match(r'^Date:\s*\d{4}-\d{2}-\d{2}', line):
                old_date = re.search(r'\d{4}-\d{2}-\d{2}', line).group()
                if old_date != update_date:
                    lines[i] = line.replace(old_date, update_date)
                    updated = True
            # Look for "Last Updated" patterns
            elif 'Last Updated' in line or 'Last updated' in line:
                date_match = re.search(r'\d{4}-\d{2}-\d{2}', line)
                if date_match:
                    old_date = date_match.group()
                    if old_date != update_date:
                        lines[i] = line.replace(old_date, update_date)
                        updated = True

        # If no date found, add one at the top if it's a markdown file with frontmatter or header
        if not updated and lines:
            # Check if first line is a header
            if lines[0].startswith('#'):
                # Insert date after header
                if len(lines) > 1 and not lines[1].strip():
                    # Empty line after header, insert date there
                    lines.insert(2, f'**Date**: {update_date}')
                    updated = True
                else:
                    lines.insert(1, f'**Date**: {update_date}')
                    updated = True

        if updated:
            md_file.write_text('\n'.join(lines), encoding='utf-8')
            return {
                'file': str(md_file),
                'success': True,
                'updated': True
            }
        else:
            return {
                'file': str(md_file),
                'success': True,
                'updated': False,
                'reason': 'No date fields found or already up to date'
            }

    except Exception as e:
        return {
            'file': str(md_file),
            'success': False,
            'error': str(e),
            'updated': False
        }

def main():
    """Main execution - updates stale docs in parallel"""
    import argparse

    parser = argparse.ArgumentParser(description='Update stale documentation files in parallel')
    parser.add_argument('--docs-dir', type=str, default='docs', help='Documentation directory')
    parser.add_argument('--workers', type=int, default=8, help='Number of parallel workers')
    parser.add_argument('--days-threshold', type=int, default=90, help='Days old to consider stale')
    parser.add_argument('--update-date', type=str, default=None,
                       help='Date to update to (YYYY-MM-DD), defaults to today')

    args = parser.parse_args()

    docs_dir = Path(args.docs_dir).resolve()
    if not docs_dir.exists():
        print(f"❌ Error: Documentation directory not found: {docs_dir}")
        sys.exit(1)

    # Determine update date
    if args.update_date:
        update_date = args.update_date
    else:
        update_date = datetime.now().strftime('%Y-%m-%d')

    # Find stale files from health report or scan
    stale_files = []
    health_report = docs_dir.parent / 'docs' / 'DOCUMENTATION_HEALTH_REPORT.md'

    if health_report.exists():
        # Try to extract stale files from report
        content = health_report.read_text(encoding='utf-8')
        # Look for stale files list
        for line in content.split('\n'):
            if 'stale_files' in line.lower() or 'days_old' in line.lower():
                # Try to extract file paths
                file_match = re.search(r'docs/[^\s,]+\.md', line)
                if file_match:
                    file_path = docs_dir.parent / file_match.group()
                    if file_path.exists():
                        stale_files.append(file_path)

    # If no stale files found in report, scan all files
    if not stale_files:
        print(f"⚠️  No stale files found in report, scanning all files...")
        threshold_date = datetime.now() - timedelta(days=args.days_threshold)

        for md_file in docs_dir.rglob('*.md'):
            try:
                # Check file modification time
                mtime = datetime.fromtimestamp(md_file.stat().st_mtime)
                if mtime < threshold_date:
                    stale_files.append(md_file)
            except Exception:
                pass

    print(f"📋 Found {len(stale_files)} stale documentation files")
    print(f"📅 Updating dates to: {update_date}")
    print(f"⚙️  Using {args.workers} parallel workers")

    if not stale_files:
        print("✅ No stale files to update!")
        return 0

    all_results = []

    # Update files in parallel
    with ProcessPoolExecutor(max_workers=args.workers) as executor:
        futures = [
            executor.submit(update_stale_file, (str(f), update_date))
            for f in stale_files
        ]

        completed = 0
        for future in as_completed(futures):
            result = future.result()
            all_results.append(result)
            completed += 1
            if result.get('updated'):
                print(f"  ✅ {completed}/{len(stale_files)} - Updated {Path(result['file']).name}")
            else:
                print(f"  ⏳ {completed}/{len(stale_files)} - No update needed", end='\r')

    # Summary
    files_updated = sum(1 for r in all_results if r.get('updated'))

    print(f"\n{'='*60}")
    print(f"📊 SUMMARY")
    print(f"{'='*60}")
    print(f"Files processed: {len(stale_files)}")
    print(f"Files updated: {files_updated}")
    print(f"Update date: {update_date}")
    print(f"{'='*60}")

    # Save report
    report_path = Path('docs/STALE_DOCS_UPDATE_REPORT.json')
    with open(report_path, 'w') as f:
        json.dump(all_results, f, indent=2)
    print(f"\n✅ Report saved to {report_path}")

    return 0

if __name__ == '__main__':
    sys.exit(main())
