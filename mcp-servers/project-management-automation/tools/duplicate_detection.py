"""
MCP Tool Wrapper for Duplicate Task Detection

Wraps Todo2DuplicateDetector to expose as MCP tool.
"""

import json
import logging
import time
from pathlib import Path
from typing import Optional

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


def detect_duplicate_tasks(
    similarity_threshold: float = 0.85,
    auto_fix: bool = False,
    output_path: Optional[str] = None
) -> str:
    """
    Find and consolidate duplicate Todo2 tasks.

    Args:
        similarity_threshold: Similarity threshold for duplicate detection (0.0-1.0)
        auto_fix: Whether to automatically fix duplicates (default: False)
        output_path: Path for report output (default: docs/TODO2_DUPLICATE_DETECTION_REPORT.md)

    Returns:
        JSON string with detection results
    """
    start_time = time.time()

    try:
        # Import from package
        from project_management_automation.scripts.automate_todo2_duplicate_detection import Todo2DuplicateDetector
        from project_management_automation.utils import find_project_root

        # Find project root
        project_root = find_project_root(Path(__file__).parent.parent.parent.parent)

        # Build config
        config = {
            'similarity_threshold': similarity_threshold,
            'auto_fix': auto_fix,
            'output_path': output_path or 'docs/TODO2_DUPLICATE_DETECTION_REPORT.md'
        }

        # Create detector and run
        detector = Todo2DuplicateDetector(config, project_root)
        results = detector.run()

        # Extract key metrics
        duplicates = results.get('results', {}).get('duplicates', {})

        # Format response
        response_data = {
            'duplicate_ids': len(duplicates.get('duplicate_ids', [])),
            'exact_name_matches': len(duplicates.get('exact_name_matches', [])),
            'similar_name_matches': len(duplicates.get('similar_name_matches', [])),
            'similar_description_matches': len(duplicates.get('similar_description_matches', [])),
            'self_dependencies': len(duplicates.get('self_dependencies', [])),
            'total_duplicates_found': (
                len(duplicates.get('duplicate_ids', [])) +
                len(duplicates.get('exact_name_matches', [])) +
                len(duplicates.get('similar_name_matches', [])) +
                len(duplicates.get('similar_description_matches', [])) +
                len(duplicates.get('self_dependencies', []))
            ),
            'report_path': str(Path(config['output_path']).absolute()),
            'auto_fix_applied': auto_fix,
            'status': results.get('status', 'unknown')
        }

        duration = time.time() - start_time
        log_automation_execution('detect_duplicate_tasks', duration, True)

        return json.dumps(format_success_response(response_data), indent=2)

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('detect_duplicate_tasks', duration, False, e)
        error_response = format_error_response(e, ErrorCode.AUTOMATION_ERROR)
        return json.dumps(error_response, indent=2)
