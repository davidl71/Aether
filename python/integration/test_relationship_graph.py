"""
test_relationship_graph.py - Unit tests for relationship_graph module

Tests graph building, path finding, and ledger integration.
"""

import pytest
from pathlib import Path
import tempfile
import sqlite3
import json

from python.integration.relationship_graph import (
    RelationshipGraph,
    Relationship,
    RelationshipType,
    Position,
    build_relationship_graph,
    load_positions_from_ledger,
)


class TestRelationshipGraph:
    """Test RelationshipGraph class."""

    def test_add_position(self):
        """Test adding positions to graph."""
        graph = RelationshipGraph()
        position = Position(id="loan-1", type="loan", rate=0.05, currency="USD", amount=10000.0)
        graph.add_position(position)

        assert "loan-1" in graph.graph
        assert graph.graph.nodes["loan-1"]["type"] == "loan"
        assert graph.graph.nodes["loan-1"]["rate"] == 0.05

    def test_add_relationship(self):
        """Test adding relationships to graph."""
        graph = RelationshipGraph()
        graph.add_position(Position(id="loan-1", type="loan", rate=0.05))
        graph.add_position(Position(id="box-spread-1", type="box_spread", rate=0.04))

        relationship = Relationship(
            source_id="loan-1",
            target_id="box-spread-1",
            relationship_type=RelationshipType.MARGIN,
            benefit=100.0,  # $100 benefit
            weight=1.0,
        )
        graph.add_relationship(relationship)

        assert graph.graph.has_edge("loan-1", "box-spread-1")
        assert graph.graph["loan-1"]["box-spread-1"]["benefit"] == 100.0

    def test_add_relationship_missing_node(self):
        """Test that adding relationship with missing node raises error."""
        graph = RelationshipGraph()
        graph.add_position(Position(id="loan-1", type="loan"))

        relationship = Relationship(
            source_id="loan-1",
            target_id="missing-node",
            relationship_type=RelationshipType.MARGIN,
        )

        with pytest.raises(ValueError, match="Target position 'missing-node' not in graph"):
            graph.add_relationship(relationship)

    def test_find_optimal_chain_simple(self):
        """Test finding optimal chain in simple graph."""
        graph = RelationshipGraph()

        # Create chain: loan → margin → box-spread → fund → cheaper-loan
        positions = [
            Position(id="loan-5%", type="loan", rate=0.05),
            Position(id="box-spread", type="box_spread", rate=0.04),
            Position(id="fund", type="fund", rate=0.06),
            Position(id="loan-3%", type="loan", rate=0.03),
        ]

        for pos in positions:
            graph.add_position(pos)

        relationships = [
            Relationship(
                source_id="loan-5%",
                target_id="box-spread",
                relationship_type=RelationshipType.MARGIN,
                benefit=100.0,
                weight=1.0,
            ),
            Relationship(
                source_id="box-spread",
                target_id="fund",
                relationship_type=RelationshipType.FINANCING,
                benefit=200.0,
                weight=1.0,
            ),
            Relationship(
                source_id="fund",
                target_id="loan-3%",
                relationship_type=RelationshipType.FINANCING,
                benefit=300.0,
                weight=1.0,
            ),
        ]

        for rel in relationships:
            graph.add_relationship(rel)

        result = graph.find_optimal_chain("loan-5%", "loan-3%", max_length=5)

        assert result is not None
        assert result["path"] == ["loan-5%", "box-spread", "fund", "loan-3%"]
        assert result["benefit"] == 600.0  # 100 + 200 + 300
        assert result["path_count"] == 1

    def test_find_optimal_chain_multiple_paths(self):
        """Test finding optimal chain when multiple paths exist."""
        graph = RelationshipGraph()

        # Create two paths: A → B → D and A → C → D
        positions = [
            Position(id="A", type="loan"),
            Position(id="B", type="box_spread"),
            Position(id="C", type="box_spread"),
            Position(id="D", type="loan"),
        ]

        for pos in positions:
            graph.add_position(pos)

        relationships = [
            Relationship(
                source_id="A",
                target_id="B",
                relationship_type=RelationshipType.MARGIN,
                benefit=50.0,  # Lower benefit
                weight=1.0,
            ),
            Relationship(
                source_id="A",
                target_id="C",
                relationship_type=RelationshipType.MARGIN,
                benefit=150.0,  # Higher benefit
                weight=1.0,
            ),
            Relationship(
                source_id="B",
                target_id="D",
                relationship_type=RelationshipType.FINANCING,
                benefit=100.0,
                weight=1.0,
            ),
            Relationship(
                source_id="C",
                target_id="D",
                relationship_type=RelationshipType.FINANCING,
                benefit=100.0,
                weight=1.0,
            ),
        ]

        for rel in relationships:
            graph.add_relationship(rel)

        result = graph.find_optimal_chain("A", "D", max_length=3)

        assert result is not None
        # Should choose path with higher benefit: A → C → D (150 + 100 = 250)
        assert result["path"] == ["A", "C", "D"]
        assert result["benefit"] == 250.0
        assert result["path_count"] == 2  # Two paths exist

    def test_find_optimal_chain_no_path(self):
        """Test finding chain when no path exists."""
        graph = RelationshipGraph()
        graph.add_position(Position(id="A", type="loan"))
        graph.add_position(Position(id="B", type="loan"))

        result = graph.find_optimal_chain("A", "B", max_length=5)

        assert result is None

    def test_calculate_path_benefit(self):
        """Test calculating path benefit."""
        graph = RelationshipGraph()

        positions = [
            Position(id="A", type="loan"),
            Position(id="B", type="box_spread"),
            Position(id="C", type="fund"),
        ]

        for pos in positions:
            graph.add_position(pos)

        relationships = [
            Relationship(
                source_id="A",
                target_id="B",
                relationship_type=RelationshipType.MARGIN,
                benefit=100.0,
            ),
            Relationship(
                source_id="B",
                target_id="C",
                relationship_type=RelationshipType.FINANCING,
                benefit=200.0,
            ),
        ]

        for rel in relationships:
            graph.add_relationship(rel)

        path = ["A", "B", "C"]
        benefit = graph.calculate_path_benefit(path)

        assert benefit == 300.0  # 100 + 200

    def test_get_relationships(self):
        """Test getting relationships for a position."""
        graph = RelationshipGraph()

        positions = [
            Position(id="loan-1", type="loan"),
            Position(id="box-spread-1", type="box_spread"),
            Position(id="box-spread-2", type="box_spread"),
        ]

        for pos in positions:
            graph.add_position(pos)

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
        ]

        for rel in relationships:
            graph.add_relationship(rel)

        rels = graph.get_relationships("loan-1")
        assert len(rels) == 2

        # Filter by type
        margin_rels = graph.get_relationships("loan-1", RelationshipType.MARGIN)
        assert len(margin_rels) == 2

        financing_rels = graph.get_relationships("loan-1", RelationshipType.FINANCING)
        assert len(financing_rels) == 0


class TestBuildRelationshipGraph:
    """Test build_relationship_graph function."""

    def test_build_graph(self):
        """Test building graph from positions and relationships."""
        positions = [
            Position(id="loan-1", type="loan", rate=0.05),
            Position(id="box-spread-1", type="box_spread", rate=0.04),
        ]

        relationships = [
            Relationship(
                source_id="loan-1",
                target_id="box-spread-1",
                relationship_type=RelationshipType.MARGIN,
                benefit=100.0,
            ),
        ]

        graph = build_relationship_graph(positions, relationships)

        assert "loan-1" in graph.graph
        assert "box-spread-1" in graph.graph
        assert graph.graph.has_edge("loan-1", "box-spread-1")


class TestLoadPositionsFromLedger:
    """Test loading positions from ledger database."""

    def test_load_positions_empty_db(self):
        """Test loading from non-existent database."""
        positions = load_positions_from_ledger(Path("/nonexistent/db.db"))
        assert positions == []

    def test_load_positions_from_mock_db(self):
        """Test loading positions from mock ledger database."""
        # Create temporary database
        with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tmp:
            db_path = Path(tmp.name)

        try:
            # Create database schema
            conn = sqlite3.connect(str(db_path))
            cursor = conn.cursor()
            cursor.execute(
                """
                CREATE TABLE transactions (
                    id TEXT PRIMARY KEY,
                    transaction_json TEXT
                )
                """
            )

            # Insert test transaction
            transaction = {
                "id": "test-1",
                "date": "2025-01-01",
                "description": "Test transaction",
                "postings": [
                    {
                        "account": {"segments": ["Assets", "IBKR", "SPY"]},
                        "amount": {"value": "100.0", "currency": "USD"},
                    },
                    {
                        "account": {"segments": ["Assets", "Bank", "Checking"]},
                        "amount": {"value": "-10000.0", "currency": "USD"},
                    },
                ],
            }

            cursor.execute(
                "INSERT INTO transactions (id, transaction_json) VALUES (?, ?)",
                ("test-1", json.dumps(transaction)),
            )
            conn.commit()
            conn.close()

            # Load positions
            positions = load_positions_from_ledger(db_path)

            # Should find positions
            assert len(positions) > 0
            position_ids = [p.id for p in positions]
            assert any("spy" in pid.lower() for pid in position_ids)

        finally:
            # Cleanup
            if db_path.exists():
                db_path.unlink()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
