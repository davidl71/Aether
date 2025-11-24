"""
MCP Tool Wrapper for Dependency Security Scan

Wraps DependencySecurityAnalyzer to expose as MCP tool.
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


def scan_dependency_security(
    languages: Optional[List[str]] = None,
    config_path: Optional[str] = None
) -> str:
    """
    Scan project dependencies for security vulnerabilities.

    Args:
        languages: List of languages to scan (python, rust, npm). If None, scans all.
        config_path: Path to dependency security config file (default: scripts/dependency_security_config.json)

    Returns:
        JSON string with scan results
    """
    start_time = time.time()

    try:
        # Import from package
        from project_management_automation.scripts.automate_dependency_security import DependencySecurityAnalyzer
        from project_management_automation.utils import find_project_root

        # Find project root
        project_root = find_project_root(Path(__file__).parent.parent.parent.parent)

        # Use default config if not provided
        if not config_path:
            config_path = str(project_root / 'scripts' / 'dependency_security_config.json')

        # Load and modify config if languages specified
        if languages:
            import json as json_module
            with open(config_path, 'r') as f:
                config_data = json_module.load(f)

            # Enable only specified languages
            scan_configs = config_data.get('scan_configs', {})
            for lang in ['python', 'rust', 'npm']:
                if lang in scan_configs:
                    scan_configs[lang]['enabled'] = lang in languages

            # Write temporary config
            temp_config_path = project_root / 'scripts' / '.temp_dependency_security_config.json'
            with open(temp_config_path, 'w') as f:
                json_module.dump(config_data, f, indent=2)
            config_path = str(temp_config_path)

        # Create analyzer and run
        analyzer = DependencySecurityAnalyzer(config_path, project_root)
        results = analyzer.run()

        # Extract key metrics
        summary = results.get('summary', {})
        scan_results = results

        # Format response
        response_data = {
            'total_vulnerabilities': summary.get('total_vulnerabilities', 0),
            'by_severity': summary.get('by_severity', {}),
            'by_language': summary.get('by_language', {}),
            'critical_vulnerabilities': len(summary.get('critical_vulnerabilities', [])),
            'python_vulnerabilities': len(scan_results.get('python', [])),
            'rust_vulnerabilities': len(scan_results.get('rust', [])),
            'npm_vulnerabilities': len(scan_results.get('npm', [])),
            'report_path': str(analyzer.output_file.absolute()),
            'status': results.get('status', 'unknown')
        }

        duration = time.time() - start_time
        log_automation_execution('scan_dependency_security', duration, True)

        return json.dumps(format_success_response(response_data), indent=2)

    except Exception as e:
        duration = time.time() - start_time
        log_automation_execution('scan_dependency_security', duration, False, e)
        error_response = format_error_response(e, ErrorCode.AUTOMATION_ERROR)
        return json.dumps(error_response, indent=2)
