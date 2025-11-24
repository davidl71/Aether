#!/usr/bin/env python3
"""
Automated Todo2 Duplicate Task Detection Script

Detects duplicate tasks in Todo2 by analyzing:
- Task names (exact and fuzzy matches)
- Task descriptions (similarity)
- Task IDs (should be unique)
- Task content (long descriptions)

Uses IntelligentAutomationBase for consistency with other automation tools.
"""

import argparse
import json
import logging
import sys
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from difflib import SequenceMatcher

# Add project root to path
# Project root will be passed to __init__

# Import base class
from .base.intelligent_automation_base import IntelligentAutomationBase

# Configure logging (will be configured after project_root is set)
logger = logging.getLogger(__name__)


class Todo2DuplicateDetector(IntelligentAutomationBase):
    """Intelligent Todo2 duplicate task detector using base class."""

    def __init__(self, config: Dict, project_root: Optional[Path] = None):
        from project_management_automation.utils import find_project_root
        if project_root is None:
            project_root = find_project_root()
        super().__init__(config, "Todo2 Duplicate Detection"), project_root)
        self.todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
        self.output_path = self.project_root / config.get('output_path', 'docs/TODO2_DUPLICATE_DETECTION_REPORT.md')
        self.similarity_threshold = config.get('similarity_threshold', 0.85)
        self.auto_fix = config.get('auto_fix', False)

        # Detection results
        self.duplicates = {
            'duplicate_ids': [],  # Multiple tasks with same ID
            'exact_name_matches': [],  # Tasks with identical names
            'similar_name_matches': [],  # Tasks with similar names (>threshold)
            'similar_description_matches': [],  # Tasks with similar descriptions
            'self_dependencies': []  # Tasks that depend on themselves
        }

    def _get_tractatus_concept(self) -> str:
        """Tractatus concept: What is duplicate detection?"""
        return "What is duplicate detection? Duplicate Detection = Unique ID × Name Uniqueness × Description Similarity × Dependency Validity"

    def _get_sequential_problem(self) -> str:
        """Sequential problem: How do we detect duplicates?"""
        return "How do we systematically detect duplicate tasks in Todo2?"

    def _execute_analysis(self) -> Dict:
        """Execute duplicate detection analysis."""
        logger.info("Executing Todo2 duplicate detection...")

        # Load tasks
        tasks = self._load_todo2_tasks()
        if not tasks:
            logger.error("No tasks found in Todo2 state file")
            return {'error': 'No tasks found'}

        logger.info(f"Analyzing {len(tasks)} tasks...")

        # Detect duplicates
        self._detect_duplicate_ids(tasks)
        self._detect_exact_name_matches(tasks)
        self._detect_similar_name_matches(tasks)
        self._detect_similar_descriptions(tasks)
        self._detect_self_dependencies(tasks)

        # Summary
        total_duplicates = (
            len(self.duplicates['duplicate_ids']) +
            len(self.duplicates['exact_name_matches']) +
            len(self.duplicates['similar_name_matches']) +
            len(self.duplicates['similar_description_matches']) +
            len(self.duplicates['self_dependencies'])
        )

        results = {
            'total_tasks': len(tasks),
            'duplicates_found': total_duplicates,
            'duplicate_ids': self.duplicates['duplicate_ids'],
            'exact_name_matches': self.duplicates['exact_name_matches'],
            'similar_name_matches': self.duplicates['similar_name_matches'],
            'similar_description_matches': self.duplicates['similar_description_matches'],
            'self_dependencies': self.duplicates['self_dependencies']
        }

        logger.info(f"Found {total_duplicates} duplicate issues")

        # Store in results for reporting
        results['duplicates_found'] = total_duplicates
        return results

    def _load_todo2_tasks(self) -> List[Dict]:
        """Load tasks from Todo2 state file."""
        try:
            with open(self.todo2_path, 'r') as f:
                data = json.load(f)
            return data.get('todos', [])
        except FileNotFoundError:
            logger.error(f"Todo2 state file not found: {self.todo2_path}")
            return []
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON in Todo2 state file: {e}")
            return []

    def _detect_duplicate_ids(self, tasks: List[Dict]):
        """Detect tasks with duplicate IDs (should never happen)."""
        id_counts = Counter([t['id'] for t in tasks])
        duplicates = {id: count for id, count in id_counts.items() if count > 1}

        for task_id, count in duplicates.items():
            duplicate_tasks = [t for t in tasks if t['id'] == task_id]
            self.duplicates['duplicate_ids'].append({
                'id': task_id,
                'count': count,
                'tasks': [
                    {
                        'id': t['id'],
                        'name': t.get('name', ''),
                        'status': t.get('status', 'unknown'),
                        'created': t.get('created', 'unknown')
                    }
                    for t in duplicate_tasks
                ]
            })

    def _detect_exact_name_matches(self, tasks: List[Dict]):
        """Detect tasks with identical names."""
        name_to_tasks = defaultdict(list)
        for task in tasks:
            name = task.get('name', '').strip().lower()
            if name:
                name_to_tasks[name].append(task)

        for name, task_list in name_to_tasks.items():
            if len(task_list) > 1:
                # Filter out tasks with different IDs (true duplicates)
                task_ids = [t['id'] for t in task_list]
                if len(set(task_ids)) > 1:  # Different IDs = potential duplicates
                    self.duplicates['exact_name_matches'].append({
                        'name': task_list[0].get('name', ''),
                        'count': len(task_list),
                        'tasks': [
                            {
                                'id': t['id'],
                                'name': t.get('name', ''),
                                'status': t.get('status', 'unknown'),
                                'created': t.get('created', 'unknown'),
                                'priority': t.get('priority', 'none')
                            }
                            for t in task_list
                        ]
                    })

    def _detect_similar_name_matches(self, tasks: List[Dict]):
        """Detect tasks with similar names (fuzzy matching)."""
        for i, task1 in enumerate(tasks):
            name1 = task1.get('name', '').strip().lower()
            if not name1 or len(name1) < 10:  # Skip very short names
                continue

            for task2 in tasks[i + 1:]:
                name2 = task2.get('name', '').strip().lower()
                if not name2 or task1['id'] == task2['id']:
                    continue

                similarity = SequenceMatcher(None, name1, name2).ratio()
                if similarity >= self.similarity_threshold:
                    self.duplicates['similar_name_matches'].append({
                        'similarity': similarity,
                        'tasks': [
                            {
                                'id': task1['id'],
                                'name': task1.get('name', ''),
                                'status': task1.get('status', 'unknown')
                            },
                            {
                                'id': task2['id'],
                                'name': task2.get('name', ''),
                                'status': task2.get('status', 'unknown')
                            }
                        ]
                    })

    def _detect_similar_descriptions(self, tasks: List[Dict]):
        """Detect tasks with similar long descriptions."""
        for i, task1 in enumerate(tasks):
            desc1 = task1.get('long_description', '').strip()
            if not desc1 or len(desc1) < 50:  # Skip very short descriptions
                continue

            for task2 in tasks[i + 1:]:
                desc2 = task2.get('long_description', '').strip()
                if not desc2 or task1['id'] == task2['id']:
                    continue

                similarity = SequenceMatcher(None, desc1, desc2).ratio()
                if similarity >= self.similarity_threshold:
                    self.duplicates['similar_description_matches'].append({
                        'similarity': similarity,
                        'tasks': [
                            {
                                'id': task1['id'],
                                'name': task1.get('name', ''),
                                'status': task1.get('status', 'unknown')
                            },
                            {
                                'id': task2['id'],
                                'name': task2.get('name', ''),
                                'status': task2.get('status', 'unknown')
                            }
                        ]
                    })

    def _detect_self_dependencies(self, tasks: List[Dict]):
        """Detect tasks that depend on themselves (invalid)."""
        for task in tasks:
            deps = task.get('dependencies', [])
            task_id = task['id']
            if task_id in deps:
                self.duplicates['self_dependencies'].append({
                    'id': task_id,
                    'name': task.get('name', ''),
                    'dependencies': deps
                })

    def _generate_insights(self, analysis_results: Dict) -> str:
        """Generate insights from duplicate detection results."""
        insights = []

        total_duplicates = analysis_results.get('duplicates_found', 0)
        if total_duplicates == 0:
            insights.append("✅ **No duplicates found!** Your Todo2 task list is clean.")
        else:
            insights.append(f"⚠️ **Found {total_duplicates} duplicate issues**")

            if self.duplicates['duplicate_ids']:
                insights.append(f"🚨 **CRITICAL**: {len(self.duplicates['duplicate_ids'])} duplicate task IDs (data integrity issue)")

            if self.duplicates['exact_name_matches']:
                insights.append(f"🔍 **{len(self.duplicates['exact_name_matches'])} exact name matches** (likely duplicates)")

            if self.duplicates['similar_name_matches']:
                insights.append(f"🔍 **{len(self.duplicates['similar_name_matches'])} similar name matches** (potential duplicates)")

            if self.duplicates['self_dependencies']:
                insights.append(f"⚠️ **{len(self.duplicates['self_dependencies'])} self-dependencies** (invalid)")

        return "\n\n".join(insights)

    def _generate_report(self, analysis_results: Dict, insights: Optional[str] = None) -> str:
        """Generate markdown report."""
        report = f"""# Todo2 Duplicate Task Detection Report

**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
**Total Tasks Analyzed**: {analysis_results.get('total_tasks', 0)}
**Duplicates Found**: {analysis_results.get('duplicates_found', 0)}

---

## Summary

"""

        # Duplicate IDs (critical)
        if self.duplicates['duplicate_ids']:
            report += f"### ⚠️ Critical: Duplicate Task IDs ({len(self.duplicates['duplicate_ids'])})\n\n"
            report += "**This should never happen!** Multiple tasks with the same ID found.\n\n"
            for dup in self.duplicates['duplicate_ids']:
                report += f"**Task ID: {dup['id']}** (appears {dup['count']} times)\n\n"
                for task in dup['tasks']:
                    report += f"- `{task['id']}`: {task['name']} (Status: {task['status']}, Created: {task['created']})\n"
                report += "\n"

        # Exact name matches
        if self.duplicates['exact_name_matches']:
            report += f"### 🔍 Exact Name Matches ({len(self.duplicates['exact_name_matches'])})\n\n"
            report += "Tasks with identical names but different IDs:\n\n"
            for match in self.duplicates['exact_name_matches']:
                report += f"**Name**: \"{match['name']}\" ({match['count']} tasks)\n\n"
                for task in match['tasks']:
                    report += f"- `{task['id']}`: {task['name']} (Status: {task['status']}, Priority: {task['priority']})\n"
                report += "\n"

        # Similar name matches
        if self.duplicates['similar_name_matches']:
            report += f"### 🔍 Similar Name Matches ({len(self.duplicates['similar_name_matches'])})\n\n"
            report += f"Tasks with similar names (similarity ≥ {self.similarity_threshold * 100:.0f}%):\n\n"
            for match in self.duplicates['similar_name_matches'][:20]:  # Limit to top 20
                report += f"**Similarity**: {match['similarity'] * 100:.1f}%\n"
                for task in match['tasks']:
                    report += f"- `{task['id']}`: {task['name']} (Status: {task['status']})\n"
                report += "\n"

        # Similar descriptions
        if self.duplicates['similar_description_matches']:
            report += f"### 📝 Similar Description Matches ({len(self.duplicates['similar_description_matches'])})\n\n"
            report += f"Tasks with similar descriptions (similarity ≥ {self.similarity_threshold * 100:.0f}%):\n\n"
            for match in self.duplicates['similar_description_matches'][:10]:  # Limit to top 10
                report += f"**Similarity**: {match['similarity'] * 100:.1f}%\n"
                for task in match['tasks']:
                    report += f"- `{task['id']}`: {task['name']} (Status: {task['status']})\n"
                report += "\n"

        # Self-dependencies
        if self.duplicates['self_dependencies']:
            report += f"### ⚠️ Self-Dependencies ({len(self.duplicates['self_dependencies'])})\n\n"
            report += "Tasks that depend on themselves (invalid):\n\n"
            for task in self.duplicates['self_dependencies']:
                report += f"- `{task['id']}`: {task['name']}\n"
                report += f"  Dependencies: {', '.join(task['dependencies'])}\n\n"

        # Recommendations
        report += "---\n\n## Recommendations\n\n"
        if analysis_results.get('duplicates_found', 0) == 0:
            report += "✅ **No duplicates found!** Your Todo2 task list is clean.\n\n"
        else:
            report += "### Action Items\n\n"
            if self.duplicates['duplicate_ids']:
                report += "1. **CRITICAL**: Fix duplicate IDs immediately - this is a data integrity issue\n"
            if self.duplicates['exact_name_matches']:
                report += "2. Review exact name matches and consolidate duplicates\n"
            if self.duplicates['similar_name_matches']:
                report += "3. Review similar name matches for potential consolidation\n"
            if self.duplicates['self_dependencies']:
                report += "4. Remove self-dependencies (tasks cannot depend on themselves)\n"
            report += "\n"

        report += "### How to Fix\n\n"
        report += "1. Review the duplicate tasks listed above\n"
        report += "2. Determine which task to keep (usually the one with more comments/history)\n"
        report += "3. Update dependencies pointing to deleted tasks\n"
        report += "4. Delete duplicate tasks\n"
        report += "5. Re-run this script to verify fixes\n\n"

        report += "---\n\n"
        report += f"*Report generated by `scripts/automate_todo2_duplicate_detection.py`*\n"

        return report

    def _create_followup_tasks(self, analysis_results: Dict):
        """Create follow-up tasks if duplicates found."""
        if analysis_results.get('duplicates_found', 0) > 0:
            # Only create follow-up if not auto-fixing
            if not self.auto_fix:
                self.results['followup_tasks'].append({
                    'name': 'Review and fix duplicate Todo2 tasks',
                    'description': f"Found {analysis_results.get('duplicates_found', 0)} duplicate issues. Review report and fix.",
                    'priority': 'high' if self.duplicates['duplicate_ids'] else 'medium'
                })


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Detect duplicate Todo2 tasks')
    parser.add_argument('--config', type=str, default='scripts/todo2_duplicate_config.json',
                       help='Path to configuration file')
    parser.add_argument('--output', type=str, help='Override output path')
    parser.add_argument('--threshold', type=float, default=0.85,
                       help='Similarity threshold (0.0-1.0, default: 0.85)')
    parser.add_argument('--auto-fix', action='store_true',
                       help='Automatically fix duplicates (experimental)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Dry run mode (no changes)')

    args = parser.parse_args()

    # Load config
    config_path = project_root / args.config
    if config_path.exists():
        with open(config_path, 'r') as f:
            config = json.load(f)
    else:
        config = {}

    # Override with command-line args
    if args.output:
        config['output_path'] = args.output
    if args.threshold:
        config['similarity_threshold'] = args.threshold
    if args.auto_fix:
        config['auto_fix'] = True

    # Run detector
    detector = Todo2DuplicateDetector(config)
    results = detector.run()

    if results and 'error' not in results:
        # Write report to file
        report = results.get('report', '')
        if report:
            detector.output_path.parent.mkdir(parents=True, exist_ok=True)
            with open(detector.output_path, 'w') as f:
                f.write(report)
            logger.info(f"Report written to: {detector.output_path}")

        duplicates_found = results.get('duplicates_found', 0)
        print(f"\n✅ Duplicate detection complete!")
        print(f"   Report: {detector.output_path}")
        print(f"   Duplicates found: {duplicates_found}")
    else:
        logger.error("Duplicate detection failed")
        sys.exit(1)


if __name__ == '__main__':
    main()
