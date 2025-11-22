#!/usr/bin/env python3
"""
Automated Todo2 Task Alignment Analysis Script

This script automates Todo2 task alignment analysis by:
1. Reading Todo2 task files
2. Analyzing task priorities against investment strategy framework
3. Identifying misaligned tasks
4. Using AI API to generate insights (optional)
5. Writing updated alignment analysis document

Usage:
    python3 scripts/automate_todo2_alignment.py [--config config.json] [--output docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md]

Configuration:
    See scripts/todo2_alignment_config.json for configuration options.
"""

import argparse
import json
import logging
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Add project root to path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(project_root / 'scripts' / 'todo2_alignment.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


class Todo2AlignmentAnalyzer:
    """Analyzes Todo2 task alignment with investment strategy framework."""

    def __init__(self, config: Dict):
        self.config = config
        self.project_root = project_root
        self.todo2_path = project_root / '.todo2' / 'state.todo2.json'
        self.docs_path = project_root / 'docs'
        self.strategy_framework_path = self.docs_path / 'INVESTMENT_STRATEGY_FRAMEWORK.md'

        # Strategy phases from framework
        self.strategy_phases = {
            'phase1': {
                'name': 'Foundation (Weeks 1-2)',
                'keywords': ['portfolio aggregation', 'multi-account', 'position import',
                           'currency conversion', 'swiftness', 'discount bank', 'multi-broker'],
                'required_tasks': []
            },
            'phase2': {
                'name': 'Core Calculations (Weeks 3-6)',
                'keywords': ['cash flow', 'greeks', 'convexity', 'barbell', 'nlopt',
                           'cpi', 'loan', 'bond'],
                'required_tasks': []
            },
            'phase3': {
                'name': 'Advanced Features (Weeks 7-12)',
                'keywords': ['cash management', 't-bill', 'bond ladder', 'etf',
                           'rebalancing', 'allocation'],
                'required_tasks': []
            }
        }

    def load_todo2_tasks(self) -> List[Dict]:
        """Load Todo2 tasks from state file."""
        try:
            with open(self.todo2_path, 'r') as f:
                data = json.load(f)
                return data.get('todos', [])
        except FileNotFoundError:
            logger.warning(f"Todo2 state file not found: {self.todo2_path}")
            return []
        except json.JSONDecodeError as e:
            logger.error(f"Error parsing Todo2 JSON: {e}")
            return []

    def analyze_task_alignment(self, tasks: List[Dict]) -> Dict:
        """Analyze task alignment with investment strategy framework."""
        analysis = {
            'total_tasks': len(tasks),
            'by_priority': {'high': 0, 'medium': 0, 'low': 0, 'critical': 0},
            'by_status': {'todo': 0, 'in_progress': 0, 'review': 0, 'done': 0},
            'by_phase': {phase: {'total': 0, 'high_priority': 0, 'aligned': 0}
                        for phase in self.strategy_phases},
            'strategy_critical': [],
            'misaligned_tasks': [],
            'stale_tasks': [],
            'blocked_tasks': []
        }

        # Analyze each task
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

            # Identify misaligned tasks (high priority but not strategy-related)
            if priority == 'high' and not aligned_phases:
                # Check if it's infrastructure/research/meta-analysis (these are OK)
                infrastructure_keywords = [
                    'research', 'config', 'infrastructure', 'testing',
                    'documentation', 'setup', 'build', 'analysis', 'alignment',
                    'review', 'prioritization', 'coordination', 'automation',
                    'meta', 'task management', 'project management', 'audit',
                    'health check', 'monitoring', 'validation', 'sync'
                ]
                if not any(keyword in task_text for keyword in infrastructure_keywords):
                    analysis['misaligned_tasks'].append({
                        'id': task_id,
                        'content': task.get('content', ''),
                        'priority': priority,
                        'status': status
                    })
                else:
                    # Track infrastructure tasks separately
                    if 'infrastructure_tasks' not in analysis:
                        analysis['infrastructure_tasks'] = []
                    analysis['infrastructure_tasks'].append({
                        'id': task_id,
                        'content': task.get('content', ''),
                        'priority': priority,
                        'status': status,
                        'category': 'infrastructure'
                    })

            # Identify stale tasks (no updates in 30+ days)
            last_modified = task.get('lastModified', '')
            if last_modified:
                try:
                    from datetime import datetime, timezone
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

            # Identify blocked tasks (has dependencies that aren't done)
            dependencies = task.get('dependencies', [])
            if dependencies:
                # Check if any dependency is not done
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

    def calculate_alignment_score(self, analysis: Dict) -> float:
        """Calculate overall alignment score (0-100)."""
        if analysis['total_tasks'] == 0:
            return 0.0

        # Factors:
        # 1. Strategy-critical tasks are high priority (weight: 40%)
        # 2. High priority tasks are strategy-aligned (weight: 30%)
        # 3. Tasks are not stale (weight: 20%)
        # 4. Tasks are not blocked (weight: 10%)

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

    def generate_ai_insights(self, analysis: Dict, alignment_score: float) -> str:
        """Generate AI insights using configured API."""
        api_provider = self.config.get('ai_api', {}).get('provider', 'none')

        if api_provider == 'none':
            return self._generate_basic_insights(analysis, alignment_score)

        # Prepare prompt
        prompt = self._build_ai_prompt(analysis, alignment_score)

        try:
            if api_provider == 'openai':
                return self._call_openai_api(prompt)
            elif api_provider == 'anthropic':
                return self._call_anthropic_api(prompt)
            else:
                logger.warning(f"Unknown AI provider: {api_provider}, using basic insights")
                return self._generate_basic_insights(analysis, alignment_score)
        except Exception as e:
            logger.error(f"Error calling AI API: {e}")
            logger.info("Falling back to basic insights")
            return self._generate_basic_insights(analysis, alignment_score)

    def _build_ai_prompt(self, analysis: Dict, alignment_score: float) -> str:
        """Build prompt for AI API."""
        return f"""Analyze Todo2 task alignment with investment strategy framework.

Task Statistics:
- Total Tasks: {analysis['total_tasks']}
- High Priority: {analysis['by_priority']['high']}
- Strategy Critical: {len(analysis['strategy_critical'])}
- Misaligned: {len(analysis['misaligned_tasks'])}
- Stale Tasks: {len(analysis['stale_tasks'])}
- Blocked Tasks: {len(analysis['blocked_tasks'])}

Alignment Score: {alignment_score}%

Phase Distribution:
- Phase 1 (Foundation): {analysis['by_phase']['phase1']['total']} tasks, {analysis['by_phase']['phase1']['high_priority']} high priority
- Phase 2 (Core Calculations): {analysis['by_phase']['phase2']['total']} tasks, {analysis['by_phase']['phase2']['high_priority']} high priority
- Phase 3 (Advanced Features): {analysis['by_phase']['phase3']['total']} tasks, {analysis['by_phase']['phase3']['high_priority']} high priority

Provide:
1. Key findings about alignment
2. Priority recommendations for misaligned tasks
3. Suggestions for stale/blocked tasks
4. Phase-specific recommendations
"""

    def _call_openai_api(self, prompt: str) -> str:
        """Call OpenAI API for insights."""
        try:
            import openai

            api_key = os.getenv('OPENAI_API_KEY') or self.config.get('ai_api', {}).get('api_key')
            if not api_key:
                raise ValueError("OpenAI API key not found")

            client = openai.OpenAI(api_key=api_key)
            response = client.chat.completions.create(
                model=self.config.get('ai_api', {}).get('model', 'gpt-4'),
                messages=[
                    {'role': 'system', 'content': 'You are a project management expert analyzing task alignment with investment strategy goals.'},
                    {'role': 'user', 'content': prompt}
                ],
                temperature=0.7,
                max_tokens=2000
            )

            return response.choices[0].message.content
        except ImportError:
            logger.warning("OpenAI library not installed. Install with: pip install openai")
            return self._generate_basic_insights({}, 0.0)

    def _call_anthropic_api(self, prompt: str) -> str:
        """Call Anthropic API for insights."""
        try:
            import anthropic

            api_key = os.getenv('ANTHROPIC_API_KEY') or self.config.get('ai_api', {}).get('api_key')
            if not api_key:
                raise ValueError("Anthropic API key not found")

            client = anthropic.Anthropic(api_key=api_key)
            response = client.messages.create(
                model=self.config.get('ai_api', {}).get('model', 'claude-3-5-sonnet-20241022'),
                max_tokens=2000,
                messages=[
                    {'role': 'user', 'content': prompt}
                ]
            )

            return response.content[0].text
        except ImportError:
            logger.warning("Anthropic library not installed. Install with: pip install anthropic")
            return self._generate_basic_insights({}, 0.0)

    def _generate_basic_insights(self, analysis: Dict, alignment_score: float) -> str:
        """Generate basic insights without AI API."""
        insights = []

        if alignment_score < 70:
            insights.append(f"⚠️ Alignment score is {alignment_score}% - below target of 80%+")

        if analysis.get('misaligned_tasks'):
            insights.append(f"⚠️ {len(analysis['misaligned_tasks'])} high-priority tasks are not aligned with strategy")

        if analysis.get('stale_tasks'):
            insights.append(f"⚠️ {len(analysis['stale_tasks'])} tasks haven't been updated in 30+ days")

        if analysis.get('blocked_tasks'):
            insights.append(f"⚠️ {len(analysis['blocked_tasks'])} tasks are blocked by incomplete dependencies")

        phase1_high = analysis.get('by_phase', {}).get('phase1', {}).get('high_priority', 0)
        phase2_high = analysis.get('by_phase', {}).get('phase2', {}).get('high_priority', 0)
        phase3_high = analysis.get('by_phase', {}).get('phase3', {}).get('high_priority', 0)

        if phase1_high == 0:
            insights.append("⚠️ Phase 1 (Foundation) has no high-priority tasks")
        if phase2_high == 0:
            insights.append("⚠️ Phase 2 (Core Calculations) has no high-priority tasks")

        return '\n'.join(insights) if insights else "✅ Task alignment looks good!"

    def generate_analysis_document(self, analysis: Dict, alignment_score: float,
                                   ai_insights: str) -> str:
        """Generate the analysis document markdown."""
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')

        # Build infrastructure tasks section
        infrastructure_section = ""
        if analysis.get('infrastructure_tasks'):
            infrastructure_section = "\n### ✅ Infrastructure Tasks (High Priority, Valid)\n\n"
            infrastructure_section += "These high-priority tasks support the project but aren't direct strategy implementation:\n\n"
            infrastructure_section += "| Task ID | Task | Priority | Status |\n"
            infrastructure_section += "|---------|------|----------|--------|\n"
            for task in analysis['infrastructure_tasks'][:10]:  # Limit to 10
                content = task.get('content', '') or task.get('name', '')
                infrastructure_section += f"| {task['id']} | {content[:50]}... | {task['priority']} | {task['status']} |\n"
            if len(analysis['infrastructure_tasks']) > 10:
                infrastructure_section += f"\n*... and {len(analysis['infrastructure_tasks']) - 10} more*\n"

        # Build misaligned tasks section
        misaligned_section = ""
        if analysis['misaligned_tasks']:
            misaligned_section = "\n### ⚠️ Misaligned Tasks (High Priority, Not Strategy-Related)\n\n"
            misaligned_section += "| Task ID | Task | Priority | Status |\n"
            misaligned_section += "|---------|------|----------|--------|\n"
            for task in analysis['misaligned_tasks'][:10]:  # Limit to 10
                content = task.get('content', '') or task.get('name', '')
                misaligned_section += f"| {task['id']} | {content[:50]}... | {task['priority']} | {task['status']} |\n"
            if len(analysis['misaligned_tasks']) > 10:
                misaligned_section += f"\n*... and {len(analysis['misaligned_tasks']) - 10} more*\n"

        # Build stale tasks section
        stale_section = ""
        if analysis['stale_tasks']:
            stale_section = "\n### ⚠️ Stale Tasks (No Updates in 30+ Days)\n\n"
            stale_section += "| Task ID | Task | Days Old | Status |\n"
            stale_section += "|---------|------|----------|--------|\n"
            for task in analysis['stale_tasks'][:10]:  # Limit to 10
                stale_section += f"| {task['id']} | {task['content'][:50]}... | {task['days_old']} | {task['status']} |\n"
            if len(analysis['stale_tasks']) > 10:
                stale_section += f"\n*... and {len(analysis['stale_tasks']) - 10} more*\n"

        # Build blocked tasks section
        blocked_section = ""
        if analysis['blocked_tasks']:
            blocked_section = "\n### ⚠️ Blocked Tasks (Dependencies Not Complete)\n\n"
            blocked_section += "| Task ID | Task | Dependencies | Status |\n"
            blocked_section += "|---------|------|--------------|--------|\n"
            for task in analysis['blocked_tasks'][:10]:  # Limit to 10
                deps = ', '.join(task['dependencies'][:3])
                if len(task['dependencies']) > 3:
                    deps += f" (+{len(task['dependencies']) - 3} more)"
                blocked_section += f"| {task['id']} | {task['content'][:50]}... | {deps} | {task['status']} |\n"
            if len(analysis['blocked_tasks']) > 10:
                blocked_section += f"\n*... and {len(analysis['blocked_tasks']) - 10} more*\n"

        doc = f"""# Todo2 Task Priority Alignment with Investment Strategy Framework

*Generated: {timestamp}*
*Analysis of {analysis['total_tasks']} active Todo2 tasks*
*Generated By: Automated Todo2 Alignment Script*

## Executive Summary

**Overall Alignment: {alignment_score}%** {'✅' if alignment_score >= 80 else '⚠️'}

**Key Metrics:**
- **Total Tasks**: {analysis['total_tasks']}
- **High Priority**: {analysis['by_priority']['high']}
- **Strategy Critical**: {len(analysis['strategy_critical'])}
- **Misaligned Tasks**: {len(analysis['misaligned_tasks'])}
- **Stale Tasks**: {len(analysis['stale_tasks'])}
- **Blocked Tasks**: {len(analysis['blocked_tasks'])}

---

## Investment Strategy Framework Priorities

From `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`, the strategy requires (in priority order):

### Phase 1: Foundation (Weeks 1-2)
1. **Portfolio Aggregation** - Multi-account, multi-broker position aggregation
2. **Position Sources** - IBKR, Israeli brokers, Discount Bank, Swiftness
3. **Currency Conversion** - ILS → USD for unified portfolio view
4. **Net Portfolio Value** - Calculate including all positions minus loan liabilities

**Tasks in Phase 1:**
- Total: {analysis['by_phase']['phase1']['total']}
- High Priority: {analysis['by_phase']['phase1']['high_priority']}
- Aligned: {analysis['by_phase']['phase1']['aligned']}

### Phase 2: Core Calculations (Weeks 3-6)
5. **Cash Flow Forecasting** - Loan payments, option expirations, bond coupons
6. **Greeks Calculation** - Portfolio-level risk metrics
7. **Convexity Optimization** - Barbell strategy with NLopt

**Tasks in Phase 2:**
- Total: {analysis['by_phase']['phase2']['total']}
- High Priority: {analysis['by_phase']['phase2']['high_priority']}
- Aligned: {analysis['by_phase']['phase2']['aligned']}

### Phase 3: Advanced Features (Weeks 7-12)
8. **Cash Management** - Immediate cash, spare cash allocation
9. **T-Bill/Bond Ladder** - Target rate allocation
10. **ETF Integration** - Rebalancing and allocation

**Tasks in Phase 3:**
- Total: {analysis['by_phase']['phase3']['total']}
- High Priority: {analysis['by_phase']['phase3']['high_priority']}
- Aligned: {analysis['by_phase']['phase3']['aligned']}

---

## Task Distribution

### By Priority

| Priority | Count | Percentage |
|----------|-------|------------|
| High | {analysis['by_priority']['high']} | {round(analysis['by_priority']['high'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| Medium | {analysis['by_priority']['medium']} | {round(analysis['by_priority']['medium'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| Low | {analysis['by_priority']['low']} | {round(analysis['by_priority']['low'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| Critical | {analysis['by_priority']['critical']} | {round(analysis['by_priority']['critical'] / max(analysis['total_tasks'], 1) * 100, 1)}% |

### By Status

| Status | Count | Percentage |
|--------|-------|------------|
| Todo | {analysis['by_status']['todo']} | {round(analysis['by_status']['todo'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| In Progress | {analysis['by_status']['in_progress']} | {round(analysis['by_status']['in_progress'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| Review | {analysis['by_status']['review']} | {round(analysis['by_status']['review'] / max(analysis['total_tasks'], 1) * 100, 1)}% |
| Done | {analysis['by_status']['done']} | {round(analysis['by_status']['done'] / max(analysis['total_tasks'], 1) * 100, 1)}% |

---

## Strategy-Critical Tasks

**High-priority tasks aligned with investment strategy:**

| Task ID | Task | Phase | Status |
|---------|------|-------|--------|
{infrastructure_section}{misaligned_section}{stale_section}{blocked_section}
---

## AI-Generated Insights

{ai_insights}

---

## Recommendations

### Immediate Actions

1. **Review Misaligned Tasks**
   - {len(analysis['misaligned_tasks'])} high-priority tasks may need priority adjustment
   - Consider if they should be medium priority or if they're actually strategy-related

2. **Address Stale Tasks**
   - {len(analysis['stale_tasks'])} tasks haven't been updated in 30+ days
   - Review and either complete, cancel, or update these tasks

3. **Unblock Blocked Tasks**
   - {len(analysis['blocked_tasks'])} tasks are waiting on dependencies
   - Prioritize dependency completion to unblock work

### Phase-Specific Recommendations

**Phase 1 (Foundation):**
- Ensure high-priority tasks are progressing
- Focus on portfolio aggregation and position sources

**Phase 2 (Core Calculations):**
- Prioritize cash flow and Greeks calculations
- Ensure convexity optimization tasks are on track

**Phase 3 (Advanced Features):**
- Plan for cash management and T-bill ladder features
- Prepare for ETF integration work

---

## Alignment Score Breakdown

**Overall Score: {alignment_score}%**

**Components:**
- Strategy-critical tasks are high priority: {round(len(analysis['strategy_critical']) / max(analysis['by_priority']['high'], 1) * 100, 1)}%
- High priority tasks are strategy-aligned: {round((analysis['by_priority']['high'] - len(analysis['misaligned_tasks'])) / max(analysis['by_priority']['high'], 1) * 100, 1)}%
- Tasks are not stale: {round((1.0 - len(analysis['stale_tasks']) / max(analysis['total_tasks'], 1)) * 100, 1)}%
- Tasks are not blocked: {round((1.0 - len(analysis['blocked_tasks']) / max(analysis['total_tasks'], 1)) * 100, 1)}%

**Target: 80%+ alignment**

---

## Next Steps

1. Review this analysis
2. Update task priorities as needed
3. Address stale and blocked tasks
4. Focus on strategy-critical tasks
5. Re-run analysis to track improvement

---

## References

- `docs/INVESTMENT_STRATEGY_FRAMEWORK.md` - Investment strategy framework
- `docs/TODO2_PRIORITIZED_ACTION_PLAN.md` - Action plan
- `docs/TODO2_SYNTHETIC_FINANCING_ALIGNMENT_ANALYSIS.md` - Synthetic financing alignment

---

*This analysis was automatically generated. Review and update task priorities as needed.*
"""
        return doc

    def run(self, output_path: Optional[Path] = None) -> bool:
        """Run the complete analysis."""
        logger.info("Starting Todo2 alignment analysis...")

        # Load data
        tasks = self.load_todo2_tasks()
        logger.info(f"Loaded {len(tasks)} Todo2 tasks")

        # Analyze
        analysis = self.analyze_task_alignment(tasks)
        logger.info(f"Analyzed {analysis['total_tasks']} tasks")

        # Calculate alignment score
        alignment_score = self.calculate_alignment_score(analysis)
        logger.info(f"Alignment score: {alignment_score}%")

        # Generate insights
        ai_insights = self.generate_ai_insights(analysis, alignment_score)
        logger.info("Generated insights")

        # Generate document
        doc = self.generate_analysis_document(analysis, alignment_score, ai_insights)

        # Write output
        if output_path is None:
            output_path = self.docs_path / 'TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md'

        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(doc)

        logger.info(f"Analysis written to: {output_path}")
        return True


def load_config(config_path: Optional[Path] = None) -> Dict:
    """Load configuration from file or use defaults."""
    if config_path is None:
        config_path = project_root / 'scripts' / 'todo2_alignment_config.json'

    default_config = {
        'ai_api': {
            'provider': 'none',  # 'openai', 'anthropic', or 'none'
            'model': 'gpt-4',
            'api_key': None  # Set via environment variable
        },
        'output_path': 'docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md'
    }

    if config_path.exists():
        try:
            with open(config_path, 'r') as f:
                user_config = json.load(f)
                default_config.update(user_config)
        except json.JSONDecodeError as e:
            logger.warning(f"Error loading config: {e}, using defaults")
    else:
        logger.info(f"Config file not found: {config_path}, using defaults")

    return default_config


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Automated Todo2 Alignment Analysis')
    parser.add_argument('--config', type=Path, help='Path to config file')
    parser.add_argument('--output', type=Path, help='Output path for analysis document')
    args = parser.parse_args()

    config = load_config(args.config)
    analyzer = Todo2AlignmentAnalyzer(config)

    try:
        success = analyzer.run(args.output)
        sys.exit(0 if success else 1)
    except Exception as e:
        logger.error(f"Error running analysis: {e}", exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()
