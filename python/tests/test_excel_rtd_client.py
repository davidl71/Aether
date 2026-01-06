"""
Tests for Excel RTD client.

Note: These tests use mocks since xlwings requires Windows OS and Excel installed.
"""

import pytest
from unittest.mock import patch, MagicMock

# Mock xlwings before importing the client
import sys
mock_xlwings = MagicMock()
sys.modules['xlwings'] = mock_xlwings

from integration.excel_rtd_client import ExcelRTDClient, ExcelRTDClientError


class TestExcelRTDClient:
    """Test cases for ExcelRTDClient."""

    def test_init_success(self, tmp_path):
        """Test successful initialization."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            client = ExcelRTDClient(
                excel_file_path=str(excel_file),
                range_name="Positions",
                polling_interval=5
            )

            assert client.excel_file == excel_file
            assert client.range_name == "Positions"
            assert client.polling_interval == 5
            assert not client._connected

    def test_init_xlwings_not_available(self, tmp_path):
        """Test initialization fails when xlwings not available."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', False):
            with pytest.raises(ExcelRTDClientError, match="xlwings not available"):
                ExcelRTDClient(excel_file_path=str(excel_file))

    def test_init_file_not_found(self):
        """Test initialization fails when Excel file doesn't exist."""
        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            with pytest.raises(ExcelRTDClientError, match="Excel file not found"):
                ExcelRTDClient(excel_file_path="nonexistent.xlsx")

    def test_connect_success(self, tmp_path):
        """Test successful connection to Excel."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        mock_app = MagicMock()
        mock_wb = MagicMock()
        mock_name = MagicMock()
        mock_name.refers_to_range = MagicMock()
        mock_wb.names = {"Positions": mock_name}

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            with patch('integration.excel_rtd_client.xw') as mock_xw:
                mock_xw.App.return_value = mock_app
                mock_app.books.open.return_value = mock_wb

                client = ExcelRTDClient(excel_file_path=str(excel_file))
                result = client.connect()

                assert result is True
                assert client._connected is True
                assert client.app == mock_app
                assert client.wb == mock_wb
                mock_xw.App.assert_called_once_with(visible=False, add_book=False)
                mock_app.books.open.assert_called_once_with(str(excel_file))

    def test_connect_failure(self, tmp_path):
        """Test connection failure handling."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            with patch('integration.excel_rtd_client.xw') as mock_xw:
                mock_xw.App.side_effect = Exception("Excel not available")

                client = ExcelRTDClient(excel_file_path=str(excel_file))
                result = client.connect()

                assert result is False
                assert client._connected is False

    def test_get_position_data_success(self, tmp_path):
        """Test successful position data retrieval."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        mock_app = MagicMock()
        mock_wb = MagicMock()
        mock_range = MagicMock()
        mock_range.value = [
            ["Symbol", "Quantity", "Cost Basis", "Price", "Currency"],
            ["AAPL", 100, 150.0, 175.0, "USD"],
            ["TSLA", 50, 200.0, 250.0, "USD"],
        ]
        mock_name = MagicMock()
        mock_name.refers_to_range = mock_range
        mock_wb.names = {"Positions": mock_name}

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            with patch('integration.excel_rtd_client.xw') as mock_xw:
                mock_xw.App.return_value = mock_app
                mock_app.books.open.return_value = mock_wb

                client = ExcelRTDClient(excel_file_path=str(excel_file))
                client.connect()

                positions = client.get_position_data()

                assert len(positions) == 2
                assert positions[0]["symbol"] == "AAPL"
                assert positions[0]["quantity"] == 100.0
                assert positions[1]["symbol"] == "TSLA"

    def test_get_position_data_not_connected(self, tmp_path):
        """Test get_position_data fails when not connected."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            client = ExcelRTDClient(excel_file_path=str(excel_file))

            with pytest.raises(ExcelRTDClientError, match="Not connected"):
                client.get_position_data()

    def test_parse_rtd_data(self, tmp_path):
        """Test RTD data parsing."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            client = ExcelRTDClient(excel_file_path=str(excel_file))

            values = [
                ["Symbol", "Quantity", "Cost Basis", "Price", "Currency"],
                ["AAPL", 100, 150.0, 175.0, "USD"],
                ["TSLA", 50, 200.0, 250.0, "USD"],
            ]

            positions = client._parse_rtd_data(values)

            assert len(positions) == 2
            assert positions[0]["symbol"] == "AAPL"
            assert positions[0]["quantity"] == 100.0
            assert positions[0]["cost_basis"] == 150.0
            assert positions[0]["current_price"] == 175.0
            assert positions[0]["currency"] == "USD"

    def test_context_manager(self, tmp_path):
        """Test context manager usage."""
        excel_file = tmp_path / "test.xlsx"
        excel_file.write_text("test")

        mock_app = MagicMock()
        mock_wb = MagicMock()
        mock_name = MagicMock()
        mock_name.refers_to_range = MagicMock()
        mock_wb.names = {"Positions": mock_name}

        with patch('integration.excel_rtd_client.XLWINGS_AVAILABLE', True):
            with patch('integration.excel_rtd_client.xw') as mock_xw:
                mock_xw.App.return_value = mock_app
                mock_app.books.open.return_value = mock_wb

                with ExcelRTDClient(excel_file_path=str(excel_file)) as client:
                    assert client._connected is True

                # Disconnect should be called
                mock_wb.close.assert_called_once()
                mock_app.quit.assert_called_once()
