"""
Task Clarification Resolution Tool

MCP Tool for resolving task clarifications by updating task descriptions with decisions.
Replaces Python heredocs with a clean MCP interface.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional


def resolve_task_clarification(
    task_id: str,
    clarification: str,
    decision: str,
    move_to_todo: bool = True,
    dry_run: bool = False
) -> Dict[str, Any]:
    """
    Resolve a single task clarification.

    Args:
        task_id: Task ID (e.g., "T-76")
        clarification: Clarification text
        decision: Decision text
        move_to_todo: Whether to move task to Todo status (default: True)
        dry_run: Preview mode without making changes (default: False)

    Returns:
        Dictionary with resolution result
    """
    project_root = Path(__file__).parent.parent.parent.parent
    script_path = project_root / "scripts" / "resolve_task_clarifications.py"
    state_file = project_root / ".todo2" / "state.todo2.json"

    if not script_path.exists():
        return {
            "status": "error",
            "error": f"Script not found: {script_path}"
        }

    # Build command
    cmd = [
        sys.executable,
        str(script_path),
        "--task-id", task_id,
        "--clarification", clarification,
        "--decision", decision,
        "--state-file", str(state_file)
    ]

    if not move_to_todo:
        cmd.append("--no-move-to-todo")

    if dry_run:
        cmd.append("--dry-run")

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            return {
                "status": "error",
                "error": result.stderr or result.stdout,
                "returncode": result.returncode
            }

        # Parse output to extract task info
        output = result.stdout
        success = "✅" in output or "Updated" in output

        return {
            "status": "success" if success else "error",
            "task_id": task_id,
            "dry_run": dry_run,
            "moved_to_todo": move_to_todo and success and not dry_run,
            "output": output
        }

    except subprocess.TimeoutExpired:
        return {
            "status": "error",
            "error": "Command timed out after 30 seconds"
        }
    except Exception as e:
        return {
            "status": "error",
            "error": str(e)
        }


def resolve_multiple_clarifications(
    decisions: Dict[str, Dict[str, str]],
    move_to_todo: bool = True,
    dry_run: bool = False
) -> Dict[str, Any]:
    """
    Resolve multiple task clarifications from a decisions dictionary.

    Args:
        decisions: Dictionary mapping task IDs to decision data:
                   {"T-76": {"clarification": "...", "decision": "..."}, ...}
        move_to_todo: Whether to move tasks to Todo status (default: True)
        dry_run: Preview mode without making changes (default: False)

    Returns:
        Dictionary with resolution results
    """
    project_root = Path(__file__).parent.parent.parent.parent
    script_path = project_root / "scripts" / "resolve_task_clarifications.py"
    state_file = project_root / ".todo2" / "state.todo2.json"

    if not script_path.exists():
        return {
            "status": "error",
            "error": f"Script not found: {script_path}"
        }

    # Create temporary decisions file
    import tempfile
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(decisions, f, indent=2)
        decisions_file = Path(f.name)

    try:
        # Build command
        cmd = [
            sys.executable,
            str(script_path),
            "--file", str(decisions_file),
            "--state-file", str(state_file)
        ]

        if dry_run:
            cmd.append("--dry-run")

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=60
        )

        # Clean up temp file
        decisions_file.unlink()

        if result.returncode != 0:
            return {
                "status": "error",
                "error": result.stderr or result.stdout,
                "returncode": result.returncode
            }

        # Parse output
        output = result.stdout
        updated_count = output.count("✅") or output.count("Updated")

        return {
            "status": "success",
            "tasks_processed": len(decisions),
            "tasks_updated": updated_count,
            "dry_run": dry_run,
            "moved_to_todo": move_to_todo and not dry_run,
            "output": output
        }

    except subprocess.TimeoutExpired:
        decisions_file.unlink()
        return {
            "status": "error",
            "error": "Command timed out after 60 seconds"
        }
    except Exception as e:
        if decisions_file.exists():
            decisions_file.unlink()
        return {
            "status": "error",
            "error": str(e)
        }


def list_tasks_awaiting_clarification() -> Dict[str, Any]:
    """
    List all tasks in Review status that need clarification.

    Returns:
        Dictionary with list of tasks awaiting clarification
    """
    project_root = Path(__file__).parent.parent.parent.parent
    state_file = project_root / ".todo2" / "state.todo2.json"

    if not state_file.exists():
        return {
            "status": "error",
            "error": f"State file not found: {state_file}"
        }

    try:
        with open(state_file, 'r') as f:
            data = json.load(f)

        todos = data.get('todos', [])
        review_tasks = [t for t in todos if t.get('status') == 'Review']

        # Extract clarification questions
        import re
        tasks_with_clarifications = []

        for task in review_tasks:
            task_id = task.get('id', '')
            name = task.get('name', '')
            long_desc = task.get('long_description', '')
            priority = task.get('priority', 'medium')

            # Extract clarification requirement
            clar_match = re.search(
                r'Clarification Required:\s*\*\*?\s*(.+?)(?:\n|$)',
                long_desc,
                re.IGNORECASE | re.DOTALL
            )
            clarification = clar_match.group(1).strip() if clar_match else 'No clarification text found'

            tasks_with_clarifications.append({
                'task_id': task_id,
                'name': name,
                'priority': priority,
                'clarification': clarification[:200] + '...' if len(clarification) > 200 else clarification
            })

        return {
            "status": "success",
            "total_tasks": len(tasks_with_clarifications),
            "tasks": tasks_with_clarifications
        }

    except Exception as e:
        return {
            "status": "error",
            "error": str(e)
        }
