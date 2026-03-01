#!/usr/bin/env python3
"""
Test script for NATS client integration.

Tests connection, publishing, and subscription functionality.
"""
import asyncio
import sys
from pathlib import Path

# Add python directory to path for imports (conftest adds repo root when run via pytest)
_tests_dir = Path(__file__).resolve().parent
_python_dir = _tests_dir.parent
if str(_python_dir) not in sys.path:
    sys.path.insert(0, str(_python_dir))

from integration.nats_client import NATSClient
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


import pytest

@pytest.mark.asyncio
async def test_nats_client():
    """Test NATS client functionality."""
    client = NATSClient(url="nats://localhost:4222")

    # Test connection
    logger.info("Testing NATS connection...")
    connected = await client.connect()
    if not connected:
        logger.error("Failed to connect to NATS server")
        return False

    logger.info("✅ Connected to NATS")

    # Test publishing strategy signal
    logger.info("Testing strategy signal publishing...")
    signal_published = await client.publish_strategy_signal(
        symbol="SPX",
        price=4500.0,
        signal_type="opportunity"
    )
    if signal_published:
        logger.info("✅ Strategy signal published")
    else:
        logger.error("❌ Failed to publish strategy signal")

    # Test publishing strategy decision
    logger.info("Testing strategy decision publishing...")
    decision_published = await client.publish_strategy_decision(
        symbol="SPX",
        quantity=10,
        side="BUY",
        mark=4500.0,
        decision_type="trade"
    )
    if decision_published:
        logger.info("✅ Strategy decision published")
    else:
        logger.error("❌ Failed to publish strategy decision")

    # Test subscription (with timeout)
    logger.info("Testing market data subscription...")
    received_messages = []

    async def message_callback(data):
        received_messages.append(data)
        logger.info(f"Received market data: {data}")

    sub_id = await client.subscribe_market_data(message_callback)
    if sub_id:
        logger.info(f"✅ Subscribed to {sub_id}")
        # Wait a bit for any messages
        await asyncio.sleep(2)
    else:
        logger.error("❌ Failed to subscribe to market data")

    # Cleanup
    await client.disconnect()
    logger.info("✅ Disconnected from NATS")

    # Summary
    logger.info("\n📊 Test Summary:")
    logger.info(f"  Connection: {'✅' if connected else '❌'}")
    logger.info(f"  Signal Published: {'✅' if signal_published else '❌'}")
    logger.info(f"  Decision Published: {'✅' if decision_published else '❌'}")
    logger.info(f"  Subscription: {'✅' if sub_id else '❌'}")
    logger.info(f"  Messages Received: {len(received_messages)}")

    return connected and signal_published and decision_published and sub_id is not None


if __name__ == "__main__":
    try:
        result = asyncio.run(test_nats_client())
        sys.exit(0 if result else 1)
    except KeyboardInterrupt:
        logger.info("\nTest interrupted by user")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Test failed with error: {e}", exc_info=True)
        sys.exit(1)
