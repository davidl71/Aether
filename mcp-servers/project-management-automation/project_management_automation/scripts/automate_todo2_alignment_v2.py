#!/usr/bin/env python3
"""
Automated Todo2 Task Alignment Analysis Script (v2 - Intelligent)

Refactored to use IntelligentAutomationBase with:
- Tractatus Thinking for structure analysis
- Sequential Thinking for workflow planning
- Todo2 integration for tracking
- NetworkX for task dependency graph analysis
"""

import argparse
import json
import logging
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

# Add project root to path
# Project root will be passed to __init__

# Import base class
from .base.intelligent_automation_base import IntelligentAutomationBase

# Configure logging (will be configured after project_root is set)
logger = logging.getLogger(__name__)


class Todo2AlignmentAnalyzerV2(IntelligentAutomationBase):
    """Intelligent Todo2 alignment analyzer using base class."""

    def __init__(self, config: Dict, project_root: Optional[Path] = None):
        from project_management_automation.utils import find_project_root

        if project_root is None:
            project_root = find_project_root()

        super().__init__(config, "Todo2 Alignment Analysis", project_root)

        # Configure logging after project_root is set
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(self.project_root / 'scripts' / 'todo2_alignment.log'),
                logging.StreamHandler()
            ],
            force=True
        )
        self.todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
        self.docs_path = self.project_root / 'docs'
        self.strategy_framework_path = self.docs_path / 'INVESTMENT_STRATEGY_FRAMEWORK.md'

        # Strategy phases
        self.strategy_phases = {
            'phase1': {
                'name': 'Foundation (Weeks 1-2)',
                'keywords': ['portfolio aggregation', 'multi-account', 'position import',
                           'currency conversion', 'swiftness', 'discount bank', 'multi-broker'],
            },
            'phase2': {
                'name': 'Core Calculations (Weeks 3-6)',
                'keywords': ['cash flow', 'greeks', 'convexity', 'barbell', 'nlopt',
                           'cpi', 'loan', 'bond'],
            },
            'phase3': {
                'name': 'Advanced Features (Weeks 7-12)',
                'keywords': ['cash management', 't-bill', 'bond ladder', 'etf',
                           'rebalancing', 'allocation'],
            }
        }

    def _get_tractatus_concept(self) -> str:
        """Tractatus concept: What is task alignment?"""
        return "What is task alignment? Task Alignment = Strategy Relevance × Priority Match × Dependency Completeness × Currency"

    def _get_sequential_problem(self) -> str:
        """Sequential problem: How do we analyze task alignment?"""
        return "How do we systematically analyze Todo2 task alignment with investment strategy framework?"

    def _execute_analysis(self) -> Dict:
        """Execute Todo2 alignment analysis."""
        logger.info("Executing Todo2 alignment analysis...")

        # Load tasks
        tasks = self._load_todo2_tasks()
        logger.info(f"Loaded {len(tasks)} tasks")

        # Analyze alignment
        analysis = self._analyze_task_alignment(tasks)

        # Calculate alignment score
        alignment_score = self._calculate_alignment_score(analysis)
        analysis['alignment_score'] = alignment_score

        return analysis

    def _load_todo2_tasks(self) -> List[Dict]:
        """Load Todo2 tasks."""
        try:
            with open(self.todo2_path, 'r') as f:
                data = json.load(f)
                return data.get('todos', [])
        except Exception as e:
            logger.error(f"Error loading Todo2 tasks: {e}")
            return []

    def _analyze_task_alignment(self, tasks: List[Dict]) -> Dict:
        """Analyze task alignment."""
        analysis = {
            'total_tasks': len(tasks),
            'by_priority': {'high': 0, 'medium': 0, 'low': 0, 'critical': 0},
            'by_status': {'todo': 0, 'in_progress': 0, 'review': 0, 'done': 0},
            'by_phase': {phase: {'total': 0, 'high_priority': 0, 'aligned': 0}
                        for phase in self.strategy_phases},
            'strategy_critical': [],
            'misaligned_tasks': [],
            'stale_tasks': [],
            'blocked_tasks': [],
            'infrastructure_tasks': []
        }

        for task in tasks:
            content = str(task.get('content', '')).lower()
            long_desc = str(task.get('long_description', '')).lower()
            tags = [tag.lower() for tag in task.get('tags', [])]
            priority = task.get('priority', 'medium').lower()
            status = task.get('status', 'todo').lower()
            task_id = task.get('id', 'unknown')

            # Count by priority
            if priority in analysis['by_priority']:
                analysis['by_priority'][priority] += 1

            # Count by status
            if 'todo' in status:
                analysis['by_status']['todo'] += 1
            elif 'progress' in status:
                analysis['by_status']['in_progress'] += 1
            elif 'review' in status:
                analysis['by_status']['review'] += 1
            elif 'done' in status:
                analysis['by_status']['done'] += 1

            # Check alignment with strategy phases
            task_text = f"{content} {long_desc} {' '.join(tags)}"
            aligned_phases = []

            for phase_key, phase_info in self.strategy_phases.items():
                if any(keyword in task_text for keyword in phase_info['keywords']):
                    aligned_phases.append(phase_key)
                    analysis['by_phase'][phase_key]['total'] += 1
                    if priority == 'high':
                        analysis['by_phase'][phase_key]['high_priority'] += 1
                    analysis['by_phase'][phase_key]['aligned'] += 1

            # Identify strategy-critical tasks
            if aligned_phases and priority == 'high':
                analysis['strategy_critical'].append({
                    'id': task_id,
                    'content': task.get('content', ''),
                    'phases': aligned_phases,
                    'priority': priority,
                    'status': status
                })

            # Identify misaligned or infrastructure tasks
            if priority == 'high' and not aligned_phases:
                infrastructure_keywords = [
                    'research', 'config', 'infrastructure', 'testing',
                    'documentation', 'setup', 'build', 'analysis', 'alignment',
                    'review', 'prioritization', 'coordination', 'automation'
                ]
                if any(keyword in task_text for keyword in infrastructure_keywords):
                    analysis['infrastructure_tasks'].append({
                        'id': task_id,
                        'content': task.get('content', ''),
                        'priority': priority,
                        'status': status
                    })
                else:
                    analysis['misaligned_tasks'].append({
                        'id': task_id,
                        'content': task.get('content', ''),
                        'priority': priority,
                        'status': status
                    })

            # Check for stale tasks
            last_modified = task.get('lastModified', '')
            if last_modified:
                try:
                    modified_date = datetime.fromisoformat(last_modified.replace('Z', '+00:00'))
                    days_old = (datetime.now(timezone.utc) - modified_date).days
                    if days_old > 30 and status not in ['done', 'cancelled']:
                        analysis['stale_tasks'].append({
                            'id': task_id,
                            'content': task.get('content', ''),
                            'days_old': days_old,
                            'status': status
                        })
                except Exception:
                    pass

            # Check for blocked tasks
            dependencies = task.get('dependencies', [])
            if dependencies:
                dep_tasks = {t.get('id'): t for t in tasks}
                blocked = False
                for dep_id in dependencies:
                    dep_task = dep_tasks.get(dep_id)
                    if dep_task and dep_task.get('status', '').lower() not in ['done', 'completed']:
                        blocked = True
                        break
                if blocked:
                    analysis['blocked_tasks'].append({
                        'id': task_id,
                        'content': task.get('content', ''),
                        'dependencies': dependencies,
                        'status': status
                    })

        return analysis

    def _calculate_alignment_score(self, analysis: Dict) -> float:
        """Calculate alignment score."""
        if analysis['total_tasks'] == 0:
            return 0.0

        strategy_critical_ratio = len(analysis['strategy_critical']) / max(analysis['by_priority']['high'], 1)
        high_priority_aligned = (analysis['by_priority']['high'] - len(analysis['misaligned_tasks'])) / max(analysis['by_priority']['high'], 1)
        not_stale_ratio = 1.0 - (len(analysis['stale_tasks']) / max(analysis['total_tasks'], 1))
        not_blocked_ratio = 1.0 - (len(analysis['blocked_tasks']) / max(analysis['total_tasks'], 1))

        score = (
            strategy_critical_ratio * 0.4 +
            high_priority_aligned * 0.3 +
            not_stale_ratio * 0.2 +
            not_blocked_ratio * 0.1
        ) * 100

        return round(score, 1)

    def _generate_insights(self, analysis_results: Dict) -> str:
        """Generate insights."""
        insights = []

        alignment_score = analysis_results.get('alignment_score', 0)
        insights.append(f"**Alignment Score: {alignment_score}%**")

        if alignment_score < 70:
            insights.append("⚠️ Alignment score is below target (80%+)")

        misaligned = len(analysis_results.get('misaligned_tasks', []))
        if misaligned > 0:
            insights.append(f"⚠️ {misaligned} high-priority tasks are not strategy-aligned")

        blocked = len(analysis_results.get('blocked_tasks', []))
        if blocked > 0:
            insights.append(f"⚠️ {blocked} tasks are blocked by incomplete dependencies")

        if alignment_score >= 80:
            insights.append("✅ Task alignment is good!")

        return '\n'.join(insights)

    def _generate_report(self, analysis_results: Dict, insights: str) -> str:
        """Generate report."""
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        alignment_score = analysis_results.get('alignment_score', 0)

        # Include NetworkX analysis if available
        networkx_section = ""
        if 'networkx_analysis' in self.results:
            nx_analysis = self.results['networkx_analysis']
            networkx_section = f"""
## NetworkX Task Dependency Analysis

**Graph Statistics:**
- Tasks (Nodes): {nx_analysis.get('nodes', 0)}
- Dependencies (Edges): {nx_analysis.get('edges', 0)}
- Is DAG: {nx_analysis.get('is_dag', False)}

**Critical Path:** {len(nx_analysis.get('critical_path', []))} tasks
**Bottlenecks:** {len(nx_analysis.get('bottlenecks', []))} identified
**Orphaned Tasks:** {len(nx_analysis.get('orphans', []))} found

"""

        return f"""# Todo2 Task Priority Alignment Analysis

*Generated: {timestamp}*
*Generated By: Intelligent Todo2 Alignment Analyzer*

## Executive Summary

**Overall Alignment: {alignment_score}%** {'✅' if alignment_score >= 80 else '⚠️'}

**Key Metrics:**
- Total Tasks: {analysis_results.get('total_tasks', 0)}
- High Priority: {analysis_results.get('by_priority', {}).get('high', 0)}
- Strategy Critical: {len(analysis_results.get('strategy_critical', []))}
- Misaligned: {len(analysis_results.get('misaligned_tasks', []))}
- Blocked: {len(analysis_results.get('blocked_tasks', []))}

---

## Insights

{insights}

{networkx_section}
---

*This report was generated using intelligent automation with Tractatus Thinking, Sequential Thinking, and NetworkX analysis.*
"""

    def _needs_networkx(self) -> bool:
        """NetworkX is useful for task dependency analysis."""
        return True

    def _build_networkx_graph(self):
        """Build task dependency graph."""
        try:
            import networkx as nx

            G = nx.DiGraph()
            tasks = self._load_todo2_tasks()

            # Add tasks as nodes
            for task in tasks:
                G.add_node(
                    task.get('id', 'unknown'),
                    name=task.get('content', ''),
                    priority=task.get('priority', 'medium'),
                    status=task.get('status', 'todo')
                )

            # Add dependencies as edges
            for task in tasks:
                task_id = task.get('id', 'unknown')
                dependencies = task.get('dependencies', [])
                for dep_id in dependencies:
                    if dep_id in G:
                        G.add_edge(dep_id, task_id, relationship='depends_on')

            return G
        except ImportError:
            return None

    def _identify_followup_tasks(self, analysis_results: Dict) -> List[Dict]:
        """Identify follow-up tasks."""
        followups = []

        # Create tasks for misaligned tasks
        misaligned = analysis_results.get('misaligned_tasks', [])
        if misaligned:
            followups.append({
                'name': 'Review misaligned high-priority tasks',
                'description': f'Review {len(misaligned)} high-priority tasks that are not strategy-aligned',
                'priority': 'high',
                'tags': ['todo2', 'alignment', 'review']
            })

        # Create tasks for blocked tasks
        blocked = analysis_results.get('blocked_tasks', [])
        if blocked:
            followups.append({
                'name': 'Unblock tasks by completing dependencies',
                'description': f'Complete dependencies for {len(blocked)} blocked tasks',
                'priority': 'medium',
                'tags': ['todo2', 'dependencies']
            })

        return followups


def load_config(config_path: Optional[Path] = None) -> Dict:
    """Load configuration."""
    if config_path is None:
        config_path = project_root / 'scripts' / 'todo2_alignment_config.json'

    default_config = {
        'output_path': 'docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md'
    }

    if config_path.exists():
        try:
            with open(config_path, 'r') as f:
                user_config = json.load(f)
                default_config.update(user_config)
        except json.JSONDecodeError:
            pass

    return default_config


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Intelligent Todo2 Alignment Analysis')
    parser.add_argument('--config', type=Path, help='Path to config file')
    parser.add_argument('--output', type=Path, help='Output path for report')
    args = parser.parse_args()

    config = load_config(args.config)
    analyzer = Todo2AlignmentAnalyzerV2(config)

    try:
        results = analyzer.run()

        # Write report
        if args.output:
            output_path = args.output
        else:
            output_path = project_root / config['output_path']

        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(results['report'])

        logger.info(f"Report written to: {output_path}")
        logger.info(f"Alignment score: {results.get('results', {}).get('alignment_score', 0)}%")

        sys.exit(0)
    except Exception as e:
        logger.error(f"Error running analysis: {e}", exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()
