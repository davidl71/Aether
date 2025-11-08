"""
connection_manager.py - Supervises IB connection health and scheduled re-authentication.
Inspired by QuantConnect Lean CLI workflow for Interactive Brokers live trading.
"""
from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from datetime import datetime, time as dt_time, timedelta
from typing import Optional, TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover - typing only
    from .notification_center import NotificationCenter


logger = logging.getLogger(__name__)


WEEKDAY_MAP = {
    "monday": 0,
    "tuesday": 1,
    "wednesday": 2,
    "thursday": 3,
    "friday": 4,
    "saturday": 5,
    "sunday": 6,
}


@dataclass
class ReauthConfig:
    enabled: bool = False
    day_of_week: int = 6  # Sunday
    time_utc: dt_time = dt_time(hour=21, minute=0)
    reminder_minutes_before: int = 15
    reauth_window_minutes: int = 10
    auto_reconnect: bool = True


class ReauthScheduler:
    """Schedules weekly re-authentication windows for the IB connection."""

    def __init__(self, config: Optional[ReauthConfig] = None) -> None:
        self.config = config or ReauthConfig(enabled=False)
        self._next_reauth_utc: Optional[datetime] = None
        self._reminder_issued: bool = False

        if self.config.enabled:
            self._next_reauth_utc = self._calculate_next_occurrence()
            logger.info(
                "Next IB re-authentication window scheduled for %s UTC",
                self._next_reauth_utc,
            )

    @staticmethod
    def parse_config(raw: Optional[dict]) -> ReauthConfig:
        if not raw:
            return ReauthConfig(enabled=False)

        enabled = bool(raw.get("enabled", False))

        day_str = str(raw.get("day_of_week", "sunday")).lower()
        day_idx = WEEKDAY_MAP.get(day_str, 6)

        time_str = str(raw.get("time_utc", "21:00"))
        try:
            hour, minute = [int(part) for part in time_str.split(":", 1)]
        except ValueError:
            logger.warning("Invalid time_utc format '%s', defaulting to 21:00", time_str)
            hour, minute = 21, 0

        reminder = int(raw.get("reminder_minutes_before", 15))
        window = int(raw.get("reauth_window_minutes", 10))
        auto_reconnect = bool(raw.get("auto_reconnect", True))

        return ReauthConfig(
            enabled=enabled,
            day_of_week=day_idx,
            time_utc=dt_time(hour=hour, minute=minute),
            reminder_minutes_before=reminder,
            reauth_window_minutes=window,
            auto_reconnect=auto_reconnect,
        )

    def _calculate_next_occurrence(self, reference: Optional[datetime] = None) -> datetime:
        reference = reference or datetime.utcnow()
        target_time = self.config.time_utc

        # Start from today at target time
        candidate = datetime.combine(reference.date(), target_time)

        # Adjust to target weekday
        current_weekday = candidate.weekday()
        delta_days = (self.config.day_of_week - current_weekday) % 7
        candidate += timedelta(days=delta_days)

        # If candidate already passed, schedule for next week
        if candidate <= reference:
            candidate += timedelta(days=7)

        return candidate

    def should_issue_reminder(self, now: Optional[datetime] = None) -> bool:
        if not self.config.enabled or not self._next_reauth_utc:
            return False

        now = now or datetime.utcnow()
        reminder_delta = timedelta(minutes=self.config.reminder_minutes_before)
        reminder_time = self._next_reauth_utc - reminder_delta

        if self._reminder_issued:
            return False

        if reminder_time <= now < self._next_reauth_utc:
            self._reminder_issued = True
            return True

        return False

    def should_trigger_reauth(self, now: Optional[datetime] = None) -> bool:
        if not self.config.enabled or not self._next_reauth_utc:
            return False

        now = now or datetime.utcnow()
        return now >= self._next_reauth_utc

    def mark_reauth_complete(self) -> None:
        if not self.config.enabled:
            return

        self._next_reauth_utc = self._calculate_next_occurrence()
        self._reminder_issued = False
        logger.info(
            "Next IB re-authentication window scheduled for %s UTC",
            self._next_reauth_utc,
        )

    def perform_reauthentication(self, client, runner, notifier: Optional["NotificationCenter"] = None) -> bool:
        """Attempt IB re-authentication using provided client and runner."""
        if not self.config.enabled:
            return True

        logger.info("Starting scheduled IB re-authentication window")

        if notifier:
            notifier.notify(
                event_type="reauth_start",
                title="IB re-authentication window",
                message="Initiating scheduled IB re-authentication",
                severity="info",
            )

        # Pause trading activity
        if runner and runner.is_running:
            logger.info("Pausing strategy before IB re-authentication")
            runner.pause()

        try:
            client.disconnect()
        except Exception as exc:
            logger.warning("Error during IB disconnect: %s", exc)

        logger.info(
            "Please complete IB Key approval on your device within %d minutes",
            self.config.reauth_window_minutes,
        )

        # Give the operator time to approve 2FA
        deadline = time.monotonic() + self.config.reauth_window_minutes * 60
        reconnect_success = False

        if not self.config.auto_reconnect:
            logger.info("Auto reconnect disabled; waiting for manual confirmation")
            time.sleep(max(0, deadline - time.monotonic()))
            reconnect_success = client.connect()
        else:
            # Poll reconnect until window expires
            while time.monotonic() < deadline:
                if client.connect():
                    reconnect_success = True
                    break
                logger.warning("Reconnect attempt failed; retrying in 30 seconds")
                time.sleep(30)

            if not reconnect_success:
                logger.error("Unable to reconnect to IB during re-auth window")
                if notifier:
                    notifier.notify(
                        event_type="reauth_failure",
                        title="IB re-authentication failed",
                        message="Automatic reconnect attempts exhausted",
                        severity="critical",
                    )
                return False

        if not reconnect_success:
            logger.error("Manual reconnect to IB failed; strategy remains paused")
            if notifier:
                notifier.notify(
                    event_type="reauth_failure",
                    title="IB re-authentication failed",
                    message="Manual reconnect unsuccessful",
                    severity="critical",
                )
            return False

        logger.info("IB re-authentication successful; resuming strategy")

        if notifier:
            notifier.notify(
                event_type="reauth_success",
                title="IB re-authentication successful",
                message="IB connection restored after scheduled re-auth",
                severity="info",
            )

        if runner and not runner.is_running:
            runner.resume()

        self.mark_reauth_complete()
        return True


class ConnectionSupervisor:
    """High-level helper to supervise IB connection health."""

    def __init__(
        self,
        scheduler: ReauthScheduler,
        notifier: Optional["NotificationCenter"] = None,
    ) -> None:
        self.scheduler = scheduler
        self.notifier = notifier

    def run_housekeeping(self, client, runner) -> None:
        if not self.scheduler.config.enabled:
            return

        now = datetime.utcnow()

        if self.scheduler.should_issue_reminder(now):
            logger.info(
                "IB re-authentication scheduled at %s UTC. Ensure IB Key device is charged.",
                self.scheduler._next_reauth_utc,
            )
            if self.notifier:
                self.notifier.notify(
                    event_type="reauth_reminder",
                    title="Upcoming IB re-authentication",
                    message=f"Scheduled at {self.scheduler._next_reauth_utc} UTC",
                    severity="info",
                )

        if self.scheduler.should_trigger_reauth(now):
            success = self.scheduler.perform_reauthentication(client, runner, self.notifier)
            if not success:
                logger.error(
                    "Re-authentication failed; strategy remains paused until connection restored"
                )
                if self.notifier:
                    self.notifier.notify(
                        event_type="reauth_failure",
                        title="Re-authentication failed",
                        message="Strategy paused until manual intervention",
                        severity="critical",
                    )


