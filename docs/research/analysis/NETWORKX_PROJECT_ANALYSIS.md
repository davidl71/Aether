# NetworkX Project Analysis Tool

## Overview

We've created a comprehensive NetworkX-based analysis tool that helps you understand and improve your project by analyzing:

1. **Todo2 Task Dependencies** - Task relationships, critical paths, bottlenecks
2. **Documentation Cross-References** - Documentation structure and connectivity
3. **Architecture Component Relationships** - Code dependencies and module interactions

## What Was Created

### 1. Project Analyzer Tool (`python/tools/project_analyzer.py`)

A comprehensive Python tool that uses NetworkX to analyze:

- **Todo2Analyzer**: Analyzes task dependencies, finds critical paths, identifies bottlenecks
- **DocumentationAnalyzer**: Maps documentation cross-references, finds central docs
- **ArchitectureAnalyzer**: Analyzes code dependencies (imports/includes)

### 2. Documentation (`python/tools/README_PROJECT_ANALYZER.md`)

Complete usage guide with:
- Installation instructions
- Usage examples
- Output interpretation
- Use cases
- Extension guidelines

### 3. Requirements Update

Added `networkx>=3.2.0` to `requirements.in` for graph analysis capabilities.

## Key Features

### Todo2 Task Analysis

**Insights Provided:**
- Total tasks and dependencies
- Critical path (longest dependency chain)
- Bottlenecks (tasks blocking others)
- Isolated tasks (no dependencies)
- Status distribution (Done, In Progress, Todo)
- Tag distribution
- Cycle detection (circular dependencies)

**Use Cases:**
- Identify blocking tasks
- Find critical path for project completion
- Understand task relationships
- Plan task prioritization

### Documentation Analysis

**Insights Provided:**
- Total documents and cross-references
- Central documents (most referenced)
- Isolated documents (no references)
- Average references per document

**Use Cases:**
- Find key reference documents
- Identify documentation gaps
- Improve documentation structure
- Plan documentation improvements

### Architecture Analysis

**Insights Provided:**
- Total source files
- Total dependencies (imports/includes)
- Average dependencies per file

**Use Cases:**
- Understand code structure
- Identify tightly coupled components
- Plan refactoring efforts
- Track architecture evolution

## Usage

### Basic Usage

```bash
# Analyze Todo2 tasks
python python/tools/project_analyzer.py --tasks

# Analyze documentation
python python/tools/project_analyzer.py --docs

# Analyze architecture
python python/tools/project_analyzer.py --architecture

# Run all analyses
python python/tools/project_analyzer.py --all
```

### Installation

NetworkX is included in `requirements-notebooks.txt`. Install with:

```bash
pip install networkx>=3.2.0
```

Or install all requirements:

```bash
pip install -r requirements-notebooks.txt
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
  - ('T-123', 3)  # Task T-123 blocks 3 other tasks
  - ('T-124', 2)  # Task T-124 blocks 2 other tasks
isolated_tasks: ['T-127', 'T-128']  # Tasks with no dependencies
status_distribution:
  Done: 79
  In Progress: 22
  Todo: 33
tag_distribution:
  implementation: 45
  research: 23
  configuration: 18
is_dag: True  # No circular dependencies
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
  - ('docs/API_DOCUMENTATION_INDEX.md', 45)  # Referenced 45 times
  - ('docs/CODEBASE_ARCHITECTURE.md', 32)    # Referenced 32 times
isolated_docs: ['docs/archive/OLD_DOC.md']  # No references
avg_references_per_doc: 4.23
```

## How It Helps Improve the Project

### 1. Task Management

**Identify Blocking Tasks:**
- Find tasks that many others depend on
- Prioritize these tasks to unblock others
- Consider breaking them down if too large

**Find Critical Path:**
- Understand minimum time to complete project
- Focus resources on critical path tasks
- Identify opportunities to parallelize work

**Detect Issues:**
- Find isolated tasks (may be forgotten)
- Detect circular dependencies (planning errors)
- Understand task distribution (status, tags)

### 2. Documentation Quality

**Find Key Documents:**
- Central documents are important reference points
- Keep them up-to-date and comprehensive
- Ensure they're well-maintained

**Identify Gaps:**
- Isolated documents may need better integration
- Add cross-references to improve navigation
- Consolidate related documentation

**Improve Structure:**
- Understand documentation connectivity
- Plan documentation improvements
- Ensure comprehensive coverage

### 3. Architecture Understanding

**Understand Dependencies:**
- See which modules are most connected
- Identify tightly coupled components
- Plan refactoring efforts

**Track Evolution:**
- Run analysis regularly to track changes
- Understand architecture growth
- Plan for scalability

## Integration with Existing Tools

### Todo2 MCP

The analyzer reads from `.todo2/state.todo2.json`, the same format used by Todo2 MCP. This means:
- Analysis stays in sync with task management
- No additional data format needed
- Works with existing Todo2 workflow

### Documentation System

The analyzer scans all markdown files in `docs/` directory, matching your existing documentation structure. It:
- Works with existing documentation
- No changes needed to documentation format
- Automatically detects cross-references

### Codebase Structure

The analyzer understands your multi-language codebase:
- C++ includes (`#include`)
- Python imports (`import`, `from`)
- TypeScript/JavaScript imports (`import`)
- Rust imports (future support)

## Future Enhancements

Potential improvements:

1. **Graph Visualization**
   - Export to GraphML, DOT, or PNG
   - Interactive visualizations
   - Web dashboard

2. **Temporal Analysis**
   - Track changes over time
   - Historical trends
   - Progress metrics

3. **Predictive Metrics**
   - Estimate completion times
   - Risk assessment
   - Resource planning

4. **CI/CD Integration**
   - Automated analysis on commits
   - Report generation
   - Trend tracking

5. **Export Formats**
   - JSON for programmatic access
   - CSV for spreadsheet analysis
   - HTML reports

## Related Documentation

- **Tool README**: `python/tools/README_PROJECT_ANALYZER.md`
- **NetworkX Integration**: `docs/NETWORKX_INTEGRATION_TASK.md`
- **Todo2 Format**: `docs/TASKS_MD_ANALYSIS.md`
- **Architecture**: `docs/CODEBASE_ARCHITECTURE.md`

## References

- **NetworkX Documentation**: https://networkx.org/
- **NetworkX Tutorial**: https://networkx.org/documentation/latest/tutorial/
- **Graph Theory**: https://en.wikipedia.org/wiki/Graph_theory

## Summary

The NetworkX Project Analysis Tool provides powerful insights into your project's structure, helping you:

✅ **Understand** task dependencies and relationships
✅ **Identify** bottlenecks and critical paths
✅ **Improve** documentation structure and connectivity
✅ **Analyze** code architecture and dependencies
✅ **Plan** refactoring and improvements
✅ **Track** project evolution over time

This tool complements your existing workflow and provides actionable insights to help you learn and improve the project.
