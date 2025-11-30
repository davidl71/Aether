#!/usr/bin/env python3
"""
Automate Todo2 Duplicate Cleanup

This script automates the cleanup of duplicate tasks identified by duplicate detection:
1. Auto-closes completed duplicate tasks
2. Fixes duplicate task IDs
3. Merges duplicate automation tasks
4. Consolidates completed work

Can run in background/parallel without human intervention.
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple

TODO2_FILE = Path(__file__).parent.parent / ".todo2" / "state.todo2.json"


def load_todos() -> Dict:
    """Load Todo2 state file."""
    with open(TODO2_FILE, 'r') as f:
        return json.load(f)


def save_todos(data: Dict) -> None:
    """Save Todo2 state file."""
    with open(TODO2_FILE, 'w') as f:
        json.dump(data, f, indent=2)


def auto_close_completed_link_tasks(data: Dict, dry_run: bool = True) -> Tuple[int, List[str]]:
    """
    Auto-close 'Fix broken documentation links' tasks that are todo but work is complete.

    Returns: (count_closed, task_ids)
    """
    todos = data.get('todos', [])
    closed = []

    # Find todo tasks for broken links (work is already complete - 0 links remaining)
    link_tasks = [
        t for t in todos
        if 'broken documentation links' in t.get('name', '').lower()
        and t.get('status') == 'todo'
    ]

    for task in link_tasks:
        if not dry_run:
            task['status'] = 'done'
            task['updated_at'] = datetime.now(timezone.utc).isoformat()
            # Add note comment
            if 'comments' not in task:
                task['comments'] = []
            task['comments'].append({
                "type": "note",
                "content": "Auto-closed: Work already complete (0 broken links remaining per T-20251130001249)",
                "created": datetime.now(timezone.utc).isoformat()
            })
        closed.append(task['id'])

    return len(closed), closed


def fix_duplicate_task_id(data: Dict, duplicate_id: str, dry_run: bool = True) -> Tuple[bool, str]:
    """
    Fix duplicate task ID by assigning new ID to one of the duplicates.

    Returns: (success, new_id)
    """
    todos = data.get('todos', [])

    # Find all tasks with the duplicate ID
    duplicates = [t for t in todos if t.get('id') == duplicate_id]

    if len(duplicates) < 2:
        return False, ""

    # Keep ID for 'done' task, assign new ID to 'in_progress' task
    done_task = next((t for t in duplicates if t.get('status') == 'done'), None)
    in_progress_task = next((t for t in duplicates if t.get('status') == 'in_progress'), None)

    if not done_task or not in_progress_task:
        return False, ""

    # Generate new ID for in_progress task
    new_id = f"AUTO-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}"

    if not dry_run:
        # Update ID
        old_id = in_progress_task['id']
        in_progress_task['id'] = new_id

        # Update any references in dependencies
        for task in todos:
            deps = task.get('dependencies', [])
            if old_id in deps:
                deps[deps.index(old_id)] = new_id

        # Update comment IDs
        for comment in in_progress_task.get('comments', []):
            if comment.get('todoId') == old_id:
                comment['todoId'] = new_id
            if comment.get('id', '').startswith(old_id):
                comment['id'] = comment['id'].replace(old_id, new_id)

        in_progress_task['updated_at'] = datetime.now(timezone.utc).isoformat()

    return True, new_id


def auto_merge_duplicate_automation_tasks(data: Dict, task_name: str, dry_run: bool = True) -> Tuple[int, List[str]]:
    """
    Auto-merge duplicate automation tasks with the same name.
    Keeps one task, merges comments/results, marks others as merged.

    Returns: (count_merged, merged_task_ids)
    """
    todos = data.get('todos', [])

    # Find all tasks with the exact name
    duplicates = [t for t in todos if t.get('name') == task_name]

    if len(duplicates) <= 1:
        return 0, []

    # Keep the oldest task (first created)
    duplicates.sort(key=lambda x: x.get('created_at', ''))
    keep_task = duplicates[0]
    merge_tasks = duplicates[1:]

    # Merge comments/results from all tasks
    all_comments = []
    for task in duplicates:
        all_comments.extend(task.get('comments', []))

    merged_ids = [t['id'] for t in merge_tasks]

    if not dry_run:
        # Update keep task with merged comments
        keep_task['comments'] = all_comments
        keep_task['updated_at'] = datetime.now(timezone.utc).isoformat()

        # Mark merge tasks as done with note
        for task in merge_tasks:
            task['status'] = 'done'
            task['updated_at'] = datetime.now(timezone.utc).isoformat()
            if 'comments' not in task:
                task['comments'] = []
            task['comments'].append({
                "type": "note",
                "content": f"Merged into {keep_task['id']}",
                "created": datetime.now(timezone.utc).isoformat()
            })

    return len(merge_tasks), merged_ids


def main():
    """Main automation function."""
    import argparse
    parser = argparse.ArgumentParser(description="Automate Todo2 duplicate cleanup")
    parser.add_argument('--dry-run', action='store_true', help='Dry run mode (no changes)')
    parser.add_argument('--fix-duplicate-id', action='store_true', help='Fix duplicate task ID')
    parser.add_argument('--close-link-tasks', action='store_true', help='Auto-close completed link tasks')
    parser.add_argument('--merge-automation', action='store_true', help='Merge duplicate automation tasks')
    parser.add_argument('--all', action='store_true', help='Run all automations')
    args = parser.parse_args()

    if not args.dry_run and not args.all and not any([args.fix_duplicate_id, args.close_link_tasks, args.merge_automation]):
        print("Error: Must specify --all or at least one automation action")
        sys.exit(1)

    data = load_todos()
    changes_made = False

    print("=== Todo2 Duplicate Cleanup Automation ===\n")
    print(f"Mode: {'DRY RUN' if args.dry_run else 'APPLY CHANGES'}\n")

    # 1. Auto-close completed link tasks
    if args.all or args.close_link_tasks:
        count, task_ids = auto_close_completed_link_tasks(data, dry_run=args.dry_run)
        if count > 0:
            print(f"✅ Auto-closed {count} completed link tasks:")
            for tid in task_ids:
                print(f"   • {tid}")
            changes_made = True

    # 2. Fix duplicate task ID
    if args.all or args.fix_duplicate_id:
        success, new_id = fix_duplicate_task_id(data, "AUTO-20251129200049", dry_run=args.dry_run)
        if success:
            print(f"\n✅ Fixed duplicate task ID:")
            print(f"   • AUTO-20251129200049 (in_progress) → {new_id}")
            changes_made = True

    # 3. Merge duplicate automation tasks
    if args.all or args.merge_automation:
        count, merged_ids = auto_merge_duplicate_automation_tasks(
            data, "Automation: Documentation Health Analysis", dry_run=args.dry_run
        )
        if count > 0:
            print(f"\n✅ Merged {count} duplicate 'Documentation Health Analysis' tasks")
            print(f"   • Kept: AUTO-20251129173956")
            print(f"   • Merged: {len(merged_ids)} tasks")
            changes_made = True

    if changes_made and not args.dry_run:
        save_todos(data)
        print("\n✅ Changes saved to .todo2/state.todo2.json")
    elif args.dry_run:
        print("\n⚠️  DRY RUN - No changes made")
    else:
        print("\n⚠️  No changes to apply")

    return 0 if changes_made or args.dry_run else 1


if __name__ == '__main__':
    sys.exit(main())
