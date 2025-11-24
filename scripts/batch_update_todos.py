#!/usr/bin/env python3
"""
Batch update TODO2 tasks.

Provides a command-line interface for batch operations on TODO2 tasks,
replacing the need for Python heredocs in terminal commands.

Usage:
    # Batch approve tasks (move Review -> Todo)
    python3 scripts/batch_update_todos.py approve --status Review --clarification-none

    # Update status for specific tasks
    python3 scripts/batch_update_todos.py update-status --task-ids T-156,T-157 --status Todo

    # Add comments to tasks
    python3 scripts/batch_update_todos.py add-comment --task-ids T-156 --comment "Approved for execution"

    # Filter and update by criteria
    python3 scripts/batch_update_todos.py update-status --status Review --filter-tag research --new-status Todo
"""

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Any, Optional

# Project root
project_root = Path(__file__).parent.parent
todo2_path = project_root / '.todo2' / 'state.todo2.json'


def load_todo2_state() -> Dict[str, Any]:
    """Load TODO2 state from JSON file."""
    if not todo2_path.exists():
        print(f"Error: TODO2 state file not found: {todo2_path}", file=sys.stderr)
        sys.exit(1)

    with open(todo2_path, 'r') as f:
        return json.load(f)


def save_todo2_state(state: Dict[str, Any]) -> bool:
    """Save TODO2 state to JSON file."""
    try:
        with open(todo2_path, 'w') as f:
            json.dump(state, f, indent=2)
        return True
    except Exception as e:
        print(f"Error saving TODO2 state: {e}", file=sys.stderr)
        return False


def find_tasks_by_criteria(
    todos: List[Dict[str, Any]],
    status: Optional[str] = None,
    tags: Optional[List[str]] = None,
    clarification_none: bool = False,
    task_ids: Optional[List[str]] = None
) -> List[Dict[str, Any]]:
    """Find tasks matching criteria."""
    matches = []

    for task in todos:
        # Filter by status
        if status and task.get('status') != status:
            continue

        # Filter by tags
        if tags:
            task_tags = [t.lower() for t in task.get('tags', [])]
            if not any(tag.lower() in task_tags for tag in tags):
                continue

        # Filter by clarification requirement
        if clarification_none:
            long_desc = task.get('long_description', '')
            clarification_match = re.search(
                r'Clarification Required[:\s]*\*\* (.*?)(?=\n|$)',
                long_desc,
                re.IGNORECASE | re.DOTALL
            )
            if clarification_match:
                clarification = clarification_match.group(1).strip()
                if clarification.lower() not in ['none', 'n/a', '']:
                    continue
            else:
                # Try alternative pattern
                alt_match = re.search(
                    r'Clarification Required[:\s]*(None|N/A)',
                    long_desc,
                    re.IGNORECASE
                )
                if not alt_match:
                    continue

        # Filter by task IDs
        if task_ids:
            if task.get('id') not in task_ids:
                continue

        matches.append(task)

    return matches


def update_task_status(
    task: Dict[str, Any],
    new_status: str,
    comment: Optional[str] = None
) -> Dict[str, Any]:
    """Update task status and add change record."""
    old_status = task.get('status', '')

    task['status'] = new_status
    task['lastModified'] = datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')

    # Add change record
    if 'changes' not in task:
        task['changes'] = []
    task['changes'].append({
        'field': 'status',
        'oldValue': old_status,
        'newValue': new_status,
        'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
    })

    # Add comment if provided
    if comment:
        if 'comments' not in task:
            task['comments'] = []

        comment_id = f"{task.get('id')}-C-{int(datetime.now(timezone.utc).timestamp())}"
        task['comments'].append({
            'id': comment_id,
            'todoId': task.get('id'),
            'type': 'note',
            'content': comment,
            'created': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
        })

    return task


def add_comment_to_task(
    task: Dict[str, Any],
    comment: str,
    comment_type: str = 'note'
) -> Dict[str, Any]:
    """Add a comment to a task."""
    if 'comments' not in task:
        task['comments'] = []

    comment_id = f"{task.get('id')}-C-{int(datetime.now(timezone.utc).timestamp())}"
    task['comments'].append({
        'id': comment_id,
        'todoId': task.get('id'),
        'type': comment_type,
        'content': comment,
        'created': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
    })

    task['lastModified'] = datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')

    return task


def cmd_approve(args):
    """Approve tasks (move Review -> Todo)."""
    state = load_todo2_state()
    todos = state.get('todos', [])

    # Find tasks to approve
    tasks_to_approve = find_tasks_by_criteria(
        todos,
        status=args.status,
        tags=args.filter_tag,
        clarification_none=args.clarification_none,
        task_ids=args.task_ids.split(',') if args.task_ids else None
    )

    if not tasks_to_approve:
        print("No tasks found matching criteria.")
        return 0

    print(f"Found {len(tasks_to_approve)} tasks to approve:")
    for task in tasks_to_approve:
        print(f"  • {task.get('id')}: {task.get('name', '')[:60]}")

    if not args.yes:
        response = input(f"\nApprove {len(tasks_to_approve)} tasks? (y/N): ")
        if response.lower() != 'y':
            print("Cancelled.")
            return 0

    # Update tasks
    comment = args.comment or "**Batch Approved:** Task approved for autonomous execution."
    for task in tasks_to_approve:
        update_task_status(task, args.new_status, comment)

    # Save state
    if save_todo2_state(state):
        print(f"\n✅ Approved {len(tasks_to_approve)} tasks")
        print(f"   Status changed: {args.status} → {args.new_status}")
        return 0
    else:
        return 1


def cmd_update_status(args):
    """Update status for tasks."""
    state = load_todo2_state()
    todos = state.get('todos', [])

    # Find tasks
    tasks_to_update = find_tasks_by_criteria(
        todos,
        status=args.status,
        tags=args.filter_tag,
        clarification_none=args.clarification_none,
        task_ids=args.task_ids.split(',') if args.task_ids else None
    )

    if not tasks_to_update:
        print("No tasks found matching criteria.")
        return 0

    print(f"Found {len(tasks_to_update)} tasks to update:")
    for task in tasks_to_update:
        print(f"  • {task.get('id')}: {task.get('name', '')[:60]} ({task.get('status')} → {args.new_status})")

    if not args.yes:
        response = input(f"\nUpdate {len(tasks_to_update)} tasks? (y/N): ")
        if response.lower() != 'y':
            print("Cancelled.")
            return 0

    # Update tasks
    for task in tasks_to_update:
        update_task_status(task, args.new_status, args.comment)

    # Save state
    if save_todo2_state(state):
        print(f"\n✅ Updated {len(tasks_to_update)} tasks")
        print(f"   Status changed to: {args.new_status}")
        return 0
    else:
        return 1


def cmd_add_comment(args):
    """Add comments to tasks."""
    state = load_todo2_state()
    todos = state.get('todos', [])

    # Find tasks
    task_ids = args.task_ids.split(',') if args.task_ids else []
    tasks_to_update = [t for t in todos if t.get('id') in task_ids]

    if not tasks_to_update:
        print("No tasks found matching IDs.")
        return 1

    print(f"Adding comment to {len(tasks_to_update)} tasks:")
    for task in tasks_to_update:
        print(f"  • {task.get('id')}: {task.get('name', '')[:60]}")

    # Add comments
    for task in tasks_to_update:
        add_comment_to_task(task, args.comment, args.comment_type)

    # Save state
    if save_todo2_state(state):
        print(f"\n✅ Added comments to {len(tasks_to_update)} tasks")
        return 0
    else:
        return 1


def cmd_list(args):
    """List tasks matching criteria."""
    state = load_todo2_state()
    todos = state.get('todos', [])

    # Find tasks
    tasks = find_tasks_by_criteria(
        todos,
        status=args.status,
        tags=args.filter_tag,
        clarification_none=args.clarification_none,
        task_ids=args.task_ids.split(',') if args.task_ids else None
    )

    if not tasks:
        print("No tasks found matching criteria.")
        return 0

    print(f"Found {len(tasks)} tasks:")
    for task in tasks:
        task_id = task.get('id', '')
        name = task.get('name', '')
        status = task.get('status', '')
        priority = task.get('priority', 'unknown')
        print(f"  • {task_id}: {name[:60]} (Status: {status}, Priority: {priority})")

    return 0


def main():
    parser = argparse.ArgumentParser(
        description='Batch update TODO2 tasks',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )

    subparsers = parser.add_subparsers(dest='command', help='Command to execute')

    # Approve command
    approve_parser = subparsers.add_parser('approve', help='Approve tasks (move Review -> Todo)')
    approve_parser.add_argument('--status', default='Review', help='Current status to filter (default: Review)')
    approve_parser.add_argument('--new-status', default='Todo', help='New status (default: Todo)')
    approve_parser.add_argument('--clarification-none', action='store_true', help='Only approve tasks with no clarification needed')
    approve_parser.add_argument('--filter-tag', help='Filter by tag (e.g., research)')
    approve_parser.add_argument('--task-ids', help='Comma-separated list of task IDs')
    approve_parser.add_argument('--comment', help='Comment to add (default: auto-generated)')
    approve_parser.add_argument('--yes', '-y', action='store_true', help='Skip confirmation')

    # Update status command
    update_parser = subparsers.add_parser('update-status', help='Update task status')
    update_parser.add_argument('--status', help='Current status to filter')
    update_parser.add_argument('--new-status', required=True, help='New status')
    update_parser.add_argument('--clarification-none', action='store_true', help='Only tasks with no clarification needed')
    update_parser.add_argument('--filter-tag', help='Filter by tag')
    update_parser.add_argument('--task-ids', help='Comma-separated list of task IDs')
    update_parser.add_argument('--comment', help='Comment to add')
    update_parser.add_argument('--yes', '-y', action='store_true', help='Skip confirmation')

    # Add comment command
    comment_parser = subparsers.add_parser('add-comment', help='Add comments to tasks')
    comment_parser.add_argument('--task-ids', required=True, help='Comma-separated list of task IDs')
    comment_parser.add_argument('--comment', required=True, help='Comment content')
    comment_parser.add_argument('--comment-type', default='note', help='Comment type (default: note)')

    # List command
    list_parser = subparsers.add_parser('list', help='List tasks matching criteria')
    list_parser.add_argument('--status', help='Filter by status')
    list_parser.add_argument('--filter-tag', help='Filter by tag')
    list_parser.add_argument('--clarification-none', action='store_true', help='Only tasks with no clarification needed')
    list_parser.add_argument('--task-ids', help='Comma-separated list of task IDs')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    # Execute command
    if args.command == 'approve':
        return cmd_approve(args)
    elif args.command == 'update-status':
        return cmd_update_status(args)
    elif args.command == 'add-comment':
        return cmd_add_comment(args)
    elif args.command == 'list':
        return cmd_list(args)
    else:
        parser.print_help()
        return 1


if __name__ == '__main__':
    sys.exit(main())
