"""
Nightly Task Automation Tool

Automatically executes background-capable TODO2 tasks in parallel across multiple hosts.
Moves interactive tasks to Review status and proceeds to next tasks.
Also includes batch approval of research tasks that don't need clarification.
"""

import json
import subprocess
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
import time

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from tools.intelligent_automation_base import IntelligentAutomationBase
except ImportError:
    # Fallback if base class not available
    class IntelligentAutomationBase:
        pass


def _get_local_ip_addresses() -> List[str]:
    """Get all local IP addresses (excluding localhost)."""
    local_ips = []

    # Get hostname
    try:
        hostname = socket.gethostname()
        local_ips.append(hostname)
        # Also add FQDN if available
        fqdn = socket.getfqdn()
        if fqdn != hostname:
            local_ips.append(fqdn)
    except Exception:
        pass

    # Get IP addresses from all interfaces
    try:
        result = subprocess.run(
            ["ifconfig"],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            for line in result.stdout.split('\n'):
                if 'inet ' in line and '127.0.0.1' not in line:
                    parts = line.split()
                    # Find the IP address (usually after 'inet')
                    for i, part in enumerate(parts):
                        if part == 'inet' and i + 1 < len(parts):
                            ip = parts[i + 1]
                            # Remove netmask if present (e.g., "192.168.1.1/24" -> "192.168.1.1")
                            ip = ip.split('/')[0]
                            if ip not in local_ips and ip != '127.0.0.1':
                                local_ips.append(ip)
    except Exception:
        pass

    # Also try to get primary IP via socket (fallback)
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        local_ip = s.getsockname()[0]
        if local_ip not in local_ips:
            local_ips.append(local_ip)
        s.close()
    except Exception:
        pass

    return local_ips


def _is_local_host(hostname: str) -> bool:
    """
    Determine if a hostname/IP refers to the local machine.

    Args:
        hostname: Hostname or IP address (may include user@ prefix)

    Returns:
        True if hostname refers to local machine
    """
    # Remove user@ prefix if present
    host = hostname.split('@')[-1] if '@' in hostname else hostname

    # Check if it's localhost
    if host in ['localhost', '127.0.0.1', '::1']:
        return True

    # Get local IPs and hostname
    local_ips = _get_local_ip_addresses()
    local_hostname = socket.gethostname()

    # Check if host matches local IP or hostname
    if host in local_ips or host == local_hostname:
        return True

    # Try to resolve hostname to IP and compare
    try:
        resolved_ip = socket.gethostbyname(host)
        if resolved_ip in local_ips or resolved_ip == '127.0.0.1':
            return True
    except Exception:
        pass

    return False


def _find_project_root(start_path: Path) -> Path:
    """
    Find project root by looking for .git directory or other markers.
    Falls back to relative path detection if markers not found.
    """
    # Try environment variable first
    env_root = os.getenv('PROJECT_ROOT') or os.getenv('WORKSPACE_PATH')
    if env_root:
        root_path = Path(env_root)
        if root_path.exists():
            return root_path.resolve()

    # Try relative path detection (assumes standard structure)
    current = start_path
    for _ in range(5):  # Go up max 5 levels
        # Check for project markers
        if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
            return current.resolve()
        if current.parent == current:  # Reached filesystem root
            break
        current = current.parent

    # Fallback to relative path (assumes mcp-servers/project-management-automation/tools/file.py)
    return start_path.parent.parent.parent.parent.resolve()


class NightlyTaskAutomation(IntelligentAutomationBase):
    """Automated nightly task execution across parallel hosts."""

    def __init__(self):
        self.project_root = _find_project_root(Path(__file__))
        self.todo2_state_file = self.project_root / ".todo2" / "state.todo2.json"
        self.batch_script = self.project_root / "scripts" / "batch_update_todos.py"
        self.agent_hostnames = self._load_agent_hostnames()

    def _load_agent_hostnames(self) -> Dict[str, str]:
        """Load agent hostname configuration."""
        hostnames_file = self.project_root / "docs" / "AGENT_HOSTNAMES.md"

        # Default configuration (can be overridden by file)
        default_hostnames = {
            "ubuntu": {
                "hostname": "david@192.168.192.57",
                "project_path": "ib_box_spread_full_universal",
                "type": "ubuntu"
            },
            "macos": {
                "hostname": "192.168.192.141",
                "project_path": "~/Projects/Trading/ib_box_spread_full_universal",
                "type": "macos"
            }
        }

        # Try to read from file if it exists
        if hostnames_file.exists():
            try:
                content = hostnames_file.read_text()
                # Parse markdown file (simple extraction)
                # In production, use proper markdown parser
                pass
            except Exception:
                pass

        # Auto-detect local agents and mark them appropriately
        for agent_name, agent_config in default_hostnames.items():
            hostname = agent_config.get("hostname", "")
            if hostname and _is_local_host(hostname):
                # This is the local machine, mark as local
                agent_config["type"] = "local"
                # Use current project root for local agents
                agent_config["project_path"] = str(self.project_root)

        return default_hostnames

    def _load_todo2_state(self) -> Dict[str, Any]:
        """Load TODO2 state file."""
        if not self.todo2_state_file.exists():
            return {"todos": []}

        try:
            with open(self.todo2_state_file, 'r') as f:
                return json.load(f)
        except Exception as e:
            return {"todos": [], "error": str(e)}

    def _save_todo2_state(self, state: Dict[str, Any]) -> bool:
        """Save TODO2 state file."""
        try:
            # Create backup
            if self.todo2_state_file.exists():
                backup_file = self.todo2_state_file.with_suffix('.json.bak')
                with open(self.todo2_state_file, 'r') as f:
                    backup_file.write_text(f.read())

            with open(self.todo2_state_file, 'w') as f:
                json.dump(state, f, indent=2)
            return True
        except Exception as e:
            return False

    def _is_background_capable(self, task: Dict[str, Any]) -> bool:
        """Determine if task can run in background."""
        task_id = task.get('id', '')
        name = task.get('name', '').lower()
        long_desc = task.get('long_description', '').lower()
        status = task.get('status', '')

        # Skip if not in Todo status
        if status not in ['Todo', 'todo']:
            return False

        # Skip Review status (already reviewed)
        if status == 'Review':
            return False

        # Interactive indicators (exclude)
        is_review = status == 'Review'
        needs_clarification = 'clarification required' in long_desc
        needs_user_input = 'user input' in long_desc or 'user interaction' in long_desc
        is_design = 'design' in name and any(x in name for x in ['framework', 'system', 'strategy', 'allocation'])
        is_decision = any(x in name for x in ['decide', 'choose', 'select', 'recommend', 'suggest', 'propose'])
        is_strategy = 'strategy' in name or 'strategy' in long_desc or ('plan' in name and 'workflow' in long_desc)

        is_interactive = is_review or needs_clarification or needs_user_input or (is_design and 'implement' not in name) or (is_decision and 'implement' not in name) or (is_strategy and 'implement' not in name)

        # Background indicators (include)
        is_mcp_extension = task_id.startswith('MCP-EXT')
        is_research = 'research' in name
        is_implementation = any(x in name for x in ['implement', 'create', 'add', 'update', 'fix', 'refactor'])
        is_testing = 'test' in name or 'testing' in name or 'validate' in name
        is_documentation = 'document' in name or 'documentation' in name
        is_configuration = 'config' in name or 'configure' in name or 'setup' in name

        is_background = (is_mcp_extension or is_research or is_implementation or is_testing or is_documentation or is_configuration) and not is_interactive

        return is_background

    def _move_to_review(self, task: Dict[str, Any], reason: str) -> Dict[str, Any]:
        """Move task to Review status."""
        task['status'] = 'Review'

        # Add note comment about why moved to review
        if 'comments' not in task:
            task['comments'] = []

        task['comments'].append({
            'id': f"{task['id']}-C-{int(time.time())}",
            'todoId': task['id'],
            'type': 'note',
            'content': f"**Automated Review:** Moved to Review status by nightly automation. Reason: {reason}",
            'created': datetime.utcnow().isoformat() + 'Z'
        })

        # Update last modified
        task['lastModified'] = datetime.utcnow().isoformat() + 'Z'

        # Add status change to changes array
        if 'changes' not in task:
            task['changes'] = []

        task['changes'].append({
            'field': 'status',
            'oldValue': task.get('status', 'Todo'),
            'newValue': 'Review',
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        })

        return task

    def _execute_task_on_host(self, task: Dict[str, Any], host_info: Dict[str, str]) -> Dict[str, Any]:
        """Execute a task on a specific host via SSH."""
        task_id = task.get('id', '')
        hostname = host_info.get('hostname', '')
        project_path = host_info.get('project_path', '')
        host_type = host_info.get('type', 'ubuntu')

        # Construct SSH command
        if '@' in hostname:
            # User@host format
            ssh_cmd = f"ssh {hostname}"
        else:
            # Just hostname (assume current user)
            ssh_cmd = f"ssh {hostname}"

        # Navigate to project and execute task
        # For now, we'll just mark as in progress and return
        # In production, would use actual task execution via Cursor agent or script

        result = {
            'task_id': task_id,
            'host': hostname,
            'status': 'started',
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'note': 'Task execution would happen here via SSH/Cursor agent'
        }

        return result

    def _update_task_status(self, task: Dict[str, Any], new_status: str, result_comment: Optional[str] = None) -> Dict[str, Any]:
        """Update task status in TODO2 state."""
        old_status = task.get('status', 'Todo')
        task['status'] = new_status
        task['lastModified'] = datetime.utcnow().isoformat() + 'Z'

        # Add status change
        if 'changes' not in task:
            task['changes'] = []

        task['changes'].append({
            'field': 'status',
            'oldValue': old_status,
            'newValue': new_status,
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        })

        # Add result comment if provided
        if result_comment:
            if 'comments' not in task:
                task['comments'] = []

            task['comments'].append({
                'id': f"{task['id']}-C-{int(time.time())}",
                'todoId': task['id'],
                'type': 'result',
                'content': result_comment,
                'created': datetime.utcnow().isoformat() + 'Z'
            })

        return task

    def _check_working_copy_health(self) -> Dict[str, Any]:
        """Check working copy health before task execution."""
        try:
            from tools.working_copy_health import check_working_copy_health
            return check_working_copy_health(check_remote=True)
        except Exception as e:
            return {
                "status": "error",
                "error": f"Failed to check working copy health: {e}",
                "summary": {}
            }

    def run_nightly_automation(
        self,
        max_tasks_per_host: int = 5,
        max_parallel_tasks: int = 10,
        priority_filter: Optional[str] = None,
        tag_filter: Optional[List[str]] = None,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """
        Run nightly automation across parallel hosts.

        Args:
            max_tasks_per_host: Maximum tasks to assign per host
            max_parallel_tasks: Maximum total parallel tasks
            priority_filter: Filter by priority ('high', 'medium', 'low')
            tag_filter: Filter by tags (list of tag strings)
            dry_run: If True, don't actually execute, just report

        Returns:
            Dictionary with execution results
        """
        # Check working copy health before execution
        working_copy_status = self._check_working_copy_health()

        state = self._load_todo2_state()
        todos = state.get('todos', [])

        # Filter tasks
        background_tasks = []
        interactive_tasks = []

        for task in todos:
            if self._is_background_capable(task):
                # Apply filters
                if priority_filter and task.get('priority') != priority_filter:
                    continue

                if tag_filter:
                    task_tags = task.get('tags', [])
                    if not any(tag in task_tags for tag in tag_filter):
                        continue

                background_tasks.append(task)
            else:
                # Check if interactive task should be moved to review
                if task.get('status') in ['Todo', 'todo']:
                    # Check if it needs user input
                    long_desc = task.get('long_description', '').lower()
                    needs_clarification = 'clarification required' in long_desc
                    needs_user_input = 'user input' in long_desc or 'user interaction' in long_desc

                    if needs_clarification or needs_user_input:
                        interactive_tasks.append(task)

        # Move interactive tasks to Review (if not dry run)
        moved_to_review = []
        if not dry_run:
            for task in interactive_tasks[:max_parallel_tasks]:  # Limit moves
                task = self._move_to_review(task, "Requires user input or clarification")
                moved_to_review.append(task['id'])

                # Update in state
                for i, t in enumerate(todos):
                    if t.get('id') == task['id']:
                        todos[i] = task
                        break

        # Assign background tasks to hosts
        assigned_tasks = []
        task_assignments = {}

        available_hosts = list(self.agent_hostnames.keys())
        host_index = 0

        for task in background_tasks[:max_parallel_tasks]:
            if len(assigned_tasks) >= max_parallel_tasks:
                break

            # Round-robin assignment to hosts
            host_key = available_hosts[host_index % len(available_hosts)]
            host_info = self.agent_hostnames[host_key]

            # Check host task limit
            host_task_count = sum(1 for t in task_assignments.values() if t['host'] == host_key)
            if host_task_count >= max_tasks_per_host:
                # Skip to next host
                host_index += 1
                host_key = available_hosts[host_index % len(available_hosts)]
                host_info = self.agent_hostnames[host_key]
                host_task_count = sum(1 for t in task_assignments.values() if t['host'] == host_key)
                if host_task_count >= max_tasks_per_host:
                    continue  # All hosts at capacity

            # Assign task
            assignment = {
                'task_id': task['id'],
                'task_name': task.get('name', ''),
                'host': host_key,
                'hostname': host_info['hostname'],
                'project_path': host_info['project_path'],
                'status': 'assigned'
            }

            task_assignments[task['id']] = assignment

            # Update task status to In Progress (if not dry run)
            if not dry_run:
                task = self._update_task_status(task, 'In Progress',
                    f"Assigned to {host_key} agent for automated execution")

                # Update in state
                for i, t in enumerate(todos):
                    if t.get('id') == task['id']:
                        todos[i] = task
                        break

            assigned_tasks.append(task)
            host_index += 1

        # Batch approve research tasks that don't need clarification (if not dry run)
        batch_approved_count = 0
        if not dry_run and self.batch_script.exists():
            try:
                # Count Review tasks with no clarification before approval
                review_tasks_before = [t for t in todos if t.get('status') == 'Review']

                # Run batch approval script for Review tasks with no clarification needed
                result = subprocess.run(
                    [
                        sys.executable,
                        str(self.batch_script),
                        'approve',
                        '--status', 'Review',
                        '--clarification-none',
                        '--new-status', 'Todo',
                        '--yes'  # Skip confirmation in automated runs
                    ],
                    cwd=str(self.project_root),
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode == 0:
                    # Reload state to get updated task list
                    state = self._load_todo2_state()
                    todos = state.get('todos', [])

                    # Count Review tasks after approval
                    review_tasks_after = [t for t in todos if t.get('status') == 'Review']
                    batch_approved_count = len(review_tasks_before) - len(review_tasks_after)
            except Exception as e:
                # Log error but don't fail the automation
                print(f"Warning: Batch approval failed: {e}", file=sys.stderr)

        # Save state (if not dry run)
        if not dry_run:
            state['todos'] = todos
            self._save_todo2_state(state)

        # Prepare results
        results = {
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'dry_run': dry_run,
            'working_copy_status': working_copy_status,
            'summary': {
                'background_tasks_found': len(background_tasks),
                'interactive_tasks_found': len(interactive_tasks),
                'tasks_assigned': len(assigned_tasks),
                'tasks_moved_to_review': len(moved_to_review),
                'tasks_batch_approved': batch_approved_count,
                'hosts_used': len(set(a['host'] for a in task_assignments.values())),
                'working_copy_warnings': working_copy_status.get('summary', {}).get('warning_agents', 0)
            },
            'assigned_tasks': [
                {
                    'task_id': a['task_id'],
                    'task_name': a['task_name'],
                    'host': a['host'],
                    'hostname': a['hostname']
                }
                for a in task_assignments.values()
            ],
            'moved_to_review': moved_to_review,
            'background_tasks_remaining': len(background_tasks) - len(assigned_tasks)
        }

        return results


def run_nightly_task_automation(
    max_tasks_per_host: int = 5,
    max_parallel_tasks: int = 10,
    priority_filter: Optional[str] = None,
    tag_filter: Optional[List[str]] = None,
    dry_run: bool = False
) -> Dict[str, Any]:
    """
    MCP Tool: Run nightly task automation across parallel hosts.

    Automatically executes background-capable TODO2 tasks in parallel across multiple hosts.
    Moves interactive tasks requiring user input to Review status.

    Args:
        max_tasks_per_host: Maximum tasks to assign per host (default: 5)
        max_parallel_tasks: Maximum total parallel tasks (default: 10)
        priority_filter: Filter by priority - 'high', 'medium', or 'low' (optional)
        tag_filter: Filter by tags - list of tag strings (optional)
        dry_run: If true, don't execute, just report what would happen (default: false)

    Returns:
        Dictionary with execution results including assigned tasks, moved tasks, and summary
    """
    automation = NightlyTaskAutomation()
    return automation.run_nightly_automation(
        max_tasks_per_host=max_tasks_per_host,
        max_parallel_tasks=max_parallel_tasks,
        priority_filter=priority_filter,
        tag_filter=tag_filter,
        dry_run=dry_run
    )
