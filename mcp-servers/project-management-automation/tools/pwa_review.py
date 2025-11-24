"""
MCP Tool Wrapper for PWA Review

Wraps PWAAnalyzer to expose as MCP tool.
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


def review_pwa_config(
    output_path: Optional[str] = None,
    config_path: Optional[str] = None
) -> str:
    """
    Review PWA configuration and generate improvement recommendations.

    Args:
        output_path: Path for analysis output (default: docs/PWA_IMPROVEMENT_ANALYSIS.md)
        config_path: Path to PWA review config file (default: scripts/pwa_review_config.json)

    Returns:
        JSON string with review results
    """
    start_time = time.time()

    try:
        # Import from package
        from project_management_automation.scripts.automate_pwa_review import PWAAnalyzer, load_config
        from project_management_automation.utils import find_project_root

        # Find project root
        project_root = find_project_root(Path(__file__).parent.parent.parent.parent)

        # Load config
        if config_path:
            config = load_config(Path(config_path))
        else:
            config = load_config(project_root)

        # Override output path if provided
        if output_path:
            config['output_path'] = output_path

        # Create analyzer and run
        analyzer = PWAAnalyzer(config, project_root)
        success = analyzer.run(Path(config['output_path']))

        if not success:
            raise Exception("PWA review analysis failed")

        # Analyze PWA structure to get metrics
        pwa_analysis = analyzer.analyze_pwa_structure()
        tasks = analyzer.load_todo2_tasks()
        todo2_alignment = analyzer.analyze_todo2_alignment(tasks)

        # Format response
        response_data = {
            'components_count': len(pwa_analysis.get('components', [])),
            'hooks_count': len(pwa_analysis.get('hooks', [])),
            'api_integrations_count': len(pwa_analysis.get('api_integrations', [])),
            'pwa_features': pwa_analysis.get('pwa_features', []),
            'missing_features': pwa_analysis.get('missing_features', []),
            'goal_aligned_tasks': todo2_alignment.get('goal_aligned', 0),
            'pwa_related_tasks': todo2_alignment.get('pwa_related', 0),
            'report_path': str(Path(config['output_path']).absolute()),
            'status': 'success'
        }

        duration = time.time() - start_time
        log_automation_execution('review_pwa_config', duration, True)

        return json.dumps(format_success_response(response_data), indent=2)

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('review_pwa_config', duration, False, e)
        error_response = format_error_response(e, ErrorCode.AUTOMATION_ERROR)
        return json.dumps(error_response, indent=2)
