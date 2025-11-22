#!/usr/bin/env python3
"""
relationship_graph_example.py - Example usage of relationship_graph module

Demonstrates how to build relationship graphs and find optimal paths for
opportunity simulation (e.g., loan → margin → box spread → fund → cheaper loan).
"""

from python.integration.relationship_graph import (
    RelationshipGraph,
    Position,
    Relationship,
    RelationshipType,
    build_relationship_graph,
    load_positions_from_ledger,
)


def example_basic_usage():
    """Basic example: Build graph and find optimal chain."""
    print("=" * 80)
    print("Example 1: Basic Relationship Graph")
    print("=" * 80)

    # Create positions
    positions = [
        Position(id="bank-loan-5%", type="loan", rate=0.05, currency="USD", amount=10000.0),
        Position(id="box-spread-4%", type="box_spread", rate=0.04, currency="USD"),
        Position(id="fund-6%", type="fund", rate=0.06, currency="USD"),
        Position(id="cheaper-loan-3%", type="loan", rate=0.03, currency="USD"),
    ]

    # Define relationships
    relationships = [
        Relationship(
            source_id="bank-loan-5%",
            target_id="box-spread-4%",
            relationship_type=RelationshipType.MARGIN,
            benefit=100.0,  # $100 benefit from using loan as margin
            weight=1.0,
            rate_benefit=100,  # 100 bps advantage
        ),
        Relationship(
            source_id="box-spread-4%",
            target_id="fund-6%",
            relationship_type=RelationshipType.FINANCING,
            benefit=200.0,  # $200 benefit
            weight=1.0,
            rate_benefit=200,  # 200 bps advantage
        ),
        Relationship(
            source_id="fund-6%",
            target_id="cheaper-loan-3%",
            relationship_type=RelationshipType.FINANCING,
            benefit=300.0,  # $300 benefit from accessing cheaper loan
            weight=1.0,
            rate_benefit=200,  # 200 bps advantage (6% - 3% = 3% = 300 bps, but shown as 200)
        ),
    ]

    # Build graph
    graph = build_relationship_graph(positions, relationships)

    # Find optimal chain
    result = graph.find_optimal_chain("bank-loan-5%", "cheaper-loan-3%", max_length=5)

    if result:
        print(f"\n✅ Optimal path found:")
        print(f"   Path: {' → '.join(result['path'])}")
        print(f"   Total benefit: ${result['benefit']:.2f}")
        print(f"   Path count: {result['path_count']}")
    else:
        print("\n❌ No path found")


def example_multiple_paths():
    """Example with multiple paths to compare."""
    print("\n" + "=" * 80)
    print("Example 2: Multiple Paths Comparison")
    print("=" * 80)

    graph = RelationshipGraph()

    # Create positions
    positions = [
        Position(id="loan-A", type="loan", rate=0.05),
        Position(id="box-spread-B", type="box_spread", rate=0.04),
        Position(id="box-spread-C", type="box_spread", rate=0.045),
        Position(id="fund-D", type="fund", rate=0.06),
    ]

    for pos in positions:
        graph.add_position(pos)

    # Create two paths: A → B → D and A → C → D
    relationships = [
        Relationship(
            source_id="loan-A",
            target_id="box-spread-B",
            relationship_type=RelationshipType.MARGIN,
            benefit=50.0,  # Lower benefit path
            weight=1.0,
        ),
        Relationship(
            source_id="loan-A",
            target_id="box-spread-C",
            relationship_type=RelationshipType.MARGIN,
            benefit=150.0,  # Higher benefit path
            weight=1.0,
        ),
        Relationship(
            source_id="box-spread-B",
            target_id="fund-D",
            relationship_type=RelationshipType.FINANCING,
            benefit=100.0,
            weight=1.0,
        ),
        Relationship(
            source_id="box-spread-C",
            target_id="fund-D",
            relationship_type=RelationshipType.FINANCING,
            benefit=100.0,
            weight=1.0,
        ),
    ]

    for rel in relationships:
        graph.add_relationship(rel)

    # Find optimal chain
    result = graph.find_optimal_chain("loan-A", "fund-D", max_length=3)

    if result:
        print(f"\n✅ Optimal path found:")
        print(f"   Path: {' → '.join(result['path'])}")
        print(f"   Total benefit: ${result['benefit']:.2f}")
        print(f"   All paths: {result['path_count']}")
        print(f"\n   All possible paths:")
        for i, path in enumerate(result["all_paths"], 1):
            benefit = graph.calculate_path_benefit(path)
            print(f"   {i}. {' → '.join(path)} (benefit: ${benefit:.2f})")


def example_ledger_integration():
    """Example loading positions from ledger."""
    print("\n" + "=" * 80)
    print("Example 3: Ledger Integration")
    print("=" * 80)

    # Load positions from ledger
    positions = load_positions_from_ledger()

    if positions:
        print(f"\n✅ Loaded {len(positions)} positions from ledger:")
        for pos in positions[:5]:  # Show first 5
            print(f"   - {pos.id}: {pos.type} ({pos.currency})")
        if len(positions) > 5:
            print(f"   ... and {len(positions) - 5} more")
    else:
        print("\n⚠️  No positions found in ledger (database may not exist or be empty)")
        print("   This is expected if ledger.db doesn't exist yet.")


def example_relationship_queries():
    """Example querying relationships."""
    print("\n" + "=" * 80)
    print("Example 4: Relationship Queries")
    print("=" * 80)

    graph = RelationshipGraph()

    # Create positions
    positions = [
        Position(id="loan-1", type="loan", rate=0.05),
        Position(id="box-spread-1", type="box_spread", rate=0.04),
        Position(id="box-spread-2", type="box_spread", rate=0.045),
        Position(id="bond-1", type="bond", rate=0.03),
    ]

    for pos in positions:
        graph.add_position(pos)

    # Add relationships
    relationships = [
        Relationship(
            source_id="loan-1",
            target_id="box-spread-1",
            relationship_type=RelationshipType.MARGIN,
            benefit=100.0,
        ),
        Relationship(
            source_id="loan-1",
            target_id="box-spread-2",
            relationship_type=RelationshipType.MARGIN,
            benefit=150.0,
        ),
        Relationship(
            source_id="loan-1",
            target_id="bond-1",
            relationship_type=RelationshipType.COLLATERAL,
            benefit=50.0,
        ),
    ]

    for rel in relationships:
        graph.add_relationship(rel)

    # Query relationships
    print("\n📊 All relationships from 'loan-1':")
    all_rels = graph.get_relationships("loan-1")
    for rel in all_rels:
        print(f"   {rel['source_id']} → {rel['target_id']}: {rel['relationship_type']} (benefit: ${rel['benefit']:.2f})")

    print("\n📊 Margin relationships only:")
    margin_rels = graph.get_relationships("loan-1", RelationshipType.MARGIN)
    for rel in margin_rels:
        print(f"   {rel['source_id']} → {rel['target_id']}: {rel['relationship_type']} (benefit: ${rel['benefit']:.2f})")


if __name__ == "__main__":
    try:
        example_basic_usage()
        example_multiple_paths()
        example_ledger_integration()
        example_relationship_queries()

        print("\n" + "=" * 80)
        print("✅ All examples completed successfully!")
        print("=" * 80)
    except ImportError as e:
        print(f"\n❌ Import error: {e}")
        print("   Install NetworkX with: pip install networkx>=3.2.0")
        print("   Or install all notebook requirements: pip install -r requirements-notebooks.txt")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback

        traceback.print_exc()
