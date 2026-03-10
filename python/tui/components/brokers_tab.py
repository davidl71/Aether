"""Brokers/accounts tab: data source, live/paper mode, health, total positions."""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

from textual.containers import Vertical
from textual.widgets import Label, DataTable
from textual.app import ComposeResult

from .base import SnapshotTabBase
from .snapshot_display import BACKEND_DISPLAY_NAMES
from ..models import SnapshotPayload

# provider_type (snapshot source) -> backend key for health map (same as setup_screen)
PROVIDER_TYPE_TO_BACKEND_KEY: Dict[str, str] = {
    "rest_ib": "ib",
    "rest_tws_gateway": "tws",
    "rest_alpaca": "alpaca",
    "rest_tradestation": "tradestation",
    "rest_tastytrade": "tastytrade",
    "mock": "mock",
    "nats": "nats",
    "file": "file",
}


def _health_status_label(payload: Dict[str, Any]) -> str:
    """Return Health column: Running, Stopped, Checking, Disabled, or No API key."""
    if not isinstance(payload, dict):
        return "—"
    s = (payload.get("status") or "").lower()
    err = (payload.get("error") or "").strip()
    if s == "ok":
        return "Running"
    if s == "disabled":
        if err and "api key" in err.lower():
            return "No API key"
        if err and "credential" in err.lower():
            return "No credentials"
        return "Disabled"
    if s == "error":
        return "Stopped"
    if s == "checking":
        return "Checking"
    return "—"


def _mode_label(payload: Dict[str, Any]) -> str:
    """Return Mode column: LIVE, PAPER, or — from health payload (e.g. session_mode)."""
    if not isinstance(payload, dict):
        return "—"
    mode = (payload.get("session_mode") or "").strip().upper()
    if mode in ("LIVE", "PAPER"):
        return mode
    return "—"


def _build_brokers_table_rows(
    backend_health: Optional[Dict[str, Dict[str, Any]]],
    snapshot_positions_count: int,
    current_backend_key: Optional[str],
) -> List[Tuple[str, str, str, str]]:
    """Build (Data source, Mode, Health, Total positions) rows. One row per backend in health."""
    rows: List[Tuple[str, str, str, str]] = []
    health_map = (
        backend_health
        if isinstance(backend_health, dict) and backend_health.get("status") is None
        else {}
    )
    if not health_map:
        return [("—", "—", "No backends", "—")]

    for key in sorted(health_map.keys()):
        payload = health_map.get(key) or {}
        display = BACKEND_DISPLAY_NAMES.get(key, key.replace("_", " ").title())
        mode = _mode_label(payload)
        health = _health_status_label(payload)
        if key == current_backend_key:
            positions_str = str(snapshot_positions_count)
        else:
            positions_str = "—"
        rows.append((display, mode, health, positions_str))
    return rows


class BrokersTab(SnapshotTabBase):
    """Tab showing brokers/accounts: data source, live/paper mode, health, total positions."""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        *,
        name: Optional[str] = None,
        id: Optional[str] = None,
        classes: Optional[str] = None,
        disabled: bool = False,
    ) -> None:
        self._backend_health: Optional[Dict[str, Any]] = None
        self._current_provider_type: Optional[str] = None
        super().__init__(
            snapshot=snapshot,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )

    def update_snapshot(self, snapshot: SnapshotPayload, **kwargs: object) -> None:
        backend_health = kwargs.pop("backend_health", None)
        current_provider_type = kwargs.pop("current_provider_type", None)
        self._backend_health = backend_health if isinstance(backend_health, dict) else None
        self._current_provider_type = (
            current_provider_type if isinstance(current_provider_type, str) else None
        )
        super().update_snapshot(snapshot, **kwargs)

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Brokers / Accounts", classes="tab-title")
            yield DataTable(id="brokers-table")

    def on_mount(self) -> None:
        table = self.query_one("#brokers-table", DataTable)
        table.add_columns("Data source", "Mode", "Health", "Total positions")
        self._update_data()

    def _update_data(self) -> None:
        table = self.query_one("#brokers-table", DataTable)
        table.clear(columns=False)

        current_key = (
            PROVIDER_TYPE_TO_BACKEND_KEY.get((self._current_provider_type or "").lower(), None)
            if self._current_provider_type
            else None
        )
        positions_count = len(self.snapshot.positions) if self.snapshot else 0
        rows = _build_brokers_table_rows(
            self._backend_health,
            positions_count,
            current_key,
        )
        for row in rows:
            table.add_row(*row)
