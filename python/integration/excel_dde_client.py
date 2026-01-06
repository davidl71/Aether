"""
excel_dde_client.py - Excel DDE (Dynamic Data Exchange) client for legacy broker systems

Connects via DDE to broker-provided data feeds for real-time position updates.
DDE is legacy Windows technology but still used by some Israeli brokers.

Based on research completed in T-146 (Research Excel RTD/DDE connectors).
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

# Optional pywin32 import (Windows-only, graceful degradation)
try:
    import dde  # noqa: F401
    PYWIN32_AVAILABLE = True
except ImportError:
    dde = None  # type: ignore
    PYWIN32_AVAILABLE = False

logger = logging.getLogger(__name__)


class ExcelDDEClientError(RuntimeError):
    """Generic error raised for Excel DDE client failures."""


class ExcelDDEClient:
    """
    Excel DDE client for connecting to broker-provided DDE data feeds.

    Requires:
    - Windows OS
    - Broker-provided DDE server
    - pywin32 library installed

    Note: DDE is legacy technology. RTD is preferred when available.
    """

    def __init__(
        self,
        server_name: str,
        topic: str,
        timeout_seconds: int = 10,
    ) -> None:
        """
        Initialize DDE client.

        Args:
            server_name: DDE server name (broker-specific, e.g., "BROKER", "TWS")
            topic: DDE topic identifier (e.g., "POSITIONS", "MARKETDATA")
            timeout_seconds: Timeout for DDE operations in seconds

        Raises:
            ExcelDDEClientError: If pywin32 not available or invalid parameters
        """
        if not PYWIN32_AVAILABLE:
            raise ExcelDDEClientError(
                "pywin32 not available. Install with: pip install pywin32\n"
                "Note: pywin32 requires Windows OS."
            )

        if not server_name or not topic:
            raise ExcelDDEClientError("server_name and topic are required")

        self.server_name = server_name
        self.topic = topic
        self.timeout_ms = timeout_seconds * 1000

        # DDE conversation
        self.conversation: Optional[Any] = None
        self._connected = False

    def connect(self) -> bool:
        """
        Establish DDE conversation with broker server.

        Returns:
            True if connected successfully, False otherwise
        """
        if self._connected:
            logger.warning("Already connected to DDE server")
            return True

        try:
            # Create DDE conversation
            server_handle = dde.CreateStringHandle(self.server_name)
            topic_handle = dde.CreateStringHandle(self.topic)

            self.conversation = dde.CreateConversation(
                server_handle,
                topic_handle,
                dde.CBF_FAIL_ALLSVRXACTIONS | dde.CBF_SKIP_ALLNOTIFICATIONS
            )

            # Connect to DDE server
            self.conversation.Connect()
            logger.info(f"Connected to DDE server: {self.server_name}, topic: {self.topic}")

            # Clean up handles
            dde.FreeStringHandle(server_handle)
            dde.FreeStringHandle(topic_handle)

            self._connected = True
            return True

        except Exception as e:
            logger.error(f"Failed to connect to DDE server {self.server_name}: {e}")
            self._connected = False
            self.conversation = None
            return False

    def request_data(self, item: str) -> Optional[str]:
        """
        Request data from DDE server.

        Args:
            item: DDE item identifier (e.g., "POSITIONS", "SYMBOL_PRICE")

        Returns:
            Data string from DDE server, or None if request fails

        Raises:
            ExcelDDEClientError: If not connected
        """
        if not self._connected:
            raise ExcelDDEClientError("Not connected to DDE server. Call connect() first.")

        if not item:
            raise ExcelDDEClientError("DDE item is required")

        try:
            # Request data from DDE server
            handle = self.conversation.RequestData(item, self.timeout_ms)
            if not handle:
                logger.warning(f"DDE request for item '{item}' returned no data")
                return None

            # Get string data from handle
            data = dde.GetString(handle)
            dde.FreeStringHandle(handle)

            logger.debug(f"Received DDE data for item '{item}': {len(data)} characters")
            return data

        except Exception as e:
            logger.error(f"Failed to request DDE data for item '{item}': {e}")
            return None

    def request_position_data(self) -> List[Dict[str, Any]]:
        """
        Request position data from DDE server.

        Returns:
            List of position dictionaries

        Raises:
            ExcelDDEClientError: If not connected or data parsing fails
        """
        if not self._connected:
            raise ExcelDDEClientError("Not connected to DDE server. Call connect() first.")

        try:
            # Request positions item (broker-specific)
            data = self.request_data("POSITIONS")
            if not data:
                logger.warning("No position data received from DDE server")
                return []

            # Parse DDE data (format is broker-specific, typically tab-separated or CSV)
            positions = self._parse_dde_data(data)
            return positions

        except Exception as e:
            logger.error(f"Failed to request position data: {e}")
            raise ExcelDDEClientError(f"Failed to request position data: {e}") from e

    def _parse_dde_data(self, data: str) -> List[Dict[str, Any]]:
        """
        Parse DDE data string into position dictionaries.

        Args:
            data: DDE data string (format is broker-specific)

        Returns:
            List of position dictionaries
        """
        positions = []

        # DDE data format is broker-specific
        # Common formats: tab-separated, comma-separated, or custom delimiter
        # Try tab-separated first (most common for DDE)
        lines = data.strip().split('\n')
        if not lines:
            return positions

        # Skip header if present
        start_idx = 1 if len(lines) > 1 and 'symbol' in lines[0].lower() else 0

        for line_idx, line in enumerate(lines[start_idx:], start=start_idx + 1):
            try:
                # Try tab-separated first, then comma-separated
                if '\t' in line:
                    parts = line.split('\t')
                elif ',' in line:
                    parts = line.split(',')
                else:
                    # Space-separated or single value
                    parts = line.split()

                if len(parts) < 4:
                    logger.warning(f"Line {line_idx}: Insufficient columns, skipping")
                    continue

                position = {
                    "symbol": parts[0].strip(),
                    "quantity": float(parts[1]) if parts[1] else 0.0,
                    "cost_basis": float(parts[2]) if parts[2] else 0.0,
                    "current_price": float(parts[3]) if parts[3] else 0.0,
                    "currency": parts[4].strip() if len(parts) > 4 else "ILS",
                }

                if position["symbol"]:
                    positions.append(position)

            except (ValueError, IndexError) as e:
                logger.warning(f"Line {line_idx}: Failed to parse DDE data: {line}, error: {e}")
                continue

        return positions

    def disconnect(self) -> None:
        """Close DDE conversation and cleanup resources."""
        try:
            if self.conversation:
                self.conversation.Disconnect()
                logger.info("Disconnected from DDE server")
            self.conversation = None
            self._connected = False

        except Exception as e:
            logger.error(f"Error during DDE disconnect: {e}")

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.disconnect()
