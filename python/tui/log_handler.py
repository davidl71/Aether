"""
TUI log handler: capture log records into a queue for display in the Logs tab.
"""

from __future__ import annotations

import logging
import queue
import sys
from collections import deque
from typing import Deque, Optional

# Max lines to keep in memory for the Logs tab
TUI_LOG_MAX_LINES = 500

# Global queue and buffer; populated by TuiLogHandler, drained by app
_log_queue: Optional[queue.Queue] = None
_log_buffer: Optional[Deque[str]] = None


def get_log_queue() -> queue.Queue:
    """Return the global log queue (create if needed)."""
    global _log_queue
    if _log_queue is None:
        _log_queue = queue.Queue()
    return _log_queue


def get_log_buffer() -> Deque[str]:
    """Return the global ring buffer of formatted log lines (create if needed)."""
    global _log_buffer
    if _log_buffer is None:
        _log_buffer = deque(maxlen=TUI_LOG_MAX_LINES)
    return _log_buffer


class TuiLogHandler(logging.Handler):
    """Handler that enqueues log records for the TUI to display."""

    def __init__(self) -> None:
        super().__init__()
        self._queue = get_log_queue()
        self._buffer = get_log_buffer()

    def emit(self, record: logging.LogRecord) -> None:
        try:
            msg = self.format(record)
            self._buffer.append(msg)
            self._queue.put_nowait(msg)
        except Exception:
            self.handleError(record)


def install_tui_log_handler(
    level: int = logging.DEBUG,
    attach_to_root: bool = False,
) -> TuiLogHandler:
    """
    Add TuiLogHandler for TUI Logs tab.

    If attach_to_root is True, attach to the root logger so all log output
    (from any library) goes to the Logs tab. Existing root handlers that
    write to stderr are removed so the terminal is not overwritten.
    Returns the handler for removal on unmount.
    If attach_to_root is False, attach only to python.tui (for tests).
    """
    handler = TuiLogHandler()
    handler.setLevel(level)
    handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(name)s: %(message)s"))
    if attach_to_root:
        root = logging.getLogger()
        root.setLevel(level)
        # Remove handlers that write to stderr so logs only appear in the Logs tab
        to_remove = [
            h for h in root.handlers
            if isinstance(h, logging.StreamHandler) and getattr(h, "stream", None) is sys.stderr
        ]
        for h in to_remove:
            root.removeHandler(h)
        root.addHandler(handler)
    else:
        logging.getLogger("python.tui").addHandler(handler)
    return handler


def remove_tui_log_handler(handler: logging.Handler, from_root: bool = False) -> None:
    """Remove the TUI log handler from python.tui or from the root logger."""
    if from_root:
        logging.getLogger().removeHandler(handler)
    else:
        logging.getLogger("python.tui").removeHandler(handler)


def drain_log_queue() -> list[str]:
    """Drain the log queue and return new lines (for the main thread to display)."""
    q = get_log_queue()
    lines = []
    while True:
        try:
            lines.append(q.get_nowait())
        except queue.Empty:
            break
    return lines


def get_buffered_log_lines() -> list[str]:
    """Return current buffer contents (e.g. for initial display when opening Logs tab)."""
    return list(get_log_buffer())
