"""
Tests for TUI data models

These tests verify that the Python models match the TypeScript types
and can serialize/deserialize correctly.
"""

import json
import pytest
from datetime import datetime

from ..models import (
    Candle, OptionStrike, OptionSeries, SymbolSnapshot,
    PositionSnapshot, TimelineEvent, AccountMetrics, SnapshotPayload,
    Severity
)


def test_candle_serialization():
    """Test Candle serialization/deserialization"""
    candle = Candle(
        open=100.0,
        high=105.0,
        low=99.0,
        close=103.0,
        volume=1000.0,
        entry=100.0,
        updated="2024-01-01T12:00:00Z"
    )

    data = candle.to_dict()
    assert data["open"] == 100.0
    assert data["high"] == 105.0

    restored = Candle.from_dict(data)
    assert restored.open == candle.open
    assert restored.high == candle.high


def test_snapshot_payload_serialization():
    """Test SnapshotPayload serialization/deserialization"""
    snapshot = SnapshotPayload(
        generated_at="2024-01-01T12:00:00Z",
        mode="DRY-RUN",
        strategy="RUNNING",
        account_id="DU123456",
        metrics=AccountMetrics(
            net_liq=100000.0,
            portal_ok=True,
            tws_ok=True
        ),
        symbols=[
            SymbolSnapshot(
                symbol="SPX",
                last=4000.0,
                bid=3999.5,
                ask=4000.5,
                spread=1.0,
                roi=2.5
            )
        ],
        positions=[],
        historic=[],
        orders=[],
        alerts=[]
    )

    # Test JSON serialization
    json_str = snapshot.to_json()
    assert "SPX" in json_str
    assert "DRY-RUN" in json_str

    # Test deserialization
    restored = SnapshotPayload.from_json(json_str)
    assert restored.mode == snapshot.mode
    assert restored.account_id == snapshot.account_id
    assert len(restored.symbols) == 1
    assert restored.symbols[0].symbol == "SPX"


def test_timeline_event_severity():
    """Test TimelineEvent with different severity levels"""
    event = TimelineEvent(
        timestamp="2024-01-01T12:00:00Z",
        text="Test alert",
        severity=Severity.ERROR
    )

    data = event.to_dict()
    assert data["severity"] == "error"

    restored = TimelineEvent.from_dict(data)
    assert restored.severity == Severity.ERROR


def test_empty_snapshot():
    """Test empty snapshot creation"""
    snapshot = SnapshotPayload()
    assert snapshot.mode == "DRY-RUN"
    assert snapshot.strategy == "STOPPED"
    assert len(snapshot.symbols) == 0
    assert len(snapshot.positions) == 0


def test_symbol_snapshot_with_option_chains():
    """Test SymbolSnapshot with option chains"""
    symbol = SymbolSnapshot(
        symbol="SPX",
        last=4000.0,
        option_chains=[
            OptionSeries(
                expiration="2024-01-19",
                strikes=[
                    OptionStrike(
                        strike=4000.0,
                        call_bid=100.0,
                        call_ask=101.0,
                        put_bid=99.0,
                        put_ask=100.0
                    )
                ]
            )
        ]
    )

    data = symbol.to_dict()
    assert len(data["option_chains"]) == 1
    assert len(data["option_chains"][0]["strikes"]) == 1

    restored = SymbolSnapshot.from_dict(data)
    assert len(restored.option_chains) == 1
    assert restored.option_chains[0].expiration == "2024-01-19"
