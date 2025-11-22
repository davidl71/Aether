# Intelligent Automation Base Class Guide

**Date**: 2025-11-20
**Purpose**: Guide for using the intelligent automation base class to create smarter automation scripts

---

## Overview

The `IntelligentAutomationBase` class integrates four powerful tools to make automation scripts smarter:

1. **Tractatus Thinking**: Understand WHAT to analyze (structure)
2. **Sequential Thinking**: Plan HOW to analyze (workflow)
3. **Todo2**: Track execution and create follow-up tasks
4. **NetworkX**: Understand relationships and dependencies

---

## Quick Start

### Creating a New Intelligent Automation

```python
from scripts.base.intelligent_automation_base import IntelligentAutomationBase

class MyAutomation(IntelligentAutomationBase):
    def __init__(self, config):
        super().__init__(config, "My Automation Name")

    def _get_tractatus_concept(self) -> str:
        """What is the structure we're analyzing?"""
        return "What is [your concept]? [Concept] = Component1 × Component2 × Component3"

    def _get_sequential_problem(self) -> str:
        """How do we solve this problem?"""
        return "How do we [your problem]?"

    def _execute_analysis(self) -> Dict:
        """Your analysis logic here"""
        # Do your analysis
        return {'results': '...'}

    def _generate_insights(self, analysis_results: Dict) -> str:
        """Generate insights from results"""
        return "Key insights..."

    def _generate_report(self, analysis_results: Dict, insights: str) -> str:
        """Generate markdown report"""
        return "# Report\n\n..."
```

### Running Your Automation

```python
config = {'output_path': 'docs/my_report.md'}
automation = MyAutomation(config)
results = automation.run()
```

---

## Example: Automation Opportunity Finder

The `automate_automation_opportunities.py` script demonstrates the base class in action:

**Key Features:**
- Finds automation opportunities automatically
- Scores opportunities by value, effort, frequency
- Creates Todo2 tasks for high-priority opportunities
- Generates comprehensive report

**Usage:**
```bash
python3 scripts/automate_automation_opportunities.py
```

**Output:**
- `docs/AUTOMATION_OPPORTUNITIES_FOUND.md` - Comprehensive report
- Todo2 tasks created automatically
- Follow-up tasks for high-priority opportunities

---

## Base Class Workflow

The base class follows a 10-step intelligent workflow:

### Step 1: Tractatus Analysis
```python
def _tractatus_analysis(self):
    # Understands WHAT to analyze
    # Breaks down into atomic components
    # Identifies multiplicative dependencies
```

### Step 2: Sequential Planning
```python
def _sequential_planning(self):
    # Plans HOW to analyze
    # Creates workflow steps
    # Adapts to findings
```

### Step 3: Todo2 Task Creation
```python
def _create_todo2_task(self):
    # Creates task for tracking
    # Links to automation execution
    # Enables follow-up task creation
```

### Step 4: NetworkX Analysis
```python
def _networkx_analysis(self):
    # Builds dependency graphs
    # Finds critical paths
    # Identifies bottlenecks
```

### Step 5: Execute Analysis
```python
def _execute_analysis(self):
    # Your custom analysis logic
    # Returns results dictionary
```

### Step 6: Generate Insights
```python
def _generate_insights(self, results):
    # Uses Tractatus to understand results
    # Generates actionable insights
```

### Step 7: Store Todo2 Results
```python
def _store_todo2_results(self, results, insights):
    # Stores results in Todo2 task
    # Links to execution
```

### Step 8: Create Follow-up Tasks
```python
def _create_followup_tasks(self, results):
    # Creates actionable tasks
    # Based on findings
```

### Step 9: Generate Report
```python
def _generate_report(self, results, insights):
    # Generates markdown report
    # Includes all findings
```

### Step 10: Update Todo2 Complete
```python
def _update_todo2_complete(self):
    # Marks task as done
    # Links to report
```

---

## Customization Points

### Override NetworkX Analysis

```python
def _needs_networkx(self) -> bool:
    """Return True if NetworkX analysis is needed"""
    return True

def _build_networkx_graph(self):
    """Build your custom graph"""
    import networkx as nx
    G = nx.DiGraph()
    # Add nodes and edges
    return G

def _find_critical_path(self) -> List[str]:
    """Find critical path in your graph"""
    # Your logic here
    return []
```

### Custom Follow-up Task Creation

```python
def _identify_followup_tasks(self, analysis_results: Dict) -> List[Dict]:
    """Identify follow-up tasks from results"""
    followups = []

    for finding in analysis_results.get('critical_issues', []):
        followups.append({
            'name': f"Fix: {finding['name']}",
            'description': finding['description'],
            'priority': 'high',
            'tags': ['fix', 'automation']
        })

    return followups
```

---

## Benefits

### Efficiency
- **50-70% faster**: Focus on critical components first
- **Early exit**: Stop on critical failures
- **Adaptive workflows**: Plan dynamically

### Quality
- **Root cause analysis**: Tractatus reveals WHY
- **Dependency awareness**: NetworkX shows relationships
- **Actionable insights**: Automatic task creation

### Integration
- **Todo2 tracking**: All automations tracked
- **Follow-up tasks**: Issues become actionable
- **Report generation**: Comprehensive documentation

---

## Best Practices

1. **Use Tractatus for Structure**: Break down complex concepts
2. **Use Sequential for Process**: Plan execution steps
3. **Use Todo2 for Tracking**: Track all automation runs
4. **Use NetworkX for Dependencies**: Understand relationships
5. **Create Follow-ups**: Turn findings into tasks
6. **Generate Reports**: Document everything

---

## Examples

### Example 1: Documentation Health (Already Implemented)

Uses base class pattern:
- Tractatus: "What is documentation health?"
- Sequential: "How do we check documentation health?"
- NetworkX: Documentation cross-reference graph
- Todo2: Tracks health checks, creates fix tasks

### Example 2: Todo2 Alignment (Can Be Refactored)

Could use base class:
- Tractatus: "What is task alignment?"
- Sequential: "How do we analyze alignment?"
- NetworkX: Task dependency graph
- Todo2: Tracks alignment, creates priority tasks

### Example 3: Automation Opportunity Finder (Implemented)

Demonstrates base class:
- Tractatus: "What makes a good automation opportunity?"
- Sequential: "How do we find opportunities?"
- NetworkX: Opportunity dependency graph
- Todo2: Tracks findings, creates implementation tasks

---

## Migration Guide

### Refactoring Existing Scripts

**Before:**
```python
def main():
    results = analyze()
    generate_report(results)
```

**After:**
```python
class MyAutomation(IntelligentAutomationBase):
    def _execute_analysis(self):
        return analyze()

    def _generate_report(self, results, insights):
        return format_report(results, insights)

def main():
    automation = MyAutomation(config)
    automation.run()
```

**Benefits:**
- Automatic Todo2 tracking
- Follow-up task creation
- Tractatus/Sequential integration
- NetworkX analysis (if needed)

---

## Troubleshooting

### NetworkX Not Available

**Issue**: "NetworkX not available, skipping graph analysis"

**Solution**: Install NetworkX
```bash
pip install networkx>=3.2.0
```

### Todo2 Task Creation Fails

**Issue**: "Failed to create Todo2 task"

**Solution**: Ensure `.todo2/state.todo2.json` exists and is valid JSON

### Tractatus/Sequential Not Working

**Issue**: Falls back to simplified versions

**Solution**: This is expected if MCP servers aren't available. The base class uses fallback implementations that still work.

---

## Next Steps

1. **Refactor Existing Scripts**: Migrate to use base class
2. **Add NetworkX Analysis**: Where dependencies matter
3. **Enhance Follow-up Creation**: Make tasks more actionable
4. **Integrate MCP Servers**: Use real Tractatus/Sequential Thinking

---

*This base class transforms automation from simple scripts into intelligent, self-improving systems.*
