#!/usr/bin/env python3
"""
Clean up old Cursor project cache directories for ib_box_spread_full_universal.

Removes cache directories for old paths while keeping the active workspace at:
/Users/davidl/Projects/Trading/ib_box_spread_full_universal
"""

import os
import shutil
from pathlib import Path

CURSOR_PROJECTS = Path.home() / ".cursor" / "projects"
KEEP_DIR = "Volumes-SSD1-APFS-ib-box-spread-full-universal"

def main():
    print("=== Cleaning up old Cursor project cache directories ===\n")

    if not CURSOR_PROJECTS.exists():
        print(f"❌ Cursor projects directory not found: {CURSOR_PROJECTS}")
        return

    # Find old ib_box_spread cache directories
    old_dirs = []
    for item in CURSOR_PROJECTS.iterdir():
        if item.is_dir() and "ib-box-spread" in item.name.lower():
            if KEEP_DIR not in item.name:
                old_dirs.append(item)

    if not old_dirs:
        print("✅ No old project cache directories found")
        return

    print(f"Found {len(old_dirs)} old project cache directory(ies):")
    for dir_path in old_dirs:
        print(f"  - {dir_path.name}")

    print(f"\n✅ KEEP: {KEEP_DIR}")
    print(f"\nThese directories will be removed (safe - they're auto-generated cache)")

    response = input("\nProceed with cleanup? (y/N): ").strip().lower()

    if response == 'y':
        removed = []
        for dir_path in old_dirs:
            try:
                print(f"Removing: {dir_path.name}")
                shutil.rmtree(dir_path)
                removed.append(dir_path.name)
            except Exception as e:
                print(f"  ⚠️  Error removing {dir_path.name}: {e}")

        print(f"\n✅ Cleanup complete! Removed {len(removed)} directory(ies)")

        # Verify keep directory still exists
        keep_path = CURSOR_PROJECTS / KEEP_DIR
        if keep_path.exists():
            print(f"✅ Active workspace cache verified: {keep_path}")
        else:
            print(f"⚠️  Note: Active workspace cache directory not found (will be recreated when needed)")
    else:
        print("Cleanup cancelled")

if __name__ == "__main__":
    main()
