"""
dxlink_client.py - DXLink WebSocket client for Tastytrade real-time market data streaming

DXLink is used by Tastytrade for streaming market data via WebSocket.
Requires a quote token from Tastytrade API to authenticate.

Usage:
    client = DXLinkClient(tastytrade_client, sandbox=False)
    await client.connect()
    await client.subscribe(["SPY", "QQQ"])
    client.on_quote(lambda quote: print(f"{quote['symbol']}: {quote['bid']}/{quote['ask']}"))
"""

from __future__ import annotations

import asyncio
import json
import logging
from typing import Dict, List, Optional, Callable, Any
from datetime import datetime

try:
    import websockets
    from websockets.client import WebSocketClientProtocol

    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False
    WebSocketClientProtocol = None

from .tastytrade_client import TastytradeClient, TastytradeError

logger = logging.getLogger(__name__)


class DXLinkError(RuntimeError):
    """Generic error raised for DXLink WebSocket failures."""


class DXLinkClient:
    """DXLink WebSocket client for Tastytrade real-time market data streaming."""

    def __init__(
        self,
        tastytrade_client: TastytradeClient,
        sandbox: Optional[bool] = None,
        websocket_url: Optional[str] = None,
    ) -> None:
        """
        Initialize DXLink client.

        Args:
            tastytrade_client: Authenticated TastytradeClient instance
            sandbox: Whether to use sandbox endpoint (defaults to client's sandbox mode)
            websocket_url: Override WebSocket URL (optional)
        """
        if not WEBSOCKETS_AVAILABLE:
            raise RuntimeError(
                "websockets library not installed. Install with: pip install websockets"
            )

        self.tastytrade_client = tastytrade_client
        self.sandbox = sandbox if sandbox is not None else tastytrade_client.sandbox

        # DXLink WebSocket endpoints
        if websocket_url:
            self.websocket_url = websocket_url
        elif self.sandbox:
            self.websocket_url = "wss://streamer.cert.tastyworks.com"
        else:
            self.websocket_url = "wss://streamer.tastytrade.com"

        self.quote_token: Optional[str] = None
        self.websocket: Optional[WebSocketClientProtocol] = None
        self.connected = False
        self.subscribed_symbols: set[str] = set()
        self.quote_callbacks: List[Callable[[Dict[str, Any]], None]] = []
        self._receive_task: Optional[asyncio.Task] = None
        self._reconnect_delay = 1.0
        self._max_reconnect_delay = 60.0

    async def get_quote_token(self) -> str:
        """
        Get quote token from Tastytrade API for DXLink authentication.

        Returns:
            Quote token string

        Raises:
            DXLinkError: If token retrieval fails
        """
        try:
            # Tastytrade API endpoint for quote token
            endpoint = f"{self.tastytrade_client.base_url}/api-quote-token"
            data = self.tastytrade_client._post(endpoint)

            # Extract token from response
            if "data" in data:
                token_data = data["data"]
            else:
                token_data = data

            token = token_data.get("token") or token_data.get("quote_token")
            if not token:
                raise DXLinkError("No quote token in API response")

            self.quote_token = token
            logger.info("DXLink quote token obtained successfully")
            return token
        except TastytradeError as e:
            raise DXLinkError(f"Failed to get quote token: {e}") from e

    async def connect(self) -> None:
        """
        Connect to DXLink WebSocket and authenticate.

        Raises:
            DXLinkError: If connection fails
        """
        if self.connected:
            logger.warning("Already connected to DXLink")
            return

        # Get quote token if not already obtained
        if not self.quote_token:
            await self.get_quote_token()

        try:
            # Connect to DXLink WebSocket
            # DXLink typically uses token in connection URL or initial message
            ws_url = f"{self.websocket_url}?token={self.quote_token}"
            logger.info(f"Connecting to DXLink WebSocket: {self.websocket_url}")

            self.websocket = await websockets.connect(
                ws_url,
                ping_interval=20,
                ping_timeout=10,
                close_timeout=10,
            )

            self.connected = True
            self._reconnect_delay = (
                1.0  # Reset reconnect delay on successful connection
            )

            # Start receiving messages
            self._receive_task = asyncio.create_task(self._receive_loop())

            logger.info("DXLink WebSocket connected successfully")
        except Exception as e:
            self.connected = False
            raise DXLinkError(f"Failed to connect to DXLink: {e}") from e

    async def disconnect(self) -> None:
        """Disconnect from DXLink WebSocket."""
        if self._receive_task:
            self._receive_task.cancel()
            try:
                await self._receive_task
            except asyncio.CancelledError:
                pass
            self._receive_task = None

        if self.websocket:
            try:
                await self.websocket.close()
            except Exception as e:
                logger.warning(f"Error closing WebSocket: {e}")
            self.websocket = None

        self.connected = False
        logger.info("DXLink WebSocket disconnected")

    async def subscribe(self, symbols: List[str]) -> None:
        """
        Subscribe to symbols for real-time quotes.

        Args:
            symbols: List of symbols to subscribe to (e.g., ["SPY", "QQQ"])

        Raises:
            DXLinkError: If subscription fails
        """
        if not self.connected or not self.websocket:
            raise DXLinkError("Not connected to DXLink. Call connect() first.")

        try:
            # DXLink subscription message format (may vary)
            # Common format: {"type": "subscribe", "symbols": [...]}
            subscribe_msg = {
                "type": "subscribe",
                "symbols": symbols,
            }

            await self.websocket.send(json.dumps(subscribe_msg))
            self.subscribed_symbols.update(symbols)
            logger.info(f"Subscribed to symbols: {symbols}")
        except Exception as e:
            raise DXLinkError(f"Failed to subscribe to symbols: {e}") from e

    async def unsubscribe(self, symbols: List[str]) -> None:
        """
        Unsubscribe from symbols.

        Args:
            symbols: List of symbols to unsubscribe from
        """
        if not self.connected or not self.websocket:
            logger.warning("Not connected to DXLink. Cannot unsubscribe.")
            return

        try:
            unsubscribe_msg = {
                "type": "unsubscribe",
                "symbols": symbols,
            }

            await self.websocket.send(json.dumps(unsubscribe_msg))
            self.subscribed_symbols.difference_update(symbols)
            logger.info(f"Unsubscribed from symbols: {symbols}")
        except Exception as e:
            logger.warning(f"Failed to unsubscribe from symbols: {e}")

    def on_quote(self, callback: Callable[[Dict[str, Any]], None]) -> None:
        """
        Register callback for quote updates.

        Args:
            callback: Function that receives quote dict with symbol, bid, ask, last, etc.
        """
        self.quote_callbacks.append(callback)

    async def _receive_loop(self) -> None:
        """Receive and process messages from DXLink WebSocket."""
        if not self.websocket:
            return

        try:
            async for message in self.websocket:
                try:
                    data = json.loads(message)
                    await self._handle_message(data)
                except json.JSONDecodeError:
                    logger.warning(f"Invalid JSON message from DXLink: {message[:100]}")
                except Exception as e:
                    logger.error(f"Error handling DXLink message: {e}")

        except websockets.exceptions.ConnectionClosed:
            logger.warning("DXLink WebSocket connection closed")
            self.connected = False
            # Trigger reconnection
            asyncio.create_task(self._reconnect())
        except Exception as e:
            logger.error(f"Error in DXLink receive loop: {e}")
            self.connected = False
            # Trigger reconnection
            asyncio.create_task(self._reconnect())

    async def _handle_message(self, data: Dict[str, Any]) -> None:
        """Handle incoming message from DXLink."""
        msg_type = data.get("type") or data.get("event")

        if msg_type == "quote" or "bid" in data or "ask" in data:
            # Quote update message
            quote = self._parse_quote(data)
            if quote:
                # Call all registered callbacks
                for callback in self.quote_callbacks:
                    try:
                        callback(quote)
                    except Exception as e:
                        logger.error(f"Error in quote callback: {e}")
        elif msg_type == "heartbeat" or msg_type == "ping":
            # Heartbeat/ping message - respond if needed
            if msg_type == "ping":
                await self._send_pong()
        elif msg_type == "error":
            logger.error(f"DXLink error message: {data}")
        else:
            logger.debug(f"Unhandled DXLink message type: {msg_type}")

    def _parse_quote(self, data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Parse quote data from DXLink message.

        Args:
            data: Raw message data

        Returns:
            Parsed quote dict or None if invalid
        """
        try:
            # DXLink quote format may vary - handle common patterns
            symbol = data.get("symbol") or data.get("eventSymbol") or data.get("s")
            bid = float(data.get("bid", 0.0) or data.get("bidPrice", 0.0))
            ask = float(data.get("ask", 0.0) or data.get("askPrice", 0.0))
            last = float(data.get("last", 0.0) or data.get("lastPrice", 0.0))
            volume = int(data.get("volume", 0) or data.get("vol", 0))

            if not symbol:
                return None

            return {
                "symbol": symbol,
                "bid": bid,
                "ask": ask,
                "last": last,
                "spread": ask - bid if ask > 0 and bid > 0 else 0.0,
                "volume": volume,
                "timestamp": datetime.now().isoformat(),
            }
        except (ValueError, TypeError, KeyError) as e:
            logger.warning(f"Failed to parse quote data: {e}")
            return None

    async def _send_pong(self) -> None:
        """Send pong response to ping."""
        if self.websocket and self.connected:
            try:
                pong_msg = {"type": "pong"}
                await self.websocket.send(json.dumps(pong_msg))
            except Exception as e:
                logger.warning(f"Failed to send pong: {e}")

    async def _reconnect(self) -> None:
        """Reconnect to DXLink with exponential backoff."""
        if self.connected:
            return

        while not self.connected:
            try:
                logger.info(
                    f"Attempting to reconnect to DXLink (delay: {self._reconnect_delay}s)"
                )
                await asyncio.sleep(self._reconnect_delay)

                # Reset quote token (may have expired)
                self.quote_token = None

                # Reconnect
                await self.connect()

                # Resubscribe to symbols
                if self.subscribed_symbols:
                    await self.subscribe(list(self.subscribed_symbols))

                # Reset reconnect delay on success
                self._reconnect_delay = 1.0
                logger.info("DXLink reconnected successfully")
                break

            except Exception as e:
                logger.warning(f"Reconnection attempt failed: {e}")
                # Exponential backoff
                self._reconnect_delay = min(
                    self._reconnect_delay * 2, self._max_reconnect_delay
                )
