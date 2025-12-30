#!/usr/bin/env python3
"""
Direct update script for Todo2 tasks - modifies state.todo2.json directly.

⚠️ WARNING: This script modifies the Todo2 state file directly.
Back up .todo2/state.todo2.json before running!
"""

import json
import sys
from pathlib import Path


def get_tasks_to_update():
    """Get list of all In Progress task IDs that should be moved to Review."""
    return [
        "T-1",
        "T-2",
        "T-9",
        "T-14",
        "T-15",
        "T-22",
        "T-48",
        "T-56",
        "T-57",
        "T-58",
        "T-59",
        "T-85",
        "T-86",
        "T-87",
        "T-88",
        "T-89",
        "T-90",
        "T-91",
        "T-93",
        "T-94",
        "T-96",
        "T-97",
        "T-139",
        "T-162",
        "T-163",
        "T-164",
        "T-167",
        "T-169",
        "T-171",
        "T-172",
        "T-173",
        "T-174",
        "T-175",
        "T-176",
        "T-177",
        "T-178",
        "T-179",
        "T-180",
        "T-185",
        "T-186",
        "T-187",
        "T-188",
        "T-189",
        "T-191",
        "T-192",
        "T-194",
        "T-197",
        "T-208",
    ]


def main():
    """Main function to update Todo2 state file directly."""
    project_root = Path(__file__).parent.parent
    state_file = project_root / ".todo2" / "state.todo2.json"

    if not state_file.exists():
        print(f"Error: {state_file} not found")
        sys.exit(1)

    # Backup the file
    backup_file = state_file.with_suffix(".json.backup")
    print(f"Creating backup: {backup_file}")
    import shutil

    shutil.copy2(state_file, backup_file)

    # Load state
    with open(state_file, "r") as f:
        data = json.load(f)

    # Get tasks to update
    task_ids = get_tasks_to_update()
    print(f"\nUpdating {len(task_ids)} tasks from 'In Progress' to 'Review'...")

    # Update tasks
    updated_count = 0
    for task in data.get("todos", []):
        if task.get("id") in task_ids:
            if task.get("status") == "In Progress":
                task["status"] = "Review"
                updated_count += 1
                print(f"  ✅ Updated {task.get('id')}: {task.get('name', '')[:50]}...")

    if updated_count == 0:
        print("\n⚠️  No tasks were updated. They may already be in Review status.")
    else:
        # Save updated state
        with open(state_file, "w") as f:
            json.dump(data, f, indent=2)

        print(f"\n✅ Successfully updated {updated_count} tasks!")
        print(f"📁 Backup saved to: {backup_file}")
        print("\n⚠️  Please verify the changes in Todo2 interface.")


if __name__ == "__main__":
    print("=" * 90)
    print("Todo2 Direct State Update Script")
    print("=" * 90)
    print("\n⚠️  WARNING: This script modifies .todo2/state.todo2.json directly!")
    print("A backup will be created automatically.\n")

    response = input("Continue? (yes/no): ")
    if response.lower() not in ["yes", "y"]:
        print("Aborted.")
        sys.exit(0)

    main()
