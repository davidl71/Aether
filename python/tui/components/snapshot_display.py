"""Snapshot header widget: provider, time, mode, strategy, account, backend/service status."""

from __future__ import annotations

from typing import Optional, Dict, Any

from textual.widgets import Static
from textual.reactive import reactive

from ..models import SnapshotPayload


def _format_one_backend_health(name: str, health: dict) -> str:
    """Format a single backend's health for status line. IB status and gateway are grouped."""
    status = health.get("status", "unknown")
    ib_connected = health.get("ib_connected", False)
    gateway_logged_in = health.get("gateway_logged_in", ib_connected)
    gateway_port = health.get("gateway_port")
    label = name.upper() if name != "current" else "Service"
    if name == "connection":
        label = "Connection"
    if status == "ok":
        # Group IB status + Gateway into one segment: "IB: ok (Gateway 5000: logged in)"
        if gateway_port is not None and name.lower() == "ib":
            gw_str = "logged in" if gateway_logged_in else "not logged in"
            return f"{label}: ok (Gateway {gateway_port}: {gw_str})"
        ib_str = "connected" if ib_connected else "disconnected"
        return f"{label}: ok | IB: {ib_str}"
    if status == "disabled":
        err = health.get("error", "not configured")
        if len(err) > 30:
            err = err[:27] + "..."
        return f"{label}: disabled ({err})"
    if status == "checking":
        return f"{label}: checking..."
    err = health.get("error", "unreachable")
    if len(err) > 30:
        err = err[:27] + "..."
    return f"{label}: unreachable ({err})"


def _format_backend_health(health: Optional[Dict[str, Any]]) -> str:
    """Format backend/service status for status line. Supports single dict or dict of backends."""
    if not health:
        return ""
    # New shape: dict of backend name -> health payload
    if isinstance(health, dict) and health and not health.get("status"):
        parts = []
        for name, payload in sorted(health.items()):
            if isinstance(payload, dict):
                parts.append(_format_one_backend_health(name, payload))
        if parts:
            return " | " + " | ".join(parts)
    # Legacy single-dict shape (status, ib_connected, error)
    if isinstance(health, dict) and health.get("status") is not None:
        return " | " + _format_one_backend_health("Service", health)
    return ""


def _format_provider_label(provider_label: str) -> str:
    """Format provider segment for status line."""
    if not provider_label:
        return ""
    return f"Provider: {provider_label}"


def format_status_line(
    provider_label: str,
    snapshot: Optional[SnapshotPayload],
    backend_health: Optional[Dict[str, Any]],
) -> str:
    """Build the full status bar line: Provider | backend health | Time | Mode | Strategy | Account."""
    parts: list[str] = []
    if provider_label:
        parts.append(_format_provider_label(provider_label))
    # Show backend health (IB, Alpaca, etc.) right after Provider so it's visible and not truncated
    backend_str = _format_backend_health(backend_health)
    if backend_str.strip():
        parts.append(backend_str.strip().lstrip(" |"))
    if snapshot:
        time_str = (
            snapshot.generated_at.split("T")[1].split(".")[0]
            if snapshot.generated_at
            else "--:--:--"
        )
        parts.append(f"Time: {time_str}")
        parts.append(f"Mode: {snapshot.mode}")
        parts.append(f"Strategy: {snapshot.strategy}")
        parts.append(f"Account: {snapshot.account_id}")
    else:
        parts.append("Time: --:--:--")
        parts.append("Mode: --")
        parts.append("Strategy: --")
        parts.append("Account: --")
    line = " | ".join(parts)
    return line or "Waiting for data..."


class StatusBar(Static):
    """Single line at bottom: Provider | Time | Mode | Strategy | Account | backend health."""

    provider_label: reactive[str] = reactive("")
    snapshot: reactive[Optional[SnapshotPayload]] = reactive(None)
    backend_health: reactive[Optional[Dict[str, Any]]] = reactive(None)

    def watch_provider_label(self) -> None:
        self._refresh()

    def watch_snapshot(self) -> None:
        self._refresh()

    def watch_backend_health(self) -> None:
        self._refresh()

    def _refresh(self) -> None:
        self.update(
            format_status_line(self.provider_label, self.snapshot, self.backend_health)
        )


class SnapshotDisplay(Static):
    """Widget that displays snapshot data, provider, and backend/service status."""

    snapshot: reactive[Optional[SnapshotPayload]] = reactive(None)
    provider_label: reactive[str] = reactive("")
    backend_health: reactive[Optional[Dict[str, Any]]] = reactive(None)

    def watch_snapshot(self, snapshot: Optional[SnapshotPayload]) -> None:
        self._refresh_display()

    def watch_backend_health(self, _health: Optional[Dict[str, Any]]) -> None:
        self._refresh_display()

    def watch_provider_label(self, _label: str) -> None:
        self._refresh_display()

    def _refresh_display(self) -> None:
        if self.snapshot:
            self.update(self._format_snapshot(self.snapshot))
        else:
            line = "Waiting for data..."
            if self.provider_label:
                line += " | " + _format_provider_label(self.provider_label)
            line += _format_backend_health(self.backend_health)
            self.update(line)

    def _format_snapshot(self, snapshot: SnapshotPayload) -> str:
        parts: list[str] = []
        if self.provider_label:
            parts.append(_format_provider_label(self.provider_label))
        time_str = (
            snapshot.generated_at.split("T")[1].split(".")[0]
            if snapshot.generated_at
            else "--:--:--"
        )
        parts.append(f"Time: {time_str}")
        parts.append(f"Mode: {snapshot.mode}")
        parts.append(f"Strategy: {snapshot.strategy}")
        parts.append(f"Account: {snapshot.account_id}")
        line = " | ".join(parts)
        line += _format_backend_health(self.backend_health)
        return line
