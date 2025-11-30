# NetworkX Integration Task Details

**Task ID**: T-138
**Status**: ✅ Complete
**Priority**: High

## Task Description

**Name**: Integrate NetworkX for multi-instrument relationship modeling and optimal path finding

## Detailed Description

🎯 **Objective:** Integrate NetworkX Python library to model multi-instrument relationships as graphs and find optimal paths for opportunity simulation (e.g., loan → margin → box spread → fund → cheaper loan).

📋 **Acceptance Criteria:**

- NetworkX added to Python requirements (`requirements.txt` or `requirements-notebooks.txt`)
- Relationship graph builder module created (`python/integration/relationship_graph.py`)
- Functions to build graph from positions and relationships
- Optimal path finding function (finds best chain from start to end instrument)
- Integration with existing ledger system to load positions
- Integration with synthetic financing architecture design
- Unit tests for graph building and path finding
- Example usage documented

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** NetworkX installation, graph building, path finding, basic integration
- **Excluded:** UI visualization (separate task), advanced optimization (NLopt integration), full opportunity simulation engine
- **Clarification Required:** Should this be in main requirements or notebooks-only?

🔧 **Technical Requirements:**

- Python 3.11+ (compatible with existing setup)
- NetworkX library (latest stable version)
- Integration with ledger system for position data
- Follow existing Python module patterns in `python/integration/`
- Use directed graphs (DiGraph) for relationship modeling
- Support relationship attributes (rate_benefit, collateral_ratio, etc.)

📁 **Files/Components:**

- Create: `python/integration/relationship_graph.py` (new module)
- Update: `requirements.txt` or `requirements-notebooks.txt` (add networkx)
- Create: `python/tests/test_relationship_graph.py` (unit tests)
- Reference: `docs/SYNTHETIC_FINANCING_ARCHITECTURE.md` (relationship design)
- Reference: `docs/LIBRARIES_FOR_PRIMARY_GOALS.md` (usage examples)

🧪 **Testing Requirements:**

- Test graph building from positions
- Test optimal path finding
- Test with multiple relationship chains
- Test edge cases (no path, multiple paths, cycles)
- Verify path benefit calculations

⚠️ **Edge Cases:**

- No path exists between instruments
- Multiple optimal paths (tie-breaking)
- Circular relationships (cycles)
- Missing position data
- Invalid relationship data

📚 **Dependencies:** None (can start immediately)

## Implementation Notes

### Key Functions to Implement

1. `build_relationship_graph(positions, relationships) -> nx.DiGraph`
   - Builds directed graph from positions and relationships
   - Nodes = instruments (loans, box spreads, bonds, etc.)
   - Edges = relationships (collateral, financing, etc.)

2. `find_optimal_chain(graph, start_id, end_id, max_length=5) -> dict`
   - Finds optimal path from start to end instrument
   - Calculates total benefit of path
   - Returns path, benefit, and all alternative paths

3. `calculate_path_benefit(graph, path) -> float`
   - Calculates total benefit of a relationship chain
   - Sums edge benefits along path

### Integration Points

- **Ledger System**: Load positions from ledger
- **Synthetic Financing Architecture**: Use relationship types from design doc
- **Opportunity Simulation**: Use graph paths for what-if scenarios

### Example Usage

```python
from python.integration.relationship_graph import (
    build_relationship_graph,
    RelationshipGraph,
    Position,
    Relationship,
    RelationshipType,
)

# Create positions

positions = [
    Position(id="bank-loan-5%", type="loan", rate=0.05, currency="USD"),
    Position(id="box-spread-4%", type="box_spread", rate=0.04, currency="USD"),
    Position(id="cheaper-loan-3%", type="loan", rate=0.03, currency="USD"),
]

# Define relationships

relationships = [
    Relationship(
        source_id="bank-loan-5%",
        target_id="box-spread-4%",
        relationship_type=RelationshipType.MARGIN,
        benefit=100.0,
        weight=1.0,
    ),
    Relationship(
        source_id="box-spread-4%",
        target_id="cheaper-loan-3%",
        relationship_type=RelationshipType.FINANCING,
        benefit=200.0,
        weight=1.0,
    ),
]

# Build graph

graph = build_relationship_graph(positions, relationships)

# Find optimal chain: loan → margin → box spread → cheaper loan

result = graph.find_optimal_chain("bank-loan-5%", "cheaper-loan-3%", max_length=5)

if result:
    print(f"Optimal path: {' → '.join(result['path'])}")
    print(f"Total benefit: ${result['benefit']:.2f}")
```

See `python/integration/relationship_graph_example.py` for complete examples.

## References

- `docs/LIBRARIES_FOR_PRIMARY_GOALS.md` - NetworkX usage examples
- `docs/SYNTHETIC_FINANCING_ARCHITECTURE.md` - Relationship architecture design
- NetworkX Documentation: https://networkx.org/
- NetworkX GitHub: https://github.com/networkx/networkx
