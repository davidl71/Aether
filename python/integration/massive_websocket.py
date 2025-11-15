"""
massive_websocket.py - WebSocket client for Massive.com real-time quotes
Provides real-time quote streaming for cross-validation with TWS.
"""
import logging
import json
import asyncio
from typing import Dict, List, Optional, Callable, Any
from datetime import datetime
import websockets
from websockets.client import WebSocketClientProtocol

logger = logging.getLogger(__name__)


class MassiveWebSocketClient:
    """
    WebSocket client for Massive.com real-time quotes.
    Provides continuous market updates for quote cross-validation.

    Reference: https://massive.com/docs (WebSocket API documentation)
    """

    def __init__(
        self,
        api_key: str,
        websocket_url: str = "wss://api.massive.com/ws",
    ):
        """
        Initialize Massive.com WebSocket client.

        Args:
            api_key: Massive.com API key
            websocket_url: WebSocket URL
        """
        self.api_key = api_key
        self.websocket_url = websocket_url
        self.ws: Optional[WebSocketClientProtocol] = None
        self.connected = False
        self.subscribed_symbols: List[str] = []
        self.quote_callbacks: List[Callable[[Dict], None]] = []
        self._running = False
        self._reconnect_delay = 5  # seconds

    async def connect(self) -> bool:
        """
        Connect to Massive.com WebSocket.

        Returns:
            True if connected successfully
        """
        try:
            # Build connection URL with API key
            url = f"{self.websocket_url}?apiKey={self.api_key}"

            logger.info(f"Connecting to Massive.com WebSocket: {self.websocket_url}")
            self.ws = await websockets.connect(url)
            self.connected = True
            self._running = True

            logger.info("Connected to Massive.com WebSocket")
            return True

        except Exception as e:
            logger.error(f"Failed to connect to Massive.com WebSocket: {e}")
            self.connected = False
            return False

    async def disconnect(self):
        """Disconnect from WebSocket."""
        self._running = False
        self.connected = False

        if self.ws:
            try:
                await self.ws.close()
                logger.info("Disconnected from Massive.com WebSocket")
            except Exception as e:
                logger.error(f"Error disconnecting: {e}")
            finally:
                self.ws = None

    async def subscribe_quotes(self, symbols: List[str]) -> bool:
        """
        Subscribe to real-time quotes for symbols.

        Args:
            symbols: List of stock tickers

        Returns:
            True if subscription successful
        """
        if not self.connected or not self.ws:
            logger.error("Not connected to WebSocket")
            return False

        try:
            # Build subscription message
            # Note: Actual message format depends on Massive.com WebSocket API
            # This is a placeholder - adjust based on actual API documentation
            message = {
                "action": "subscribe",
                "type": "quotes",
                "symbols": symbols
            }

            await self.ws.send(json.dumps(message))
            self.subscribed_symbols.extend(symbols)

            logger.info(f"Subscribed to quotes for {len(symbols)} symbols: {symbols}")
            return True

        except Exception as e:
            logger.error(f"Failed to subscribe to quotes: {e}")
            return False

    async def unsubscribe_quotes(self, symbols: List[str]) -> bool:
        """
        Unsubscribe from real-time quotes for symbols.

        Args:
            symbols: List of stock tickers

        Returns:
            True if unsubscription successful
        """
        if not self.connected or not self.ws:
            return False

        try:
            message = {
                "action": "unsubscribe",
                "type": "quotes",
                "symbols": symbols
            }

            await self.ws.send(json.dumps(message))
            self.subscribed_symbols = [s for s in self.subscribed_symbols if s not in symbols]

            logger.info(f"Unsubscribed from quotes for {len(symbols)} symbols")
            return True

        except Exception as e:
            logger.error(f"Failed to unsubscribe from quotes: {e}")
            return False

    def on_quote(self, callback: Callable[[Dict], None]):
        """
        Register callback for quote updates.

        Args:
            callback: Function to call with quote data (dict with symbol, bid, ask, etc.)
        """
        self.quote_callbacks.append(callback)
        logger.debug(f"Registered quote callback (total: {len(self.quote_callbacks)})")

    async def _handle_message(self, message: str):
        """
        Handle incoming WebSocket message.

        Args:
            message: JSON message string
        """
        try:
            data = json.loads(message)

            # Handle different message types
            msg_type = data.get("type", "")

            if msg_type == "quote" or "quote" in data:
                # Quote update
                quote_data = data.get("quote") or data
                self._notify_quote_callbacks(quote_data)

            elif msg_type == "error":
                logger.error(f"WebSocket error: {data.get('message', 'Unknown error')}")

            elif msg_type == "heartbeat" or msg_type == "ping":
                # Respond to heartbeat/ping
                await self._send_heartbeat()

            else:
                logger.debug(f"Unknown message type: {msg_type}")

        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse WebSocket message: {e}")
        except Exception as e:
            logger.error(f"Error handling WebSocket message: {e}")

    def _notify_quote_callbacks(self, quote_data: Dict):
        """
        Notify all registered callbacks of quote update.

        Args:
            quote_data: Quote data dictionary
        """
        for callback in self.quote_callbacks:
            try:
                callback(quote_data)
            except Exception as e:
                logger.error(f"Error in quote callback: {e}")

    async def _send_heartbeat(self):
        """Send heartbeat/pong response."""
        if self.ws and self.connected:
            try:
                await self.ws.send(json.dumps({"type": "pong"}))
            except Exception as e:
                logger.debug(f"Error sending heartbeat: {e}")

    async def listen(self):
        """
        Listen for incoming messages (run in background task).
        """
        if not self.ws:
            return

        try:
            async for message in self.ws:
                if not self._running:
                    break
                await self._handle_message(message)

        except websockets.exceptions.ConnectionClosed:
            logger.warning("WebSocket connection closed")
            self.connected = False
            if self._running:
                # Attempt to reconnect
                await self._reconnect()
        except Exception as e:
            logger.error(f"Error in WebSocket listen loop: {e}")
            self.connected = False

    async def _reconnect(self):
        """Attempt to reconnect to WebSocket."""
        logger.info(f"Attempting to reconnect in {self._reconnect_delay} seconds...")
        await asyncio.sleep(self._reconnect_delay)

        if await self.connect():
            # Resubscribe to symbols
            if self.subscribed_symbols:
                await self.subscribe_quotes(self.subscribed_symbols)

            # Restart listen loop
            asyncio.create_task(self.listen())

    def get_quote(self, symbol: str) -> Optional[Dict]:
        """
        Get latest quote for symbol (from cache if available).

        Note: This is a synchronous method that returns cached data.
        For real-time updates, use callbacks.

        Args:
            symbol: Stock ticker

        Returns:
            Latest quote data or None
        """
        # In a real implementation, you'd maintain a cache of latest quotes
        # For now, this is a placeholder
        logger.debug(f"Getting quote for {symbol} (cached)")
        return None

    async def run(self):
        """
        Run WebSocket client (connect and listen).
        Use this in an async context.
        """
        if await self.connect():
            await self.listen()


class QuoteCrossValidator:
    """
    Helper class to cross-validate quotes from TWS and Massive.com.
    """

    def __init__(
        self,
        massive_ws_client: MassiveWebSocketClient,
        discrepancy_threshold: float = 0.01  # 1% threshold
    ):
        """
        Initialize quote cross-validator.

        Args:
            massive_ws_client: Massive.com WebSocket client
            discrepancy_threshold: Maximum allowed discrepancy (0.01 = 1%)
        """
        self.massive_client = massive_ws_client
        self.threshold = discrepancy_threshold
        self.latest_quotes: Dict[str, Dict] = {}  # symbol -> quote data

        # Register callback to update latest quotes
        self.massive_client.on_quote(self._update_quote)

    def _update_quote(self, quote_data: Dict):
        """Update latest quote cache."""
        symbol = quote_data.get("symbol")
        if symbol:
            self.latest_quotes[symbol] = quote_data

    def validate_quote(
        self,
        symbol: str,
        tws_bid: float,
        tws_ask: float
    ) -> tuple[bool, Optional[str]]:
        """
        Validate TWS quote against Massive.com quote.

        Args:
            symbol: Stock ticker
            tws_bid: TWS bid price
            tws_ask: TWS ask price

        Returns:
            Tuple of (is_valid, reason)
        """
        massive_quote = self.latest_quotes.get(symbol)
        if not massive_quote:
            return (True, "No Massive.com quote available for comparison")

        massive_bid = massive_quote.get("bid")
        massive_ask = massive_quote.get("ask")

        if massive_bid is None or massive_ask is None:
            return (True, "Incomplete Massive.com quote data")

        # Calculate discrepancies
        bid_diff = abs(tws_bid - massive_bid) / tws_bid if tws_bid > 0 else 0
        ask_diff = abs(tws_ask - massive_ask) / tws_ask if tws_ask > 0 else 0

        if bid_diff > self.threshold or ask_diff > self.threshold:
            reason = (
                f"Quote discrepancy detected: "
                f"TWS bid={tws_bid:.2f}, Massive bid={massive_bid:.2f} "
                f"(diff={bid_diff*100:.2f}%), "
                f"TWS ask={tws_ask:.2f}, Massive ask={massive_ask:.2f} "
                f"(diff={ask_diff*100:.2f}%)"
            )
            logger.warning(reason)
            return (False, reason)

        return (True, "Quotes match within threshold")

    def get_discrepancy(
        self,
        symbol: str,
        tws_bid: float,
        tws_ask: float
    ) -> Optional[Dict]:
        """
        Get detailed discrepancy information.

        Args:
            symbol: Stock ticker
            tws_bid: TWS bid price
            tws_ask: TWS ask price

        Returns:
            Discrepancy dict or None if no comparison available
        """
        massive_quote = self.latest_quotes.get(symbol)
        if not massive_quote:
            return None

        massive_bid = massive_quote.get("bid")
        massive_ask = massive_quote.get("ask")

        if massive_bid is None or massive_ask is None:
            return None

        bid_diff = abs(tws_bid - massive_bid) / tws_bid if tws_bid > 0 else 0
        ask_diff = abs(tws_ask - massive_ask) / tws_ask if tws_ask > 0 else 0

        return {
            "symbol": symbol,
            "tws_bid": tws_bid,
            "tws_ask": tws_ask,
            "massive_bid": massive_bid,
            "massive_ask": massive_ask,
            "bid_discrepancy_pct": bid_diff * 100,
            "ask_discrepancy_pct": ask_diff * 100,
            "max_discrepancy_pct": max(bid_diff, ask_diff) * 100,
            "within_threshold": max(bid_diff, ask_diff) <= self.threshold
        }
