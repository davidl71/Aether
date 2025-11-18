"""
event_bridge.py - Event Bridge for LEAN to WebSocket

This module bridges LEAN algorithm events to WebSocket broadcasts.
It subscribes to LEAN callbacks and converts them to WebSocket messages.
"""

import asyncio
import logging
from typing import Optional, Dict, Any
from datetime import datetime, timezone
from threading import Thread

from .websocket_manager import websocket_manager
from .lean_client import LeanClient

logger = logging.getLogger(__name__)


class EventBridge:
    """
    Bridges LEAN algorithm events to WebSocket broadcasts.

    This class hooks into LEAN algorithm callbacks and converts
    events to WebSocket messages for real-time client updates.
    """

    def __init__(self, lean_client: LeanClient):
        """
        Initialize event bridge.

        Args:
            lean_client: LEAN client instance
        """
        self.lean_client = lean_client
        self._event_loop: Optional[asyncio.AbstractEventLoop] = None
        self._bridge_thread: Optional[Thread] = None
        self._running = False

    def start(self):
        """Start the event bridge in a separate thread."""
        if self._running:
            logger.warning("Event bridge already running")
            return

        self._running = True
        self._bridge_thread = Thread(target=self._run_event_loop, daemon=True)
        self._bridge_thread.start()
        logger.info("Event bridge started")

    def stop(self):
        """Stop the event bridge."""
        if not self._running:
            return

        self._running = False
        if self._event_loop:
            self._event_loop.call_soon_threadsafe(self._event_loop.stop)

        if self._bridge_thread:
            self._bridge_thread.join(timeout=5.0)

        logger.info("Event bridge stopped")

    def _run_event_loop(self):
        """Run asyncio event loop in bridge thread."""
        self._event_loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self._event_loop)
        try:
            self._event_loop.run_forever()
        finally:
            self._event_loop.close()

    def _schedule_broadcast(self, event_type: str, data: Dict[str, Any]):
        """
        Schedule a WebSocket broadcast from any thread.

        Args:
            event_type: Type of event
            data: Event data
        """
        if not self._event_loop:
            logger.warning("Event loop not initialized, cannot broadcast")
            return

        # Schedule broadcast in event loop
        asyncio.run_coroutine_threadsafe(
            websocket_manager.broadcast_event(event_type, data),
            self._event_loop
        )

    def on_order_filled(self, order_id: int, order_info: Dict[str, Any], fill_price: float):
        """
        Handle order filled event from LEAN.

        Args:
            order_id: Order ID
            order_info: Order information dictionary
            fill_price: Fill price
        """
        data = {
            "order_id": str(order_id),
            "status": "FILLED",
            "fill_price": float(fill_price),
            "symbol": order_info.get("symbol", "UNKNOWN"),
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("order_filled", data)
        logger.debug(f"Order filled event: {order_id}")

    def on_order_cancelled(self, order_id: int, order_info: Dict[str, Any]):
        """
        Handle order cancelled event from LEAN.

        Args:
            order_id: Order ID
            order_info: Order information dictionary
        """
        data = {
            "order_id": str(order_id),
            "status": "CANCELLED",
            "symbol": order_info.get("symbol", "UNKNOWN"),
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("order_cancelled", data)
        logger.debug(f"Order cancelled event: {order_id}")

    def on_position_updated(self, position_data: Dict[str, Any]):
        """
        Handle position updated event.

        Args:
            position_data: Position data dictionary
        """
        data = {
            "position": position_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("position_updated", data)
        logger.debug("Position updated event")

    def on_symbol_updated(self, symbol: str, market_data: Dict[str, Any]):
        """
        Handle symbol market data updated event.

        Args:
            symbol: Symbol name
            market_data: Market data dictionary
        """
        data = {
            "symbol": symbol,
            "market_data": market_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("symbol_updated", data)
        logger.debug(f"Symbol updated event: {symbol}")

    def on_alert(self, level: str, message: str):
        """
        Handle alert/notification event.

        Args:
            level: Alert level ("info", "warning", "error")
            message: Alert message
        """
        data = {
            "level": level,
            "message": message,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("alert", data)
        logger.debug(f"Alert event: {level} - {message}")

    def on_snapshot_update(self, snapshot_data: Dict[str, Any]):
        """
        Handle periodic snapshot update.

        Args:
            snapshot_data: Snapshot data dictionary
        """
        data = {
            "snapshot": snapshot_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self._schedule_broadcast("snapshot", data)
        logger.debug("Snapshot update event")


# Global event bridge instance (will be initialized when algorithm is set)
event_bridge: Optional[EventBridge] = None


def get_event_bridge(lean_client: LeanClient) -> EventBridge:
    """
    Get or create the global event bridge instance.

    Args:
        lean_client: LEAN client instance

    Returns:
        EventBridge instance
    """
    global event_bridge
    if event_bridge is None:
        event_bridge = EventBridge(lean_client)
    return event_bridge
