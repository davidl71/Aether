"""
nautilus_client.py - NautilusTrader client wrapper for broker integrations
Supports Interactive Brokers and Alpaca via NautilusTrader adapters.
"""

import logging
from typing import Optional, Dict, TYPE_CHECKING
from nautilus_trader.core.nautilus_pyo3 import (
    LiveDataClient,
    LiveExecutionClient,
    InstrumentId,
    Venue,
)
from nautilus_trader.adapters.interactive_brokers import (
    InteractiveBrokersDataClient,
    InteractiveBrokersExecClient,
    InteractiveBrokersDataClientConfig,
    InteractiveBrokersExecClientConfig,
)

# Alpaca adapters (available in NautilusTrader)
try:
    from nautilus_trader.adapters.alpaca import (
        AlpacaDataClient,
        AlpacaExecClient,
        AlpacaDataClientConfig,
        AlpacaExecClientConfig,
    )

    _ALPACA_AVAILABLE = True
except Exception:  # pragma: no cover - optional dependency
    _ALPACA_AVAILABLE = False

if TYPE_CHECKING:  # pragma: no cover - typing only
    from .notification_center import NotificationCenter

logger = logging.getLogger(__name__)


class NautilusClient:
    """
    Wrapper for NautilusTrader broker adapters providing market data and execution.
    """

    def __init__(
        self,
        data_config: Optional[Dict] = None,
        exec_config: Optional[Dict] = None,
        venue: str = "ALPACA",
        notification_center: Optional["NotificationCenter"] = None,
    ):
        """
        Initialize NautilusTrader client.

        Args:
            data_config: Configuration for data client
            exec_config: Configuration for execution client
            venue: Trading venue identifier (default: "ALPACA")
        """
        self.venue = Venue(venue)
        self.data_client: Optional[LiveDataClient] = None
        self.exec_client: Optional[LiveExecutionClient] = None
        self._data_config = data_config or {}
        self._exec_config = exec_config or {}
        self._connected = False
        self.notifier = notification_center

    def create_data_client(self) -> LiveDataClient:
        """Create and configure the data client."""
        if not self.data_client:
            venue_name = str(self.venue)
            if venue_name.upper() == "ALPACA":
                if not _ALPACA_AVAILABLE:
                    raise RuntimeError(
                        "Alpaca adapter not available in NautilusTrader installation"
                    )
                config = AlpacaDataClientConfig(**self._data_config)
                self.data_client = AlpacaDataClient(
                    config=config,
                    venue=self.venue,
                )
            else:
                config = InteractiveBrokersDataClientConfig(**self._data_config)
                self.data_client = InteractiveBrokersDataClient(
                    config=config,
                    venue=self.venue,
                )
        return self.data_client

    def create_exec_client(self) -> LiveExecutionClient:
        """Create and configure the execution client."""
        if not self.exec_client:
            venue_name = str(self.venue)
            if venue_name.upper() == "ALPACA":
                if not _ALPACA_AVAILABLE:
                    raise RuntimeError(
                        "Alpaca adapter not available in NautilusTrader installation"
                    )
                config = AlpacaExecClientConfig(**self._exec_config)
                self.exec_client = AlpacaExecClient(
                    config=config,
                    venue=self.venue,
                )
            else:
                config = InteractiveBrokersExecClientConfig(**self._exec_config)
                self.exec_client = InteractiveBrokersExecClient(
                    config=config,
                    venue=self.venue,
                )
        return self.exec_client

    def connect(self) -> bool:
        """Connect both data and execution clients."""
        try:
            if self.data_client:
                self.data_client.connect()
                logger.info("Data client connected")

            if self.exec_client:
                self.exec_client.connect()
                logger.info("Execution client connected")

            self._connected = True
            if self.notifier:
                self.notifier.notify(
                    event_type="connection_established",
                    title=f"{str(self.venue)} connection established",
                    message="Successfully connected data and execution clients",
                    severity="info",
                )
            return True
        except Exception as e:
            logger.error(f"Failed to connect: {e}")
            self._connected = False
            if self.notifier:
                self.notifier.notify(
                    event_type="connection_failure",
                    title=f"{str(self.venue)} connection failed",
                    message=str(e),
                    severity="critical",
                )
            return False

    def disconnect(self):
        """Disconnect both clients."""
        try:
            if self.data_client:
                self.data_client.disconnect()
            if self.exec_client:
                self.exec_client.disconnect()
            self._connected = False
            logger.info("Clients disconnected")
        except Exception as e:
            logger.error(f"Error disconnecting: {e}")
            if self.notifier:
                self.notifier.notify(
                    event_type="disconnect_error",
                    title=f"Error during {str(self.venue)} disconnect",
                    message=str(e),
                    severity="warning",
                )

    def is_connected(self) -> bool:
        """Check if clients are connected."""
        return self._connected

    def subscribe_market_data(self, instrument_id: InstrumentId):
        """Subscribe to market data for an instrument."""
        if not self.data_client:
            raise RuntimeError("Data client not initialized")
        self.data_client.subscribe_trade_ticks(instrument_id)
        self.data_client.subscribe_quote_ticks(instrument_id)
        logger.info(f"Subscribed to market data for {instrument_id}")

    def unsubscribe_market_data(self, instrument_id: InstrumentId):
        """Unsubscribe from market data."""
        if not self.data_client:
            return
        self.data_client.unsubscribe_trade_ticks(instrument_id)
        self.data_client.unsubscribe_quote_ticks(instrument_id)
        logger.info(f"Unsubscribed from market data for {instrument_id}")
