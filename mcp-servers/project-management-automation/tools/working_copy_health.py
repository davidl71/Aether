"""
Working Copy Health Check Tool

MCP Tool for checking git working copy status across all agents and runners.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional


def check_working_copy_health(
    agent_name: Optional[str] = None,
    check_remote: bool = True
) -> Dict[str, Any]:
    """
    Check working copy health across agents.

    Args:
        agent_name: Specific agent to check (optional, checks all if None)
        check_remote: Whether to check remote agents (default: True)

    Returns:
        Dictionary with working copy status for each agent
    """
    project_root = Path(__file__).parent.parent.parent.parent

    # Agent configurations
    agents = {
        "local": {
            "path": str(project_root),
            "type": "local"
        }
    }

    if check_remote:
        agents.update({
            "ubuntu": {
                "host": "david@192.168.192.57",
                "path": "~/ib_box_spread_full_universal",
                "type": "remote"
            },
            "macos": {
                "host": "davidl@192.168.192.141",
                "path": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal",
                "type": "remote"
            }
        })

    results = {}

    # Filter to specific agent if requested
    if agent_name and agent_name in agents:
        agents = {agent_name: agents[agent_name]}

    for agent_name, agent_config in agents.items():
        agent_type = agent_config.get("type", "local")

        if agent_type == "local":
            # Check local working copy
            try:
                result = subprocess.run(
                    ["git", "status", "--porcelain"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    text=True,
                    timeout=5
                )

                has_changes = bool(result.stdout.strip())

                # Get branch and commit info
                branch_result = subprocess.run(
                    ["git", "branch", "--show-current"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                branch = branch_result.stdout.strip() or "unknown"

                # Get latest commit
                commit_result = subprocess.run(
                    ["git", "log", "-1", "--oneline"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                latest_commit = commit_result.stdout.strip() or "unknown"

                # Check sync status
                subprocess.run(
                    ["git", "fetch", "--quiet"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    timeout=10
                )

                behind_result = subprocess.run(
                    ["git", "rev-list", "--count", "HEAD..origin/main"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                behind = int(behind_result.stdout.strip() or "0")

                ahead_result = subprocess.run(
                    ["git", "rev-list", "--count", "origin/main..HEAD"],
                    cwd=agent_config["path"],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                ahead = int(ahead_result.stdout.strip() or "0")

                results[agent_name] = {
                    "status": "ok" if not has_changes and behind == 0 and ahead == 0 else "warning",
                    "has_uncommitted_changes": has_changes,
                    "uncommitted_files": result.stdout.strip().split('\n') if has_changes else [],
                    "branch": branch,
                    "latest_commit": latest_commit,
                    "behind_remote": behind,
                    "ahead_remote": ahead,
                    "in_sync": behind == 0 and ahead == 0,
                    "type": "local"
                }

            except Exception as e:
                results[agent_name] = {
                    "status": "error",
                    "error": str(e),
                    "type": "local"
                }

        else:
            # Check remote agent
            host = agent_config["host"]
            path = agent_config["path"]

            try:
                # Check SSH connectivity
                ssh_test = subprocess.run(
                    ["ssh", "-o", "ConnectTimeout=5", "-o", "BatchMode=yes", host, "exit"],
                    capture_output=True,
                    timeout=10
                )

                if ssh_test.returncode != 0:
                    results[agent_name] = {
                        "status": "error",
                        "error": f"Cannot connect to {host}",
                        "type": "remote"
                    }
                    continue

                # Get git status
                status_cmd = f"cd {path} && git status --porcelain 2>/dev/null || echo ''"
                status_result = subprocess.run(
                    ["ssh", host, status_cmd],
                    capture_output=True,
                    text=True,
                    timeout=10
                )

                has_changes = bool(status_result.stdout.strip())
                uncommitted_files = status_result.stdout.strip().split('\n') if has_changes else []

                # Get branch
                branch_cmd = f"cd {path} && git branch --show-current 2>/dev/null || echo 'unknown'"
                branch_result = subprocess.run(
                    ["ssh", host, branch_cmd],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                branch = branch_result.stdout.strip() or "unknown"

                # Get latest commit
                commit_cmd = f"cd {path} && git log -1 --oneline 2>/dev/null || echo 'unknown'"
                commit_result = subprocess.run(
                    ["ssh", host, commit_cmd],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                latest_commit = commit_result.stdout.strip() or "unknown"

                # Check sync status
                fetch_cmd = f"cd {path} && git fetch --quiet 2>/dev/null || true"
                subprocess.run(
                    ["ssh", host, fetch_cmd],
                    capture_output=True,
                    timeout=15
                )

                behind_cmd = f"cd {path} && git rev-list --count HEAD..origin/main 2>/dev/null || echo '0'"
                behind_result = subprocess.run(
                    ["ssh", host, behind_cmd],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                behind = int(behind_result.stdout.strip() or "0")

                ahead_cmd = f"cd {path} && git rev-list --count origin/main..HEAD 2>/dev/null || echo '0'"
                ahead_result = subprocess.run(
                    ["ssh", host, ahead_cmd],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                ahead = int(ahead_result.stdout.strip() or "0")

                results[agent_name] = {
                    "status": "ok" if not has_changes and behind == 0 and ahead == 0 else "warning",
                    "has_uncommitted_changes": has_changes,
                    "uncommitted_files": uncommitted_files,
                    "branch": branch,
                    "latest_commit": latest_commit,
                    "behind_remote": behind,
                    "ahead_remote": ahead,
                    "in_sync": behind == 0 and ahead == 0,
                    "type": "remote",
                    "host": host
                }

            except Exception as e:
                results[agent_name] = {
                    "status": "error",
                    "error": str(e),
                    "type": "remote",
                    "host": host
                }

    # Calculate summary
    total_agents = len(results)
    ok_agents = sum(1 for r in results.values() if r.get("status") == "ok")
    warning_agents = sum(1 for r in results.values() if r.get("status") == "warning")
    error_agents = sum(1 for r in results.values() if r.get("status") == "error")

    agents_with_changes = sum(1 for r in results.values() if r.get("has_uncommitted_changes", False))
    agents_behind = sum(1 for r in results.values() if r.get("behind_remote", 0) > 0)

    return {
        "timestamp": __import__("datetime").datetime.utcnow().isoformat() + "Z",
        "summary": {
            "total_agents": total_agents,
            "ok_agents": ok_agents,
            "warning_agents": warning_agents,
            "error_agents": error_agents,
            "agents_with_uncommitted_changes": agents_with_changes,
            "agents_behind_remote": agents_behind
        },
        "agents": results,
        "recommendations": _generate_recommendations(results)
    }


def _generate_recommendations(results: Dict[str, Any]) -> List[str]:
    """Generate recommendations based on working copy status."""
    recommendations = []

    for agent_name, agent_data in results.items():
        if agent_data.get("status") == "error":
            recommendations.append(f"{agent_name}: Fix connection/access issues")
            continue

        if agent_data.get("has_uncommitted_changes", False):
            file_count = len(agent_data.get("uncommitted_files", []))
            recommendations.append(f"{agent_name}: Commit {file_count} uncommitted file(s)")

        if agent_data.get("behind_remote", 0) > 0:
            behind = agent_data.get("behind_remote", 0)
            recommendations.append(f"{agent_name}: Pull {behind} commit(s) from remote")

        if agent_data.get("ahead_remote", 0) > 0:
            ahead = agent_data.get("ahead_remote", 0)
            recommendations.append(f"{agent_name}: Push {ahead} commit(s) to remote")

    if not recommendations:
        recommendations.append("All agents have clean working copies and are in sync")

    return recommendations
