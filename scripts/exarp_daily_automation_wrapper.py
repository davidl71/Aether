#!/usr/bin/env python3
"""
Exarp Daily Automation Wrapper (optional fallback).

Exarp automation is primarily via the exarp-go MCP server in Cursor. This script
is an optional fallback when `uvx exarp` is installed. It runs all three checks
(docs health, todo2 alignment, duplicate detection) via the uvx exarp CLI.

Usage:
    python3 scripts/exarp_daily_automation_wrapper.py [project_dir] [--dry-run] [--json]

Options:
    --dry-run    Run in dry-run mode (no changes)
    --json       Output results as JSON
"""

import sys
import subprocess
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Optional


class ExarpDailyAutomation:
    """Wrapper for Exarp daily automation tasks"""

    def __init__(self, project_dir: Path, dry_run: bool = False, json_output: bool = False):
        self.project_dir = project_dir.resolve()
        self.dry_run = dry_run
        self.json_output = json_output
        self.results: Dict[str, Any] = {
            'timestamp': datetime.now().isoformat(),
            'project_dir': str(self.project_dir),
            'dry_run': dry_run,
            'tasks': {}
        }

    def run_command(self, command: list, task_name: str) -> Dict[str, Any]:
        """Run an Exarp command and capture results"""
        try:
            result = subprocess.run(
                command,
                cwd=str(self.project_dir),
                capture_output=True,
                text=True,
                timeout=300
            )

            task_result = {
                'success': result.returncode == 0,
                'exit_code': result.returncode,
                'stdout': result.stdout,
                'stderr': result.stderr,
                'command': ' '.join(command)
            }

            if not self.json_output:
                if result.returncode == 0:
                    print(f"✅ {task_name}: Success")
                else:
                    print(f"❌ {task_name}: Failed (exit code {result.returncode})")
                    if result.stderr:
                        print(f"   Error: {result.stderr[:200]}")

            return task_result

        except subprocess.TimeoutExpired:
            error_msg = f"{task_name} timed out after 300 seconds"
            if not self.json_output:
                print(f"⏱️  {error_msg}")
            return {
                'success': False,
                'exit_code': -1,
                'error': error_msg,
                'command': ' '.join(command)
            }
        except FileNotFoundError:
            error_msg = f"uvx or exarp not found. Is uvx installed?"
            if not self.json_output:
                print(f"❌ {error_msg}")
            return {
                'success': False,
                'exit_code': -1,
                'error': error_msg,
                'command': ' '.join(command)
            }

    def check_documentation_health(self) -> Dict[str, Any]:
        """Run documentation health check"""
        if not self.json_output:
            print("\n📚 Task 1: Checking documentation health...")

        command = ['uvx', 'exarp', 'check-documentation-health', str(self.project_dir)]
        if self.dry_run:
            command.append('--dry-run')

        result = self.run_command(command, 'Documentation Health')
        self.results['tasks']['docs_health'] = result
        return result

    def analyze_todo2_alignment(self) -> Dict[str, Any]:
        """Run Todo2 alignment analysis"""
        if not self.json_output:
            print("\n🎯 Task 2: Analyzing Todo2 alignment...")

        command = ['uvx', 'exarp', 'analyze-todo2-alignment', str(self.project_dir)]
        if self.dry_run:
            command.append('--dry-run')

        result = self.run_command(command, 'Todo2 Alignment')
        self.results['tasks']['todo2_alignment'] = result
        return result

    def detect_duplicate_tasks(self, auto_fix: bool = False) -> Dict[str, Any]:
        """Run duplicate task detection"""
        if not self.json_output:
            print("\n🔍 Task 3: Detecting duplicate tasks...")

        command = ['uvx', 'exarp', 'detect-duplicate-tasks', str(self.project_dir)]
        if self.dry_run:
            command.append('--dry-run')
        elif auto_fix:
            command.append('--auto-fix')

        result = self.run_command(command, 'Duplicate Detection')
        self.results['tasks']['duplicate_detection'] = result
        return result

    def run_all(self, auto_fix_duplicates: bool = False) -> Dict[str, Any]:
        """Run all Exarp daily automation tasks"""
        if not self.json_output:
            print("🚀 Starting Exarp daily automation...")
            print(f"Project directory: {self.project_dir}")
            if self.dry_run:
                print("Mode: DRY-RUN (no changes will be made)")
            print()

        # Run all tasks
        self.check_documentation_health()
        self.analyze_todo2_alignment()
        self.detect_duplicate_tasks(auto_fix=auto_fix_duplicates)

        # Calculate summary
        all_success = all(
            task['success']
            for task in self.results['tasks'].values()
        )

        self.results['summary'] = {
            'all_success': all_success,
            'tasks_completed': len(self.results['tasks']),
            'tasks_succeeded': sum(1 for t in self.results['tasks'].values() if t['success']),
            'tasks_failed': sum(1 for t in self.results['tasks'].values() if not t['success'])
        }

        if not self.json_output:
            print("\n" + "=" * 70)
            print("📊 Summary:")
            print(f"   Tasks completed: {self.results['summary']['tasks_completed']}")
            print(f"   Tasks succeeded: {self.results['summary']['tasks_succeeded']}")
            print(f"   Tasks failed: {self.results['summary']['tasks_failed']}")
            if all_success:
                print("   ✅ All tasks completed successfully")
            else:
                print("   ⚠️  Some tasks failed - check output above")
            print("=" * 70)

        return self.results


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Exarp daily automation wrapper - orchestrates Exarp MCP tools'
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project directory (default: current directory)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Run in dry-run mode (no changes)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )
    parser.add_argument(
        '--auto-fix',
        action='store_true',
        help='Auto-fix duplicate tasks (ignored in dry-run mode)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()

    if not project_dir.exists():
        print(f"Error: Project directory does not exist: {project_dir}", file=sys.stderr)
        sys.exit(1)

    automation = ExarpDailyAutomation(
        project_dir=project_dir,
        dry_run=args.dry_run,
        json_output=args.json
    )

    results = automation.run_all(auto_fix_duplicates=args.auto_fix and not args.dry_run)

    if args.json:
        print(json.dumps(results, indent=2))

    # Exit with error code if any task failed
    if not results['summary']['all_success']:
        sys.exit(1)

    sys.exit(0)


if __name__ == '__main__':
    main()
