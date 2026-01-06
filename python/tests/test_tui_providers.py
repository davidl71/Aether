"""
Tests for TUI providers module.

Tests Provider abstract class, MockProvider, RestProvider, and FileProvider.
"""
import unittest
import json
import tempfile
import time
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock, mock_open

import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from tui.providers import Provider, MockProvider, RestProvider, FileProvider
from tui.models import SnapshotPayload


class TestProvider(unittest.TestCase):
    """Tests for Provider abstract base class."""

    def test_provider_cannot_instantiate(self):
        """Test that Provider cannot be instantiated directly."""
        with self.assertRaises(TypeError):
            Provider()  # Abstract class cannot be instantiated

    def test_provider_has_abstract_methods(self):
        """Test that Provider has abstract methods."""
        # Check that Provider has abstract methods
        assert hasattr(Provider, 'start')
        assert hasattr(Provider, 'stop')
        assert hasattr(Provider, 'get_snapshot')
        assert hasattr(Provider, 'is_running')


class TestMockProvider(unittest.TestCase):
    """Tests for MockProvider class."""

    def setUp(self):
        """Set up test fixtures."""
        self.provider = MockProvider(update_interval_ms=100)

    def tearDown(self):
        """Clean up after tests."""
        if self.provider.is_running():
            self.provider.stop()

    def test_mock_provider_init(self):
        """Test MockProvider initialization."""
        provider = MockProvider(update_interval_ms=500)
        assert provider.update_interval_ms == 500
        assert not provider.is_running()
        assert provider._symbols == ["SPX", "XSP", "NDX"]

    def test_mock_provider_start_stop(self):
        """Test MockProvider start and stop."""
        assert not self.provider.is_running()
        self.provider.start()
        assert self.provider.is_running()
        self.provider.stop()
        assert not self.provider.is_running()

    def test_mock_provider_start_twice(self):
        """Test MockProvider start when already running."""
        self.provider.start()
        assert self.provider.is_running()
        # Starting again should not raise error
        self.provider.start()
        assert self.provider.is_running()
        self.provider.stop()

    def test_mock_provider_get_snapshot_not_running(self):
        """Test MockProvider.get_snapshot() when not running."""
        snapshot = self.provider.get_snapshot()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "DRY-RUN"
        assert len(snapshot.symbols) > 0

    def test_mock_provider_get_snapshot_running(self):
        """Test MockProvider.get_snapshot() when running."""
        self.provider.start()
        time.sleep(0.15)  # Wait for snapshot generation
        snapshot = self.provider.get_snapshot()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "DRY-RUN"
        self.provider.stop()

    def test_mock_provider_add_symbol(self):
        """Test MockProvider.add_symbol() method."""
        initial_count = len(self.provider._symbols)
        self.provider.add_symbol("AAPL")
        assert len(self.provider._symbols) == initial_count + 1
        assert "AAPL" in self.provider._symbols

    def test_mock_provider_add_symbol_duplicate(self):
        """Test MockProvider.add_symbol() with duplicate symbol."""
        initial_count = len(self.provider._symbols)
        self.provider.add_symbol("SPX")  # Already exists
        assert len(self.provider._symbols) == initial_count  # Should not add duplicate

    def test_mock_provider_generate_snapshot(self):
        """Test MockProvider._generate_snapshot() method."""
        snapshot = self.provider._generate_snapshot()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "DRY-RUN"
        assert snapshot.strategy == "RUNNING"
        assert len(snapshot.symbols) == 3  # SPX, XSP, NDX
        assert snapshot.symbols[0]["symbol"] == "SPX"
        assert snapshot.metrics.net_liq == 100000.0


class TestRestProvider(unittest.TestCase):
    """Tests for RestProvider class."""

    def setUp(self):
        """Set up test fixtures."""
        self.provider = RestProvider(
            endpoint="http://localhost:8080/api/snapshot",
            update_interval_ms=100,
            timeout_ms=1000
        )

    def tearDown(self):
        """Clean up after tests."""
        if self.provider.is_running():
            self.provider.stop()

    def test_rest_provider_init(self):
        """Test RestProvider initialization."""
        provider = RestProvider(
            endpoint="http://test.com/api",
            update_interval_ms=2000,
            timeout_ms=5000
        )
        assert provider.endpoint == "http://test.com/api"
        assert provider.update_interval_ms == 2000
        assert provider.timeout_sec == 5.0
        assert not provider.is_running()

    def test_rest_provider_start_stop(self):
        """Test RestProvider start and stop."""
        assert not self.provider.is_running()
        self.provider.start()
        assert self.provider.is_running()
        self.provider.stop()
        assert not self.provider.is_running()

    def test_rest_provider_start_twice(self):
        """Test RestProvider start when already running."""
        self.provider.start()
        assert self.provider.is_running()
        self.provider.start()  # Should not raise error
        assert self.provider.is_running()
        self.provider.stop()

    @patch('tui.providers.requests.Session')
    def test_rest_provider_fetch_success(self, mock_session_class):
        """Test RestProvider._fetch() with successful response."""
        mock_response = Mock()
        mock_response.json.return_value = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "strategy": "RUNNING",
            "account_id": "DU123456",
            "symbols": [],
            "positions": [],
            "metrics": {"net_liq": 100000.0}
        }
        mock_response.raise_for_status = Mock()

        mock_session = Mock()
        mock_session.get.return_value = mock_response
        mock_session_class.return_value = mock_session

        provider = RestProvider(endpoint="http://test.com/api")
        snapshot = provider._fetch()

        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "LIVE"
        assert snapshot.metrics.net_liq == 100000.0

    @patch('tui.providers.requests.Session')
    def test_rest_provider_fetch_error(self, mock_session_class):
        """Test RestProvider._fetch() with error response."""
        mock_session = Mock()
        mock_session.get.side_effect = Exception("Connection error")
        mock_session_class.return_value = mock_session

        provider = RestProvider(endpoint="http://test.com/api")
        snapshot = provider._fetch()

        # Should return empty snapshot on error
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "DRY-RUN"  # Default

    @patch('tui.providers.requests.Session')
    def test_rest_provider_fetch_http_error(self, mock_session_class):
        """Test RestProvider._fetch() with HTTP error."""
        mock_response = Mock()
        mock_response.raise_for_status.side_effect = Exception("404 Not Found")
        mock_session = Mock()
        mock_session.get.return_value = mock_response
        mock_session_class.return_value = mock_session

        provider = RestProvider(endpoint="http://test.com/api")
        snapshot = provider._fetch()

        # Should return empty snapshot on error
        assert isinstance(snapshot, SnapshotPayload)

    def test_rest_provider_get_snapshot_not_running(self):
        """Test RestProvider.get_snapshot() when not running."""
        with patch.object(self.provider, '_fetch') as mock_fetch:
            mock_fetch.return_value = SnapshotPayload(mode="LIVE")
            snapshot = self.provider.get_snapshot()
            assert snapshot.mode == "LIVE"
            mock_fetch.assert_called_once()

    def test_rest_provider_get_snapshot_running(self):
        """Test RestProvider.get_snapshot() when running."""
        with patch.object(self.provider, '_fetch') as mock_fetch:
            mock_fetch.return_value = SnapshotPayload(mode="LIVE")
            self.provider.start()
            time.sleep(0.15)  # Wait for poll loop
            snapshot = self.provider.get_snapshot()
            assert isinstance(snapshot, SnapshotPayload)
            self.provider.stop()


class TestFileProvider(unittest.TestCase):
    """Tests for FileProvider class."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.temp_file = Path(self.temp_dir) / "snapshot.json"
        self.provider = FileProvider(str(self.temp_file), update_interval_ms=100)

    def tearDown(self):
        """Clean up after tests."""
        if self.provider.is_running():
            self.provider.stop()
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_file_provider_init(self):
        """Test FileProvider initialization."""
        provider = FileProvider("/path/to/file.json", update_interval_ms=2000)
        assert provider.file_path == Path("/path/to/file.json")
        assert provider.update_interval_ms == 2000
        assert not provider.is_running()

    def test_file_provider_start_stop(self):
        """Test FileProvider start and stop."""
        assert not self.provider.is_running()
        self.provider.start()
        assert self.provider.is_running()
        self.provider.stop()
        assert not self.provider.is_running()

    def test_file_provider_start_twice(self):
        """Test FileProvider start when already running."""
        self.provider.start()
        assert self.provider.is_running()
        self.provider.start()  # Should not raise error
        assert self.provider.is_running()
        self.provider.stop()

    def test_file_provider_load_from_file_success(self):
        """Test FileProvider._load_from_file() with valid file."""
        snapshot_data = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "strategy": "RUNNING",
            "account_id": "DU123456",
            "symbols": [{"symbol": "SPY", "last": 450.0}],
            "positions": [],
            "metrics": {"net_liq": 100000.0}
        }

        with open(self.temp_file, 'w') as f:
            json.dump(snapshot_data, f)

        snapshot = self.provider._load_from_file()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "LIVE"
        assert len(snapshot.symbols) == 1
        assert snapshot.symbols[0].symbol == "SPY"

    def test_file_provider_load_from_file_not_found(self):
        """Test FileProvider._load_from_file() with missing file."""
        # File doesn't exist
        snapshot = self.provider._load_from_file()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "DRY-RUN"  # Default

    def test_file_provider_load_from_file_invalid_json(self):
        """Test FileProvider._load_from_file() with invalid JSON."""
        with open(self.temp_file, 'w') as f:
            f.write("invalid json")

        snapshot = self.provider._load_from_file()
        # Should return empty snapshot on error
        assert isinstance(snapshot, SnapshotPayload)

    def test_file_provider_get_snapshot_not_running(self):
        """Test FileProvider.get_snapshot() when not running."""
        snapshot_data = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "symbols": [],
            "positions": []
        }
        with open(self.temp_file, 'w') as f:
            json.dump(snapshot_data, f)

        snapshot = self.provider.get_snapshot()
        assert snapshot.mode == "LIVE"

    def test_file_provider_get_snapshot_running(self):
        """Test FileProvider.get_snapshot() when running."""
        snapshot_data = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "symbols": [],
            "positions": []
        }
        with open(self.temp_file, 'w') as f:
            json.dump(snapshot_data, f)

        self.provider.start()
        time.sleep(0.15)  # Wait for poll loop
        snapshot = self.provider.get_snapshot()
        assert isinstance(snapshot, SnapshotPayload)
        assert snapshot.mode == "LIVE"
        self.provider.stop()

    def test_file_provider_poll_loop_detects_changes(self):
        """Test FileProvider poll loop detects file changes."""
        # Create initial file
        initial_data = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "DRY-RUN",
            "symbols": [],
            "positions": []
        }
        with open(self.temp_file, 'w') as f:
            json.dump(initial_data, f)

        self.provider.start()
        time.sleep(0.15)  # Wait for initial load

        # Update file
        updated_data = {
            "generated_at": "2025-01-01T01:00:00Z",
            "mode": "LIVE",
            "symbols": [],
            "positions": []
        }
        with open(self.temp_file, 'w') as f:
            json.dump(updated_data, f)

        time.sleep(0.15)  # Wait for poll to detect change
        snapshot = self.provider.get_snapshot()
        assert snapshot.mode == "LIVE"
        self.provider.stop()


if __name__ == "__main__":
    unittest.main()
