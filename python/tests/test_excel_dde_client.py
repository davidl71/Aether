"""
Tests for Excel DDE client.

Note: These tests use mocks since pywin32 requires Windows OS.
"""

import pytest
from unittest.mock import patch, MagicMock

# Mock pywin32 before importing the client
import sys
mock_dde = MagicMock()
mock_win32ui = MagicMock()
sys.modules['dde'] = mock_dde
sys.modules['win32ui'] = mock_win32ui

from integration.excel_dde_client import ExcelDDEClient, ExcelDDEClientError


class TestExcelDDEClient:
    """Test cases for ExcelDDEClient."""

    def test_init_success(self):
        """Test successful initialization."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            client = ExcelDDEClient(
                server_name="BROKER",
                topic="POSITIONS",
                timeout_seconds=10
            )

            assert client.server_name == "BROKER"
            assert client.topic == "POSITIONS"
            assert client.timeout_ms == 10000
            assert not client._connected

    def test_init_pywin32_not_available(self):
        """Test initialization fails when pywin32 not available."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', False):
            with pytest.raises(ExcelDDEClientError, match="pywin32 not available"):
                ExcelDDEClient(server_name="BROKER", topic="POSITIONS")

    def test_init_missing_parameters(self):
        """Test initialization fails with missing parameters."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with pytest.raises(ExcelDDEClientError, match="required"):
                ExcelDDEClient(server_name="", topic="POSITIONS")

            with pytest.raises(ExcelDDEClientError, match="required"):
                ExcelDDEClient(server_name="BROKER", topic="")

    def test_connect_success(self):
        """Test successful DDE connection."""
        mock_server_handle = MagicMock()
        mock_topic_handle = MagicMock()
        mock_conversation = MagicMock()

        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with patch('integration.excel_dde_client.dde') as mock_dde_module:
                mock_dde_module.CreateStringHandle.side_effect = [
                    mock_server_handle,
                    mock_topic_handle
                ]
                mock_dde_module.CreateConversation.return_value = mock_conversation

                client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")
                result = client.connect()

                assert result is True
                assert client._connected is True
                assert client.conversation == mock_conversation
                mock_conversation.Connect.assert_called_once()

    def test_connect_failure(self):
        """Test connection failure handling."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with patch('integration.excel_dde_client.dde') as mock_dde_module:
                mock_dde_module.CreateStringHandle.side_effect = Exception("DDE error")

                client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")
                result = client.connect()

                assert result is False
                assert client._connected is False

    def test_request_data_success(self):
        """Test successful data request."""
        mock_server_handle = MagicMock()
        mock_topic_handle = MagicMock()
        mock_conversation = MagicMock()
        mock_data_handle = MagicMock()

        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with patch('integration.excel_dde_client.dde') as mock_dde_module:
                mock_dde_module.CreateStringHandle.side_effect = [
                    mock_server_handle,
                    mock_topic_handle
                ]
                mock_dde_module.CreateConversation.return_value = mock_conversation
                mock_conversation.RequestData.return_value = mock_data_handle
                mock_dde_module.GetString.return_value = "AAPL\t100\t150.0\t175.0\tUSD"

                client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")
                client.connect()

                data = client.request_data("POSITIONS")

                assert data == "AAPL\t100\t150.0\t175.0\tUSD"
                mock_conversation.RequestData.assert_called_once_with("POSITIONS", 10000)

    def test_request_data_not_connected(self):
        """Test request_data fails when not connected."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")

            with pytest.raises(ExcelDDEClientError, match="Not connected"):
                client.request_data("POSITIONS")

    def test_request_position_data(self):
        """Test position data request and parsing."""
        mock_server_handle = MagicMock()
        mock_topic_handle = MagicMock()
        mock_conversation = MagicMock()
        mock_data_handle = MagicMock()

        dde_data = "Symbol\tQuantity\tCost Basis\tPrice\tCurrency\nAAPL\t100\t150.0\t175.0\tUSD\nTSLA\t50\t200.0\t250.0\tUSD"

        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with patch('integration.excel_dde_client.dde') as mock_dde_module:
                mock_dde_module.CreateStringHandle.side_effect = [
                    mock_server_handle,
                    mock_topic_handle
                ]
                mock_dde_module.CreateConversation.return_value = mock_conversation
                mock_conversation.RequestData.return_value = mock_data_handle
                mock_dde_module.GetString.return_value = dde_data

                client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")
                client.connect()

                positions = client.request_position_data()

                assert len(positions) == 2
                assert positions[0]["symbol"] == "AAPL"
                assert positions[0]["quantity"] == 100.0
                assert positions[1]["symbol"] == "TSLA"

    def test_parse_dde_data_tab_separated(self):
        """Test parsing tab-separated DDE data."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")

            data = "Symbol\tQuantity\tCost Basis\tPrice\nAAPL\t100\t150.0\t175.0"
            positions = client._parse_dde_data(data)

            assert len(positions) == 1
            assert positions[0]["symbol"] == "AAPL"
            assert positions[0]["quantity"] == 100.0

    def test_parse_dde_data_comma_separated(self):
        """Test parsing comma-separated DDE data."""
        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            client = ExcelDDEClient(server_name="BROKER", topic="POSITIONS")

            data = "Symbol,Quantity,Cost Basis,Price\nAAPL,100,150.0,175.0"
            positions = client._parse_dde_data(data)

            assert len(positions) == 1
            assert positions[0]["symbol"] == "AAPL"

    def test_context_manager(self):
        """Test context manager usage."""
        mock_server_handle = MagicMock()
        mock_topic_handle = MagicMock()
        mock_conversation = MagicMock()

        with patch('integration.excel_dde_client.PYWIN32_AVAILABLE', True):
            with patch('integration.excel_dde_client.dde') as mock_dde_module:
                mock_dde_module.CreateStringHandle.side_effect = [
                    mock_server_handle,
                    mock_topic_handle
                ]
                mock_dde_module.CreateConversation.return_value = mock_conversation

                with ExcelDDEClient(server_name="BROKER", topic="POSITIONS") as client:
                    assert client._connected is True

                # Disconnect should be called
                mock_conversation.Disconnect.assert_called_once()
