"""Logs tab: live TUI log output."""

from __future__ import annotations

from textual.containers import Container, Vertical
from textual.widgets import Log, Static


class LogsTab(Container):
    """Tab that displays captured TUI log lines."""

    def __init__(self, max_lines: int = 500, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._max_lines = max_lines

    def compose(self):
        with Vertical(classes="fill"):
            yield Static(
                "TUI log output (all captured logs). Scroll with mouse or keyboard.",
                classes="logs-tab-header",
            )
            yield Log(id="tui-log", max_lines=self._max_lines, highlight=True)

    def write_lines(self, lines: list[str]) -> None:
        """Append new log lines to the Log widget."""
        try:
            log = self.query_one("#tui-log", Log)
            for line in lines:
                log.write(line)
        except Exception:
            pass

    def load_buffer(self, lines: list[str]) -> None:
        """Load initial buffered lines (e.g. when tab is first shown)."""
        try:
            log = self.query_one("#tui-log", Log)
            log.clear()
            for line in lines:
                log.write(line)
        except Exception:
            pass
