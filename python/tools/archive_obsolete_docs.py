#!/usr/bin/env python3
"""
Archive obsolete markdown documentation files.

This script identifies and archives:
- Status/summary/completion reports
- JSON task reports
- Files referencing removed functionality
"""

import re
import shutil
from pathlib import Path
from typing import List, Tuple


def find_obsolete_files(docs_dir: Path) -> Tuple[List[Path], List[Path], List[Path]]:
    """Find obsolete files by pattern."""
    status_files = []
    json_reports = []
    deprecated_refs = []

    # Patterns for obsolete status/summary files
    obsolete_patterns = [
        r'.*COMPLETE\.md$',
        r'.*SUMMARY\.md$',
        r'.*STATUS\.md$',
        r'.*REVIEW\.md$',
        r'.*FIX.*\.md$',
        r'.*REPORT\.md$',
        r'TASKS?_\d+.*\.md$',
        r'T-\d+.*\.md$',
        r'.*AUDIT.*\.md$',
        r'.*PROGRESS.*\.md$',
    ]

    # Patterns for JSON reports
    json_patterns = [
        r'TASK_\d+_FIX_REPORT\.json$',
        r'.*_REPORT\.json$',
        r'.*_TASKS.*\.json$',
        r'ACTIONABLE_TASKS\.json$',
        r'TASK_DISCOVERY_REPORT\.json$',
        r'STALE_DOCS_UPDATE_REPORT\.json$',
        r'BROKEN_LINKS.*\.json$',
        r'DOCUMENTATION_FIX_REPORT\.json$',
    ]

    # Files referencing removed functionality
    deprecated_ref_patterns = [
        r'.*NOTEBOOKLM.*\.md$',
        r'.*DESKTOP.*COMMANDER.*\.md$',
    ]

    # Exclude files that should be kept
    keep_files = {
        'MCP_DESKTOP_COMMANDER_REMOVAL.md',  # Documents removal
        'MCP_NOTEBOOKLM_DISABLED.md',  # Documents removal
        'DOCUMENTATION_INDEX.md',  # Main index
        'MARKDOWN_FILES_REVIEW.md',  # This review
        'MARKDOWN_LINT_ERRORS_SUMMARY.md',  # Linting summary
        'PROJECT_SCORECARD_ACCURATE.md',  # Current scorecard
    }

    for md_file in docs_dir.rglob('*.md'):
        if md_file.name in keep_files:
            continue

        # Check status/summary patterns
        for pattern in obsolete_patterns:
            if re.match(pattern, md_file.name, re.IGNORECASE):
                # Exclude core documentation
                if not any(exclude in str(md_file) for exclude in [
                    '/platform/',
                    '/strategies/',
                    '/research/architecture/',
                    'API_DOCUMENTATION_INDEX.md',
                    'DOCUMENTATION_INDEX.md',
                ]):
                    status_files.append(md_file)
                break

    for json_file in docs_dir.rglob('*.json'):
        # Exclude message schemas and config files
        if 'message_schemas' in str(json_file) or 'config' in str(json_file).lower():
            continue

        for pattern in json_patterns:
            if re.match(pattern, json_file.name, re.IGNORECASE):
                json_reports.append(json_file)
                break

    for md_file in docs_dir.rglob('*.md'):
        if md_file.name in keep_files:
            continue

        for pattern in deprecated_ref_patterns:
            if re.match(pattern, md_file.name, re.IGNORECASE):
                # Exclude removal documentation
                if 'REMOVAL' not in md_file.name and 'DISABLED' not in md_file.name:
                    deprecated_refs.append(md_file)
                break

    return status_files, json_reports, deprecated_refs


def create_archive_structure(archive_dir: Path):
    """Create archive directory structure."""
    (archive_dir / 'status-reports').mkdir(parents=True, exist_ok=True)
    (archive_dir / 'json-reports').mkdir(parents=True, exist_ok=True)
    (archive_dir / 'deprecated-refs').mkdir(parents=True, exist_ok=True)


def archive_files(files: List[Path], archive_dir: Path, subdir: str, docs_dir: Path, dry_run: bool = False):
    """Archive files to specified subdirectory."""
    archived = []
    skipped = []

    for file_path in files:
        # Get relative path from docs directory
        try:
            rel_path = file_path.relative_to(docs_dir)
        except ValueError:
            # File is not under docs_dir, use just the filename
            rel_path = Path(file_path.name)

        archive_path = archive_dir / subdir / rel_path

        # Create parent directories
        archive_path.parent.mkdir(parents=True, exist_ok=True)

        if dry_run:
            print(f"Would archive: {file_path} -> {archive_path}")
            archived.append(file_path)
        else:
            try:
                shutil.move(str(file_path), str(archive_path))
                archived.append(file_path)
                print(f"Archived: {file_path.name}")
            except Exception as e:
                print(f"Error archiving {file_path}: {e}")
                skipped.append(file_path)

    return archived, skipped


def main():
    import argparse

    parser = argparse.ArgumentParser(description='Archive obsolete documentation files')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be archived without archiving')
    parser.add_argument('--docs-dir', type=str, default='docs', help='Documentation directory')
    parser.add_argument('--archive-dir', type=str, default='docs/archive', help='Archive directory')

    args = parser.parse_args()

    docs_dir = Path(args.docs_dir)
    archive_dir = Path(args.archive_dir)

    if not docs_dir.exists():
        print(f"Error: Documentation directory not found: {docs_dir}")
        return 1

    print(f"Scanning {docs_dir} for obsolete files...")
    status_files, json_reports, deprecated_refs = find_obsolete_files(docs_dir)

    print("\nFound:")
    print(f"  Status/Summary files: {len(status_files)}")
    print(f"  JSON reports: {len(json_reports)}")
    print(f"  Deprecated references: {len(deprecated_refs)}")
    print(f"  Total: {len(status_files) + len(json_reports) + len(deprecated_refs)}")

    if args.dry_run:
        print("\n=== DRY RUN MODE ===")
        print("\nStatus/Summary files:")
        for f in sorted(status_files)[:20]:
            print(f"  {f}")
        if len(status_files) > 20:
            print(f"  ... and {len(status_files) - 20} more")

        print("\nJSON reports:")
        for f in sorted(json_reports):
            print(f"  {f}")

        print("\nDeprecated references:")
        for f in sorted(deprecated_refs):
            print(f"  {f}")
    else:
        create_archive_structure(archive_dir)

        print("\nArchiving files...")
        archived_status, skipped_status = archive_files(status_files, archive_dir, 'status-reports', docs_dir, args.dry_run)
        archived_json, skipped_json = archive_files(json_reports, archive_dir, 'json-reports', docs_dir, args.dry_run)
        archived_deprecated, skipped_deprecated = archive_files(deprecated_refs, archive_dir, 'deprecated-refs', docs_dir, args.dry_run)

        print("\nArchived:")
        print(f"  Status/Summary: {len(archived_status)}")
        print(f"  JSON reports: {len(archived_json)}")
        print(f"  Deprecated refs: {len(archived_deprecated)}")
        print(f"  Total: {len(archived_status) + len(archived_json) + len(archived_deprecated)}")

        if skipped_status or skipped_json or skipped_deprecated:
            print(f"\nSkipped: {len(skipped_status) + len(skipped_json) + len(skipped_deprecated)}")

    return 0


if __name__ == '__main__':
    exit(main())
