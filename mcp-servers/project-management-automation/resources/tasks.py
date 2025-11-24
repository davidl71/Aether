"""
MCP Resource Handler for Todo2 Tasks

Provides resource access to cached Todo2 task lists, filtered by agent, status, etc.
"""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional

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


def _load_todo2_state() -> Dict[str, Any]:
    """Load Todo2 state file."""
    project_root = _find_project_root()
    todo2_file = project_root / '.todo2' / 'state.todo2.json'

    if not todo2_file.exists():
        return {"todos": []}

    try:
        with open(todo2_file, 'r') as f:
            return json.load(f)
    except Exception as e:
        logger.error(f"Error loading Todo2 state: {e}")
        return {"todos": [], "error": str(e)}


def _get_agent_names() -> List[str]:
    """Get list of agent names from cursor-agent.json files."""
    project_root = _find_project_root()
    agents_dir = project_root / 'agents'

    agent_names = []
    if agents_dir.exists():
        for agent_dir in agents_dir.iterdir():
            if agent_dir.is_dir():
                cursor_agent_file = agent_dir / 'cursor-agent.json'
                if cursor_agent_file.exists():
                    try:
                        with open(cursor_agent_file, 'r') as f:
                            config = json.load(f)
                            agent_name = config.get('name', agent_dir.name)
                            agent_names.append(agent_name)
                    except Exception as e:
                        logger.warning(f"Error reading {cursor_agent_file}: {e}")

    return sorted(agent_names)


def _filter_tasks_by_agent(tasks: List[Dict[str, Any]], agent_name: str) -> List[Dict[str, Any]]:
    """Filter tasks by agent name (checks name, description, tags)."""
    agent_lower = agent_name.lower()
    filtered = []

    for task in tasks:
        name = task.get('name', '').lower()
        desc = task.get('long_description', '').lower()
        tags = [tag.lower() for tag in task.get('tags', [])]

        # Check if agent name appears in task name, description, or tags
        if (agent_lower in name or
            agent_lower in desc or
            agent_lower in tags or
            f"{agent_lower}-agent" in name or
            f"{agent_lower}-agent" in desc):
            filtered.append(task)

    return filtered


def get_tasks_resource(agent: Optional[str] = None, status: Optional[str] = None, limit: int = 100) -> str:
    """
    Get Todo2 tasks as resource, optionally filtered by agent or status.

    Args:
        agent: Optional agent name to filter by
        status: Optional status to filter by (Todo, In Progress, Review, Done, etc.)
        limit: Maximum number of tasks to return

    Returns:
        JSON string with filtered task list
    """
    try:
        state = _load_todo2_state()
        tasks = state.get('todos', [])

        # Apply filters
        if agent:
            tasks = _filter_tasks_by_agent(tasks, agent)

        if status:
            tasks = [t for t in tasks if t.get('status', '').lower() == status.lower()]

        # Limit results
        tasks = tasks[:limit]

        # Count by status
        status_counts = {}
        for task in state.get('todos', []):
            task_status = task.get('status', 'Unknown')
            status_counts[task_status] = status_counts.get(task_status, 0) + 1

        result = {
            "tasks": tasks,
            "total_tasks": len(tasks),
            "total_in_state": len(state.get('todos', [])),
            "filters": {
                "agent": agent,
                "status": status,
                "limit": limit
            },
            "status_counts": status_counts,
            "timestamp": datetime.now().isoformat()
        }

        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error getting tasks resource: {e}")
        return json.dumps({
            "tasks": [],
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }, indent=2)


def get_agent_tasks_resource(agent_name: str, status: Optional[str] = None, limit: int = 50) -> str:
    """
    Get tasks for a specific agent.

    Args:
        agent_name: Agent name (e.g., 'backend-agent', 'web-agent')
        status: Optional status filter
        limit: Maximum number of tasks to return

    Returns:
        JSON string with agent's tasks
    """
    return get_tasks_resource(agent=agent_name, status=status, limit=limit)


def get_agents_resource() -> str:
    """
    Get list of available agents with their configurations.

    Returns:
        JSON string with agent list and metadata
    """
    try:
        project_root = _find_project_root()
        agents_dir = project_root / 'agents'

        agents = []
        if agents_dir.exists():
            for agent_dir in agents_dir.iterdir():
                if agent_dir.is_dir():
                    cursor_agent_file = agent_dir / 'cursor-agent.json'
                    if cursor_agent_file.exists():
                        try:
                            with open(cursor_agent_file, 'r') as f:
                                config = json.load(f)
                                agent_info = {
                                    "name": config.get('name', agent_dir.name),
                                    "directory": str(agent_dir.relative_to(project_root)),
                                    "working_directory": config.get('workingDirectory', ''),
                                    "env": config.get('env', {}),
                                    "startup_commands": config.get('startupCommands', []),
                                    "runtime_commands": config.get('runtimeCommands', [])
                                }
                                agents.append(agent_info)
                        except Exception as e:
                            logger.warning(f"Error reading {cursor_agent_file}: {e}")

        # Get task counts per agent
        state = _load_todo2_state()
        all_tasks = state.get('todos', [])

        agent_task_counts = {}
        for agent_info in agents:
            agent_name = agent_info['name']
            agent_tasks = _filter_tasks_by_agent(all_tasks, agent_name)
            agent_task_counts[agent_name] = len(agent_tasks)

        result = {
            "agents": agents,
            "total_agents": len(agents),
            "task_counts": agent_task_counts,
            "timestamp": datetime.now().isoformat()
        }

        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error getting agents resource: {e}")
        return json.dumps({
            "agents": [],
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }, indent=2)
