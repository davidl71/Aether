"""Setup screen: show current TUI config, paths, and env overrides."""

from __future__ import annotations

import logging
import os
import socket
import subprocess
import webbrowser
from pathlib import Path
from typing import Optional, Any, Dict, List, Tuple
from urllib.parse import urlparse
from textual.screen import Screen
from textual.containers import Container, Horizontal
from textual.widgets import Header, Footer, Static, Button, DataTable
from textual.binding import Binding

from ..config import TUIConfig, PRESET_REST_ENDPOINTS, DEFAULT_TCP_BACKEND_PORTS, DEFAULT_BACKEND_PORTS
from ..display_utils import format_endpoint_display
from ...integration.shared_config_loader import SharedConfigLoader
from .onepassword_screen import OnePasswordScreen
from .snapshot_display import BACKEND_DISPLAY_NAMES, BACKEND_ROLES

logger = logging.getLogger(__name__)

# Backend key -> provider_type for "Set preferred" (snapshot source)
BACKEND_KEY_TO_PROVIDER_TYPE: Dict[str, str] = {
    "ib": "rest_ib",
    "tws": "rest_tws_gateway",
    "alpaca": "rest_alpaca",
    "tastytrade": "rest_tastytrade",
    "mock": "mock",
    "nats": "nats",
    "file": "file",
}
PROVIDER_TYPE_TO_BACKEND_KEY: Dict[str, str] = {v: k for k, v in BACKEND_KEY_TO_PROVIDER_TYPE.items()}

# Backend key -> scripts/service.sh service name (for start on enable)
BACKEND_KEY_TO_SERVICE_NAME: Dict[str, str] = {
    "ib": "ib",
    "alpaca": "alpaca",
    "tastytrade": "tastytrade",
    "risk_free_rate": "riskfree",
    "discount_bank": "discount",
}


def _pid_for_port(port: int) -> str:
    """Return PID(s) listening on port, or '—' if none or error."""
    try:
        r = subprocess.run(
            ["lsof", "-ti", f":{port}"],
            capture_output=True,
            text=True,
            timeout=2,
        )
        if r.returncode == 0 and r.stdout and r.stdout.strip():
            pids = " ".join(p.strip() for p in r.stdout.strip().split() if p.strip())
            return pids[:20] + ("…" if len(pids) > 20 else "")
    except Exception:
        pass
    return "—"


def _tws_gateway_status(port: int, timeout: float = 2.0) -> str:
    """Return 'reachable' if TWS/Gateway port accepts a connection, else 'unreachable'."""
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=timeout):
            pass
        return "reachable"
    except Exception:
        return "unreachable"


def _running_summary(backend_health: Optional[Dict[str, Any]], display_names: Dict[str, str]) -> str:
    """Return a short 'Running: a, b | Stopped: c' summary from backend_health."""
    if not isinstance(backend_health, dict) or backend_health.get("status") is not None:
        return ""
    running: List[str] = []
    stopped: List[str] = []
    for key, payload in sorted(backend_health.items()):
        if not isinstance(payload, dict):
            continue
        s = (payload.get("status") or "").lower()
        name = display_names.get(key, key.replace("_", " "))
        if s == "ok":
            running.append(name)
        elif s == "error":
            stopped.append(name)
    parts = []
    if running:
        parts.append("Running: " + ", ".join(running))
    if stopped:
        parts.append("Stopped: " + ", ".join(stopped))
    return " | ".join(parts)


def _effective_health_source(config: TUIConfig) -> str:
    """Return short label for where backend health is sourced: dashboard URL or per-backend polling."""
    url = getattr(config, "health_dashboard_url", None)
    if url and (url or "").strip():
        return f"Health: dashboard ({url.strip()})"
    base = getattr(config, "api_base_url", None)
    if base and (base or "").strip():
        health_path = base.strip().rstrip("/") + "/api/health"
        return f"Health: dashboard ({health_path})"
    return "Health: per-backend polling"


def _provides_label(role: str) -> str:
    """Map backend role to 'what it provides' (market data, execution, soft, other)."""
    if role == "trading":
        return "Execution, Market data"
    if role == "market_data":
        return "Market data"
    if role == "banking":
        return "Banking (soft)"
    if role == "rates":
        return "Rates"
    if role == "platform":
        return "Platform (other)"
    return "Other"


# Separate columns for "Provides" (✓ or — per column)
PROVIDE_COLUMNS: Tuple[str, ...] = ("Execution", "Market data", "Banking", "Rates", "Other")


def _provides_cells(role: str) -> Tuple[str, str, str, str, str]:
    """Return (Execution, Market data, Banking, Rates, Other) as '✓' or '—' for the given role."""
    if role == "trading":
        return ("✓", "✓", "—", "—", "—")
    if role == "market_data":
        return ("—", "✓", "—", "—", "—")
    if role == "banking":
        return ("—", "—", "✓", "—", "—")
    if role == "rates":
        return ("—", "—", "—", "✓", "—")
    return ("—", "—", "—", "—", "✓")  # platform, other, snapshot source


def _mode_label(config: TUIConfig, key: str, port: int, health_payload: Optional[Dict[str, Any]] = None) -> str:
    """Return mode label for Setup table. Prefer inferred session_mode from health; else Paper/Live from config/port."""
    if isinstance(health_payload, dict):
        mode = (health_payload.get("session_mode") or "").strip().upper()
        if mode in ("LIVE", "PAPER"):
            return mode
    if key == "tws":
        return "Live (7496)" if port == 7496 else "Paper (7497)"
    if key == "ib":
        # IB: session_mode from health (account ID) when available; else show config hint
        return "—"
    if key == "alpaca":
        paper = getattr(config, "alpaca_paper", None)
        if paper is False:
            return "Live"
        return "Paper"
    return "—"


def _build_provider_table_rows(
    config: TUIConfig,
    backend_health: Optional[Dict[str, Any]] = None,
    current_provider_type: Optional[str] = None,
) -> Tuple[List[Tuple[str, ...]], List[str]]:
    """Build rows with (display, endpoint, enabled, running, pid, other_config, Execution, Market data, Banking, Rates, Other, preferred, mode). Returns (rows, row_keys)."""
    rows: List[Tuple[str, ...]] = []
    row_keys: List[str] = []
    disabled = config.disabled_backends or {}
    user_disabled = set(config.user_disabled_backends or [])
    backend_ports = config.backend_ports or {}
    tcp_ports = config.tcp_backend_ports or {}
    all_http = {**DEFAULT_BACKEND_PORTS, **backend_ports}
    all_tcp = {**DEFAULT_TCP_BACKEND_PORTS, **tcp_ports}
    health_map = backend_health if isinstance(backend_health, dict) and backend_health.get("status") is None else {}
    preferred_key = PROVIDER_TYPE_TO_BACKEND_KEY.get(current_provider_type or "", "")

    def _health_status(key: str, health_map: Dict[str, Any], disabled: Dict[str, str]) -> str:
        """Return Running column label: Running, Disabled, Stopped, Checking, or No API key when enabled but service reports disabled."""
        payload = health_map.get(key)
        if not isinstance(payload, dict):
            return "—"
        s = (payload.get("status") or "").lower()
        err = (payload.get("error") or "").strip()
        if s == "ok":
            return "Running"
        if s == "disabled":
            # Backend is enabled in config but service reports disabled (e.g. no API key at service)
            if key not in disabled:
                if err and "api key" in err.lower():
                    return "No API key"
                if err and "credential" in err.lower():
                    return "No credentials"
                return "Not connected"
            return "Disabled"
        if s == "error":
            return "Stopped"
        if s == "checking":
            return "Checking"
        return "—"

    for key, port in sorted(all_http.items()):
        display = BACKEND_DISPLAY_NAMES.get(key, key.replace("_", " ").title())
        endpoint = format_endpoint_display(f"http://127.0.0.1:{port}")
        if key in disabled:
            enabled = "☐"  # Disabled (credentials)
            other = disabled[key]
        elif key in user_disabled:
            enabled = "☐"  # Disabled by user
            other = "Disabled by user"
        else:
            enabled = "☑"
            other = f"port {port}, /api/health"
        role = BACKEND_ROLES.get(key, "platform")
        provide_cells = _provides_cells(role)
        health = _health_status(key, health_map, disabled)
        pid = _pid_for_port(port)
        preferred = "☑" if key == preferred_key else "☐"
        mode = _mode_label(config, key, port, health_map.get(key))
        rows.append((display, endpoint, enabled, health, pid, other, *provide_cells, preferred, mode))
        row_keys.append(key)

    for key, port in sorted(all_tcp.items()):
        display = BACKEND_DISPLAY_NAMES.get(key, key.replace("_", " ").title())
        endpoint = format_endpoint_display(f"127.0.0.1:{port} (TCP)")
        if key in disabled:
            enabled = "☐"
            other = disabled.get(key, "")
        elif key in user_disabled:
            enabled = "☐"
            other = "Disabled by user"
        else:
            enabled = "☑"
            other = f"port {port}, socket connect"
        role = BACKEND_ROLES.get(key, "platform")
        provide_cells = _provides_cells(role)
        health = _health_status(key, health_map, disabled)
        pid = _pid_for_port(port)
        preferred = "☑" if key == preferred_key else "☐"
        mode = _mode_label(config, key, port, health_map.get(key))
        rows.append((display, endpoint, enabled, health, pid, other, *provide_cells, preferred, mode))
        row_keys.append(key)

    for key, label, endpoint_raw, mode_val in (
        ("mock", "Mock (synthetic)", "—", "Mock"),
        ("nats", "NATS", format_endpoint_display(getattr(config, "nats_url", "nats://localhost:4222")), "—"),
        ("file", "File (JSON)", (config.file_path or "—")[:40] + ("..." if (config.file_path or "") and len(config.file_path or "") > 40 else ""), "—"),
    ):
        preferred = "☑" if key == preferred_key else "☐"
        provide_cells = _provides_cells(BACKEND_ROLES.get(key, "platform"))
        rows.append((label, endpoint_raw, "—", "—", "—", "—", *provide_cells, preferred, mode_val))
        row_keys.append(key)

    return rows, row_keys


class SetupScreen(Screen[Optional[Dict[str, Any]]]):
    """Modal setup screen showing provider, config paths, and env overrides."""

    BINDINGS = [
        Binding("escape", "close", "Close"),
        Binding("q", "close", "Close"),
    ]

    CSS = """
    #setup-body {
        padding: 1 2;
        width: 100%;
        height: 1fr;
    }
    #setup-title {
        margin-bottom: 1;
    }
    .setup-heading {
        color: $accent;
        margin-top: 1;
    }
    .setup-hint {
        color: $text-muted;
        margin: 1 0;
    }
    #switch-section Input {
        width: 1fr;
    }
    #setup-table-section {
        height: 1fr;
        min-height: 6;
    }
    #setup-providers-table {
        height: 1fr;
        scrollbar-size: 1 1;
    }
    .setup-heading {
        color: $accent;
        margin-top: 0;
    }
    .setup-hint {
        color: $text-muted;
        margin: 0 0;
    }
    .setup-line {
        margin: 0 0;
        overflow: hidden;
    }
    """

    def __init__(
        self,
        config: TUIConfig,
        provider_label: str,
        loans_path: str,
        backend_health: Optional[Dict[str, Any]] = None,
    ) -> None:
        super().__init__()
        self._config = config
        self._provider_label = provider_label
        self._loans_path = Path(loans_path)
        self._backend_health = backend_health
        self._table_row_keys: List[str] = []
        self._selected_row_index: Optional[int] = None

    def on_mount(self) -> None:
        table = self.query_one("#setup-providers-table", DataTable)
        table.add_columns(
            "Provider type",
            "Endpoint",
            "Enabled",
            "Running",
            "PID",
            "Other config",
            *PROVIDE_COLUMNS,
            "Preferred",
            "Mode",
        )
        table.cursor_type = "row"
        rows, self._table_row_keys = _build_provider_table_rows(
            self._config, self._backend_health, self._config.provider_type
        )
        for row in rows:
            table.add_row(*row)
        self._update_status_lines()

    def compose(self):
        yield Header(show_clock=False)
        with Container(id="setup-body"):
            yield Static("[bold]Setup[/bold] — Row → Set preferred / Toggle enabled / Switch mode / Restart", id="setup-title")
            with Container(id="setup-table-section"):
                yield DataTable(id="setup-providers-table")
            with Horizontal(classes="setup-buttons"):
                yield Button("Set preferred", id="set-preferred", variant="primary")
                yield Button("Toggle enabled", id="toggle-enabled")
                yield Button("Switch mode", id="switch-mode")
                yield Button("Restart", id="restart-backend")
                yield Button("1Password", id="op-secrets")
                yield Button("Open IB Gateway", id="open-ib-gateway")
                yield Button("Close", id="close")
            yield Static("", id="setup-current-line", classes="setup-line")
            yield Static("", id="setup-gateway-line", classes="setup-line")
            yield Static(
                "[dim]IB: Client Portal and TWS are exclusive — only one can be logged in at a time.[/dim]",
                id="setup-ib-exclusive-hint",
                classes="setup-hint setup-line",
            )
            yield Static("", id="setup-config-line", classes="setup-line")
        yield Footer()

    def _update_status_lines(self) -> None:
        """Populate compact status lines so the screen fits without scrolling."""
        current_parts = [f"Current: {self._provider_label}"]
        # Show inferred mode (PAPER/LIVE) from current backend health when available
        preferred_key = PROVIDER_TYPE_TO_BACKEND_KEY.get(self._config.provider_type or "", "")
        health_map = (
            self._backend_health
            if isinstance(self._backend_health, dict) and self._backend_health.get("status") is None
            else {}
        )
        if health_map:
            payload = health_map.get(preferred_key) if preferred_key else None
            if payload is None:
                payload = health_map.get("current")
            if isinstance(payload, dict):
                mode = (payload.get("session_mode") or "").strip().upper()
                if mode in ("LIVE", "PAPER"):
                    current_parts.append(f"Inferred: {mode}")
        if self._config.provider_type in ("rest", *PRESET_REST_ENDPOINTS):
            endpoint = self._config.rest_endpoint or PRESET_REST_ENDPOINTS.get(self._config.provider_type, "")
            current_parts.append(f"REST: {format_endpoint_display(endpoint)}")
        elif self._config.provider_type == "file" and self._config.file_path:
            current_parts.append(f"File: {self._config.file_path}")
        elif self._config.provider_type == "nats":
            nats_url = getattr(self._config, "nats_url", "nats://localhost:4222")
            current_parts.append(f"NATS: {format_endpoint_display(nats_url)} snapshot.{getattr(self._config, 'nats_snapshot_backend', 'ib')}")
        current_text = " | ".join(current_parts)
        try:
            self.query_one("#setup-current-line", Static).update(
                current_text[:100] + "…" if len(current_text) > 100 else current_text
            )
        except Exception:
            pass
        tws_port = (self._config.tcp_backend_ports or {}).get("tws") or DEFAULT_TCP_BACKEND_PORTS.get("tws", 7497)
        tws_status = _tws_gateway_status(tws_port)
        portal_base = (self._config.ibkr_rest_base_url or "https://localhost:5001/v1/portal").strip().rstrip("/")
        parsed = urlparse(portal_base)
        gateway_url = f"{parsed.scheme}://{parsed.netloc}" if parsed.netloc else "https://localhost:5001"
        tws_display = format_endpoint_display(f"127.0.0.1:{tws_port} (TCP)")
        gateway_display = format_endpoint_display(gateway_url)
        running_summary = _running_summary(self._backend_health, BACKEND_DISPLAY_NAMES)
        try:
            gw = f"TWS {tws_display} — {tws_status} | IB Gateway: {gateway_display}"
            if running_summary:
                gw += " | " + running_summary
            self.query_one("#setup-gateway-line", Static).update(gw[:130] + "…" if len(gw) > 130 else gw)
        except Exception:
            pass
        tui_path = TUIConfig.get_config_path()
        loans_exists = "exists" if self._loans_path.exists() else "not found"
        envs = [f"{k}={os.getenv(k) or '(not set)'}" for k in ("TUI_BACKEND", "TUI_API_URL", "TUI_SNAPSHOT_FILE", "IB_BOX_SPREAD_CONFIG")]
        health_source = _effective_health_source(self._config)
        raw = f"{health_source} | TUI: {tui_path} | Loans: {self._loans_path} ({loans_exists}) | " + " ".join(envs)
        try:
            self.query_one("#setup-config-line", Static).update(
                raw[:150] + "…" if len(raw) > 150 else raw
            )
        except Exception:
            pass

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "close":
            self.dismiss(None)
        elif event.button.id == "op-secrets":
            self.app.push_screen(OnePasswordScreen())
        elif event.button.id == "open-ib-gateway":
            self._open_ib_gateway_login()
        elif event.button.id == "set-preferred":
            self._set_preferred()
        elif event.button.id == "toggle-enabled":
            self._toggle_enabled()
        elif event.button.id == "switch-mode":
            self._switch_mode()
        elif event.button.id == "restart-backend":
            self._restart_backend()

    def _open_ib_gateway_login(self) -> None:
        """Open IB Gateway (Client Portal) login URL in the default browser."""
        portal_base = (self._config.ibkr_rest_base_url or "https://localhost:5001/v1/portal").strip().rstrip("/")
        parsed = urlparse(portal_base)
        gateway_login_url = f"{parsed.scheme}://{parsed.netloc}" if parsed.netloc else "https://localhost:5001"
        try:
            webbrowser.open(gateway_login_url)
            self.notify(f"Opened {gateway_login_url}", title="IB Gateway")
        except Exception as e:
            self.notify(f"Could not open browser: {e}. Open {gateway_login_url} manually.", title="IB Gateway", severity="warning")

    def on_data_table_row_selected(self, event: Any) -> None:
        """Track selected row index for Set preferred / Toggle enabled."""
        try:
            table = self.query_one("#setup-providers-table", DataTable)
            cursor = table.cursor_row
            if cursor is not None and 0 <= cursor < len(self._table_row_keys):
                self._selected_row_index = cursor
        except Exception:
            self._selected_row_index = None

    def _get_selected_key(self) -> Optional[str]:
        if self._selected_row_index is not None and 0 <= self._selected_row_index < len(self._table_row_keys):
            return self._table_row_keys[self._selected_row_index]
        try:
            table = self.query_one("#setup-providers-table", DataTable)
            cursor = table.cursor_row
            if cursor is not None and 0 <= cursor < len(self._table_row_keys):
                return self._table_row_keys[cursor]
        except Exception:
            pass
        return None

    def _set_preferred(self) -> None:
        key = self._get_selected_key()
        if not key:
            self.notify("Select a row first", title="Set preferred", severity="warning")
            return
        ptype = BACKEND_KEY_TO_PROVIDER_TYPE.get(key)
        if not ptype:
            self.notify("Cannot set as snapshot source", title="Set preferred", severity="warning")
            return
        if ptype in PRESET_REST_ENDPOINTS:
            rest_endpoint = PRESET_REST_ENDPOINTS[ptype]
        elif ptype == "nats":
            rest_endpoint = None
        elif ptype == "file":
            rest_endpoint = None
        else:
            rest_endpoint = None
        self.dismiss({
            "provider_type": ptype,
            "rest_endpoint": rest_endpoint if (ptype in PRESET_REST_ENDPOINTS or ptype == "rest") else None,
            "file_path": self._config.file_path if ptype == "file" else None,
            "nats_url": getattr(self._config, "nats_url", "nats://localhost:4222") if ptype == "nats" else None,
            "nats_snapshot_backend": getattr(self._config, "nats_snapshot_backend", "ib") if ptype == "nats" else None,
        })

    def _toggle_enabled(self) -> None:
        key = self._get_selected_key()
        if not key:
            self.notify("Select a row first", title="Toggle enabled", severity="warning")
            return
        if key in ("mock", "nats", "file"):
            self.notify("Enable/disable applies only to backends (IB, Alpaca, etc.)", title="Toggle enabled", severity="information")
            return
        user_disabled = list(self._config.user_disabled_backends or [])
        was_disabled = key in user_disabled
        if key in user_disabled:
            user_disabled.remove(key)
        else:
            user_disabled.append(key)
        self._config.user_disabled_backends = user_disabled
        try:
            self._config.save_to_file(TUIConfig.get_config_path())
        except Exception:
            pass
        self._refresh_table()
        if was_disabled:
            svc_name = BACKEND_KEY_TO_SERVICE_NAME.get(key)
            if svc_name:
                self.run_worker(
                    lambda: self._start_backend_and_notify(svc_name, key),
                    exclusive=False,
                    thread=True,
                )
                return
        self.notify(f"{key}: {'enabled' if key not in user_disabled else 'disabled'}", title="Toggle enabled")

    def _switch_mode(self) -> None:
        """Toggle Live/Paper for selected provider (tws or alpaca); update home config and notify to restart."""
        key = self._get_selected_key()
        if not key:
            self.notify("Select a row first", title="Switch mode", severity="warning")
            return
        if key == "tws":
            current = (self._config.tcp_backend_ports or {}).get("tws") or DEFAULT_TCP_BACKEND_PORTS.get("tws", 7497)
            new_port = 7496 if current == 7497 else 7497
            mode_name = "Live (7496)" if new_port == 7496 else "Paper (7497)"
            try:
                SharedConfigLoader.patch_home_config({"tws": {"port": new_port}})
            except FileNotFoundError as e:
                self.notify(f"Config not found: {e}", title="Switch mode", severity="error")
                return
            except Exception as e:
                self.notify(f"Failed to update config: {e}", title="Switch mode", severity="error")
                return
            if not self._config.tcp_backend_ports:
                self._config.tcp_backend_ports = dict(DEFAULT_TCP_BACKEND_PORTS)
            self._config.tcp_backend_ports["tws"] = new_port
            try:
                self._config.save_to_file(TUIConfig.get_config_path())
            except Exception as e:
                logger.debug("Could not persist TUI config after switch mode: %s", e)
            self._refresh_table()
            self._update_status_lines()
            self.notify(
                f"TWS set to {mode_name}. Restart IB service (and ensure Gateway/TWS is on port {new_port}) to apply.",
                title="Switch mode",
            )
        elif key == "alpaca":
            current = getattr(self._config, "alpaca_paper", True)
            new_paper = not current
            base_url = "https://paper-api.alpaca.markets" if new_paper else "https://api.alpaca.markets"
            mode_name = "Paper" if new_paper else "Live"
            try:
                SharedConfigLoader.patch_home_config({
                    "alpaca": {
                        "data_client_config": {"paper": new_paper, "base_url": base_url},
                        "dataClientConfig": {"paper": new_paper, "baseUrl": base_url},
                    }
                })
            except FileNotFoundError as e:
                self.notify(f"Config not found: {e}", title="Switch mode", severity="error")
                return
            except Exception as e:
                self.notify(f"Failed to update config: {e}", title="Switch mode", severity="error")
                return
            self._config.alpaca_paper = new_paper
            try:
                self._config.save_to_file(TUIConfig.get_config_path())
            except Exception as e:
                logger.debug("Could not persist TUI config after switch mode: %s", e)
            self._refresh_table()
            self._update_status_lines()
            self.notify(
                f"Alpaca set to {mode_name}. Restart Alpaca service to apply.",
                title="Switch mode",
            )
        else:
            self.notify("Switch mode applies only to TWS/IB (7496/7497) or Alpaca (paper/live)", title="Switch mode", severity="information")

    def _restart_backend(self) -> None:
        """Restart the selected backend's service via scripts/service.sh restart."""
        key = self._get_selected_key()
        if not key:
            self.notify("Select a row first", title="Restart", severity="warning")
            return
        svc = BACKEND_KEY_TO_SERVICE_NAME.get(key)
        if not svc:
            self.notify("Restart applies only to backends with a service (IB, Alpaca, etc.)", title="Restart", severity="information")
            return
        self.run_worker(
            lambda: self._run_restart_and_notify(svc, key),
            exclusive=False,
            thread=True,
        )

    def _run_restart_and_notify(self, svc_name: str, backend_key: str) -> None:
        """Run scripts/service.sh restart <svc>; refresh table and notify from main thread."""
        project_root = Path(__file__).resolve().parent.parent.parent.parent
        script = project_root / "scripts" / "service.sh"
        if not script.exists():
            self.app.call_from_thread(
                lambda: self.notify("scripts/service.sh not found", title="Restart", severity="warning")
            )
            return
        try:
            r = subprocess.run(
                [str(script), "restart", svc_name],
                cwd=str(project_root),
                capture_output=True,
                text=True,
                timeout=60,
            )
            self.app.call_from_thread(self._refresh_table)
            if r.returncode == 0:
                self.app.call_from_thread(
                    lambda: self.notify(f"{backend_key} service restarted", title="Restart")
                )
            else:
                err = (r.stderr or r.stdout or "")[:80]
                self.app.call_from_thread(
                    lambda: self.notify(f"Restart failed: {err}", title="Restart", severity="warning")
                )
        except subprocess.TimeoutExpired:
            self.app.call_from_thread(self._refresh_table)
            self.app.call_from_thread(
                lambda: self.notify(f"{backend_key} restart timed out", title="Restart", severity="warning")
            )
        except Exception as e:
            self.app.call_from_thread(self._refresh_table)
            err_msg = str(e)
            self.app.call_from_thread(
                lambda msg=err_msg: self.notify(f"Restart failed: {msg}", title="Restart", severity="warning")
            )

    def _start_backend_and_notify(self, svc_name: str, backend_key: str) -> None:
        """Run scripts/service.sh start <svc>; refresh table and notify from main thread."""
        project_root = Path(__file__).resolve().parent.parent.parent.parent
        script = project_root / "scripts" / "service.sh"
        if not script.exists():
            self.app.call_from_thread(
                lambda: self.notify(f"scripts/service.sh not found; start {backend_key} manually", title="Start backend", severity="warning")
            )
            return
        try:
            r = subprocess.run(
                [str(script), "start", svc_name],
                cwd=str(project_root),
                capture_output=True,
                text=True,
                timeout=30,
            )
            out = (r.stdout or "").strip() + (f" {r.stderr}" if r.stderr else "")
            pid_hint = ""
            if "PID:" in out:
                for part in out.replace(",", " ").split():
                    if part.isdigit() and len(part) < 8:
                        pid_hint = f" (PID: {part})"
                        break
            self.app.call_from_thread(self._refresh_table)
            if r.returncode == 0:
                self.app.call_from_thread(
                    lambda: self.notify(f"{backend_key} started{pid_hint}", title="Start backend")
                )
            else:
                self.app.call_from_thread(
                    lambda: self.notify(f"{backend_key} start failed: {out[:80]}", title="Start backend", severity="warning")
                )
        except subprocess.TimeoutExpired:
            self.app.call_from_thread(self._refresh_table)
            self.app.call_from_thread(
                lambda: self.notify(f"{backend_key} start timed out", title="Start backend", severity="warning")
            )
        except Exception as e:
            self.app.call_from_thread(self._refresh_table)
            err_msg = str(e)
            self.app.call_from_thread(
                lambda msg=err_msg: self.notify(f"{backend_key} start failed: {msg}", title="Start backend", severity="warning")
            )

    def _refresh_table(self) -> None:
        table = self.query_one("#setup-providers-table", DataTable)
        table.clear()
        rows, self._table_row_keys = _build_provider_table_rows(
            self._config, self._backend_health, self._config.provider_type
        )
        for row in rows:
            table.add_row(*row)

    def action_close(self) -> None:
        self.dismiss(None)
