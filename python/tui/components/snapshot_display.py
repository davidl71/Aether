"""Snapshot header widget: time, mode, strategy, account."""

from __future__ import annotations

from typing import Optional

from textual.widgets import Static
from textual.reactive import reactive

from ..models import SnapshotPayload


class SnapshotDisplay(Static):
    """Widget that displays snapshot data reactively."""

    snapshot: reactive[Optional[SnapshotPayload]] = reactive(None)

    def watch_snapshot(self, snapshot: Optional[SnapshotPayload]) -> None:
        if snapshot:
            self.update(self._format_snapshot(snapshot))
        else:
            self.update("Waiting for data...")

    def _format_snapshot(self, snapshot: SnapshotPayload) -> str:
        time_str = (
            snapshot.generated_at.split("T")[1].split(".")[0]
            if snapshot.generated_at
            else "--:--:--"
        )
        return f"Time: {time_str} | Mode: {snapshot.mode} | Strategy: {snapshot.strategy} | Account: {snapshot.account_id}"
