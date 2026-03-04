"""Alerts tab: system alerts log."""

from __future__ import annotations

from textual.containers import Vertical
from textual.widgets import Label, Log
from textual.app import ComposeResult

from ..models import Severity
from .base import SnapshotTabBase


class AlertsTab(SnapshotTabBase):
    """Alerts tab showing system alerts."""

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Alerts", classes="tab-title")
            yield Log(id="alerts-log")

    def on_mount(self) -> None:
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        try:
            log = self.query_one("#alerts-log", Log)
            log.clear()

            for alert in self.snapshot.alerts:
                time_str = (
                    alert.timestamp.split("T")[1].split(".")[0]
                    if alert.timestamp
                    else "--:--:--"
                )
                severity_style = self._get_severity_style(alert.severity)
                log.write(
                    f"[{time_str}] [{severity_style}]{alert.text}[/]", markup=True
                )
        except Exception:
            pass

    def _get_severity_style(self, severity: Severity) -> str:
        styles = {
            Severity.INFO: "cyan",
            Severity.SUCCESS: "green",
            Severity.WARN: "yellow",
            Severity.WARNING: "yellow",
            Severity.ERROR: "red",
            Severity.CRITICAL: "bold red",
        }
        return styles.get(severity, "white")
