#!/usr/bin/env python3
"""
Script to update In Progress tasks with result comments to Review status.

This script generates the proper update commands for Todo2 MCP tool.
"""

import json
from pathlib import Path


def get_all_in_progress_task_ids():
    """Get list of all In Progress task IDs that should be moved to Review."""
    # Based on audit findings - all 48 tasks have result comments
    task_ids = [
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
    return task_ids


def generate_update_batches(task_ids, batch_size=10):
    """Generate update batches for Todo2 MCP tool."""
    batches = []
    for i in range(0, len(task_ids), batch_size):
        batch = task_ids[i : i + batch_size]
        batches.append(batch)
    return batches


def generate_mcp_command(batch):
    """Generate MCP tool command for a batch of tasks."""
    updates = [{"id": task_id, "status": "Review"} for task_id in batch]
    command = {"updates": updates}
    return json.dumps(command, indent=2)


def main():
    """Main function."""
    print("=" * 90)
    print("Todo2 Task Status Update Helper")
    print("=" * 90)
    print(
        "\nThis script generates update commands to move In Progress tasks to Review status."
    )
    print(
        "All 48 tasks have result comments and should be in Review per Todo2 workflow.\n"
    )

    task_ids = get_all_in_progress_task_ids()
    print(f"Total tasks to update: {len(task_ids)}\n")

    # Generate batches
    batches = generate_update_batches(task_ids, batch_size=10)
    print(f"Generated {len(batches)} batches of updates\n")

    # Save to file
    output_file = (
        Path(__file__).parent.parent / "docs" / "analysis" / "TASK_UPDATE_COMMANDS.json"
    )
    output_file.parent.mkdir(parents=True, exist_ok=True)

    all_updates = []
    for i, batch in enumerate(batches, 1):
        print(f"Batch {i} ({len(batch)} tasks):")
        print(f"  Tasks: {', '.join(batch)}")
        command = generate_mcp_command(batch)
        print(f"  Command JSON:\n{command}\n")

        all_updates.extend([{"id": task_id, "status": "Review"} for task_id in batch])

    # Save all updates to JSON file
    with open(output_file, "w") as f:
        json.dump(
            {
                "total_tasks": len(task_ids),
                "batches": len(batches),
                "all_updates": all_updates,
                "batched_updates": [
                    [{"id": task_id, "status": "Review"} for task_id in batch]
                    for batch in batches
                ],
            },
            f,
            indent=2,
        )

    print(f"\n✅ Update commands saved to: {output_file}")
    print("\n📝 Usage Instructions:")
    print("1. Use the Todo2 MCP tool: mcp_Todo2_todo2-extension-todo2_update_todos")
    print("2. Provide the 'updates' parameter with the JSON array from the file")
    print("3. Or update tasks manually through the Todo2 interface")
    print("\n" + "=" * 90)


if __name__ == "__main__":
    main()
