"""NATS bridge: publish NautilusTrader events to the existing NATS topic schema.

Replicates the Rust encode_envelope pattern from
agents/backend/crates/nats_adapter/src/serde.rs so the Rust backend and TUI
decode messages without modification.

Topic names are taken verbatim from
agents/backend/crates/nats_adapter/src/topics.rs.
"""

from __future__ import annotations

import asyncio
import time
import uuid
from typing import Any

import nats
import structlog
from google.protobuf import timestamp_pb2
from google.protobuf.message import Message as ProtoMessage

from nautilus_agent.generated import messages_pb2 as pb

log = structlog.get_logger(__name__)


class NatsBridge:
    """Async NATS publisher that wraps proto messages in NatsEnvelope."""

    def __init__(self, nats_url: str, source_id: str = "nautilus-ib") -> None:
        self._url = nats_url
        self._source_id = source_id
        self._nc: nats.aio.client.Client | None = None

    async def connect(self) -> None:
        self._nc = await nats.connect(self._url)
        log.info("nats_bridge.connected", url=self._url)

    async def disconnect(self) -> None:
        if self._nc:
            await self._nc.drain()
            log.info("nats_bridge.disconnected")

    # ------------------------------------------------------------------
    # Public publish helpers
    # ------------------------------------------------------------------

    async def publish_market_data(self, symbol: str, event: pb.MarketDataEvent) -> None:
        """market-data.tick.{symbol}"""
        topic = f"market-data.tick.{symbol}"
        await self._wrap_and_publish(topic, "MarketDataEvent", event)

    async def publish_strategy_signal(self, symbol: str, signal: pb.StrategySignal) -> None:
        """strategy.signal.{symbol}"""
        topic = f"strategy.signal.{symbol}"
        await self._wrap_and_publish(topic, "StrategySignal", signal)

    async def publish_strategy_decision(self, symbol: str, decision: pb.StrategyDecision) -> None:
        """strategy.decision.{symbol}"""
        topic = f"strategy.decision.{symbol}"
        await self._wrap_and_publish(topic, "StrategyDecision", decision)

    async def publish_order_fill(self, order_id: str, execution: pb.BoxSpreadExecution) -> None:
        """orders.fill.{order_id}"""
        topic = f"orders.fill.{order_id}"
        await self._wrap_and_publish(topic, "BoxSpreadExecution", execution)

    async def publish_position_update(self, symbol: str, position: pb.Position) -> None:
        """positions.update.{symbol}"""
        topic = f"positions.update.{symbol}"
        await self._wrap_and_publish(topic, "Position", position)

    async def publish_health(self, health: pb.BackendHealth) -> None:
        """system.health"""
        await self._wrap_and_publish("system.health", "BackendHealth", health)

    # ------------------------------------------------------------------
    # Core envelope wrapper — mirrors Rust encode_envelope in serde.rs
    # ------------------------------------------------------------------

    async def _wrap_and_publish(
        self, topic: str, message_type: str, proto_msg: ProtoMessage
    ) -> None:
        if self._nc is None:
            log.warning("nats_bridge.not_connected", topic=topic)
            return

        now = time.time()
        ts = timestamp_pb2.Timestamp(
            seconds=int(now),
            nanos=int((now % 1) * 1_000_000_000),
        )
        envelope = pb.NatsEnvelope(
            id=str(uuid.uuid4()),
            timestamp=ts,
            source=self._source_id,
            message_type=message_type,
            payload=proto_msg.SerializeToString(),
        )
        raw = envelope.SerializeToString()
        try:
            await self._nc.publish(topic, raw)
        except Exception:
            log.exception("nats_bridge.publish_failed", topic=topic, message_type=message_type)

    # ------------------------------------------------------------------
    # Fire-and-forget helper for use inside NT strategy callbacks
    # (NT runs a single asyncio loop — never create a new event loop here)
    # ------------------------------------------------------------------

    def schedule(self, coro: Any) -> None:
        """Schedule a coroutine on the running event loop without blocking."""
        asyncio.create_task(coro)
