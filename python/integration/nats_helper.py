"""NATS client helpers for async publish/subscribe in sync strategy code."""

from __future__ import annotations

import asyncio
import logging
from typing import Optional

logger = logging.getLogger(__name__)

try:
    from .nats_client import NATSClient
    NATS_AVAILABLE = True
except ImportError:
    NATSClient = None
    NATS_AVAILABLE = False


def create_nats_client(url: str) -> Optional["NATSClient"]:
    """Create a NATS client if the library is available, else return None."""
    if not NATS_AVAILABLE or NATSClient is None:
        logger.info("NATS integration not available")
        return None
    try:
        client = NATSClient(url=url)
        logger.info("NATS client initialized (will connect on start)")
        return client
    except Exception as e:
        logger.warning("Failed to initialize NATS client: %s", e)
        return None


async def connect(client: Optional["NATSClient"]) -> None:
    """Connect to NATS asynchronously."""
    if client is None:
        return
    try:
        connected = await client.connect()
        if connected:
            logger.info("NATS connected - will publish signals/decisions")
        else:
            logger.warning("NATS connection failed - continuing without NATS")
    except Exception as e:
        logger.warning("Failed to connect to NATS: %s", e)


async def disconnect(client: Optional["NATSClient"]) -> None:
    """Disconnect from NATS asynchronously."""
    if client is None:
        return
    try:
        await client.disconnect()
    except Exception as e:
        logger.warning("Error disconnecting from NATS: %s", e)


def fire_and_forget(coro) -> None:
    """Schedule an async coroutine without blocking, handling event loop state."""
    try:
        loop = asyncio.get_event_loop()
        if loop.is_running():
            asyncio.create_task(coro)
        else:
            asyncio.run(coro)
    except Exception as e:
        logger.debug("Failed to dispatch async NATS call: %s", e)


def publish_signal(
    client: Optional["NATSClient"],
    symbol: str,
    price: float,
    signal_type: str,
) -> None:
    """Publish a strategy signal to NATS (fire-and-forget)."""
    if client is None or not client.is_connected():
        return
    fire_and_forget(
        client.publish_strategy_signal(
            symbol=symbol, price=price, signal_type=signal_type,
        )
    )
