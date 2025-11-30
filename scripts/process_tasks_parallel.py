#!/usr/bin/env python3
"""
Parallel Task Processing Script

Processes Todo2 tasks in parallel batches. Can run in background to process
ready tasks (no dependencies, not critical) automatically.
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple
from concurrent.futures import ThreadPoolExecutor, as_completed
import time

TODO2_FILE = Path(__file__).parent.parent / ".todo2" / "state.todo2.json"


def load_todos() -> Dict:
    """Load Todo2 state file."""
    with open(TODO2_FILE, 'r') as f:
        return json.load(f)


def save_todos(data: Dict) -> None:
    """Save Todo2 state file."""
    with open(TODO2_FILE, 'w') as f:
        json.dump(data, f, indent=2)


def get_ready_tasks(data: Dict, max_tasks: int = None) -> List[Dict]:
    """
    Get tasks ready for parallel processing.

    Criteria:
    - Status: todo
    - No dependencies
    - Priority: not critical
    """
    todos = data.get('todos', [])

    ready = [
        t for t in todos
        if t.get('status') == 'todo'
        and not t.get('dependencies')
        and t.get('priority', '').lower() != 'critical'
    ]

    # Sort by priority (high -> medium -> low -> none)
    priority_order = {'high': 0, 'medium': 1, 'low': 2, '': 3}
    ready.sort(key=lambda x: priority_order.get(x.get('priority', '').lower(), 3))

    if max_tasks:
        ready = ready[:max_tasks]

    return ready


def process_task(task: Dict, dry_run: bool = True) -> Tuple[str, bool, str]:
    """
    Process a single task.

    Returns: (task_id, success, message)
    """
    task_id = task.get('id')
    task_name = task.get('name', '')[:50]

    if dry_run:
        return task_id, True, f"DRY RUN: Would process {task_name}"

    # For now, just mark as in_progress
    # In real implementation, this would do actual work
    task['status'] = 'in_progress'
    task['updated_at'] = datetime.now(timezone.utc).isoformat()

    # Add processing note
    if 'comments' not in task:
        task['comments'] = []
    task['comments'].append({
        "type": "note",
        "content": f"Parallel processing started at {datetime.now(timezone.utc).isoformat()}",
        "created": datetime.now(timezone.utc).isoformat()
    })

    return task_id, True, f"Processed: {task_name}"


def process_batch(tasks: List[Dict], batch_num: int, dry_run: bool = True) -> Dict:
    """
    Process a batch of tasks in parallel.

    Returns: Summary dict with counts and results
    """
    results = {
        'batch_num': batch_num,
        'total': len(tasks),
        'success': 0,
        'failed': 0,
        'task_results': []
    }

    print(f"\n{'='*60}")
    print(f"Processing Batch {batch_num} ({len(tasks)} tasks)")
    print(f"{'='*60}")

    # Process tasks in parallel (max 5 concurrent)
    with ThreadPoolExecutor(max_workers=5) as executor:
        futures = {executor.submit(process_task, task, dry_run): task for task in tasks}

        for future in as_completed(futures):
            task = futures[future]
            try:
                task_id, success, message = future.result()
                if success:
                    results['success'] += 1
                else:
                    results['failed'] += 1
                results['task_results'].append({
                    'id': task_id,
                    'success': success,
                    'message': message
                })
                print(f"  {'✅' if success else '❌'} {message}")
            except Exception as e:
                results['failed'] += 1
                task_id = task.get('id', 'unknown')
                results['task_results'].append({
                    'id': task_id,
                    'success': False,
                    'message': f"Error: {str(e)}"
                })
                print(f"  ❌ {task_id}: Error - {str(e)}")

    return results


def main():
    """Main processing function."""
    import argparse
    parser = argparse.ArgumentParser(description="Process Todo2 tasks in parallel")
    parser.add_argument('--dry-run', action='store_true', help='Dry run mode (no changes)')
    parser.add_argument('--batch-size', type=int, default=10, help='Tasks per batch')
    parser.add_argument('--max-tasks', type=int, help='Maximum tasks to process')
    parser.add_argument('--max-batches', type=int, help='Maximum batches to process')
    parser.add_argument('--delay', type=float, default=1.0, help='Delay between batches (seconds)')
    args = parser.parse_args()

    data = load_todos()

    # Get ready tasks
    ready_tasks = get_ready_tasks(data, max_tasks=args.max_tasks)

    if not ready_tasks:
        print("No tasks ready for parallel processing")
        return 0

    print(f"\n{'='*60}")
    print(f"PARALLEL TASK PROCESSING")
    print(f"{'='*60}")
    print(f"Mode: {'DRY RUN' if args.dry_run else 'APPLY CHANGES'}")
    print(f"Ready tasks: {len(ready_tasks)}")
    print(f"Batch size: {args.batch_size}")
    print(f"{'='*60}\n")

    # Create batches
    batches = [ready_tasks[i:i+args.batch_size] for i in range(0, len(ready_tasks), args.batch_size)]

    if args.max_batches:
        batches = batches[:args.max_batches]

    print(f"Processing {len(batches)} batch(es)...\n")

    all_results = []
    total_processed = 0

    for i, batch in enumerate(batches, 1):
        batch_results = process_batch(batch, i, dry_run=args.dry_run)
        all_results.append(batch_results)
        total_processed += batch_results['total']

        # Save after each batch if not dry run
        if not args.dry_run:
            save_todos(data)
            print(f"  💾 Saved progress after batch {i}")

        # Delay between batches
        if i < len(batches) and args.delay > 0:
            print(f"  ⏳ Waiting {args.delay}s before next batch...")
            time.sleep(args.delay)

    # Summary
    print(f"\n{'='*60}")
    print(f"PROCESSING COMPLETE")
    print(f"{'='*60}")
    print(f"Batches processed: {len(batches)}")
    print(f"Total tasks: {total_processed}")
    total_success = sum(r['success'] for r in all_results)
    total_failed = sum(r['failed'] for r in all_results)
    print(f"Success: {total_success}")
    print(f"Failed: {total_failed}")
    print(f"{'='*60}\n")

    if not args.dry_run:
        save_todos(data)
        print("✅ All changes saved to .todo2/state.todo2.json")
    else:
        print("⚠️  DRY RUN - No changes made")

    return 0 if total_failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
