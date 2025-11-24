#!/usr/bin/env python3
"""
Daily Automation Orchestrator

Runs routine daily maintenance tasks and generates a combined summary report.
"""

import argparse
import json
import logging
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any

# Add project root to path
# Project root will be passed to __init__

# Configure logging (will be configured after project_root is set)
logger = logging.getLogger(__name__)


# Available daily tasks
DAILY_TASKS = {
    'docs_health': {
        'name': 'Documentation Health Check',
        'script': 'scripts/automate_docs_health_v2.py',
        'mcp_tool': 'check_documentation_health',
        'quick': True,
        'description': 'Monitor documentation quality and structure'
    },
    'todo2_alignment': {
        'name': 'Todo2 Alignment Analysis',
        'script': 'scripts/automate_todo2_alignment_v2.py',
        'mcp_tool': 'analyze_todo2_alignment',
        'quick': True,
        'description': 'Ensure tasks align with project strategy'
    },
    'duplicate_detection': {
        'name': 'Duplicate Task Detection',
        'script': 'scripts/automate_todo2_duplicate_detection.py',
        'mcp_tool': 'detect_duplicate_tasks',
        'quick': True,
        'description': 'Detect and report duplicate tasks'
    },
    'dependency_security': {
        'name': 'Dependency Security Scan',
        'script': 'scripts/automate_dependency_security.py',
        'mcp_tool': 'scan_dependency_security',
        'quick': False,
        'description': 'Check for vulnerable dependencies'
    },
    'external_tool_hints': {
        'name': 'External Tool Hints',
        'script': 'scripts/automate_external_tool_hints.py',
        'mcp_tool': 'add_external_tool_hints',
        'quick': True,
        'description': 'Add Context7 hints to documentation'
    }
}


class DailyAutomation:
    """Orchestrates daily maintenance tasks."""

    def __init__(self, config: Dict, project_root: Optional[Path] = None):
        self.config = config
        self.project_root = project_root
        self.tasks_to_run = config.get('tasks', ['docs_health', 'todo2_alignment', 'duplicate_detection'])
        self.dry_run = config.get('dry_run', False)
        self.output_path = config.get('output_path', 'docs/DAILY_AUTOMATION_REPORT.md')
        self.include_slow = config.get('include_slow', False)

        # Results
        self.results = {
            'timestamp': datetime.now().isoformat(),
            'tasks_run': [],
            'tasks_succeeded': [],
            'tasks_failed': [],
            'summary': {},
            'duration_seconds': 0
        }

    def run(self) -> Dict:
        """Run all selected daily tasks."""
        start_time = time.time()
        logger.info(f"Starting daily automation (dry_run={self.dry_run})")

        # Filter tasks based on configuration
        tasks = self._filter_tasks()

        if not tasks:
            logger.warning("No tasks selected to run")
            return {
                'status': 'error',
                'error': 'No tasks selected to run',
                'results': self.results
            }

        logger.info(f"Running {len(tasks)} daily tasks: {', '.join(tasks)}")

        # Run each task
        for task_id in tasks:
            task_info = DAILY_TASKS[task_id]
            logger.info(f"Running: {task_info['name']}")

            task_result = self._run_task(task_id, task_info)
            self.results['tasks_run'].append({
                'task_id': task_id,
                'task_name': task_info['name'],
                'status': task_result['status'],
                'duration_seconds': task_result.get('duration', 0),
                'error': task_result.get('error'),
                'summary': task_result.get('summary', {})
            })

            if task_result['status'] == 'success':
                self.results['tasks_succeeded'].append(task_id)
            else:
                self.results['tasks_failed'].append(task_id)

        # Generate summary
        self.results['summary'] = self._generate_summary()
        self.results['duration_seconds'] = time.time() - start_time

        # Generate report
        report = self._generate_report()

        # Save report
        if self.output_path:
            report_path = Path(self.output_path)
            report_path.parent.mkdir(parents=True, exist_ok=True)
            with open(report_path, 'w') as f:
                f.write(report)
            logger.info(f"Report saved to: {report_path}")

        logger.info(f"Daily automation completed in {self.results['duration_seconds']:.2f}s")
        return {
            'status': 'success',
            'results': self.results,
            'report_path': str(self.output_path)
        }

    def _filter_tasks(self) -> List[str]:
        """Filter tasks based on configuration."""
        tasks = []

        for task_id in self.tasks_to_run:
            if task_id not in DAILY_TASKS:
                logger.warning(f"Unknown task: {task_id}")
                continue

            task_info = DAILY_TASKS[task_id]

            # Skip slow tasks unless explicitly included
            if not task_info['quick'] and not self.include_slow:
                logger.info(f"Skipping slow task: {task_id} (use --include-slow to include)")
                continue

            tasks.append(task_id)

        return tasks

    def _run_task(self, task_id: str, task_info: Dict) -> Dict[str, Any]:
        """Run a single task."""
        start_time = time.time()

        try:
            # Try to run via script first
            script_path = self.project_root / task_info['script']
            if script_path.exists():
                result = self._run_script(script_path, task_id)
            else:
                # Fallback: Try MCP tool (would need MCP client)
                logger.warning(f"Script not found: {script_path}, skipping")
                result = {'status': 'skipped', 'error': 'Script not found'}

            duration = float(time.time() - start_time)
            result_with_duration: Dict[str, Any] = {**result, 'duration': duration}
            return result_with_duration

        except Exception as e:
            logger.error(f"Error running task {task_id}: {e}", exc_info=True)
            return {
                'status': 'error',
                'error': str(e),
                'duration': float(time.time() - start_time)
            }

    def _run_script(self, script_path: Path, task_id: str) -> Dict[str, Any]:
        """Run a Python script."""
        import subprocess

        try:
            # Build command
            cmd = [sys.executable, str(script_path)]

            if self.dry_run:
                cmd.append('--dry-run')

            # Add task-specific arguments
            if task_id == 'duplicate_detection':
                cmd.extend(['--auto-fix', 'false'])
            elif task_id == 'external_tool_hints':
                cmd.extend(['--dry-run' if self.dry_run else ''])

            # Run script
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            if result.returncode == 0:
                # Try to parse JSON output
                try:
                    output_data = json.loads(result.stdout)
                    return {
                        'status': 'success',
                        'summary': output_data.get('data', {})
                    }
                except json.JSONDecodeError:
                    return {
                        'status': 'success',
                        'summary': {'output': str(result.stdout[:500])}  # First 500 chars
                    }
            else:
                return {
                    'status': 'error',
                    'error': result.stderr[:200] if result.stderr else 'Script failed'
                }

        except subprocess.TimeoutExpired:
            return {
                'status': 'error',
                'error': 'Task timed out after 5 minutes'
            }
        except Exception as e:
            return {
                'status': 'error',
                'error': str(e)
            }

    def _generate_summary(self) -> Dict:
        """Generate summary statistics."""
        total = len(self.results['tasks_run'])
        succeeded = len(self.results['tasks_succeeded'])
        failed = len(self.results['tasks_failed'])

        return {
            'total_tasks': total,
            'succeeded': succeeded,
            'failed': failed,
            'success_rate': (succeeded / total * 100) if total > 0 else 0,
            'duration_seconds': self.results['duration_seconds']
        }

    def _generate_report(self) -> str:
        """Generate markdown report."""
        summary = self.results['summary']

        report_lines = [
            "# Daily Automation Report",
            "",
            f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"**Mode**: {'DRY RUN' if self.dry_run else 'APPLIED'}",
            "",
            "## Summary",
            "",
            f"- **Total Tasks**: {summary['total_tasks']}",
            f"- **Succeeded**: {summary['succeeded']}",
            f"- **Failed**: {summary['failed']}",
            f"- **Success Rate**: {summary['success_rate']:.1f}%",
            f"- **Duration**: {summary['duration_seconds']:.2f} seconds",
            "",
            "## Task Results",
            "",
        ]

        for task_result in self.results['tasks_run']:
            status_icon = "✅" if task_result['status'] == 'success' else "❌"
            report_lines.append(f"### {status_icon} {task_result['task_name']}")
            report_lines.append("")
            report_lines.append(f"- **Status**: {task_result['status']}")
            report_lines.append(f"- **Duration**: {task_result['duration_seconds']:.2f}s")

            if task_result.get('error'):
                report_lines.append(f"- **Error**: {task_result['error']}")

            if task_result.get('summary'):
                report_lines.append("- **Summary**:")
                for key, value in task_result['summary'].items():
                    if isinstance(value, (int, float, str)):
                        report_lines.append(f"  - {key}: {value}")

            report_lines.append("")

        return '\n'.join(report_lines)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Run daily maintenance tasks')
    parser.add_argument('--tasks', nargs='+',
                       choices=list(DAILY_TASKS.keys()),
                       default=['docs_health', 'todo2_alignment', 'duplicate_detection'],
                       help='Tasks to run (default: quick tasks only)')
    parser.add_argument('--include-slow', action='store_true',
                       help='Include slow tasks (e.g., dependency security scan)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Preview changes without applying')
    parser.add_argument('--output-path', type=str,
                       default='docs/DAILY_AUTOMATION_REPORT.md',
                       help='Path for report output')

    args = parser.parse_args()

    config = {
        'tasks': args.tasks,
        'include_slow': args.include_slow,
        'dry_run': args.dry_run,
        'output_path': args.output_path
    }

    automation = DailyAutomation(config)
    results = automation.run()

    print(json.dumps(results, indent=2))
    return 0 if results['status'] == 'success' else 1


if __name__ == '__main__':
    sys.exit(main())
