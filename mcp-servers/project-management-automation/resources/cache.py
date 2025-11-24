"""
MCP Resource Handler for Cache Status

Provides resource access to cache status and what data is cached.
"""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)


def _find_project_root() -> Path:
    """Find project root directory."""
    current = Path(__file__).parent.parent.parent.parent
    for _ in range(5):
        if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
            return current
        if current.parent == current:
            break
        current = current.parent
    return current


def get_cache_status_resource() -> str:
    """
    Get cache status - what data is cached and when it was last updated.

    Returns:
        JSON string with cache status information
    """
    try:
        project_root = _find_project_root()
        cache_info = {
            "caches": [],
            "timestamp": datetime.now().isoformat()
        }

        # Check Todo2 state cache
        todo2_file = project_root / '.todo2' / 'state.todo2.json'
        if todo2_file.exists():
            stat = todo2_file.stat()
            cache_info["caches"].append({
                "name": "todo2_state",
                "type": "task_list",
                "path": str(todo2_file.relative_to(project_root)),
                "size_bytes": stat.st_size,
                "last_modified": datetime.fromtimestamp(stat.st_mtime).isoformat(),
                "description": "Todo2 task state - contains all tasks, statuses, and metadata"
            })

        # Check automation history caches
        scripts_dir = project_root / 'scripts'
        history_files = [
            ('.docs_health_history.json', 'documentation_health_history'),
            ('.todo2_alignment_history.json', 'todo2_alignment_history'),
            ('.todo_sync_history.json', 'todo_sync_history'),
            ('.dependency_security_history.json', 'dependency_security_history')
        ]

        for filename, cache_name in history_files:
            history_file = scripts_dir / filename
            if history_file.exists():
                stat = history_file.stat()
                cache_info["caches"].append({
                    "name": cache_name,
                    "type": "execution_history",
                    "path": str(history_file.relative_to(project_root)),
                    "size_bytes": stat.st_size,
                    "last_modified": datetime.fromtimestamp(stat.st_mtime).isoformat(),
                    "description": f"Execution history for {cache_name.replace('_', ' ')}"
                })

        # Check agent configurations (cached in memory when loaded)
        agents_dir = project_root / 'agents'
        agent_count = 0
        if agents_dir.exists():
            for agent_dir in agents_dir.iterdir():
                if agent_dir.is_dir() and (agent_dir / 'cursor-agent.json').exists():
                    agent_count += 1

        cache_info["caches"].append({
            "name": "agent_configurations",
            "type": "agent_metadata",
            "count": agent_count,
            "path": str(agents_dir.relative_to(project_root)) if agents_dir.exists() else None,
            "description": f"Agent configurations from cursor-agent.json files ({agent_count} agents)"
        })

        cache_info["summary"] = {
            "total_caches": len(cache_info["caches"]),
            "task_caches": len([c for c in cache_info["caches"] if c["type"] == "task_list"]),
            "history_caches": len([c for c in cache_info["caches"] if c["type"] == "execution_history"]),
            "agent_caches": len([c for c in cache_info["caches"] if c["type"] == "agent_metadata"])
        }

        return json.dumps(cache_info, indent=2)

    except Exception as e:
        logger.error(f"Error getting cache status resource: {e}")
        return json.dumps({
            "caches": [],
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }, indent=2)
