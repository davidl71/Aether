"""
websocket_manager.py - WebSocket Connection Manager

This module manages WebSocket connections and broadcasts events to all connected clients.
"""

import asyncio
import logging
from typing import Set, Dict, Any
from datetime import datetime, timezone

from fastapi import WebSocket

logger = logging.getLogger(__name__)


class WebSocketManager:
    """
    Manages WebSocket connections and broadcasts events.

    Thread-safe connection management with async broadcast support.
    """

    def __init__(self):
        """Initialize WebSocket manager."""
        self.active_connections: Set[WebSocket] = set()
        self._lock = asyncio.Lock()

    async def connect(self, websocket: WebSocket):
        """
        Accept and register a new WebSocket connection.

        Args:
            websocket: WebSocket connection to accept
        """
        await websocket.accept()
        async with self._lock:
            self.active_connections.add(websocket)
        logger.info(f"WebSocket client connected. Total connections: {len(self.active_connections)}")

    async def disconnect(self, websocket: WebSocket):
        """
        Remove a WebSocket connection.

        Args:
            websocket: WebSocket connection to remove
        """
        async with self._lock:
            self.active_connections.discard(websocket)
        logger.info(f"WebSocket client disconnected. Total connections: {len(self.active_connections)}")

    async def send_personal_message(self, message: Dict[str, Any], websocket: WebSocket):
        """
        Send a message to a specific WebSocket connection.

        Args:
            message: Message dictionary to send
            websocket: Target WebSocket connection
        """
        try:
            await websocket.send_json(message)
        except Exception as e:
            logger.error(f"Error sending personal message: {e}")
            await self.disconnect(websocket)

    async def broadcast(self, message: Dict[str, Any]):
        """
        Broadcast a message to all connected WebSocket clients.

        Args:
            message: Message dictionary to broadcast
        """
        if not self.active_connections:
            return

        # Create a copy of connections to avoid modification during iteration
        async with self._lock:
            connections = list(self.active_connections)

        # Send to all connections, removing failed ones
        disconnected = []
        for connection in connections:
            try:
                await connection.send_json(message)
            except Exception as e:
                logger.warning(f"Error broadcasting to client: {e}")
                disconnected.append(connection)

        # Remove disconnected clients
        if disconnected:
            async with self._lock:
                for conn in disconnected:
                    self.active_connections.discard(conn)
            logger.info(f"Removed {len(disconnected)} disconnected clients")

    async def broadcast_event(self, event_type: str, data: Dict[str, Any]):
        """
        Broadcast an event message with type and data.

        Args:
            event_type: Type of event (e.g., "order_filled", "position_updated")
            data: Event data dictionary
        """
        message = {
            "type": event_type,
            "data": data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        await self.broadcast(message)

    def get_connection_count(self) -> int:
        """Get the number of active connections."""
        return len(self.active_connections)


# Global WebSocket manager instance
websocket_manager = WebSocketManager()
