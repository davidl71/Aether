"""
relationship_graph.py - NetworkX-based multi-instrument relationship modeling

Models financial instrument relationships as directed graphs and finds optimal
paths for opportunity simulation (e.g., loan → margin → box spread → fund → cheaper loan).

Based on design in docs/SYNTHETIC_FINANCING_ARCHITECTURE.md
"""

from __future__ import annotations

import os
import sqlite3
from collections import defaultdict
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Dict, List, Any, Optional

try:
    import networkx as nx
except ImportError:
    raise ImportError(
        "networkx is required. Install with: pip install networkx>=3.2.0"
    )


class RelationshipType(str, Enum):
    """Types of relationships between financial instruments."""

    COLLATERAL = "collateral"  # Asset can be used as collateral
    MARGIN = "margin"  # Asset can satisfy margin requirements
    FINANCING = "financing"  # Asset provides financing
    HEDGE = "hedge"  # Asset hedges another position
    ARBITRAGE = "arbitrage"  # Arbitrage relationship (legacy)
    CROSS_CURRENCY = "cross_currency"  # Cross-currency relationship
    REGULATORY = "regulatory"  # Regulatory relationship (haircuts, offsets)


@dataclass
class Relationship:
    """Represents a relationship between two financial instruments."""

    source_id: str  # Source instrument ID
    target_id: str  # Target instrument ID
    relationship_type: RelationshipType
    benefit: float = 0.0  # Benefit of this relationship (e.g., rate advantage in bps)
    weight: float = 1.0  # Edge weight for path finding (lower = better)
    collateral_ratio: Optional[float] = None  # 0.0-1.0 (haircut applied)
    rate_benefit: Optional[float] = None  # Rate advantage in basis points
    currency: Optional[str] = None
    constraints: Optional[Dict[str, Any]] = None  # Additional constraints


@dataclass
class Position:
    """Represents a financial position/instrument."""

    id: str
    type: str  # e.g., "loan", "box_spread", "bond", "bank_account"
    rate: Optional[float] = None  # Interest rate or financing rate
    currency: str = "USD"
    amount: Optional[float] = None
    metadata: Optional[Dict[str, Any]] = None


class RelationshipGraph:
    """Manages relationship graph using NetworkX."""

    def __init__(self):
        """Initialize empty directed graph."""
        self.graph = nx.DiGraph()

    def add_position(self, position: Position) -> None:
        """Add a position as a node in the graph.

        Args:
            position: Position to add as node
        """
        self.graph.add_node(
            position.id,
            type=position.type,
            rate=position.rate,
            currency=position.currency,
            amount=position.amount,
            **{f"meta_{k}": v for k, v in (position.metadata or {}).items()},
        )

    def add_relationship(self, relationship: Relationship) -> None:
        """Add a relationship as an edge in the graph.

        Args:
            relationship: Relationship to add as edge
        """
        # Ensure source and target nodes exist
        if relationship.source_id not in self.graph:
            raise ValueError(f"Source position '{relationship.source_id}' not in graph")
        if relationship.target_id not in self.graph:
            raise ValueError(f"Target position '{relationship.target_id}' not in graph")

        # Add edge with relationship attributes
        self.graph.add_edge(
            relationship.source_id,
            relationship.target_id,
            relationship_type=relationship.relationship_type.value,
            benefit=relationship.benefit,
            weight=relationship.weight,
            collateral_ratio=relationship.collateral_ratio,
            rate_benefit=relationship.rate_benefit,
            currency=relationship.currency,
            constraints=relationship.constraints or {},
        )

    def find_optimal_chain(
        self,
        start_id: str,
        end_id: str,
        max_length: int = 5,
        maximize_benefit: bool = True,
    ) -> Optional[Dict[str, Any]]:
        """Find optimal path from start to end instrument.

        Args:
            start_id: Starting instrument ID
            end_id: Ending instrument ID
            max_length: Maximum path length (number of edges)
            maximize_benefit: If True, maximize benefit; if False, minimize weight

        Returns:
            Dictionary with 'path', 'benefit', 'all_paths', or None if no path exists
        """
        if start_id not in self.graph:
            raise ValueError(f"Start position '{start_id}' not in graph")
        if end_id not in self.graph:
            raise ValueError(f"End position '{end_id}' not in graph")

        # Find all simple paths (no cycles)
        try:
            all_paths = list(
                nx.all_simple_paths(self.graph, start_id, end_id, cutoff=max_length)
            )
        except nx.NetworkXNoPath:
            return None

        if not all_paths:
            return None

        # Calculate benefit/weight for each path
        path_metrics = []
        for path in all_paths:
            if maximize_benefit:
                # Sum benefits along path (negative weight = benefit)
                total_benefit = sum(
                    self.graph[u][v].get("benefit", 0.0) for u, v in zip(path[:-1], path[1:], strict=False)
                )
                metric = total_benefit
            else:
                # Sum weights along path
                total_weight = sum(
                    self.graph[u][v].get("weight", 1.0) for u, v in zip(path[:-1], path[1:], strict=False)
                )
                metric = -total_weight  # Negative for sorting (higher is better)

            path_metrics.append((path, metric))

        # Sort by metric (descending)
        path_metrics.sort(key=lambda x: x[1], reverse=True)
        optimal_path, optimal_metric = path_metrics[0]

        # Calculate total benefit for optimal path
        total_benefit = sum(
            self.graph[u][v].get("benefit", 0.0) for u, v in zip(optimal_path[:-1], optimal_path[1:], strict=False)
        )

        return {
            "path": optimal_path,
            "benefit": total_benefit,
            "metric": optimal_metric,
            "all_paths": all_paths,
            "path_count": len(all_paths),
        }

    def calculate_path_benefit(self, path: List[str]) -> float:
        """Calculate total benefit of a relationship chain.

        Args:
            path: List of node IDs representing the path

        Returns:
            Total benefit along the path
        """
        if len(path) < 2:
            return 0.0

        total_benefit = 0.0
        for u, v in zip(path[:-1], path[1:], strict=False):
            if self.graph.has_edge(u, v):
                total_benefit += self.graph[u][v].get("benefit", 0.0)

        return total_benefit

    def get_relationships(
        self, position_id: str, relationship_type: Optional[RelationshipType] = None
    ) -> List[Dict[str, Any]]:
        """Get all relationships for a position.

        Args:
            position_id: Position ID
            relationship_type: Optional filter by relationship type

        Returns:
            List of relationship dictionaries
        """
        if position_id not in self.graph:
            return []

        relationships = []
        for target_id in self.graph.successors(position_id):
            edge_data = self.graph[position_id][target_id]
            if relationship_type is None or edge_data.get("relationship_type") == relationship_type.value:
                relationships.append(
                    {
                        "source_id": position_id,
                        "target_id": target_id,
                        **edge_data,
                    }
                )

        return relationships


def build_relationship_graph(
    positions: List[Position], relationships: List[Relationship]
) -> RelationshipGraph:
    """Build relationship graph from positions and relationships.

    Args:
        positions: List of positions to add as nodes
        relationships: List of relationships to add as edges

    Returns:
        RelationshipGraph instance
    """
    graph = RelationshipGraph()

    # Add all positions as nodes
    for position in positions:
        graph.add_position(position)

    # Add all relationships as edges
    for relationship in relationships:
        graph.add_relationship(relationship)

    return graph


def load_positions_from_ledger(
    db_path: Optional[Path] = None,
) -> List[Position]:
    """Load positions from ledger database.

    Args:
        db_path: Path to ledger database (defaults to standard locations)

    Returns:
        List of Position objects
    """
    if db_path is None:
        db_path = _get_ledger_database_path()

    if db_path is None or not db_path.exists():
        return []

    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        cursor.execute(
            """
            SELECT id, transaction_json
            FROM transactions
            ORDER BY id DESC
            """
        )

        positions = []
        position_balances: Dict[str, Dict[str, float]] = defaultdict(lambda: defaultdict(float))
        position_currencies: Dict[str, str] = {}

        for row in cursor.fetchall():
            import json

            transaction_json = row["transaction_json"]
            if not transaction_json:
                continue

            try:
                transaction = json.loads(transaction_json)
                postings = transaction.get("postings", [])

                for posting in postings:
                    account_path_str = posting.get("account", "")
                    if isinstance(account_path_str, dict):
                        segments_list = account_path_str.get("segments", [])
                        if segments_list:
                            account_path_str = ":".join(segments_list)

                    # Extract position info from account path
                    # Format: Assets:Broker:Symbol or Assets:Bank:Account
                    if account_path_str and account_path_str.startswith("Assets:"):
                        parts = account_path_str.split(":")
                        if len(parts) >= 3:
                            broker_or_type = parts[1].lower()
                            symbol_or_account = parts[2]

                            # Determine position type
                            if broker_or_type in ["ibkr", "alpaca", "tastytrade", "tradestation"]:
                                pass
                            elif broker_or_type == "bank":
                                pass
                            else:
                                pass

                            # Get amount
                            amount_data = posting.get("amount", {})
                            if isinstance(amount_data, dict):
                                amount = float(amount_data.get("value", 0.0))
                                currency = amount_data.get("currency", "USD")
                            else:
                                amount = float(amount_data) if amount_data else 0.0
                                currency = "USD"

                            # Accumulate balance
                            position_balances[symbol_or_account][currency] += amount
                            position_currencies[symbol_or_account] = currency

            except (json.JSONDecodeError, KeyError, ValueError):
                # Skip invalid transactions
                continue

        # Create Position objects from accumulated balances
        for symbol, balances in position_balances.items():
            for currency, amount in balances.items():
                if abs(amount) > 0.001:  # Only non-zero positions
                    position_id = f"{symbol}-{currency}".lower()
                    positions.append(
                        Position(
                            id=position_id,
                            type=position_currencies.get(symbol, "position"),
                            currency=currency,
                            amount=amount,
                            metadata={"symbol": symbol, "broker": "ledger"},
                        )
                    )

        conn.close()
        return positions

    except sqlite3.Error:
        return []


def _get_ledger_database_path() -> Optional[Path]:
    """Get ledger database path from environment or default location."""
    db_path = os.getenv("LEDGER_DATABASE_PATH")
    if db_path:
        return Path(db_path).expanduser()

    # Try default locations
    root_dir = Path(__file__).parent.parent.parent
    default_paths = [
        root_dir / "ledger.db",
        root_dir / "agents" / "backend" / "ledger.db",
        Path.home() / ".ledger" / "ledger.db",
    ]

    for path in default_paths:
        if path.exists():
            return path

    return None
