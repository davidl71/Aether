#!/usr/bin/env python3
"""
Context-aware task list for Exarp Oh My Zsh plugin.
Queries Todo2 tasks relevant to current directory/git repo context.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def find_project_root(start_path: Path) -> Optional[Path]:
    """Find project root by looking for .git, .todo2, or CMakeLists.txt."""
    current = start_path.resolve()
    for _ in range(10):  # Max 10 levels up
        if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
            return current
        if current.parent == current:
            break
        current = current.parent
    return None


def get_git_repo_name(project_root: Path) -> Optional[str]:
    """Get git repository name from remote URL."""
    try:
        git_config = project_root / '.git' / 'config'
        if git_config.exists():
            with open(git_config, 'r') as f:
                content = f.read()
                # Look for remote URL
                for line in content.split('\n'):
                    if 'url =' in line:
                        url = line.split('url =')[1].strip()
                        # Extract repo name from URL
                        if 'github.com' in url or 'gitlab.com' in url:
                            parts = url.rstrip('.git').split('/')
                            if len(parts) >= 2:
                                return '/'.join(parts[-2:])
                        elif '@' in url:  # SSH format
                            parts = url.split(':')
                            if len(parts) >= 2:
                                repo = parts[-1].rstrip('.git')
                                return repo
        return None
    except Exception:
        return None


def get_current_folder_context(current_dir: Path, project_root: Path) -> Dict[str, str]:
    """Get context information about current directory."""
    rel_path = current_dir.relative_to(project_root) if current_dir.is_relative_to(project_root) else None

    context = {
        'current_dir': str(current_dir),
        'project_root': str(project_root),
        'relative_path': str(rel_path) if rel_path else None,
        'folder_name': current_dir.name,
        'parent_folder': current_dir.parent.name if current_dir.parent != current_dir else None,
    }

    # Detect common module patterns
    if rel_path:
        parts = str(rel_path).split('/')
        if len(parts) > 0:
            context['top_level'] = parts[0]
            if len(parts) > 1:
                context['sub_module'] = parts[1]

    return context


def load_todo2_tasks(project_root: Path) -> List[Dict]:
    """Load Todo2 tasks from state file."""
    todo2_file = project_root / '.todo2' / 'state.todo2.json'

    if not todo2_file.exists():
        return []

    try:
        with open(todo2_file, 'r') as f:
            data = json.load(f)
            return data.get('todos', [])
    except Exception as e:
        print(f"Error loading Todo2 tasks: {e}", file=sys.stderr)
        return []


def task_matches_context(task: Dict, context: Dict, repo_name: Optional[str]) -> bool:
    """Check if task matches current context."""
    name = task.get('name', '').lower()
    desc = task.get('long_description', '').lower()
    content = task.get('content', '').lower()
    tags = [tag.lower() for tag in task.get('tags', [])]

    folder_name = context.get('folder_name', '').lower()
    top_level = context.get('top_level', '').lower()
    sub_module = context.get('sub_module', '').lower()
    relative_path = context.get('relative_path', '').lower()

    # Check folder name matches
    if folder_name and folder_name in (name + desc + content):
        return True

    # Check top-level module matches
    if top_level and top_level in (name + desc + content):
        return True

    # Check sub-module matches
    if sub_module and sub_module in (name + desc + content):
        return True

    # Check tags match folder/module
    if folder_name and folder_name in tags:
        return True
    if top_level and top_level in tags:
        return True
    if sub_module and sub_module in tags:
        return True

    # Check relative path matches
    if relative_path:
        path_parts = relative_path.split('/')
        for part in path_parts:
            if part and part in (name + desc + content):
                return True

    # Check repo name matches
    if repo_name:
        repo_parts = repo_name.lower().split('/')
        for part in repo_parts:
            if part and part in (name + desc + content):
                return True

    return False


def filter_tasks_by_context(tasks: List[Dict], context: Dict, repo_name: Optional[str]) -> List[Dict]:
    """Filter tasks that match current context."""
    if not context.get('relative_path'):
        # If we're at project root, return all tasks
        return tasks

    matched = []
    for task in tasks:
        if task_matches_context(task, context, repo_name):
            matched.append(task)

    return matched


def get_task_summary(tasks: List[Dict]) -> Dict[str, int]:
    """Get summary counts by status."""
    summary = {
        'Todo': 0,
        'In Progress': 0,
        'Review': 0,
        'Done': 0,
        'Blocked': 0,
        'Total': len(tasks)
    }

    for task in tasks:
        status = task.get('status', 'Todo')
        if status in summary:
            summary[status] += 1
        else:
            summary['Todo'] += 1  # Default unknown status to Todo

    return summary


def format_motd(summary: Dict, context: Dict, repo_name: Optional[str], limit: int = 5) -> str:
    """Format MOTD with task summary."""
    lines = []

    # Header
    if repo_name:
        lines.append(f"📋 {repo_name}")
    elif context.get('folder_name'):
        lines.append(f"📋 {context['folder_name']}")

    if context.get('relative_path'):
        lines.append(f"📍 {context['relative_path']}")

    lines.append("")

    # Summary
    if summary['Total'] > 0:
        lines.append("📊 Task Summary:")
        if summary['Review'] > 0:
            lines.append(f"   ⚠️  {summary['Review']} pending review")
        if summary['In Progress'] > 0:
            lines.append(f"   🔄 {summary['In Progress']} in progress")
        if summary['Todo'] > 0:
            lines.append(f"   📝 {summary['Todo']} todo")
        if summary['Blocked'] > 0:
            lines.append(f"   🚫 {summary['Blocked']} blocked")
        if summary['Done'] > 0:
            lines.append(f"   ✅ {summary['Done']} done")
        lines.append(f"   📈 {summary['Total']} total")
    else:
        lines.append("✅ No tasks found for this context")

    return "\n".join(lines)


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Usage: exarp_context_tasks.py <command> [options]", file=sys.stderr)
        print("Commands: list, summary, motd", file=sys.stderr)
        sys.exit(1)

    command = sys.argv[1]
    current_dir = Path.cwd()

    # Find project root
    project_root = find_project_root(current_dir)
    if not project_root:
        print("❌ Not in a project directory", file=sys.stderr)
        sys.exit(1)

    # Get context
    context = get_current_folder_context(current_dir, project_root)
    repo_name = get_git_repo_name(project_root)

    # Load tasks
    all_tasks = load_todo2_tasks(project_root)
    context_tasks = filter_tasks_by_context(all_tasks, context, repo_name)

    # Execute command
    if command == 'list':
        # Output JSON list of tasks
        output = {
            'context': context,
            'repo_name': repo_name,
            'tasks': context_tasks,
            'total': len(context_tasks),
            'all_tasks_total': len(all_tasks)
        }
        print(json.dumps(output, indent=2))

    elif command == 'summary':
        # Output summary counts
        summary = get_task_summary(context_tasks)
        output = {
            'context': context,
            'repo_name': repo_name,
            'summary': summary
        }
        print(json.dumps(output, indent=2))

    elif command == 'motd':
        # Output formatted MOTD
        summary = get_task_summary(context_tasks)
        motd = format_motd(summary, context, repo_name)
        print(motd)

    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
