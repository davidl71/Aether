#!/usr/bin/env python3
"""
Exarp-compatible wrapper for shared TODO table synchronization.

Synchronizes tasks between agents/shared/TODO_OVERVIEW.md and Todo2.
"""

import json
import re
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

# Status mapping
STATUS_MAP = {
    'pending': 'Todo',
    'in_progress': 'In Progress',
    'completed': 'Done',
    'Todo': 'pending',
    'In Progress': 'in_progress',
    'Done': 'completed',
    'Review': 'in_progress',
    'Cancelled': 'completed'
}


def parse_shared_todo_table(content: str) -> Dict[str, Dict]:
    """Parse shared TODO table from markdown"""
    tasks = {}
    lines = content.split('\n')
    
    # Find table start
    table_start = -1
    for i, line in enumerate(lines):
        if '| TODO ID |' in line:
            table_start = i + 1
            break
    
    if table_start == -1:
        return tasks
    
    # Parse table rows
    for i in range(table_start, len(lines)):
        line = lines[i].strip()
        if not line.startswith('|') or line.startswith('|---'):
            continue
        
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 5:
            continue
        
        try:
            todo_id = parts[1]
            description = parts[2]
            owner = parts[3]
            status = parts[4].lower()
            
            # Skip header row and separator rows
            if todo_id == 'TODO ID' or todo_id.startswith('---') or not todo_id:
                continue
            
            # Skip CI tasks for now (they have different format)
            if todo_id.startswith('CI-'):
                continue
            
            # Only process valid task IDs (numeric or alphanumeric)
            if todo_id and description and todo_id.replace('-', '').replace('_', '').isdigit() or (len(todo_id) <= 10 and todo_id[0].isdigit()):
                tasks[todo_id] = {
                    'id': todo_id,
                    'description': description,
                    'owner': owner,
                    'status': status,
                    'source': 'shared_todo'
                }
        except (IndexError, ValueError):
            continue
    
    return tasks


def parse_todo2_state(state_file: Path) -> Dict[str, Dict]:
    """Parse Todo2 state JSON"""
    if not state_file.exists():
        return {}
    
    with open(state_file, 'r') as f:
        data = json.load(f)
    
    tasks = {}
    for todo in data.get('todos', []):
        todo_id = todo.get('id', '')
        if not todo_id:
            continue
        
        # Check if it's a shared TODO task
        metadata = todo.get('metadata', {})
        if metadata.get('source_type') == 'shared_todo_table':
            source_id = metadata.get('source_id') or todo_id.replace('SHARED-', '')
            tasks[source_id] = {
                'id': todo_id,
                'name': todo.get('name', ''),
                'status': todo.get('status', ''),
                'description': todo.get('long_description', ''),
                'source': 'todo2'
            }
    
    return tasks


def sync_tasks(shared_tasks: Dict, todo2_tasks: Dict, dry_run: bool = True) -> Dict:
    """Synchronize tasks between shared TODO and Todo2"""
    report = {
        'timestamp': datetime.now().isoformat(),
        'dry_run': dry_run,
        'shared_tasks': len(shared_tasks),
        'todo2_tasks': len(todo2_tasks),
        'created': [],
        'updated': [],
        'conflicts': []
    }
    
    # Find tasks that exist in shared TODO but not in Todo2
    for task_id, task in shared_tasks.items():
        if task_id not in todo2_tasks:
            report['created'].append({
                'id': task_id,
                'description': task['description'],
                'status': task['status']
            })
    
    # Find status conflicts
    for task_id in set(shared_tasks.keys()) & set(todo2_tasks.keys()):
        shared_status = STATUS_MAP.get(shared_tasks[task_id]['status'], shared_tasks[task_id]['status'])
        todo2_status = todo2_tasks[task_id]['status']
        
        if shared_status != todo2_status:
            report['conflicts'].append({
                'id': task_id,
                'shared_status': shared_tasks[task_id]['status'],
                'todo2_status': todo2_status,
                'description': shared_tasks[task_id]['description']
            })
    
    return report


def update_shared_todo_table(content: str, updates: List[Dict]) -> str:
    """Update shared TODO table with status changes"""
    lines = content.split('\n')
    result = []
    
    for line in lines:
        if not line.startswith('|') or '| TODO ID |' in line or line.startswith('|---'):
            result.append(line)
            continue
        
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 5:
            result.append(line)
            continue
        
        todo_id = parts[1]
        
        # Find update for this task
        update = next((u for u in updates if u['id'] == todo_id), None)
        if update:
            # Update status
            parts[4] = update['new_status']
            result.append('|' + '|'.join(parts) + '|')
        else:
            result.append(line)
    
    return '\n'.join(result)


def main():
    """Main entry point following Exarp script pattern"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Sync shared TODO table with Todo2 (Exarp-compatible)'
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project root directory (default: current directory)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        default=True,
        help='Dry run mode (default: True)'
    )
    parser.add_argument(
        '--apply',
        action='store_true',
        help='Apply sync (overrides --dry-run)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )
    parser.add_argument(
        '--output',
        type=str,
        help='Output JSON report file (optional)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()
    shared_todo_file = project_dir / 'agents' / 'shared' / 'TODO_OVERVIEW.md'
    todo2_state_file = project_dir / '.todo2' / 'state.todo2.json'

    if not shared_todo_file.exists():
        error_msg = {
            'status': 'error',
            'message': f'Shared TODO file not found: {shared_todo_file}',
            'project_dir': str(project_dir)
        }
        if args.json:
            print(json.dumps(error_msg, indent=2))
        else:
            print(f"❌ Error: {error_msg['message']}", file=sys.stderr)
        sys.exit(1)

    dry_run = not args.apply

    # Parse both sources
    shared_content = shared_todo_file.read_text()
    shared_tasks = parse_shared_todo_table(shared_content)
    todo2_tasks = parse_todo2_state(todo2_state_file)

    # Sync tasks
    report = sync_tasks(shared_tasks, todo2_tasks, dry_run=dry_run)

    # Output results
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"{'DRY RUN MODE' if dry_run else 'SYNC MODE'} - Synchronizing shared TODO table...\n")
        print(f"Shared TODO tasks: {report['shared_tasks']}")
        print(f"Todo2 tasks: {report['todo2_tasks']}")
        print(f"Tasks to create: {len(report['created'])}")
        print(f"Status conflicts: {len(report['conflicts'])}")
        
        if report['created']:
            print("\n📝 Tasks to create in Todo2:")
            for task in report['created'][:5]:
                print(f"  - {task['id']}: {task['description'][:50]}...")
            if len(report['created']) > 5:
                print(f"  ... and {len(report['created']) - 5} more")
        
        if report['conflicts']:
            print("\n⚠️  Status conflicts:")
            for conflict in report['conflicts'][:5]:
                print(f"  - {conflict['id']}: shared={conflict['shared_status']}, todo2={conflict['todo2_status']}")
            if len(report['conflicts']) > 5:
                print(f"  ... and {len(report['conflicts']) - 5} more")

    if args.output:
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        if not args.json:
            print(f"\n✅ Report saved to {args.output}")

    if dry_run and (report['created'] or report['conflicts']):
        if not args.json:
            print("\nRun with --apply to apply sync")

    sys.exit(0)


if __name__ == '__main__':
    main()
