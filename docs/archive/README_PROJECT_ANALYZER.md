# Project Analyzer - NetworkX-based Project Analysis Tool

## Overview

The Project Analyzer uses NetworkX to create graph representations of your project's structure, helping you understand and improve:

1. **Todo2 Task Dependencies** - Visualize task relationships, find critical paths, identify bottlenecks
2. **Documentation Cross-References** - Map documentation structure, find central docs, identify isolated content
3. **Architecture Component Relationships** - Analyze code dependencies, understand module interactions

## Installation

NetworkX is included in `requirements-notebooks.txt`. To install:

```bash
pip install networkx>=3.2.0
```

Or install all requirements:

```bash
pip install -r requirements-notebooks.txt
```

## Usage

### Analyze Todo2 Tasks

```bash
python python/tools/project_analyzer.py --tasks
```

**Output includes:**

- Total tasks and dependencies
- Critical path (longest dependency chain)
- Bottlenecks (tasks with most dependents)
- Isolated tasks (no dependencies)
- Status distribution
- Tag distribution
- Cycle detection

### Analyze Documentation

```bash
python python/tools/project_analyzer.py --docs
```

**Output includes:**

- Total documents and cross-references
- Central documents (most referenced)
- Isolated documents (no references)
- Average references per document

### Analyze Architecture

```bash
python python/tools/project_analyzer.py --architecture
```

**Output includes:**

- Total source files
- Total dependencies (imports/includes)
- Average dependencies per file

### Run All Analyses

```bash
python python/tools/project_analyzer.py --all
```

## Example Output

### Todo2 Task Analysis

```
============================================================
TODO2 TASK ANALYSIS
============================================================

Todo2 Task Insights
============================================================

total_tasks: 137
total_dependencies: 32
critical_path: ['T-122', 'T-123', 'T-124', 'T-125', 'T-126']
critical_path_length: 5
bottlenecks:
  - ('T-123', 3)
  - ('T-124', 2)
isolated_tasks: ['T-127', 'T-128']
status_distribution:
  Done: 79
  In Progress: 22
  Todo: 33
tag_distribution:
  implementation: 45
  research: 23
  configuration: 18
is_dag: True
```

### Documentation Analysis

```
============================================================
DOCUMENTATION ANALYSIS
============================================================

Documentation Insights
============================================================

total_docs: 295
total_references: 1247
central_docs:
  - ('docs/API_DOCUMENTATION_INDEX.md', 45)
  - ('docs/CODEBASE_ARCHITECTURE.md', 32)
central_docs: 10
isolated_docs: ['docs/archive/OLD_DOC.md']
avg_references_per_doc: 4.23
```

## Use Cases

### 1. Project Planning

**Find Critical Path:**

- Identify the longest dependency chain
- Understand minimum time to complete project
- Focus on blocking tasks

**Identify Bottlenecks:**

- Find tasks that many others depend on
- Prioritize these tasks
- Consider breaking them down

### 2. Documentation Improvement

**Find Central Documents:**

- These are key reference points
- Keep them up-to-date
- Ensure they're comprehensive

**Find Isolated Documents:**

- May need better integration
- Consider adding cross-references
- Or consolidate with related docs

### 3. Architecture Understanding

**Understand Dependencies:**

- See which modules are most connected
- Identify tightly coupled components
- Plan refactoring efforts

## Integration with Project Workflow

### Regular Analysis

Run analysis weekly to track:

- Task completion progress
- Documentation growth
- Architecture evolution

### Before Major Refactoring

Use architecture analysis to:

- Understand current structure
- Identify refactoring targets
- Plan dependency changes

### Documentation Reviews

Use documentation analysis to:

- Find outdated central docs
- Identify missing cross-references
- Plan documentation improvements

## Advanced Usage

### Custom Paths

```bash
python python/tools/project_analyzer.py --tasks --todo2-path /custom/path/todo2.json
python python/tools/project_analyzer.py --docs --docs-dir /custom/path/docs
```

### Extending the Tool

The tool is modular - you can extend it by:

1. **Adding new analyzers** - Create new analyzer classes
2. **Custom metrics** - Add new insight calculations
3. **Visualization** - Export graphs for visualization (matplotlib, graphviz)
4. **Export formats** - Add JSON/CSV export for further analysis

## Examples

### Find Tasks Blocking Others

```python
from python.tools.project_analyzer import Todo2Analyzer
from pathlib import Path

analyzer = Todo2Analyzer(Path('.todo2/state.todo2.json'))
graph = analyzer.build_graph()
bottlenecks = analyzer.find_bottlenecks(5)

for task_id, count in bottlenecks:
    task = analyzer.tasks[task_id]
    print(f"{task_id}: {task['name']} ({count} dependents)")
```

### Find Most Referenced Documentation

```python
from python.tools.project_analyzer import DocumentationAnalyzer
from pathlib import Path

analyzer = DocumentationAnalyzer(Path('docs'))
graph = analyzer.build_graph()
central = analyzer.find_central_docs(10)

for doc, count in central:
    print(f"{doc}: {count} references")
```

## Future Enhancements

Potential improvements:

1. **Graph Visualization** - Export to GraphML, DOT, or PNG
2. **Temporal Analysis** - Track changes over time
3. **Predictive Metrics** - Estimate completion times
4. **Integration** - CI/CD integration for automated analysis
5. **Web Dashboard** - Interactive web interface
6. **Export Formats** - JSON, CSV, HTML reports

## Related Tools

- **NetworkX** - Graph analysis library
- **Todo2 MCP** - Task management system
- **Documentation** - See `docs/` directory

## References

- NetworkX Documentation: https://networkx.org/
- Todo2 Format: `.todo2/state.todo2.json`
- Project Documentation: `docs/`
