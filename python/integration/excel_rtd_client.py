"""
excel_rtd_client.py - Excel RTD (Real-Time Data) client for real-time position updates

Connects to Excel RTD server for real-time position data from Israeli brokers.
Requires Windows OS, Excel installed, and broker-provided RTD server/add-in.

Based on research completed in T-146 (Research Excel RTD/DDE connectors).
"""

from __future__ import annotations

import logging
import time
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional

# Optional xlwings import (Windows-only, graceful degradation)
try:
    import xlwings as xw
    XLWINGS_AVAILABLE = True
except ImportError:
    xw = None
    XLWINGS_AVAILABLE = False

logger = logging.getLogger(__name__)


class ExcelRTDClientError(RuntimeError):
    """Generic error raised for Excel RTD client failures."""


class ExcelRTDClient:
    """
    Excel RTD client for connecting to Excel RTD server and reading real-time position data.

    Requires:
    - Windows OS
    - Excel installed with RTD add-in
    - Broker-provided RTD server or Excel workbook with RTD connections
    - xlwings library installed
    """

    def __init__(
        self,
        excel_file_path: str,
        range_name: str = "Positions",
        polling_interval: int = 5,
        visible: bool = False,
    ) -> None:
        """
        Initialize Excel RTD client.

        Args:
            excel_file_path: Path to Excel file with RTD connections
            range_name: Named range in Excel containing position data
            polling_interval: Polling interval in seconds (1-5 recommended)
            visible: Whether to show Excel window (default: False)

        Raises:
            ExcelRTDClientError: If xlwings not available or invalid parameters
        """
        if not XLWINGS_AVAILABLE:
            raise ExcelRTDClientError(
                "xlwings not available. Install with: pip install xlwings\n"
                "Note: xlwings requires Windows OS and Excel installed."
            )

        self.excel_file = Path(excel_file_path)
        if not self.excel_file.exists():
            raise ExcelRTDClientError(f"Excel file not found: {excel_file_path}")

        self.range_name = range_name
        self.polling_interval = max(1, min(60, polling_interval))  # Clamp 1-60 seconds
        self.visible = visible

        # Excel COM objects
        self.app: Optional[Any] = None
        self.wb: Optional[Any] = None
        self._connected = False

        # Last data for change detection
        self._last_data: Optional[List[Dict[str, Any]]] = None

    def connect(self) -> bool:
        """
        Connect to Excel RTD server.

        Returns:
            True if connected successfully, False otherwise
        """
        if self._connected:
            logger.warning("Already connected to Excel RTD")
            return True

        try:
            # Connect to Excel (create new instance or connect to existing)
            self.app = xw.App(visible=self.visible, add_book=False)
            logger.info(f"Connected to Excel application")

            # Open workbook with RTD connections
            self.wb = self.app.books.open(str(self.excel_file))
            logger.info(f"Opened Excel workbook: {self.excel_file.name}")

            # Verify named range exists (informational only)
            try:
                _ = self.wb.names[self.range_name]
                logger.info(f"Found named range: {self.range_name}")
            except KeyError:
                logger.warning(f"Named range '{self.range_name}' not found, will try to use directly")

            self._connected = True
            return True

        except Exception as e:
            logger.error(f"Failed to connect to Excel RTD: {e}")
            self._connected = False
            if self.app:
                try:
                    self.app.quit()
                except Exception:
                    pass
            self.app = None
            self.wb = None
            return False

    def get_position_data(self) -> List[Dict[str, Any]]:
        """
        Read position data from Excel RTD range.

        Returns:
            List of position dictionaries with keys: symbol, quantity, cost_basis, current_price, currency

        Raises:
            ExcelRTDClientError: If not connected or data read fails
        """
        if not self._connected:
            raise ExcelRTDClientError("Not connected to Excel RTD. Call connect() first.")

        try:
            # Get named range or use range name directly
            try:
                range_obj = self.wb.names[self.range_name]
                data_range = range_obj.refers_to_range
            except (KeyError, AttributeError):
                # Try to use range name as cell reference (e.g., "A1:Z100")
                try:
                    data_range = self.wb.sheets[0].range(self.range_name)
                except Exception:
                    raise ExcelRTDClientError(
                        f"Could not find named range or cell range: {self.range_name}"
                    )

            # Read values from range
            values = data_range.value

            # Handle different data formats
            if values is None:
                logger.warning("RTD range returned None, no data available")
                return []

            # Convert to list of lists if single row/column
            if not isinstance(values, list):
                values = [[values]]
            elif values and not isinstance(values[0], list):
                values = [[v] for v in values]

            # Parse RTD data into positions
            positions = self._parse_rtd_data(values)
            return positions

        except Exception as e:
            logger.error(f"Failed to read RTD data: {e}")
            raise ExcelRTDClientError(f"Failed to read RTD data: {e}") from e

    def _parse_rtd_data(self, values: List[List[Any]]) -> List[Dict[str, Any]]:
        """
        Parse RTD data array into position dictionaries.

        Args:
            values: 2D array from Excel range (rows x columns)

        Returns:
            List of position dictionaries
        """
        if not values or not values[0]:
            return []

        positions = []
        # Assume first row is header, skip it
        for row_idx, row in enumerate(values[1:], start=2):
            try:
                # Expected columns: Symbol, Quantity, Cost Basis, Current Price, Currency
                if len(row) < 4:
                    logger.warning(f"Row {row_idx}: Insufficient columns, skipping")
                    continue

                position = {
                    "symbol": str(row[0]).strip() if row[0] else "",
                    "quantity": float(row[1]) if row[1] else 0.0,
                    "cost_basis": float(row[2]) if row[2] else 0.0,
                    "current_price": float(row[3]) if row[3] else 0.0,
                    "currency": str(row[4]).strip() if len(row) > 4 and row[4] else "ILS",
                }

                if position["symbol"]:
                    positions.append(position)
            except (ValueError, TypeError) as e:
                logger.warning(f"Row {row_idx}: Failed to parse position data: {row}, error: {e}")
                continue

        return positions

    def monitor_positions(
        self, callback: Callable[[List[Dict[str, Any]]], None], stop_event: Optional[Any] = None
    ) -> None:
        """
        Monitor RTD data and call callback when positions change.

        Args:
            callback: Function to call with updated positions list
            stop_event: Optional threading.Event to stop monitoring

        Raises:
            ExcelRTDClientError: If not connected
        """
        if not self._connected:
            raise ExcelRTDClientError("Not connected to Excel RTD. Call connect() first.")

        logger.info(f"Starting RTD monitoring (interval: {self.polling_interval}s)")

        try:
            while True:
                # Check stop event if provided
                if stop_event and stop_event.is_set():
                    logger.info("Stop event set, stopping RTD monitoring")
                    break

                try:
                    current_data = self.get_position_data()

                    # Detect changes by comparing data
                    if current_data != self._last_data:
                        logger.info(f"Position data changed, {len(current_data)} positions")
                        callback(current_data)
                        self._last_data = current_data

                    time.sleep(self.polling_interval)

                except KeyboardInterrupt:
                    logger.info("RTD monitoring interrupted by user")
                    break
                except Exception as e:
                    logger.error(f"Error during RTD monitoring: {e}")
                    time.sleep(self.polling_interval)

        except Exception as e:
            logger.error(f"RTD monitoring failed: {e}")
            raise ExcelRTDClientError(f"RTD monitoring failed: {e}") from e

    def disconnect(self) -> None:
        """Disconnect from Excel and cleanup resources."""
        try:
            if self.wb:
                self.wb.close()
                logger.info("Closed Excel workbook")
            if self.app:
                self.app.quit()
                logger.info("Quit Excel application")

            self.wb = None
            self.app = None
            self._connected = False

        except Exception as e:
            logger.error(f"Error during disconnect: {e}")

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.disconnect()
