#!/usr/bin/env python3
"""
Fix In Progress task statuses by moving tasks with result comments to Review.

Based on Todo2 workflow: Tasks with result comments should be in Review status,
not In Progress.
"""

import re
import sys


def parse_todo2_list_output():
    """
    Parse the Todo2 list output to identify tasks with result comments.
    Format: "Comments: X (type1, type2, result)"
    """
    # Based on the actual list output, these tasks show 'result' in comments
    # We'll verify each one using Todo2 MCP tools
    tasks_to_check = [
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
        "T-206",
    ]

    return tasks_to_check


def main():
    """Main function."""
    print("=" * 90)
    print("IN PROGRESS TASKS STATUS AUDIT")
    print("=" * 90)
    print(
        "\nThis script identifies tasks that should be moved from In Progress to Review"
    )
    print("based on Todo2 workflow: Tasks with result comments should be in Review.\n")

    tasks_to_check = parse_todo2_list_output()
    print(f"Found {len(tasks_to_check)} In Progress tasks to verify\n")
    print("=" * 90)
    print("\n📋 MANUAL VERIFICATION REQUIRED:")
    print("\nUse Todo2 MCP get_todo_details to check each task for result comments.")
    print("Tasks with result comments should be moved to Review status.")
    print("\nExample command:")
    print(
        '  mcp_Todo2_todo2-extension-todo2_get_todo_details({ids: ["T-1", "T-2", ...]})'
    )
    print("\nThen update statuses:")
    print(
        '  mcp_Todo2_todo2-extension-todo2_update_todos({updates: [{id: "T-1", status: "Review"}, ...]})'
    )
    print("\n" + "=" * 90)


if __name__ == "__main__":
    main()
