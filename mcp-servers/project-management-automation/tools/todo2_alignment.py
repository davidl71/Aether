"""
MCP Tool Wrapper for Todo2 Alignment Analysis

Wraps Todo2AlignmentAnalyzerV2 to expose as MCP tool.
"""

import json
import logging
import time
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


def analyze_todo2_alignment(
    create_followup_tasks: bool = True,
    output_path: Optional[str] = None
) -> str:
    """
    Analyze task alignment with project goals, find misaligned tasks.

    Args:
        create_followup_tasks: Whether to create Todo2 tasks for misaligned tasks
        output_path: Path for report output (default: docs/TODO2_ALIGNMENT_REPORT.md)

    Returns:
        JSON string with analysis results
    """
    start_time = time.time()

    try:
        # Import here to avoid circular dependencies
        import sys
        project_root = Path(__file__).parent.parent.parent.parent
        sys.path.insert(0, str(project_root))

        from scripts.automate_todo2_alignment_v2 import Todo2AlignmentAnalyzerV2
        from ..error_handler import (
            format_success_response,
            format_error_response,
            log_automation_execution,
            ErrorCode
        )

        # Build config
        config = {
            'create_followup_tasks': create_followup_tasks,
            'output_path': output_path or 'docs/TODO2_ALIGNMENT_REPORT.md'
        }

        # Create analyzer and run
        analyzer = Todo2AlignmentAnalyzerV2(config)
        results = analyzer.run()

        # Extract key metrics
        analysis_results = results.get('results', {})
        alignment_scores = analysis_results.get('alignment_scores', {})
        misaligned_tasks = analysis_results.get('misaligned_tasks', [])

        # Format response
        response_data = {
            'total_tasks_analyzed': len(analysis_results.get('tasks_analyzed', [])),
            'misaligned_count': len(misaligned_tasks),
            'average_alignment_score': alignment_scores.get('average', 0),
            'report_path': str(Path(config['output_path']).absolute()),
            'tasks_created': len(results.get('followup_tasks', [])) if create_followup_tasks else 0,
            'status': results.get('status', 'unknown')
        }

        duration = time.time() - start_time
        log_automation_execution('analyze_todo2_alignment', duration, True)

        return json.dumps(format_success_response(response_data), indent=2)

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('analyze_todo2_alignment', duration, False, e)

        from ..error_handler import (
            format_error_response,
            ErrorCode
        )
        error_response = format_error_response(e, ErrorCode.AUTOMATION_ERROR)
        return json.dumps(error_response, indent=2)
