"""
Tests for preflight module.

Tests PreflightChecklist class and PreflightResult dataclass.
"""
import unittest
from unittest.mock import patch, MagicMock

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from integration.preflight import PreflightChecklist, PreflightResult


class TestPreflightResult(unittest.TestCase):
    """Tests for PreflightResult dataclass."""

    def test_preflight_result_passed(self):
        """Test PreflightResult with passed=True."""
        result = PreflightResult(passed=True, warnings=[], errors=[])
        assert result.passed is True
        assert len(result.warnings) == 0
        assert len(result.errors) == 0

    def test_preflight_result_failed(self):
        """Test PreflightResult with passed=False."""
        result = PreflightResult(
            passed=False,
            warnings=["Warning 1"],
            errors=["Error 1", "Error 2"]
        )
        assert result.passed is False
        assert len(result.warnings) == 1
        assert len(result.errors) == 2


class TestPreflightChecklist(unittest.TestCase):
    """Tests for PreflightChecklist class."""

    def setUp(self):
        """Set up test fixtures."""
        self.config = {
            "dry_run": True,
            "orats": {"enabled": False}
        }
        self.data_cfg = {"host": "127.0.0.1", "port": 7497}
        self.exec_cfg = {"host": "127.0.0.1", "port": 7497}
        self.connection_cfg = {
            "weekly_reauth": {"enabled": False}
        }

    def test_preflight_checklist_init(self):
        """Test PreflightChecklist initialization."""
        checklist = PreflightChecklist(
            config=self.config,
            nautilus_data_config=self.data_cfg,
            nautilus_exec_config=self.exec_cfg,
            connection_config=self.connection_cfg
        )
        assert checklist.config == self.config
        assert checklist.data_cfg == self.data_cfg
        assert checklist.exec_cfg == self.exec_cfg

    def test_preflight_checklist_run_passed(self):
        """Test PreflightChecklist.run() with all checks passing."""
        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is True
                assert len(result.errors) == 0

    def test_preflight_checklist_run_config_validation_failed(self):
        """Test PreflightChecklist.run() with config validation failure."""
        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (False, ["Config error 1", "Config error 2"])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is False
                assert len(result.errors) == 2
                assert "Config error 1" in result.errors

    def test_preflight_checklist_run_host_unreachable(self):
        """Test PreflightChecklist.run() with unreachable host."""
        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=False):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is False
                assert any("unreachable" in error.lower() for error in result.errors)

    def test_preflight_checklist_run_live_trading_warning(self):
        """Test PreflightChecklist.run() with live trading warning."""
        config = {"dry_run": False, "orats": {"enabled": False}}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is True
                assert any("live trading" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_orats_missing_token(self):
        """Test PreflightChecklist.run() with ORATS enabled but missing token."""
        config = {
            "dry_run": True,
            "orats": {"enabled": True, "api_token": None}
        }

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is False
                assert any("orats" in error.lower() and "token" in error.lower() for error in result.errors)

    def test_preflight_checklist_run_weekly_reauth_enabled(self):
        """Test PreflightChecklist.run() with weekly reauth enabled."""
        connection_cfg = {
            "weekly_reauth": {
                "enabled": True,
                "day_of_week": "sunday",
                "time_utc": "21:00"
            }
        }

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=connection_cfg
                )
                result = checklist.run()
                assert result.passed is True
                # Should not have warning about reauth disabled
                assert not any("reauth" in warning.lower() and "disabled" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_weekly_reauth_disabled_warning(self):
        """Test PreflightChecklist.run() with weekly reauth disabled (warning)."""
        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert any("re-authentication" in warning.lower() and "disabled" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_different_endpoints_warning(self):
        """Test PreflightChecklist.run() with different data/exec endpoints (warning)."""
        data_cfg = {"host": "127.0.0.1", "port": 7497}
        exec_cfg = {"host": "127.0.0.1", "port": 7496}  # Different port

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=data_cfg,
                    nautilus_exec_config=exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert any("different" in warning.lower() and "endpoint" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_notifications_enabled_no_channels(self):
        """Test PreflightChecklist.run() with notifications enabled but no channels."""
        notifications_cfg = {"enabled": True, "channels": []}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg,
                    notifications_config=notifications_cfg
                )
                result = checklist.run()
                assert result.passed is False
                assert any("notification" in error.lower() and "channel" in error.lower() for error in result.errors)

    def test_preflight_checklist_run_notifications_disabled_warning(self):
        """Test PreflightChecklist.run() with notifications disabled (warning)."""
        notifications_cfg = {"enabled": False}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg,
                    notifications_config=notifications_cfg
                )
                result = checklist.run()
                assert any("notification" in warning.lower() and "disabled" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_data_provider_not_ib_warning(self):
        """Test PreflightChecklist.run() with non-IB primary data provider (warning)."""
        data_provider_cfg = {"primary": "alpaca", "fallbacks": []}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg,
                    data_provider_config=data_provider_cfg
                )
                result = checklist.run()
                assert any("primary" in warning.lower() and "ib" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_questdb_disabled_warning(self):
        """Test PreflightChecklist.run() with QuestDB disabled (warning)."""
        questdb_cfg = {"enabled": False}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg,
                    questdb_config=questdb_cfg
                )
                result = checklist.run()
                assert any("questdb" in warning.lower() and "disabled" in warning.lower() for warning in result.warnings)

    def test_preflight_checklist_run_portal_disabled_warning(self):
        """Test PreflightChecklist.run() with Client Portal disabled (warning)."""
        portal_cfg = {"enabled": False}

        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.return_value = (True, [])

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg,
                    portal_config=portal_cfg
                )
                result = checklist.run()
                assert any("portal" in warning.lower() and "disabled" in warning.lower() for warning in result.warnings)

    def test_check_host_reachable_success(self):
        """Test _check_host_reachable() with successful connection."""
        with patch('integration.preflight.socket.create_connection') as mock_connect:
            mock_connect.return_value.__enter__ = MagicMock()
            mock_connect.return_value.__exit__ = MagicMock(return_value=False)

            result = PreflightChecklist._check_host_reachable("127.0.0.1", 7497)
            assert result is True

    def test_check_host_reachable_failure(self):
        """Test _check_host_reachable() with connection failure."""
        with patch('integration.preflight.socket.create_connection') as mock_connect:
            mock_connect.side_effect = OSError("Connection refused")

            result = PreflightChecklist._check_host_reachable("127.0.0.1", 7497)
            assert result is False

    def test_check_host_reachable_no_host(self):
        """Test _check_host_reachable() with no host."""
        result = PreflightChecklist._check_host_reachable(None, 7497)
        assert result is False

    def test_check_host_reachable_no_port(self):
        """Test _check_host_reachable() with no port."""
        result = PreflightChecklist._check_host_reachable("127.0.0.1", None)
        assert result is False

    def test_preflight_checklist_validate_config_exception(self):
        """Test PreflightChecklist.run() handles config validation exception."""
        with patch('python.config_adapter.ConfigAdapter') as mock_adapter:
            mock_adapter.validate_config.side_effect = Exception("Unexpected error")

            with patch('integration.preflight.PreflightChecklist._check_host_reachable', return_value=True):
                checklist = PreflightChecklist(
                    config=self.config,
                    nautilus_data_config=self.data_cfg,
                    nautilus_exec_config=self.exec_cfg,
                    connection_config=self.connection_cfg
                )
                result = checklist.run()
                assert result.passed is False
                assert any("unexpected error" in error.lower() for error in result.errors)


if __name__ == "__main__":
    unittest.main()
