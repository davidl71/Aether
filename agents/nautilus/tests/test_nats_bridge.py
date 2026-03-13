"""Tests for NatsBridge — verify NatsEnvelope wire format matches Rust expectations.

The Rust backend decodes messages with decode_envelope() in serde.rs.
We verify:
  - Correct topic names (from topics.rs)
  - NatsEnvelope fields populated correctly
  - Inner payload roundtrips to the original proto message
"""

from __future__ import annotations

import sys
import os
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch
import time

import pytest

# Allow importing generated stubs even if not yet generated (skip gracefully)
GENERATED_DIR = Path(__file__).parents[1] / "src" / "nautilus_agent" / "generated"
sys.path.insert(0, str(Path(__file__).parents[1] / "src"))

try:
    from nautilus_agent.generated import messages_pb2 as pb
    HAS_PROTO = True
except ImportError:
    HAS_PROTO = False

pytestmark = pytest.mark.skipif(
    not HAS_PROTO,
    reason="Protobuf stubs not generated yet — run: just proto-gen-nautilus",
)


@pytest.fixture
def bridge():
    """NatsBridge with mocked NATS connection."""
    from nautilus_agent.nats_bridge import NatsBridge
    b = NatsBridge("nats://localhost:4222", source_id="nautilus-ib")
    b._nc = AsyncMock()
    return b


@pytest.mark.asyncio
async def test_publish_market_data_topic(bridge):
    """Market data published to correct topic: market-data.tick.{symbol}"""
    event = pb.MarketDataEvent(symbol="SPX", bid=5100.0, ask=5101.0)
    await bridge.publish_market_data("SPX", event)

    bridge._nc.publish.assert_awaited_once()
    topic = bridge._nc.publish.call_args[0][0]
    assert topic == "market-data.tick.SPX"


@pytest.mark.asyncio
async def test_market_data_envelope_roundtrip(bridge):
    """NatsEnvelope fields and inner payload decode correctly."""
    event = pb.MarketDataEvent(
        symbol="SPX", bid=5100.25, ask=5101.75, last=5100.50, volume=1000
    )
    await bridge.publish_market_data("SPX", event)

    raw_bytes = bridge._nc.publish.call_args[0][1]
    envelope = pb.NatsEnvelope()
    envelope.ParseFromString(raw_bytes)

    assert envelope.source == "nautilus-ib"
    assert envelope.message_type == "MarketDataEvent"
    assert envelope.id != ""
    assert envelope.timestamp.seconds > 0

    inner = pb.MarketDataEvent()
    inner.ParseFromString(envelope.payload)
    assert inner.symbol == "SPX"
    assert abs(inner.bid - 5100.25) < 1e-9
    assert abs(inner.ask - 5101.75) < 1e-9
    assert inner.volume == 1000


@pytest.mark.asyncio
async def test_publish_strategy_decision_topic(bridge):
    """Strategy decision published to correct topic: strategy.decision.{symbol}"""
    decision = pb.StrategyDecision(symbol="XSP", quantity=1, side="BUY", mark=100.0)
    await bridge.publish_strategy_decision("XSP", decision)

    topic = bridge._nc.publish.call_args[0][0]
    assert topic == "strategy.decision.XSP"


@pytest.mark.asyncio
async def test_publish_order_fill_topic(bridge):
    """Order fill published to correct topic: orders.fill.{order_id}"""
    execution = pb.BoxSpreadExecution(
        symbol="SPX", lower_strike=5000, upper_strike=5100, expiry="20241220", net_debit=99.50
    )
    await bridge.publish_order_fill("ORD-123", execution)

    topic = bridge._nc.publish.call_args[0][0]
    assert topic == "orders.fill.ORD-123"


@pytest.mark.asyncio
async def test_publish_position_update_topic(bridge):
    """Position update published to correct topic: positions.update.{symbol}"""
    position = pb.Position(id="pos-1", symbol="SPX", quantity=1)
    await bridge.publish_position_update("SPX", position)

    topic = bridge._nc.publish.call_args[0][0]
    assert topic == "positions.update.SPX"


@pytest.mark.asyncio
async def test_nats_not_connected_does_not_raise(bridge):
    """Publishing when not connected logs warning and does not raise."""
    bridge._nc = None
    event = pb.MarketDataEvent(symbol="SPX", bid=5100.0, ask=5101.0)
    # Should not raise
    await bridge.publish_market_data("SPX", event)


@pytest.mark.asyncio
async def test_envelope_uuid_unique(bridge):
    """Each publish produces a unique envelope ID."""
    event = pb.MarketDataEvent(symbol="SPX", bid=5100.0, ask=5101.0)
    await bridge.publish_market_data("SPX", event)
    await bridge.publish_market_data("SPX", event)

    calls = bridge._nc.publish.call_args_list
    assert len(calls) == 2

    ids = []
    for call in calls:
        raw = call[0][1]
        env = pb.NatsEnvelope()
        env.ParseFromString(raw)
        ids.append(env.id)

    assert ids[0] != ids[1], "Envelope IDs should be unique per publish"
