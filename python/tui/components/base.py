"""
Base class for TUI tab components that display SnapshotPayload.

Reduces boilerplate: subclasses implement compose() and _update_data().
"""

from __future__ import annotations

from typing import Optional

from textual.containers import Container

from ..models import SnapshotPayload


class SnapshotTabBase(Container):
    """Base for tabs that show snapshot data. Subclasses implement compose() and _update_data()."""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None):
        super().__init__()
        self.snapshot = snapshot

    def update_snapshot(self, snapshot: SnapshotPayload, **kwargs: object) -> None:
        """Update with new snapshot data. Subclasses can override to accept extra kwargs."""
        self.snapshot = snapshot
        self._update_data()

    def _update_data(self) -> None:
        """Override in subclasses to refresh the display from self.snapshot."""
        pass
