#!/usr/bin/env python3
"""
Automated PWA Review Analysis Script

This script automates the PWA review analysis process by:
1. Reading Todo2 task files
2. Analyzing PWA codebase structure
3. Comparing current state against goals
4. Using AI API to generate insights
5. Writing updated analysis document

Usage:
    python3 scripts/automate_pwa_review.py [--config config.json] [--output docs/PWA_IMPROVEMENT_ANALYSIS.md]

Configuration:
    See scripts/pwa_review_config.json for configuration options.
"""

import argparse
import json
import logging
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Add project root to path
# Project root will be passed to __init__

# Configure logging (will be configured after project_root is set)
logger = logging.getLogger(__name__)


class PWAAnalyzer:
    """Analyzes PWA state and generates improvement recommendations."""

    def __init__(self, config: Dict, project_root: Optional[Path] = None):
        self.config = config
        self.project_root = project_root
        self.todo2_path = project_root / '.todo2' / 'state.todo2.json'
        self.pwa_path = project_root / 'web'
        self.docs_path = project_root / 'docs'

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

    def analyze_pwa_structure(self) -> Dict:
        """Analyze PWA codebase structure."""
        analysis = {
            'components': [],
            'hooks': [],
            'api_integrations': [],
            'pwa_features': [],
            'missing_features': []
        }

        # Check for components
        components_path = self.pwa_path / 'src' / 'components'
        if components_path.exists():
            analysis['components'] = [
                f.stem for f in components_path.glob('*.tsx')
            ]

        # Check for hooks
        hooks_path = self.pwa_path / 'src' / 'hooks'
        if hooks_path.exists():
            analysis['hooks'] = [
                f.stem for f in hooks_path.glob('*.ts')
            ]

        # Check for API integrations
        api_path = self.pwa_path / 'src' / 'api'
        if api_path.exists():
            analysis['api_integrations'] = [
                f.stem for f in api_path.glob('*.ts')
            ]

        # Check PWA features
        manifest_path = self.pwa_path / 'public' / 'manifest.json'
        vite_config_path = self.pwa_path / 'vite.config.ts'

        if manifest_path.exists():
            analysis['pwa_features'].append('manifest')
        if vite_config_path.exists():
            with open(vite_config_path, 'r') as f:
                content = f.read()
                if 'VitePWA' in content:
                    analysis['pwa_features'].append('service_worker')
                if 'registerType' in content:
                    analysis['pwa_features'].append('auto_update')

        # Check for goal-aligned features
        goal_features = {
            'unified_positions': 'UnifiedPositionsPanel' in str(analysis['components']),
            'cash_flow': 'CashFlowChart' in str(analysis['components']),
            'simulation': 'OpportunitySimulator' in str(analysis['components']),
            'relationships': 'RelationshipGraph' in str(analysis['components'])
        }

        analysis['missing_features'] = [
            feature for feature, exists in goal_features.items() if not exists
        ]

        return analysis

    def analyze_todo2_alignment(self, tasks: List[Dict]) -> Dict:
        """Analyze Todo2 task alignment with PWA goals."""
        alignment = {
            'total_tasks': len(tasks),
            'pwa_related': 0,
            'goal_aligned': 0,
            'high_priority': 0,
            'in_progress': 0,
            'todo': 0,
            'done': 0
        }

        goal_keywords = [
            'pwa', 'web', 'unified', 'position', 'cash flow', 'simulation',
            'opportunity', 'relationship', 'portfolio', 'aggregation'
        ]

        for task in tasks:
            content = str(task.get('content', '')).lower()
            tags = [tag.lower() for tag in task.get('tags', [])]
            long_desc = str(task.get('long_description', '')).lower()

            # Check if PWA-related
            if any(keyword in content or keyword in long_desc or keyword in tags
                   for keyword in goal_keywords):
                alignment['pwa_related'] += 1

                # Check goal alignment
                if any(keyword in content or keyword in long_desc
                       for keyword in ['unified', 'cash flow', 'simulation', 'opportunity']):
                    alignment['goal_aligned'] += 1

            # Count by priority
            if task.get('priority') == 'high':
                alignment['high_priority'] += 1

            # Count by status
            status = task.get('status', '').lower()
            if 'progress' in status:
                alignment['in_progress'] += 1
            elif 'todo' in status:
                alignment['todo'] += 1
            elif 'done' in status:
                alignment['done'] += 1

        return alignment

    def generate_ai_insights(self, pwa_analysis: Dict, todo2_alignment: Dict) -> str:
        """Generate AI insights using configured API."""
        api_provider = self.config.get('ai_api', {}).get('provider', 'openai')

        if api_provider == 'none':
            return self._generate_basic_insights(pwa_analysis, todo2_alignment)

        # Prepare prompt
        prompt = self._build_ai_prompt(pwa_analysis, todo2_alignment)

        try:
            if api_provider == 'openai':
                return self._call_openai_api(prompt)
            elif api_provider == 'anthropic':
                return self._call_anthropic_api(prompt)
            else:
                logger.warning(f"Unknown AI provider: {api_provider}, using basic insights")
                return self._generate_basic_insights(pwa_analysis, todo2_alignment)
        except Exception as e:
            logger.error(f"Error calling AI API: {e}")
            logger.info("Falling back to basic insights")
            return self._generate_basic_insights(pwa_analysis, todo2_alignment)

    def _build_ai_prompt(self, pwa_analysis: Dict, todo2_alignment: Dict) -> str:
        """Build prompt for AI API."""
        return f"""Analyze the PWA state and provide improvement recommendations.

PWA Structure:
- Components: {len(pwa_analysis['components'])} components found
- Hooks: {len(pwa_analysis['hooks'])} hooks found
- API Integrations: {len(pwa_analysis['api_integrations'])} integrations
- PWA Features: {', '.join(pwa_analysis['pwa_features'])}
- Missing Goal Features: {', '.join(pwa_analysis['missing_features'])}

Todo2 Alignment:
- Total Tasks: {todo2_alignment['total_tasks']}
- PWA Related: {todo2_alignment['pwa_related']}
- Goal Aligned: {todo2_alignment['goal_aligned']}
- High Priority: {todo2_alignment['high_priority']}
- In Progress: {todo2_alignment['in_progress']}
- Todo: {todo2_alignment['todo']}
- Done: {todo2_alignment['done']}

Provide:
1. Key findings and gaps
2. Priority recommendations
3. Implementation suggestions
4. Alignment with investment strategy goals
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
                    {'role': 'system', 'content': 'You are a PWA architecture expert analyzing investment strategy alignment.'},
                    {'role': 'user', 'content': prompt}
                ],
                temperature=0.7,
                max_tokens=2000
            )

            return response.choices[0].message.content
        except ImportError:
            logger.warning("OpenAI library not installed. Install with: pip install openai")
            return self._generate_basic_insights({}, {})

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
            return self._generate_basic_insights({}, {})

    def _generate_basic_insights(self, pwa_analysis: Dict, todo2_alignment: Dict) -> str:
        """Generate basic insights without AI API."""
        insights = []

        if pwa_analysis.get('missing_features'):
            insights.append(f"Missing Goal Features: {', '.join(pwa_analysis['missing_features'])}")

        if todo2_alignment.get('goal_aligned', 0) < todo2_alignment.get('pwa_related', 0) * 0.5:
            insights.append("Low goal alignment - consider refocusing PWA tasks on primary goals")

        if todo2_alignment.get('todo', 0) > todo2_alignment.get('in_progress', 0) * 2:
            insights.append("Many tasks in Todo state - consider prioritizing and starting work")

        return '\n'.join(insights) if insights else "No significant insights identified."

    def generate_analysis_document(self, pwa_analysis: Dict, todo2_alignment: Dict,
                                   ai_insights: str) -> str:
        """Generate the analysis document markdown."""
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')

        doc = f"""# PWA Improvement Analysis & Alignment Plan

**Date**: {timestamp}
**Generated By**: Automated PWA Review Script
**Purpose**: Analyze current PWA state, identify gaps, and create improvement plan aligned with investment strategy goals

---

## Executive Summary

**Current PWA State**: ✅ Basic PWA functionality implemented
**Gap Analysis**: ⚠️ PWA alignment with goals needs improvement
**Recommendation**: Enhance PWA to become primary interface for investment strategy framework

---

## 1. Current PWA State Analysis

### 1.1 Implemented Features

**✅ PWA Core Features:**
- Components: {len(pwa_analysis.get('components', []))} components
- Hooks: {len(pwa_analysis.get('hooks', []))} hooks
- API Integrations: {len(pwa_analysis.get('api_integrations', []))} integrations
- PWA Features: {', '.join(pwa_analysis.get('pwa_features', []))}

### 1.2 Current Limitations

**❌ Missing Goal Alignment Features:**
{chr(10).join(f"- {feature.replace('_', ' ').title()}" for feature in pwa_analysis.get('missing_features', []))}

---

## 2. Todo2 Task Alignment Analysis

### 2.1 Task Statistics

- **Total Tasks**: {todo2_alignment.get('total_tasks', 0)}
- **PWA Related**: {todo2_alignment.get('pwa_related', 0)}
- **Goal Aligned**: {todo2_alignment.get('goal_aligned', 0)}
- **High Priority**: {todo2_alignment.get('high_priority', 0)}
- **In Progress**: {todo2_alignment.get('in_progress', 0)}
- **Todo**: {todo2_alignment.get('todo', 0)}
- **Done**: {todo2_alignment.get('done', 0)}

### 2.2 Alignment Score

**Goal Alignment**: {round(todo2_alignment.get('goal_aligned', 0) / max(todo2_alignment.get('pwa_related', 1), 1) * 100, 1)}%

---

## 3. AI-Generated Insights

{ai_insights}

---

## 4. Recommendations

### 4.1 High Priority

1. **Implement Missing Goal Features**
   - Focus on: {', '.join(pwa_analysis.get('missing_features', [])[:3])}

2. **Improve Goal Alignment**
   - Current alignment: {round(todo2_alignment.get('goal_aligned', 0) / max(todo2_alignment.get('pwa_related', 1), 1) * 100, 1)}%
   - Target: 80%+ alignment

### 4.2 Next Steps

1. Review this analysis
2. Prioritize missing features
3. Update Todo2 tasks to align with goals
4. Begin implementation

---

## 5. References

- `docs/PRIMARY_GOALS_AND_REQUIREMENTS.md` - Primary goals definition
- `docs/INVESTMENT_STRATEGY_FRAMEWORK.md` - Investment strategy framework
- `docs/SYNTHETIC_FINANCING_ARCHITECTURE.md` - Architecture design
- `docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md` - Task priority analysis

---

*This analysis was automatically generated. Review and update as needed.*
"""
        return doc

    def run(self, output_path: Optional[Path] = None) -> bool:
        """Run the complete analysis."""
        logger.info("Starting PWA review analysis...")

        # Load data
        tasks = self.load_todo2_tasks()
        logger.info(f"Loaded {len(tasks)} Todo2 tasks")

        # Analyze
        pwa_analysis = self.analyze_pwa_structure()
        logger.info(f"Analyzed PWA structure: {len(pwa_analysis['components'])} components")

        todo2_alignment = self.analyze_todo2_alignment(tasks)
        logger.info(f"Todo2 alignment: {todo2_alignment['goal_aligned']}/{todo2_alignment['pwa_related']} goal-aligned")

        # Generate insights
        ai_insights = self.generate_ai_insights(pwa_analysis, todo2_alignment)
        logger.info("Generated AI insights")

        # Generate document
        doc = self.generate_analysis_document(pwa_analysis, todo2_alignment, ai_insights)

        # Write output
        if output_path is None:
            output_path = self.docs_path / 'PWA_IMPROVEMENT_ANALYSIS.md'

        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(doc)

        logger.info(f"Analysis written to: {output_path}")
        return True


def load_config(config_path: Optional[Path] = None) -> Dict:
    """Load configuration from file or use defaults."""
    if config_path is None:
        config_path = project_root / 'scripts' / 'pwa_review_config.json'

    default_config = {
        'ai_api': {
            'provider': 'none',  # 'openai', 'anthropic', or 'none'
            'model': 'gpt-4',
            'api_key': None  # Set via environment variable
        },
        'output_path': 'docs/PWA_IMPROVEMENT_ANALYSIS.md'
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
    parser = argparse.ArgumentParser(description='Automated PWA Review Analysis')
    parser.add_argument('--config', type=Path, help='Path to config file')
    parser.add_argument('--output', type=Path, help='Output path for analysis document')
    args = parser.parse_args()

    config = load_config(args.config)
    analyzer = PWAAnalyzer(config)

    try:
        success = analyzer.run(args.output)
        sys.exit(0 if success else 1)
    except Exception as e:
        logger.error(f"Error running analysis: {e}", exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()
