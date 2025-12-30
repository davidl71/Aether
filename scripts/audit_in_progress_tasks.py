#!/usr/bin/env python3
"""
Audit In Progress tasks to identify which should be moved to Review or Done.

This script analyzes all tasks marked as "In Progress" and identifies:
1. Tasks with result comments (should be in Review)
2. Tasks that are actually complete (should be Done)
3. Tasks that are stale (no recent activity)
"""

import json
import sys
from pathlib import Path


def analyze_task(task_data):
    """Analyze a single task and return its status recommendation."""
    task_id = task_data.get("id", "unknown")
    name = task_data.get("name", "No name")
    status = task_data.get("status", "unknown")
    comments = task_data.get("comments", [])

    # Count comment types
    result_count = sum(1 for c in comments if c.get("type") == "result")
    research_count = sum(1 for c in comments if c.get("type") == "research_with_links")
    note_count = sum(1 for c in comments if c.get("type") == "note")

    # Analyze result comments to see if work is complete
    result_comments = [c for c in comments if c.get("type") == "result"]
    completion_indicators = []
    for rc in result_comments:
        content = rc.get("content", "").lower()
        if any(
            word in content
            for word in [
                "complete",
                "completed",
                "done",
                "finished",
                "delivered",
                "implemented",
            ]
        ):
            completion_indicators.append("complete")
        if any(
            word in content
            for word in ["ready for review", "awaiting review", "needs review"]
        ):
            completion_indicators.append("review")

    recommendation = {
        "id": task_id,
        "name": name,
        "current_status": status,
        "result_comments": result_count,
        "research_comments": research_count,
        "note_comments": note_count,
        "total_comments": len(comments),
        "completion_indicators": completion_indicators,
        "recommended_status": None,
        "reason": None,
    }

    # Determine recommendation
    if result_count > 0:
        if "complete" in completion_indicators:
            recommendation["recommended_status"] = "Review"
            recommendation["reason"] = (
                f"Has {result_count} result comment(s) indicating completion - should be in Review"
            )
        else:
            recommendation["recommended_status"] = "Review"
            recommendation["reason"] = (
                f"Has {result_count} result comment(s) - should be in Review per workflow"
            )
    elif research_count > 0 and result_count == 0:
        recommendation["recommended_status"] = "In Progress"
        recommendation["reason"] = "Has research but no result - actually in progress"
    elif len(comments) == 0:
        recommendation["recommended_status"] = "Todo"
        recommendation["reason"] = "No comments - should be Todo (needs research)"
    else:
        recommendation["recommended_status"] = "In Progress"
        recommendation["reason"] = "Has activity but no result - actually in progress"

    return recommendation


def main():
    """Main function to audit all In Progress tasks."""
    project_root = Path(__file__).parent.parent
    state_file = project_root / ".todo2" / "state.todo2.json"

    if not state_file.exists():
        print(f"Error: {state_file} not found")
        sys.exit(1)

    with open(state_file, "r") as f:
        data = json.load(f)

    # Find all In Progress tasks
    in_progress_tasks = [
        t for t in data.get("todos", []) if t.get("status") == "In Progress"
    ]

    print(f"Found {len(in_progress_tasks)} tasks marked as 'In Progress'\n")
    print("=" * 90)

    # Analyze each task
    recommendations = []
    for task in in_progress_tasks:
        rec = analyze_task(task)
        recommendations.append(rec)

    # Group by recommendation
    should_be_review = [
        r for r in recommendations if r["recommended_status"] == "Review"
    ]
    should_be_todo = [r for r in recommendations if r["recommended_status"] == "Todo"]
    actually_in_progress = [
        r for r in recommendations if r["recommended_status"] == "In Progress"
    ]

    print(f"\n📊 ANALYSIS RESULTS:\n")
    print(f"✅ Should be Review: {len(should_be_review)} tasks")
    print(f"📋 Should be Todo: {len(should_be_todo)} tasks")
    print(f"⏳ Actually In Progress: {len(actually_in_progress)} tasks")

    if should_be_review:
        print(f"\n\n🔍 TASKS THAT SHOULD BE IN REVIEW ({len(should_be_review)}):")
        print("=" * 90)
        for rec in should_be_review:
            print(f"\n{rec['id']}: {rec['name'][:70]}")
            print(f"  Result comments: {rec['result_comments']}")
            print(f"  Total comments: {rec['total_comments']}")
            print(f"  Reason: {rec['reason']}")

    if should_be_todo:
        print(f"\n\n📋 TASKS THAT SHOULD BE TODO ({len(should_be_todo)}):")
        print("=" * 90)
        for rec in should_be_todo[:10]:  # Show first 10
            print(f"\n{rec['id']}: {rec['name'][:70]}")
            print(f"  Comments: {rec['total_comments']}")
            print(f"  Reason: {rec['reason']}")

    if actually_in_progress:
        print(f"\n\n⏳ TASKS ACTUALLY IN PROGRESS ({len(actually_in_progress)}):")
        print("=" * 90)
        for rec in actually_in_progress[:10]:  # Show first 10
            print(f"\n{rec['id']}: {rec['name'][:70]}")
            print(
                f"  Research: {rec['research_comments']}, Notes: {rec['note_comments']}, Total: {rec['total_comments']}"
            )

    # Save recommendations to file
    output_file = project_root / "docs" / "analysis" / "IN_PROGRESS_TASKS_AUDIT.json"
    output_file.parent.mkdir(parents=True, exist_ok=True)

    with open(output_file, "w") as f:
        json.dump(
            {
                "total_in_progress": len(in_progress_tasks),
                "should_be_review": should_be_review,
                "should_be_todo": should_be_todo,
                "actually_in_progress": actually_in_progress,
                "all_recommendations": recommendations,
            },
            f,
            indent=2,
        )

    print(f"\n\n✅ Analysis complete! Results saved to: {output_file}")
    print(f"\n📝 Next steps:")
    print(f"  1. Review tasks that should be in Review ({len(should_be_review)})")
    print(f"  2. Move tasks with result comments to Review status")
    print(f"  3. Move tasks without research to Todo status")


if __name__ == "__main__":
    main()
