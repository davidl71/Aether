"""
nats_client.py - NATS message queue client wrapper for Python strategy runner

Provides async NATS connection, subscription, and publishing capabilities
for market data and strategy signals/decisions.
"""
import json
import logging
import uuid
from datetime import datetime, timezone
from typing import Optional, Callable, Dict, Any

logger = logging.getLogger(__name__)

# Optional NATS import (graceful degradation)
try:
    import nats
    from nats.aio.client import Client as NATS
    NATS_AVAILABLE = True
except ImportError:
    NATS = None
    NATS_AVAILABLE = False
    logger.warning("nats-py not available - NATS integration disabled")


class NATSClient:
    """
    NATS client wrapper for Python strategy runner.

    Handles connection, subscription to market data, and publishing
    of strategy signals and decisions.
    """

    def __init__(self, url: str = "nats://localhost:4222"):
        """
        Initialize NATS client.

        Args:
            url: NATS server URL
        """
        self.url = url
        self.nc: Optional[NATS] = None
        self._connected = False
        self._subscriptions: Dict[str, Any] = {}

    async def connect(self) -> bool:
        """
        Connect to NATS server.

        Returns:
            True if connected, False otherwise
        """
        if not NATS_AVAILABLE:
            logger.warning("NATS not available - skipping connection")
            return False

        try:
            self.nc = NATS()
            await self.nc.connect(
                servers=[self.url],
                reconnect_time_wait=2,  # 2 seconds
                max_reconnect_attempts=-1,  # Unlimited
                allow_reconnect=True
            )
            self._connected = True
            logger.info(f"Connected to NATS at {self.url}")
            return True
        except Exception as e:
            logger.error(f"Failed to connect to NATS: {e}")
            self._connected = False
            return False

    async def disconnect(self):
        """Disconnect from NATS server."""
        if self.nc and self._connected:
            try:
                await self.nc.close()
                self._connected = False
                logger.info("Disconnected from NATS")
            except Exception as e:
                logger.error(f"Error disconnecting from NATS: {e}")

    def is_connected(self) -> bool:
        """Check if connected to NATS."""
        return self._connected and self.nc is not None

    async def subscribe_market_data(
        self,
        callback: Callable[[Dict[str, Any]], None],
        symbol: Optional[str] = None
    ) -> Optional[str]:
        """
        Subscribe to market data topics.

        Args:
            callback: Function to call when market data received
            symbol: Specific symbol to subscribe to, or None for all

        Returns:
            Subscription ID or None if failed
        """
        if not self.is_connected():
            logger.warning("Not connected to NATS - cannot subscribe")
            return None

        # Topic: market-data.tick.{symbol} or market-data.> for all
        if symbol:
            topic = f"market-data.tick.{symbol}"
        else:
            topic = "market-data.>"

        try:
            async def message_handler(msg):
                try:
                    data = json.loads(msg.data.decode())
                    # Extract payload from NATS message format
                    payload = data.get("payload", data)
                    callback(payload)
                except Exception as e:
                    logger.error(f"Error processing market data message: {e}")

            sub = await self.nc.subscribe(topic, cb=message_handler)
            self._subscriptions[topic] = sub
            logger.info(f"Subscribed to {topic}")
            return topic
        except Exception as e:
            logger.error(f"Failed to subscribe to {topic}: {e}")
            return None

    async def publish_strategy_signal(
        self,
        symbol: str,
        price: float,
        signal_type: str = "opportunity"
    ) -> bool:
        """
        Publish strategy signal to NATS.

        Args:
            symbol: Symbol for the signal
            price: Current price
            signal_type: Type of signal (opportunity, alert, etc.)

        Returns:
            True if published, False otherwise
        """
        if not self.is_connected():
            return False

        topic = f"strategy.signal.{symbol}"

        message = {
            "id": str(uuid.uuid4()),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "source": "python-strategy",
            "type": "StrategySignal",
            "payload": {
                "symbol": symbol,
                "price": price,
                "signal_type": signal_type,
                "timestamp": datetime.now(timezone.utc).isoformat()
            }
        }

        try:
            json_str = json.dumps(message)
            await self.nc.publish(topic, json_str.encode())
            logger.debug(f"Published strategy signal for {symbol} at {price}")
            return True
        except Exception as e:
            logger.error(f"Failed to publish strategy signal: {e}")
            return False

    async def publish_strategy_decision(
        self,
        symbol: str,
        quantity: int,
        side: str,
        mark: float,
        decision_type: str = "trade"
    ) -> bool:
        """
        Publish strategy decision to NATS.

        Args:
            symbol: Symbol for the decision
            quantity: Trade quantity
            side: Trade side (BUY/SELL)
            mark: Mark price
            decision_type: Type of decision (trade, cancel, etc.)

        Returns:
            True if published, False otherwise
        """
        if not self.is_connected():
            return False

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
                "decision_type": decision_type,
                "timestamp": datetime.now(timezone.utc).isoformat()
            }
        }

        try:
            json_str = json.dumps(message)
            await self.nc.publish(topic, json_str.encode())
            logger.debug(f"Published strategy decision for {symbol}: {side} {quantity} @ {mark}")
            return True
        except Exception as e:
            logger.error(f"Failed to publish strategy decision: {e}")
            return False

    async def unsubscribe(self, topic: str):
        """Unsubscribe from a topic."""
        if topic in self._subscriptions:
            try:
                await self._subscriptions[topic].unsubscribe()
                del self._subscriptions[topic]
                logger.info(f"Unsubscribed from {topic}")
            except Exception as e:
                logger.error(f"Error unsubscribing from {topic}: {e}")

    async def unsubscribe_all(self):
        """Unsubscribe from all topics."""
        for topic in list(self._subscriptions.keys()):
            await self.unsubscribe(topic)
