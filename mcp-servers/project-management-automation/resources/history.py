"""
MCP Resource Handler for Automation Execution History

Provides resource access to automation tool execution history.
"""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional

logger = logging.getLogger(__name__)


def get_history_resource(limit: int = 50) -> str:
    """
    Get automation execution history as resource.

    Args:
        limit: Maximum number of history entries to return

    Returns:
        JSON string with execution history
    """
    try:
        project_root = Path(__file__).parent.parent.parent.parent
        history_files = [
            project_root / 'scripts' / '.docs_health_history.json',
            project_root / 'scripts' / '.todo2_alignment_history.json',
            project_root / 'scripts' / '.todo_sync_history.json',
            project_root / 'scripts' / '.dependency_security_history.json'
        ]

        history = {
            "automation_history": [],
            "total_executions": 0,
            "timestamp": datetime.now().isoformat()
        }

        # Load history from various automation scripts
        for history_file in history_files:
            if history_file.exists():
                try:
                    with open(history_file, 'r') as f:
                        data = json.load(f)
                        runs = data.get('runs', [])
                        for run in runs[-limit:]:  # Get last N runs
                            history["automation_history"].append({
                                "automation": history_file.stem.replace('.', '_'),
                                "timestamp": run.get('timestamp', 'unknown'),
                                "status": run.get('status', 'unknown'),
                                "health_score": run.get('health_score'),
                                "issues_found": run.get('issues_found', 0)
                            })
                            history["total_executions"] += 1
                except Exception as e:
                    logger.warning(f"Error reading history file {history_file}: {e}")

        # Sort by timestamp (most recent first)
        history["automation_history"].sort(
            key=lambda x: x.get('timestamp', ''),
            reverse=True
        )

        # Limit results
        history["automation_history"] = history["automation_history"][:limit]

        return json.dumps(history, indent=2)

    except Exception as e:
        logger.error(f"Error getting history resource: {e}")
        return json.dumps({
            "automation_history": [],
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }, indent=2)
