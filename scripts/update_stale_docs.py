#!/usr/bin/env python3
"""
Update stale documentation files (date fields). Single entrypoint.

Supports:
  - Explicit file list (--files) or default list
  - Find stale by scan (--docs-dir with optional --days-threshold)
  - Parallel (--jobs N, default 8) or sequential (--jobs 1)
"""
import json
import re
import sys
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
from datetime import datetime, timedelta

DEFAULT_STALE_FILES = [
    "docs/TICKER_TUI_ANALYSIS.md",
    "docs/INTEGRATION_SUMMARY.md",
    "docs/IBKR_PRO_COMMISSIONS.md",
    "docs/CME_FEE_SCHEDULE_REBATES.md",
    "docs/GITHUB_WORKFLOWS.md",
    "docs/PHASE_CONFLICT_ANALYSIS.md",
    "docs/ZORRO_INTEGRATION_PLAN.md",
    "docs/REPOSITORY_RENAME_PLAN.md",
    "docs/ARCHITECTURE_DOCUMENTATION_OPTIONS.md",
    "docs/NEXT_STEPS_RENAME_AND_SPLIT.md",
    "docs/BREADCRUMB_LOGGING_TRADING_TESTING.md",
    "docs/FEATURE_TRACKING.md",
    "docs/API_DOCUMENTATION_SUMMARY.md",
    "docs/MCP_TRADING_SERVER_COMPLETE.md",
    "docs/CBOE_ONE_WEEK_EXPLORATION_PLAN.md",
]


def update_stale_file(args_tuple):
    """Update a single doc file's date fields. Worker-friendly (picklable)."""
    md_file_str, update_date = args_tuple
    md_file = Path(md_file_str)

    if not md_file.exists():
        return {"file": str(md_file), "success": False, "error": "File not found", "updated": False}

    try:
        content = md_file.read_text(encoding="utf-8")
        lines = content.split("\n")
        updated = False

        for i, line in enumerate(lines):
            if re.match(r"^\*\*Date\*\*:\s*\d{4}-\d{2}-\d{2}", line):
                m = re.search(r"\d{4}-\d{2}-\d{2}", line)
                if m and m.group() != update_date:
                    lines[i] = line.replace(m.group(), update_date)
                    updated = True
            elif re.match(r"^Date:\s*\d{4}-\d{2}-\d{2}", line):
                m = re.search(r"\d{4}-\d{2}-\d{2}", line)
                if m and m.group() != update_date:
                    lines[i] = line.replace(m.group(), update_date)
                    updated = True
            elif "Last Updated" in line or "Last updated" in line:
                date_match = re.search(r"\d{4}-\d{2}-\d{2}", line)
                if date_match:
                    old_date = date_match.group()
                    if old_date != update_date:
                        lines[i] = line.replace(old_date, update_date)
                        updated = True

        if not updated and lines and lines[0].startswith("#"):
            if len(lines) > 1 and not lines[1].strip():
                lines.insert(2, f"**Date**: {update_date}")
            else:
                lines.insert(1, f"**Date**: {update_date}")
            updated = True

        if updated:
            md_file.write_text("\n".join(lines), encoding="utf-8")
            return {"file": str(md_file), "success": True, "updated": True}
        return {"file": str(md_file), "success": True, "updated": False}
    except Exception as e:
        return {"file": str(md_file), "success": False, "error": str(e), "updated": False}


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Update stale documentation date fields.")
    parser.add_argument("--files", type=str, nargs="*", help="Explicit list of files to update")
    parser.add_argument("--docs-dir", type=str, default=None, help="Scan directory for stale .md files")
    parser.add_argument("--days-threshold", type=int, default=90, help="Days old to consider stale when scanning")
    parser.add_argument("--jobs", type=int, default=8, help="Parallel workers (1 = sequential)")
    parser.add_argument("--update-date", type=str, default=None, help="Date YYYY-MM-DD (default: today)")
    args = parser.parse_args()

    update_date = args.update_date or datetime.now().strftime("%Y-%m-%d")
    base_dir = Path.cwd()
    stale_files = []

    if args.files:
        stale_files = [base_dir / p if not (p := Path(f)).is_absolute() else p for f in args.files]
    elif args.docs_dir:
        docs_dir = Path(args.docs_dir).resolve()
        if not docs_dir.exists():
            print(f"Error: docs directory not found: {docs_dir}", file=sys.stderr)
            return 1
        threshold = datetime.now() - timedelta(days=args.days_threshold)
        for md in docs_dir.rglob("*.md"):
            try:
                if datetime.fromtimestamp(md.stat().st_mtime) < threshold:
                    stale_files.append(md)
            except OSError:
                pass
        if not stale_files:
            print("No stale files found by mtime scan.", file=sys.stderr)
            return 0
    else:
        stale_files = [base_dir / f for f in DEFAULT_STALE_FILES]

    stale_files = [f.resolve() if not f.is_absolute() else f for f in stale_files]
    stale_files = [f for f in stale_files if f.exists()]

    print(f"Updating dates to: {update_date}, jobs={args.jobs}, files={len(stale_files)}")

    if not stale_files:
        print("No files to update.")
        return 0

    work = [(str(f), update_date) for f in stale_files]
    if args.jobs <= 1:
        all_results = [update_stale_file(t) for t in work]
    else:
        all_results = []
        with ProcessPoolExecutor(max_workers=args.jobs) as executor:
            futures = [executor.submit(update_stale_file, t) for t in work]
            for future in as_completed(futures):
                all_results.append(future.result())

    updated_count = sum(1 for r in all_results if r.get("updated"))
    for r in all_results:
        if r.get("updated"):
            print(f"  Updated {Path(r['file']).name}")

    print(f"Processed: {len(stale_files)}, updated: {updated_count}")

    report_path = base_dir / "docs" / "STALE_DOCS_UPDATE_REPORT.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(all_results, indent=2), encoding="utf-8")
    print(f"Report: {report_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
