"""
MCP Tool Wrapper for Daily Automation

Wraps DailyAutomation to expose as MCP tool.
"""

import json
import logging
import time
from pathlib import Path
from typing import List, Optional

logger = logging.getLogger(__name__)

# Import error handler at module level to avoid scoping issues
try:
    from ..error_handler import (
        format_success_response,
        format_error_response,
        log_automation_execution,
        ErrorCode
    )
except ImportError:
    import sys
    from pathlib import Path
    server_dir = Path(__file__).parent.parent
    sys.path.insert(0, str(server_dir))
    try:
        from error_handler import (
            format_success_response,
            format_error_response,
            log_automation_execution,
            ErrorCode
        )
    except ImportError:
        # Fallback: define minimal versions if import fails
        def format_success_response(data, message=None):
            return {"success": True, "data": data, "timestamp": time.time()}
        def format_error_response(error, error_code, include_traceback=False):
            return {"success": False, "error": {"code": str(error_code), "message": str(error)}}
        def log_automation_execution(name, duration, success, error=None):
            logger.info(f"{name}: {duration:.2f}s, success={success}")
        class ErrorCode:
            AUTOMATION_ERROR = "AUTOMATION_ERROR"


def run_daily_automation(
    tasks: Optional[List[str]] = None,
    include_slow: bool = False,
    dry_run: bool = False,
    output_path: Optional[str] = None
) -> str:
    """
    [HINT: Daily automation. Returns tasks run, success rate, summary, report path.]

    Run routine daily maintenance tasks and generate a combined summary report.

    Available tasks:
    - docs_health: Documentation health check
    - todo2_alignment: Todo2 alignment analysis
    - duplicate_detection: Duplicate task detection
    - dependency_security: Dependency security scan (slow)
    - external_tool_hints: Add Context7 hints to documentation

    Args:
        tasks: List of task IDs to run (default: quick tasks only)
        include_slow: Include slow tasks like dependency security scan (default: False)
        dry_run: Preview changes without applying (default: False)
        output_path: Path for report output (default: docs/DAILY_AUTOMATION_REPORT.md)

    Returns:
        JSON string with automation results
    """
    start_time = time.time()

    try:
        # Import from package
        from project_management_automation.scripts.automate_daily import DailyAutomation
        from project_management_automation.utils import find_project_root

        # Find project root
        project_root = find_project_root(Path(__file__).parent.parent.parent.parent)

        # Build config
        config = {
            'tasks': tasks or ['docs_health', 'todo2_alignment', 'duplicate_detection'],
            'include_slow': include_slow,
            'dry_run': dry_run,
            'output_path': output_path or 'docs/DAILY_AUTOMATION_REPORT.md'
        }

        # Create automation and run
        automation = DailyAutomation(config, project_root)
        results = automation.run()

        # Extract key metrics
        summary = results.get('results', {}).get('summary', {})
        tasks_run = results.get('results', {}).get('tasks_run', [])
        tasks_succeeded = results.get('results', {}).get('tasks_succeeded', [])
        tasks_failed = results.get('results', {}).get('tasks_failed', [])

        # Format response
        response_data = {
            'tasks_run': len(tasks_run),
            'tasks_succeeded': len(tasks_succeeded),
            'tasks_failed': len(tasks_failed),
            'success_rate': summary.get('success_rate', 0),
            'duration_seconds': summary.get('duration_seconds', 0),
            'report_path': str(Path(config['output_path']).absolute()),
            'dry_run': dry_run,
            'status': results.get('status', 'unknown'),
            'task_results': [
                {
                    'task_id': tr.get('task_id'),
                    'task_name': tr.get('task_name'),
                    'status': tr.get('status'),
                    'duration_seconds': tr.get('duration_seconds', 0)
                }
                for tr in tasks_run
            ]
        }

        duration = time.time() - start_time
        log_automation_execution('run_daily_automation', duration, True)

        return json.dumps(
            format_success_response(response_data, "Daily automation completed"),
            indent=2
        )

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('run_daily_automation', duration, False, str(e))
        logger.error(f"Error in daily automation: {e}", exc_info=True)

        return json.dumps(
            format_error_response(
                f"Daily automation failed: {str(e)}",
                ErrorCode.AUTOMATION_ERROR,
                include_traceback=True
            ),
            indent=2
        )
