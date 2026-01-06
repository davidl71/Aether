"""
Tests for connection manager module.

Tests ReauthConfig, ReauthScheduler, and ConnectionSupervisor classes.
"""

import unittest
from datetime import datetime, time as dt_time, timedelta
from unittest.mock import Mock, patch

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from integration.connection_manager import (
    ReauthConfig,
    ReauthScheduler,
    ConnectionSupervisor,
    WEEKDAY_MAP,
)


class TestReauthConfig(unittest.TestCase):
    """Tests for ReauthConfig dataclass."""

    def test_default_values(self):
        """Test ReauthConfig with default values."""
        config = ReauthConfig()

        assert config.enabled is False
        assert config.day_of_week == 6  # Sunday
        assert config.time_utc == dt_time(hour=21, minute=0)
        assert config.reminder_minutes_before == 15
        assert config.reauth_window_minutes == 10
        assert config.auto_reconnect is True

    def test_custom_values(self):
        """Test ReauthConfig with custom values."""
        config = ReauthConfig(
            enabled=True,
            day_of_week=0,  # Monday
            time_utc=dt_time(hour=10, minute=30),
            reminder_minutes_before=30,
            reauth_window_minutes=15,
            auto_reconnect=False,
        )

        assert config.enabled is True
        assert config.day_of_week == 0
        assert config.time_utc == dt_time(hour=10, minute=30)
        assert config.reminder_minutes_before == 30
        assert config.reauth_window_minutes == 15
        assert config.auto_reconnect is False


class TestReauthSchedulerParseConfig(unittest.TestCase):
    """Tests for ReauthScheduler.parse_config() method."""

    def test_parse_config_none(self):
        """Test parse_config() with None input."""
        config = ReauthScheduler.parse_config(None)

        assert config.enabled is False

    def test_parse_config_empty_dict(self):
        """Test parse_config() with empty dict."""
        config = ReauthScheduler.parse_config({})

        assert config.enabled is False

    def test_parse_config_enabled_false(self):
        """Test parse_config() with enabled=False."""
        config = ReauthScheduler.parse_config({"enabled": False})

        assert config.enabled is False

    def test_parse_config_enabled_true(self):
        """Test parse_config() with enabled=True."""
        config = ReauthScheduler.parse_config({"enabled": True})

        assert config.enabled is True
        assert config.day_of_week == 6  # Default Sunday

    def test_parse_config_day_of_week_string(self):
        """Test parse_config() with day_of_week as string."""
        for day_name, day_idx in WEEKDAY_MAP.items():
            config = ReauthScheduler.parse_config(
                {"enabled": True, "day_of_week": day_name}
            )
            assert config.day_of_week == day_idx

    def test_parse_config_day_of_week_invalid(self):
        """Test parse_config() with invalid day_of_week defaults to Sunday."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "day_of_week": "invalid"}
        )
        assert config.day_of_week == 6  # Defaults to Sunday

    def test_parse_config_time_utc_valid(self):
        """Test parse_config() with valid time_utc format."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "time_utc": "10:30"}
        )
        assert config.time_utc == dt_time(hour=10, minute=30)

    def test_parse_config_time_utc_invalid(self):
        """Test parse_config() with invalid time_utc format defaults to 21:00."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "time_utc": "invalid"}
        )
        assert config.time_utc == dt_time(hour=21, minute=0)

    def test_parse_config_reminder_minutes(self):
        """Test parse_config() with reminder_minutes_before."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "reminder_minutes_before": 30}
        )
        assert config.reminder_minutes_before == 30

    def test_parse_config_window_minutes(self):
        """Test parse_config() with reauth_window_minutes."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "reauth_window_minutes": 20}
        )
        assert config.reauth_window_minutes == 20

    def test_parse_config_auto_reconnect(self):
        """Test parse_config() with auto_reconnect."""
        config = ReauthScheduler.parse_config(
            {"enabled": True, "auto_reconnect": False}
        )
        assert config.auto_reconnect is False


class TestReauthScheduler(unittest.TestCase):
    """Tests for ReauthScheduler class."""

    def test_init_disabled(self):
        """Test ReauthScheduler initialization with disabled config."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)

        assert scheduler.config.enabled is False
        assert scheduler._next_reauth_utc is None
        assert scheduler._reminder_issued is False

    def test_init_enabled(self):
        """Test ReauthScheduler initialization with enabled config."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        assert scheduler.config.enabled is True
        assert scheduler._next_reauth_utc is not None
        assert isinstance(scheduler._next_reauth_utc, datetime)

    def test_calculate_next_occurrence_same_week(self):
        """Test _calculate_next_occurrence() for same week."""
        config = ReauthConfig(
            enabled=True, day_of_week=0, time_utc=dt_time(10, 0)
        )  # Monday 10:00
        scheduler = ReauthScheduler(config)

        # Reference: Monday 9:00 (before target time)
        reference = datetime(2025, 1, 6, 9, 0)  # Monday Jan 6, 2025 9:00 UTC
        next_occurrence = scheduler._calculate_next_occurrence(reference)

        # Should be same day at 10:00
        assert next_occurrence.date() == reference.date()
        assert next_occurrence.hour == 10
        assert next_occurrence.minute == 0

    def test_calculate_next_occurrence_next_week(self):
        """Test _calculate_next_occurrence() for next week."""
        config = ReauthConfig(
            enabled=True, day_of_week=0, time_utc=dt_time(10, 0)
        )  # Monday 10:00
        scheduler = ReauthScheduler(config)

        # Reference: Monday 11:00 (after target time)
        reference = datetime(2025, 1, 6, 11, 0)  # Monday Jan 6, 2025 11:00 UTC
        next_occurrence = scheduler._calculate_next_occurrence(reference)

        # Should be next Monday at 10:00
        assert next_occurrence.date() == (reference.date() + timedelta(days=7))
        assert next_occurrence.hour == 10
        assert next_occurrence.minute == 0

    def test_should_issue_reminder_disabled(self):
        """Test should_issue_reminder() when scheduler is disabled."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)

        assert scheduler.should_issue_reminder() is False

    def test_should_issue_reminder_before_window(self):
        """Test should_issue_reminder() before reminder time."""
        config = ReauthConfig(
            enabled=True,
            day_of_week=0,
            time_utc=dt_time(10, 0),
            reminder_minutes_before=15,
        )
        scheduler = ReauthScheduler(config)

        # Set next reauth to Monday 10:00
        reference = datetime(2025, 1, 6, 9, 0)  # Monday 9:00 (before reminder)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        # Check at 9:44 (before reminder time of 9:45)
        now = datetime(2025, 1, 6, 9, 44)
        assert scheduler.should_issue_reminder(now) is False

    def test_should_issue_reminder_during_window(self):
        """Test should_issue_reminder() during reminder window."""
        config = ReauthConfig(
            enabled=True,
            day_of_week=0,
            time_utc=dt_time(10, 0),
            reminder_minutes_before=15,
        )
        scheduler = ReauthScheduler(config)

        # Set next reauth to Monday 10:00
        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        # Check at 9:46 (during reminder window 9:45-10:00)
        now = datetime(2025, 1, 6, 9, 46)
        assert scheduler.should_issue_reminder(now) is True
        assert scheduler._reminder_issued is True

    def test_should_issue_reminder_already_issued(self):
        """Test should_issue_reminder() when reminder already issued."""
        config = ReauthConfig(
            enabled=True,
            day_of_week=0,
            time_utc=dt_time(10, 0),
            reminder_minutes_before=15,
        )
        scheduler = ReauthScheduler(config)

        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)
        scheduler._reminder_issued = True

        now = datetime(2025, 1, 6, 9, 46)
        assert scheduler.should_issue_reminder(now) is False

    def test_should_trigger_reauth_disabled(self):
        """Test should_trigger_reauth() when scheduler is disabled."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)

        assert scheduler.should_trigger_reauth() is False

    def test_should_trigger_reauth_before_time(self):
        """Test should_trigger_reauth() before reauth time."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        now = datetime(2025, 1, 6, 9, 59)
        assert scheduler.should_trigger_reauth(now) is False

    def test_should_trigger_reauth_at_time(self):
        """Test should_trigger_reauth() at reauth time."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        now = datetime(2025, 1, 6, 10, 0)
        assert scheduler.should_trigger_reauth(now) is True

    def test_should_trigger_reauth_after_time(self):
        """Test should_trigger_reauth() after reauth time."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        now = datetime(2025, 1, 6, 10, 1)
        assert scheduler.should_trigger_reauth(now) is True

    def test_mark_reauth_complete_disabled(self):
        """Test mark_reauth_complete() when scheduler is disabled."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)

        scheduler.mark_reauth_complete()

        assert scheduler._next_reauth_utc is None

    def test_mark_reauth_complete_enabled(self):
        """Test mark_reauth_complete() when scheduler is enabled."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        original_next = scheduler._next_reauth_utc
        scheduler._reminder_issued = True

        scheduler.mark_reauth_complete()

        assert scheduler._next_reauth_utc is not None
        assert scheduler._next_reauth_utc != original_next
        assert scheduler._reminder_issued is False

    @patch("integration.connection_manager.time.sleep")
    @patch("integration.connection_manager.time.monotonic")
    def test_perform_reauthentication_disabled(self, mock_monotonic, mock_sleep):
        """Test perform_reauthentication() when scheduler is disabled."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_runner = Mock()

        result = scheduler.perform_reauthentication(mock_client, mock_runner)

        assert result is True
        mock_client.disconnect.assert_not_called()

    @patch("integration.connection_manager.time.sleep")
    @patch("integration.connection_manager.time.monotonic")
    def test_perform_reauthentication_auto_reconnect_success(
        self, mock_monotonic, mock_sleep
    ):
        """Test perform_reauthentication() with auto_reconnect=True, successful reconnect."""
        config = ReauthConfig(
            enabled=True, reauth_window_minutes=10, auto_reconnect=True
        )
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_client.connect.side_effect = [False, True]  # Second attempt succeeds
        mock_runner = Mock()
        mock_runner.is_running = True

        # Mock time.monotonic() to return values that allow reconnection
        mock_monotonic.side_effect = [0, 30, 60]  # Start, first attempt, second attempt

        result = scheduler.perform_reauthentication(mock_client, mock_runner)

        assert result is True
        mock_client.disconnect.assert_called_once()
        assert mock_client.connect.call_count == 2
        mock_runner.pause.assert_called_once()
        mock_runner.resume.assert_called_once()

    @patch("integration.connection_manager.time.sleep")
    @patch("integration.connection_manager.time.monotonic")
    def test_perform_reauthentication_auto_reconnect_failure(
        self, mock_monotonic, mock_sleep
    ):
        """Test perform_reauthentication() with auto_reconnect=True, reconnect fails."""
        config = ReauthConfig(
            enabled=True, reauth_window_minutes=1, auto_reconnect=True
        )
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_client.connect.return_value = False  # Always fails
        mock_runner = Mock()
        mock_runner.is_running = True
        mock_notifier = Mock()

        # Mock time.monotonic() to expire window
        mock_monotonic.side_effect = [0, 30, 60, 90]  # Exceeds 1 minute window

        result = scheduler.perform_reauthentication(
            mock_client, mock_runner, mock_notifier
        )

        assert result is False
        mock_client.disconnect.assert_called_once()
        mock_notifier.notify.assert_called()
        # Runner should remain paused
        mock_runner.resume.assert_not_called()

    @patch("integration.connection_manager.time.sleep")
    @patch("integration.connection_manager.time.monotonic")
    def test_perform_reauthentication_manual_reconnect_success(
        self, mock_monotonic, mock_sleep
    ):
        """Test perform_reauthentication() with auto_reconnect=False, successful reconnect."""
        config = ReauthConfig(
            enabled=True, reauth_window_minutes=10, auto_reconnect=False
        )
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_client.connect.return_value = True
        mock_runner = Mock()
        mock_runner.is_running = True

        # Mock time.monotonic() for manual reconnect
        mock_monotonic.side_effect = [0, 600]  # Start, end of window

        result = scheduler.perform_reauthentication(mock_client, mock_runner)

        assert result is True
        mock_client.disconnect.assert_called_once()
        mock_client.connect.assert_called_once()
        mock_runner.pause.assert_called_once()
        mock_runner.resume.assert_called_once()

    @patch("integration.connection_manager.time.sleep")
    @patch("integration.connection_manager.time.monotonic")
    def test_perform_reauthentication_manual_reconnect_failure(
        self, mock_monotonic, mock_sleep
    ):
        """Test perform_reauthentication() with auto_reconnect=False, reconnect fails."""
        config = ReauthConfig(
            enabled=True, reauth_window_minutes=10, auto_reconnect=False
        )
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_client.connect.return_value = False
        mock_runner = Mock()
        mock_runner.is_running = True
        mock_notifier = Mock()

        # Mock time.monotonic() for manual reconnect
        mock_monotonic.side_effect = [0, 600]  # Start, end of window

        result = scheduler.perform_reauthentication(
            mock_client, mock_runner, mock_notifier
        )

        assert result is False
        mock_client.disconnect.assert_called_once()
        mock_client.connect.assert_called_once()
        mock_notifier.notify.assert_called()
        # Runner should remain paused
        mock_runner.resume.assert_not_called()

    def test_perform_reauthentication_disconnect_error(self):
        """Test perform_reauthentication() handles disconnect errors gracefully."""
        config = ReauthConfig(enabled=True, auto_reconnect=False)
        scheduler = ReauthScheduler(config)
        mock_client = Mock()
        mock_client.disconnect.side_effect = Exception("Disconnect error")
        mock_runner = Mock()

        with patch("integration.connection_manager.time.monotonic", return_value=0):
            with patch("integration.connection_manager.time.sleep"):
                mock_client.connect.return_value = True
                result = scheduler.perform_reauthentication(mock_client, mock_runner)

        # Should continue despite disconnect error
        assert result is True


class TestConnectionSupervisor(unittest.TestCase):
    """Tests for ConnectionSupervisor class."""

    def test_init(self):
        """Test ConnectionSupervisor initialization."""
        config = ReauthConfig(enabled=True)
        scheduler = ReauthScheduler(config)
        notifier = Mock()

        supervisor = ConnectionSupervisor(scheduler, notifier)

        assert supervisor.scheduler == scheduler
        assert supervisor.notifier == notifier

    def test_init_no_notifier(self):
        """Test ConnectionSupervisor initialization without notifier."""
        config = ReauthConfig(enabled=True)
        scheduler = ReauthScheduler(config)

        supervisor = ConnectionSupervisor(scheduler, None)

        assert supervisor.scheduler == scheduler
        assert supervisor.notifier is None

    def test_run_housekeeping_disabled(self):
        """Test run_housekeeping() when scheduler is disabled."""
        config = ReauthConfig(enabled=False)
        scheduler = ReauthScheduler(config)
        supervisor = ConnectionSupervisor(scheduler)
        mock_client = Mock()
        mock_runner = Mock()

        supervisor.run_housekeeping(mock_client, mock_runner)

        # Should return early, no operations
        assert scheduler._next_reauth_utc is None

    @patch("integration.connection_manager.datetime")
    def test_run_housekeeping_reminder(self, mock_datetime):
        """Test run_housekeeping() issues reminder when appropriate."""
        config = ReauthConfig(
            enabled=True,
            day_of_week=0,
            time_utc=dt_time(10, 0),
            reminder_minutes_before=15,
        )
        scheduler = ReauthScheduler(config)

        # Set up next reauth
        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        # Mock current time during reminder window
        mock_datetime.utcnow.return_value = datetime(2025, 1, 6, 9, 46)
        mock_datetime.combine = datetime.combine
        mock_datetime.side_effect = lambda *args, **kw: datetime(*args, **kw)

        notifier = Mock()
        supervisor = ConnectionSupervisor(scheduler, notifier)
        mock_client = Mock()
        mock_runner = Mock()

        supervisor.run_housekeeping(mock_client, mock_runner)

        # Should issue reminder
        assert scheduler._reminder_issued is True
        notifier.notify.assert_called_once()

    @patch("integration.connection_manager.datetime")
    def test_run_housekeeping_trigger_reauth(self, mock_datetime):
        """Test run_housekeeping() triggers reauthentication when appropriate."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        # Set up next reauth
        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        # Mock current time at reauth time
        mock_datetime.utcnow.return_value = datetime(2025, 1, 6, 10, 0)
        mock_datetime.combine = datetime.combine
        mock_datetime.side_effect = lambda *args, **kw: datetime(*args, **kw)

        supervisor = ConnectionSupervisor(scheduler)
        mock_client = Mock()
        mock_runner = Mock()

        with patch.object(
            scheduler, "perform_reauthentication", return_value=True
        ) as mock_reauth:
            supervisor.run_housekeeping(mock_client, mock_runner)

            mock_reauth.assert_called_once_with(mock_client, mock_runner, None)

    @patch("integration.connection_manager.datetime")
    def test_run_housekeeping_reauth_failure(self, mock_datetime):
        """Test run_housekeeping() handles reauthentication failure."""
        config = ReauthConfig(enabled=True, day_of_week=0, time_utc=dt_time(10, 0))
        scheduler = ReauthScheduler(config)

        reference = datetime(2025, 1, 6, 9, 0)
        scheduler._next_reauth_utc = scheduler._calculate_next_occurrence(reference)

        mock_datetime.utcnow.return_value = datetime(2025, 1, 6, 10, 0)
        mock_datetime.combine = datetime.combine
        mock_datetime.side_effect = lambda *args, **kw: datetime(*args, **kw)

        notifier = Mock()
        supervisor = ConnectionSupervisor(scheduler, notifier)
        mock_client = Mock()
        mock_runner = Mock()

        with patch.object(
            scheduler, "perform_reauthentication", return_value=False
        ) as mock_reauth:
            supervisor.run_housekeeping(mock_client, mock_runner)

            mock_reauth.assert_called_once()
            # Should notify about failure
            assert notifier.notify.call_count >= 1


if __name__ == "__main__":
    unittest.main()
