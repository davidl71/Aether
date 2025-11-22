# NATS Integration Guide - Python

**Date**: 2025-11-20
**Status**: Planning/Implementation Guide
**Library**: `nats-py` (asyncio NATS client)

## Overview

This guide covers integrating NATS message queue into Python strategy runners for subscribing to market data and publishing strategy signals/decisions.

## Prerequisites

### Installation

```bash
pip install nats-py
```

### Requirements

```python
# requirements.txt
nats-py>=2.6.0
```

## Connection Management

### Async Connection

```python
import asyncio
import nats
from nats.aio.client import Client as NATS

async def connect_nats(url: str = "nats://localhost:4222"):
    """Connect to NATS server with automatic reconnection"""
    nc = NATS()

    try:
        await nc.connect(
            servers=[url],
            reconnect_time_wait=2,  # 2 seconds
            max_reconnect_attempts=-1,  # Unlimited
            allow_reconnect=True
        )
        print(f"Connected to NATS at {url}")
        return nc
    except Exception as e:
        print(f"Failed to connect to NATS: {e}")
        return None
```

## Publishing Messages

### Strategy Signal Publishing

```python
import json
import uuid
from datetime import datetime, timezone

async def publish_strategy_signal(nc: NATS, symbol: str, price: float):
    """Publish strategy signal to NATS"""
    topic = f"strategy.signal.{symbol}"

    message = {
        "id": str(uuid.uuid4()),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "source": "python-strategy",
        "type": "StrategySignal",
        "payload": {
            "symbol": symbol,
            "price": price,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    }

    json_str = json.dumps(message)

    try:
        await nc.publish(topic, json_str.encode())
        print(f"Published signal for {symbol} at {price}")
    except Exception as e:
        print(f"Failed to publish signal: {e}")
        # Send to DLQ
        await send_to_dlq(nc, topic, message, "publish_error")
```

### Strategy Decision Publishing

```python
async def publish_strategy_decision(nc: NATS,
                                    symbol: str,
                                    quantity: int,
                                    side: str,
                                    mark: float):
    """Publish strategy decision to NATS"""
    topic = f"strategy.decision.{symbol}"

    message = {
        "id": str(uuid.uuid4()),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "source": "python-strategy",
        "type": "StrategyDecision",
        "payload": {
            "symbol": symbol,
            "quantity": quantity,
            "side": side,
            "mark": mark,
            "strategy_name": "box_spread_arbitrage"
        }
    }

    json_str = json.dumps(message)

    try:
        await nc.publish(topic, json_str.encode())
    except Exception as e:
        print(f"Failed to publish decision: {e}")
        await send_to_dlq(nc, topic, message, "publish_error")
```

## Subscribing to Messages

### Subscribe to Market Data

```python
async def subscribe_market_data(nc: NATS, symbol: str, callback):
    """Subscribe to market data for a symbol"""
    topic = f"market-data.tick.{symbol}"

    async def message_handler(msg):
        try:
            data = json.loads(msg.data.decode())

            if data.get("type") == "MarketDataTick":
                payload = data["payload"]
                await callback(
                    symbol=payload["symbol"],
                    bid=payload.get("bid"),
                    ask=payload.get("ask"),
                    price=payload.get("price")
                )
        except Exception as e:
            print(f"Error processing market data: {e}")

    await nc.subscribe(topic, cb=message_handler)
    print(f"Subscribed to {topic}")

# Usage
async def on_market_data(symbol: str, bid: float, ask: float, price: float):
    """Process market data update"""
    print(f"{symbol}: bid={bid}, ask={ask}, last={price}")
    # Evaluate strategy
    decision = evaluate_strategy(symbol, bid, ask, price)
    if decision:
        await publish_strategy_decision(nc, symbol, decision.quantity,
                                       decision.side, decision.mark)
```

### Wildcard Subscriptions

```python
async def subscribe_all_signals(nc: NATS):
    """Subscribe to all strategy signals"""
    topic = "strategy.signal.>"

    async def message_handler(msg):
        try:
            data = json.loads(msg.data.decode())
            payload = data["payload"]
            print(f"Signal: {payload['symbol']} @ {payload['price']}")
        except Exception as e:
            print(f"Error processing signal: {e}")

    await nc.subscribe(topic, cb=message_handler)
```

## Topic Constants

### Module: `nats_topics.py`

```python
"""NATS topic constants and helpers"""

class MarketDataTopics:
    @staticmethod
    def tick(symbol: str) -> str:
        return f"market-data.tick.{symbol}"

    @staticmethod
    def candle(symbol: str) -> str:
        return f"market-data.candle.{symbol}"

    @staticmethod
    def all() -> str:
        return "market-data.>"

class StrategyTopics:
    @staticmethod
    def signal(symbol: str) -> str:
        return f"strategy.signal.{symbol}"

    @staticmethod
    def decision(symbol: str) -> str:
        return f"strategy.decision.{symbol}"

    @staticmethod
    def all_signals() -> str:
        return "strategy.signal.>"

    @staticmethod
    def all_decisions() -> str:
        return "strategy.decision.>"

class DLQTopics:
    @staticmethod
    def dead_letter(component: str, error_type: str) -> str:
        return f"system.dlq.{component}.{error_type}"
```

## Error Handling with DLQ

```python
async def send_to_dlq(nc: NATS,
                     original_topic: str,
                     message: dict,
                     error_type: str,
                     retry_count: int = 0):
    """Send failed message to dead letter queue"""
    dlq_topic = DLQTopics.dead_letter("python-strategy", error_type)

    dlq_message = {
        "id": str(uuid.uuid4()),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "original_topic": original_topic,
        "component": "python-strategy",
        "error_type": error_type,
        "error_message": f"Failed to publish to {original_topic}",
        "retry_count": retry_count,
        "original_payload": message.get("payload", {})
    }

    try:
        await nc.publish(dlq_topic, json.dumps(dlq_message).encode())
        print(f"Message sent to DLQ: {dlq_topic}")
    except Exception as e:
        print(f"Failed to send to DLQ: {e}")

async def publish_with_retry(nc: NATS,
                             topic: str,
                             message: dict,
                             max_retries: int = 3):
    """Publish message with retry logic"""
    json_str = json.dumps(message)
    delay_ms = 100

    for attempt in range(max_retries + 1):
        try:
            await nc.publish(topic, json_str.encode())
            return True
        except Exception as e:
            if attempt < max_retries:
                await asyncio.sleep(delay_ms / 1000.0)
                delay_ms *= 2  # Exponential backoff
            else:
                await send_to_dlq(nc, topic, message, "publish_error", attempt)
                return False

    return False
```

## Complete Example

```python
import asyncio
import json
import uuid
from datetime import datetime, timezone
from nats.aio.client import Client as NATS

async def main():
    # Connect to NATS
    nc = NATS()
    await nc.connect("nats://localhost:4222")

    # Subscribe to market data
    async def on_tick(msg):
        data = json.loads(msg.data.decode())
        if data["type"] == "MarketDataTick":
            payload = data["payload"]
            print(f"Tick: {payload['symbol']} @ {payload.get('price')}")

    await nc.subscribe("market-data.tick.>", cb=on_tick)

    # Publish strategy signal
    signal = {
        "id": str(uuid.uuid4()),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "source": "python-strategy",
        "type": "StrategySignal",
        "payload": {
            "symbol": "SPY",
            "price": 509.18,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    }

    await nc.publish("strategy.signal.SPY", json.dumps(signal).encode())

    # Keep running
    await asyncio.sleep(60)

    await nc.close()

if __name__ == "__main__":
    asyncio.run(main())
```

## Configuration

### Environment Variables

```python
import os

NATS_URL = os.getenv("NATS_URL", "nats://localhost:4222")
NATS_ENABLED = os.getenv("NATS_ENABLED", "true").lower() == "true"
NATS_DLQ_ENABLED = os.getenv("NATS_DLQ_ENABLED", "true").lower() == "true"
```

## Best Practices

1. **Async/Await**: Use async functions for all NATS operations
2. **Error Handling**: Always wrap publish/subscribe in try/except
3. **Message Validation**: Validate JSON structure before processing
4. **Topic Constants**: Use topic helper functions
5. **DLQ Integration**: Implement retry logic and DLQ publishing
6. **Connection Management**: Reuse connection, handle reconnection

## References

- [NATS Python Client Documentation](https://github.com/nats-io/nats.py)
- [NATS Topics Registry](../NATS_TOPICS_REGISTRY.md)
- [Message Schemas](../message_schemas/README.md)
