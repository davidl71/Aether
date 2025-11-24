#!/usr/bin/env python3
"""
Resolve task clarifications by updating task descriptions with decisions.

This script automates the process of updating tasks with clarification decisions,
replacing the need for Python heredocs.
"""

import json
import sys
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional


def load_todo2_state(state_file: Path) -> Dict:
    """Load TODO2 state file."""
    with open(state_file, 'r') as f:
        return json.load(f)


def save_todo2_state(state_file: Path, data: Dict) -> None:
    """Save TODO2 state file."""
    with open(state_file, 'w') as f:
        json.dump(data, f, indent=2)


def update_task_clarification(
    task: Dict,
    clarification: str,
    decision: str,
    move_to_todo: bool = True
) -> bool:
    """Update a task with clarification decision."""
    long_desc = task.get('long_description', '')
    
    # Update or add clarification section
    import re
    
    # Pattern to find existing clarification section
    clar_pattern = r'Clarification Required:\s*\*\*?\s*[^\n]+(?:\n[^\n]+)*'
    
    if 'Clarification Required:' in long_desc:
        # Replace existing clarification
        replacement = f"Clarification Required: **{clarification}**\n\n**Decision Made:** {decision}"
        long_desc = re.sub(clar_pattern, replacement, long_desc, flags=re.IGNORECASE)
    else:
        # Add new clarification section at the beginning
        if long_desc.strip():
            long_desc = f"**Clarification Required:** **{clarification}**\n\n**Decision Made:** {decision}\n\n{long_desc}"
        else:
            long_desc = f"**Clarification Required:** **{clarification}**\n\n**Decision Made:** {decision}"
    
    task['long_description'] = long_desc
    
    # Add note comment
    if 'comments' not in task:
        task['comments'] = []
    
    task['comments'].append({
        'type': 'note',
        'content': f"**Clarification Resolved:** {decision[:150]}...",
        'timestamp': datetime.now().isoformat() + 'Z'
    })
    
    # Update status if requested
    if move_to_todo:
        task['status'] = 'Todo'
        task['lastModified'] = datetime.now().isoformat() + 'Z'
    
    return True


def resolve_from_file(decisions_file: Path, state_file: Path, dry_run: bool = False) -> Dict:
    """Resolve clarifications from a JSON file."""
    with open(decisions_file, 'r') as f:
        decisions = json.load(f)
    
    data = load_todo2_state(state_file)
    todos = data.get('todos', [])
    
    results = {
        'updated': [],
        'not_found': [],
        'errors': []
    }
    
    for task_id, decision_data in decisions.items():
        task = next((t for t in todos if t.get('id') == task_id), None)
        
        if not task:
            results['not_found'].append(task_id)
            continue
        
        try:
            clarification = decision_data.get('clarification', 'Resolved')
            decision = decision_data.get('decision', '')
            move_to_todo = decision_data.get('move_to_todo', True)
            
            if not dry_run:
                update_task_clarification(task, clarification, decision, move_to_todo)
            
            results['updated'].append({
                'task_id': task_id,
                'name': task.get('name', '')[:50],
                'status': 'Todo' if move_to_todo and not dry_run else task.get('status')
            })
        except Exception as e:
            results['errors'].append({
                'task_id': task_id,
                'error': str(e)
            })
    
    if not dry_run:
        save_todo2_state(state_file, data)
    
    return results


def resolve_interactive(task_id: str, clarification: str, decision: str, 
                       state_file: Path, move_to_todo: bool = True, 
                       dry_run: bool = False) -> bool:
    """Resolve a single task clarification interactively."""
    data = load_todo2_state(state_file)
    todos = data.get('todos', [])
    
    task = next((t for t in todos if t.get('id') == task_id), None)
    
    if not task:
        print(f"❌ Task {task_id} not found")
        return False
    
    if not dry_run:
        update_task_clarification(task, clarification, decision, move_to_todo)
        save_todo2_state(state_file, data)
        print(f"✅ Updated {task_id}: {task.get('name', '')[:50]}")
    else:
        print(f"🔍 Would update {task_id}: {task.get('name', '')[:50]}")
        print(f"   Clarification: {clarification}")
        print(f"   Decision: {decision}")
        print(f"   Move to Todo: {move_to_todo}")
    
    return True


def main():
    parser = argparse.ArgumentParser(
        description='Resolve task clarifications by updating task descriptions with decisions'
    )
    parser.add_argument(
        '--file',
        type=Path,
        help='JSON file with decisions (format: {"T-36": {"clarification": "...", "decision": "..."}})'
    )
    parser.add_argument(
        '--task-id',
        help='Single task ID to update'
    )
    parser.add_argument(
        '--clarification',
        help='Clarification text (required with --task-id)'
    )
    parser.add_argument(
        '--decision',
        help='Decision text (required with --task-id)'
    )
    parser.add_argument(
        '--no-move-to-todo',
        action='store_true',
        help='Do not move task to Todo status after resolving'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Preview changes without applying them'
    )
    parser.add_argument(
        '--state-file',
        type=Path,
        default=Path('.todo2/state.todo2.json'),
        help='Path to TODO2 state file (default: .todo2/state.todo2.json)'
    )
    
    args = parser.parse_args()
    
    if args.file:
        # Batch resolve from file
        if not args.file.exists():
            print(f"❌ File not found: {args.file}")
            sys.exit(1)
        
        results = resolve_from_file(args.file, args.state_file, args.dry_run)
        
        print("\n" + "=" * 70)
        print("RESOLUTION RESULTS")
        print("=" * 70)
        print(f"\n✅ Updated: {len(results['updated'])} tasks")
        for item in results['updated']:
            print(f"   {item['task_id']}: {item['name']} → {item['status']}")
        
        if results['not_found']:
            print(f"\n⚠️  Not Found: {len(results['not_found'])} tasks")
            for task_id in results['not_found']:
                print(f"   {task_id}")
        
        if results['errors']:
            print(f"\n❌ Errors: {len(results['errors'])} tasks")
            for item in results['errors']:
                print(f"   {item['task_id']}: {item['error']}")
        
        if args.dry_run:
            print("\n🔍 DRY RUN - No changes made")
        else:
            print("\n✅ Changes saved to state file")
    
    elif args.task_id:
        # Single task resolve
        if not args.clarification or not args.decision:
            print("❌ --clarification and --decision required with --task-id")
            sys.exit(1)
        
        success = resolve_interactive(
            args.task_id,
            args.clarification,
            args.decision,
            args.state_file,
            move_to_todo=not args.no_move_to_todo,
            dry_run=args.dry_run
        )
        
        if not success:
            sys.exit(1)
    
    else:
        parser.print_help()
        print("\n" + "=" * 70)
        print("EXAMPLES")
        print("=" * 70)
        print("\n1. Resolve from JSON file:")
        print("   python3 scripts/resolve_task_clarifications.py --file decisions.json")
        print("\n2. Resolve single task:")
        print("   python3 scripts/resolve_task_clarifications.py \\")
        print("     --task-id T-76 \\")
        print("     --clarification 'Storage format preference' \\")
        print("     --decision 'Use JSON config for simplicity'")
        print("\n3. Dry run (preview):")
        print("   python3 scripts/resolve_task_clarifications.py --file decisions.json --dry-run")
        sys.exit(1)


if __name__ == '__main__':
    main()

