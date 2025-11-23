"""
MCP Tool Wrapper for Todo Sync

Wraps TodoSyncAutomation to expose as MCP tool.
"""

import json
import logging
import time
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


def sync_todo_tasks(
    dry_run: bool = False,
    output_path: Optional[str] = None
) -> str:
    """
    Synchronize tasks between shared TODO table and Todo2.

    Args:
        dry_run: Whether to simulate sync without making changes (default: False)
        output_path: Path for report output (default: docs/TODO_SYNC_REPORT.md)

    Returns:
        JSON string with sync results
    """
    start_time = time.time()

    try:
        # Import here to avoid circular dependencies
        import sys
        project_root = Path(__file__).parent.parent.parent.parent
        sys.path.insert(0, str(project_root))

        from scripts.automate_todo_sync import TodoSyncAutomation
        from ..error_handler import (
            format_success_response,
            format_error_response,
            log_automation_execution,
            ErrorCode
        )

        # Build config
        config = {
            'output_path': output_path or 'docs/TODO_SYNC_REPORT.md',
            'dry_run': dry_run
        }

        # Create sync automation and run
        sync_automation = TodoSyncAutomation(config)
        results = sync_automation.run()

        # Extract key metrics
        sync_results = results.get('results', {})
        matches = sync_results.get('matches', [])
        conflicts = sync_results.get('conflicts', [])
        new_shared = sync_results.get('new_shared_todos', [])
        new_todo2 = sync_results.get('new_todo2_tasks', [])
        updates = sync_results.get('updates', [])

        # Format response
        response_data = {
            'dry_run': dry_run,
            'matches_found': len(matches),
            'conflicts_detected': len(conflicts),
            'new_shared_todos': len(new_shared),
            'new_todo2_tasks': len(new_todo2),
            'updates_performed': len(updates),
            'report_path': str(Path(config['output_path']).absolute()),
            'status': results.get('status', 'unknown')
        }

        duration = time.time() - start_time
        log_automation_execution('sync_todo_tasks', duration, True)

        return json.dumps(format_success_response(response_data), indent=2)

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('sync_todo_tasks', duration, False, e)

        from ..error_handler import (
            format_error_response,
            ErrorCode
        )
        error_response = format_error_response(e, ErrorCode.AUTOMATION_ERROR)
        return json.dumps(error_response, indent=2)
