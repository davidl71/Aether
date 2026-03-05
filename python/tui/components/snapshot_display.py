"""Snapshot header widget: provider, time, mode, strategy, account, backend/service status."""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Optional, Dict, Any, Tuple

from textual.widgets import Static
from textual.reactive import reactive

from ..models import SnapshotPayload

# Data older than this (seconds) is shown as stale so user knows it may be outdated
STALE_THRESHOLD_SEC = 60


def format_updated_display(iso_timestamp: str) -> str:
    """Format an ISO timestamp as 'HH:MM:SS (Xs ago)' or '--' if missing. For use in tables/tooltips."""
    if not iso_timestamp or not iso_timestamp.strip():
        return "--"
    try:
        raw = iso_timestamp.strip()
        if "T" in raw:
            time_part = raw.split("T")[1].split(".")[0]
        else:
            return raw[:8] if len(raw) >= 8 else raw
        if raw.endswith("Z"):
            raw = raw[:-1] + "+00:00"
        dt = datetime.fromisoformat(raw)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        age_sec = (datetime.now(timezone.utc) - dt).total_seconds()
        if age_sec < 0:
            age_sec = 0
        if age_sec >= 3600:
            return f"{time_part} ({int(age_sec / 3600)}h ago)"
        if age_sec >= 60:
            return f"{time_part} ({int(age_sec)}s ago)"
        if age_sec >= 1:
            return f"{time_part} ({int(age_sec)}s ago)"
        return f"{time_part} (now)"
    except Exception:
        return "--"


# Friendly display names for backend health status line (key from backend_ports)
BACKEND_DISPLAY_NAMES: Dict[str, str] = {
    "ib": "TWS/IBKR",
    "tws": "TWS",
    "alpaca": "Alpaca",
    "tastytrade": "Tastytrade",
    "tradestation": "TradeStation",
    "discount_bank": "Discount Bank",
    "risk_free_rate": "Risk-Free Rate",
    "rust": "Rust",
}

# One-word (or short) labels for status bar pills, PWA-style (key -> pill text)
BACKEND_SHORT_LABELS: Dict[str, str] = {
    "ib": "IB",
    "tws": "TWS",
    "alpaca": "Alpaca",
    "tastytrade": "Tasty",
    "tradestation": "TS",
    "discount_bank": "Discount",
    "risk_free_rate": "RFR",
    "rust": "Rust",
    "current": "Service",
    "connection": "Conn",
}

# Environment badge: mock = synthetic data, paper = DRY-RUN, live = real money
ENVIRONMENT_LABELS: Dict[str, str] = {
    "mock": "MOCK",
    "paper": "PAPER",
    "live": "LIVE",
}
ENVIRONMENT_MARKUP: Dict[str, str] = {
    "mock": "[bold cyan on #2d3d4d] MOCK [/]",
    "paper": "[bold yellow on #4d4d2d] PAPER [/]",
    "live": "[bold white on #8b2020] LIVE [/]",
}


def _snapshot_time_and_stale(snapshot: Optional[SnapshotPayload]) -> Tuple[str, str]:
    """Return (time_str, age_suffix). age_suffix is e.g. ' (2s ago)', ' (1m ago)', ' (stale)'."""
    if not snapshot or not snapshot.generated_at:
        return "--:--:--", ""
    try:
        raw = snapshot.generated_at.strip()
        if "T" in raw:
            time_part = raw.split("T")[1].split(".")[0]
        else:
            time_part = "--:--:--"
        if raw.endswith("Z"):
            raw = raw[:-1] + "+00:00"
        dt = datetime.fromisoformat(raw)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        age_sec = (datetime.now(timezone.utc) - dt).total_seconds()
        if age_sec < 0:
            age_sec = 0
        if age_sec >= 3600:
            age_suffix = f" ({int(age_sec / 3600)}h ago)"
        elif age_sec >= 60:
            age_suffix = f" ({int(age_sec)}s ago)"
        elif age_sec > STALE_THRESHOLD_SEC:
            age_suffix = " (stale)"
        elif age_sec >= 1:
            age_suffix = f" ({int(age_sec)}s ago)"
        else:
            age_suffix = " (now)"
        return time_part, age_suffix
    except Exception:
        return (
            snapshot.generated_at.split("T")[1].split(".")[0]
            if "T" in snapshot.generated_at
            else "--:--:--",
            "",
        )


def _format_one_backend_health(name: str, health: dict) -> str:
    """Format a single backend's health for status line. IB status and gateway are grouped."""
    status = health.get("status", "unknown")
    ib_connected = health.get("ib_connected", False)
    gateway_logged_in = health.get("gateway_logged_in", ib_connected)
    gateway_port = health.get("gateway_port")
    label = BACKEND_DISPLAY_NAMES.get(name.lower(), name) if name != "current" else "Service"
    if name == "connection":
        label = "Connection"
    if status == "ok":
        # Group IB status + Gateway into one segment: "TWS/IBKR: ok (Gateway 5001: logged in)"
        if gateway_port is not None and name.lower() == "ib":
            gw_str = "logged in" if gateway_logged_in else "not logged in"
            return f"{label}: ok (Gateway {gateway_port}: {gw_str})"
        # TCP-only backends (e.g. TWS Gateway) have no ib_connected
        if "ib_connected" not in health:
            return f"{label}: ok"
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
    hint = health.get("hint")
    if hint:
        return f"{label}: unreachable ({err}) — {hint}"
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
            return "Backend: " + " | ".join(parts)
    # Legacy single-dict shape (status, ib_connected, error)
    if isinstance(health, dict) and health.get("status") is not None:
        return "Backend: " + _format_one_backend_health("Service", health)
    return ""


def _backend_pills_rich(health: Optional[Dict[str, Any]]):
    """
    Build Rich Text of one-word colored backend pills (PWA-style).
    ok -> green, disabled/checking -> yellow, error -> red.
    Returns a Rich Text or empty string.
    """
    from rich.text import Text
    if not health or not isinstance(health, dict):
        return Text()
    # Legacy single-dict shape (status, ib_connected, error) -> one "Service" pill
    if health.get("status") is not None:
        status = health.get("status", "unknown")
        label = BACKEND_SHORT_LABELS.get("current", "Service")
        if status == "ok":
            return Text(f" {label} ", style="bold green")
        if status in ("disabled", "checking"):
            return Text(f" {label} ", style="bold yellow")
        return Text(f" {label} ", style="bold red")
    # New shape: dict of backend name -> health payload
    out = Text()
    for name, payload in sorted(health.items()):
        if not isinstance(payload, dict):
            continue
        status = payload.get("status", "unknown")
        label = BACKEND_SHORT_LABELS.get(name.lower(), name)
        if status == "ok":
            out.append(f" {label} ", style="bold green")
        elif status in ("disabled", "checking"):
            out.append(f" {label} ", style="bold yellow")
        else:
            out.append(f" {label} ", style="bold red")
    return out


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
    """Build the full status bar line (plain string). Used for fallback and QA."""
    parts: list[str] = []
    backend_str = _format_backend_health(backend_health)
    if backend_str.strip():
        parts.append(backend_str.strip().lstrip(" |"))
    if provider_label:
        parts.append(_format_provider_label(provider_label))
    if snapshot:
        time_str, age_suffix = _snapshot_time_and_stale(snapshot)
        parts.append(f"Updated: {time_str}{age_suffix}")
        parts.append(f"Mode: {snapshot.mode}")
        parts.append(f"Strategy: {snapshot.strategy}")
        parts.append(f"Account: {snapshot.account_id}")
    else:
        parts.append("Updated: --:--:--")
        parts.append("Mode: --")
        parts.append("Strategy: --")
        parts.append("Account: --")
    line = " | ".join(parts)
    return line or "Waiting for data..."


def format_status_line_rest(provider_label: str, snapshot: Optional[SnapshotPayload]) -> str:
    """Build the right-hand part of the status line (Provider | Updated | Mode | Strategy | Account)."""
    parts: list[str] = []
    if provider_label:
        parts.append(_format_provider_label(provider_label))
    if snapshot:
        time_str, age_suffix = _snapshot_time_and_stale(snapshot)
        parts.append(f"Updated: {time_str}{age_suffix}")
        parts.append(f"Mode: {snapshot.mode}")
        parts.append(f"Strategy: {snapshot.strategy}")
        parts.append(f"Account: {snapshot.account_id}")
    else:
        parts.append("Updated: --:--:--")
        parts.append("Mode: --")
        parts.append("Strategy: --")
        parts.append("Account: --")
    return " | ".join(parts)


def get_environment(provider: Any, snapshot: Optional[SnapshotPayload]) -> str:
    """Return 'mock' | 'paper' | 'live' for status bar styling and badge."""
    from ..providers import MockProvider
    if isinstance(provider, MockProvider):
        return "mock"
    mode = (snapshot.mode or "").strip().upper() if snapshot else ""
    return "live" if mode == "LIVE" else "paper"


class StatusBar(Static):
    """Bottom status line: [MOCK|PAPER|LIVE] badge, colored one-word backend pills (PWA-style), Provider | Updated | Mode | Strategy | Account."""

    provider_label: reactive[str] = reactive("")
    snapshot: reactive[Optional[SnapshotPayload]] = reactive(None)
    backend_health: reactive[Optional[Dict[str, Any]]] = reactive(None)
    environment: reactive[str] = reactive("")  # mock | paper | live for badge and bar colour

    def watch_provider_label(self) -> None:
        self._refresh()

    def watch_snapshot(self) -> None:
        self._refresh()

    def watch_backend_health(self) -> None:
        self._refresh()

    def watch_environment(self) -> None:
        self._refresh()
        self._update_mode_class()

    def _update_mode_class(self) -> None:
        for cls in ("mode-mock", "mode-paper", "mode-live"):
            self.remove_class(cls)
        if self.environment:
            self.add_class(f"mode-{self.environment}")

    def _refresh(self) -> None:
        from rich.text import Text
        rest_str = format_status_line_rest(
            self.provider_label, self.snapshot
        )
        pills = _backend_pills_rich(self.backend_health)
        if self.environment and self.environment in ENVIRONMENT_MARKUP:
            badge = Text.from_markup(ENVIRONMENT_MARKUP[self.environment])
            if pills:
                content = badge + Text(" ") + pills + Text(" | " + rest_str)
            else:
                content = badge + Text(" ") + rest_str
        elif pills:
            content = pills + Text(" | " + rest_str)
        else:
            content = rest_str
        self.update(content)


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
        time_str, age_suffix = _snapshot_time_and_stale(snapshot)
        parts.append(f"Updated: {time_str}{age_suffix}")
        parts.append(f"Mode: {snapshot.mode}")
        parts.append(f"Strategy: {snapshot.strategy}")
        parts.append(f"Account: {snapshot.account_id}")
        line = " | ".join(parts)
        line += _format_backend_health(self.backend_health)
        return line
