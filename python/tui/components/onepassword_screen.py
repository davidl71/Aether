"""1Password / Secrets status screen: which backends use OP_*_SECRET and whether SDK can resolve."""

from __future__ import annotations

import os
import subprocess
import webbrowser
from typing import Optional, List, Tuple

from textual.screen import Screen
from textual.containers import Container
from textual.widgets import Header, Footer, Static, Button, DataTable
from textual.binding import Binding

# Backend name -> list of (OP_*_SECRET env var name, short label)
BACKEND_SECRET_VARS: List[Tuple[str, List[Tuple[str, str]]]] = [
    ("Alpaca", [
        ("OP_ALPACA_CLIENT_ID_SECRET", "Client ID"),
        ("OP_ALPACA_CLIENT_SECRET_SECRET", "Client Secret"),
        ("OP_ALPACA_API_KEY_ID_SECRET", "API Key ID"),
        ("OP_ALPACA_API_SECRET_KEY_SECRET", "API Secret Key"),
        ("OP_ALPACA_ACCESS_TOKEN_SECRET", "Access Token"),
        ("OP_ALPACA_REFRESH_TOKEN_SECRET", "Refresh Token"),
    ]),
    ("Tastytrade", [
        ("OP_TASTYTRADE_CLIENT_SECRET_SECRET", "Client Secret"),
        ("OP_TASTYTRADE_REFRESH_TOKEN_SECRET", "Refresh Token"),
        ("OP_TASTYTRADE_USERNAME_SECRET", "Username"),
        ("OP_TASTYTRADE_PASSWORD_SECRET", "Password"),
    ]),
    ("TradeStation", [
        ("OP_TRADESTATION_CLIENT_ID_SECRET", "Client ID"),
        ("OP_TRADESTATION_CLIENT_SECRET_SECRET", "Client Secret"),
        ("OP_TRADESTATION_ACCOUNT_ID_SECRET", "Account ID"),
    ]),
    ("FRED (SOFR/Treasury)", [
        ("OP_FRED_API_KEY_SECRET", "API Key"),
    ]),
    ("Alpha Vantage", [
        ("OP_ALPHA_VANTAGE_API_KEY_SECRET", "API Key"),
    ]),
    ("Finnhub", [
        ("OP_FINNHUB_API_KEY_SECRET", "API Key"),
    ]),
    ("JupyterLab", [
        ("OP_JUPYTERLAB_TOKEN_SECRET", "Token"),
        ("OP_JUPYTERLAB_PASSWORD_SECRET", "Password"),
    ]),
]


def _is_op_ref(value: Optional[str]) -> bool:
    """True if value looks like an op:// reference."""
    return bool(value and str(value).strip().startswith("op://"))


def _open_url(url: str) -> bool:
    """Open a URL (e.g. op://) with the system default handler. Returns True if launched."""
    try:
        import platform
        if platform.system() == "Darwin":
            subprocess.run(["open", url], check=False, timeout=5)
            return True
        if platform.system() == "Linux":
            subprocess.run(["xdg-open", url], check=False, timeout=5)
            return True
    except Exception:
        pass
    try:
        webbrowser.open(url)
        return True
    except Exception:
        return False


def _secret_status(env_var: str, try_fred_discovery: bool = False, try_alpha_vantage_discovery: bool = False, try_finnhub_discovery: bool = False) -> Tuple[str, bool]:
    """Return (status_text, is_placeholder) for one OP_*_SECRET.
    Status: 'Set (op://)', 'Set (vault)', 'Not set', or 'Set (plain)'.
    When try_fred_discovery is True and env var is unset, tries get_fred_api_key_from_1password() to show vault detection.
    When try_alpha_vantage_discovery is True and env var is unset, tries get_alpha_vantage_api_key_from_1password().
    When try_finnhub_discovery is True and env var is unset, tries get_finnhub_api_key_from_1password().
    """
    val = os.getenv(env_var, "").strip()
    if not val:
        if try_fred_discovery:
            try:
                from ...integration.onepassword_sdk_helper import get_fred_api_key_from_1password
                if get_fred_api_key_from_1password():
                    return "Set (vault)", False
            except Exception:
                pass
        if try_alpha_vantage_discovery:
            try:
                from ...integration.onepassword_sdk_helper import get_alpha_vantage_api_key_from_1password
                if get_alpha_vantage_api_key_from_1password():
                    return "Set (vault)", False
            except Exception:
                pass
        if try_finnhub_discovery:
            try:
                from ...integration.onepassword_sdk_helper import get_finnhub_api_key_from_1password
                if get_finnhub_api_key_from_1password():
                    return "Set (vault)", False
            except Exception:
                pass
        return "Not set", False
    if _is_op_ref(val):
        # Optionally resolve to detect placeholder (never show real secret values)
        try:
            from ...integration.onepassword_sdk_helper import resolve_secret
            resolved = resolve_secret(val)
            if resolved == "replace-me":
                return "Set (op://)", True
        except Exception:
            pass
        return "Set (op://)", False
    return "Set (plain)", False


def _op_vault_list_ok() -> bool:
    """Return True if op vault list succeeds in this process (token valid for CLI)."""
    token = os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip()
    if not token:
        return False
    env = os.environ.copy()
    env["OP_SERVICE_ACCOUNT_TOKEN"] = token
    env.pop("OP_CONNECT_HOST", None)
    env.pop("OP_CONNECT_TOKEN", None)
    try:
        r = subprocess.run(
            ["op", "vault", "list"],
            env=env,
            capture_output=True,
            timeout=10,
        )
        return r.returncode == 0
    except Exception:
        return False


def _suggested_alpaca_exports() -> Optional[str]:
    """If op is available and vault has an Alpaca item, return copy-paste export lines for OP_ALPACA_*."""
    try:
        r = subprocess.run(
            ["op", "item", "list", "--format=json"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if r.returncode != 0 or not r.stdout:
            return None
        import json
        items = json.loads(r.stdout)
        for it in items:
            title = (it.get("title") or "").strip()
            if "alpaca" not in title.lower():
                continue
            vault = it.get("vault") or {}
            vault_name = vault.get("name") or "trading"
            item_id = it.get("id")
            if not item_id:
                continue
            # Get item details to use actual field labels (e.g. username/credential vs API Key ID/API Secret Key)
            gr = subprocess.run(
                ["op", "item", "get", item_id, "--format=json"],
                capture_output=True,
                text=True,
                timeout=10,
            )
            if gr.returncode != 0 or not gr.stdout:
                break
            detail = json.loads(gr.stdout)
            fields = detail.get("fields") or []
            key_label, secret_label = None, None
            for f in fields:
                lab = (f.get("label") or "").strip()
                purpose = (f.get("purpose") or "").strip()
                if purpose == "USERNAME" or lab in ("username", "API Key ID", "Key ID"):
                    key_label = lab
                elif purpose == "PASSWORD" or (f.get("type") == "CONCEALED" and lab in ("credential", "API Secret Key", "Secret Key", "password")):
                    secret_label = lab
                if key_label and secret_label:
                    break
            key_label = key_label or "username"
            secret_label = secret_label or "credential"
            return (
                f'export OP_ALPACA_API_KEY_ID_SECRET="op://{vault_name}/{title}/{key_label}"\n'
                f'export OP_ALPACA_API_SECRET_KEY_SECRET="op://{vault_name}/{title}/{secret_label}"'
            )
    except Exception:
        pass
    return None


def _suggested_fred_export() -> Optional[str]:
    """If op is available and vault has a FRED item, return the export line for OP_FRED_API_KEY_SECRET."""
    try:
        import json
        r = subprocess.run(
            ["op", "item", "list", "--format=json"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if r.returncode != 0 or not r.stdout:
            return None
        items = json.loads(r.stdout)
        for it in items:
            title = (it.get("title") or "").strip()
            if "fred" not in title.lower():
                continue
            vault = it.get("vault") or {}
            vault_name = vault.get("name") or "trading"
            item_id = it.get("id")
            if not item_id:
                continue
            gr = subprocess.run(
                ["op", "item", "get", item_id, "--format=json"],
                capture_output=True,
                text=True,
                timeout=10,
            )
            if gr.returncode != 0 or not gr.stdout:
                continue
            detail = json.loads(gr.stdout)
            for f in detail.get("fields") or []:
                lab = (f.get("label") or "").strip()
                ref = (f.get("reference") or "").strip()
                if ref.startswith("op://") and (f.get("type") == "CONCEALED" or lab in ("credential", "API Key", "password", "secret")):
                    return f'export OP_FRED_API_KEY_SECRET="op://{vault_name}/{title}/{lab}"'
            return f'export OP_FRED_API_KEY_SECRET="op://{vault_name}/{title}/credential"'
    except Exception:
        pass
    return None


def _build_status_text() -> Tuple[str, str, Optional[str]]:
    """Build (sdk_auth_text, table_text, suggested_exports) for the screen."""
    try:
        from ...integration.onepassword_sdk_helper import sdk_available, client_available
    except ImportError:
        sdk_available = lambda: False  # noqa: E731
        client_available = lambda: False  # noqa: E731

    sdk = "installed" if sdk_available() else "not installed"
    token_set = bool(os.getenv("OP_SERVICE_ACCOUNT_TOKEN", "").strip())
    auth = "ready" if client_available() else "not ready"

    # Hints so user knows why auth is not ready and what to do
    auth_hint = ""
    if not client_available():
        if not token_set and not os.getenv("OP_1PASSWORD_ACCOUNT_NAME", "").strip():
            auth_hint = (
                " — Export token in this shell: source ./scripts/setup_op_service_account.sh  "
                "(then start TUI from the same shell). Best: start TUI with ./scripts/run_python_tui.sh so token and SDK are loaded."
            )
        elif not sdk_available():
            auth_hint = " — Install 1Password SDK: in python/ run uv sync --extra onepassword  (then restart TUI)"
        else:
            # Token set but SDK auth failed — check if CLI works
            if token_set and _op_vault_list_ok():
                auth_hint = " — Token valid (op vault list OK). SDK auth failed — uv sync --extra onepassword, restart TUI."
            else:
                auth_hint = " — Run: op vault list  (then restart TUI from same shell after exporting token)"
    sdk_auth = f"SDK: {sdk}  |  Token: {'set' if token_set else 'not set'}  |  Auth: {auth}{auth_hint}"

    lines: List[str] = []
    for backend_name, vars_list in BACKEND_SECRET_VARS:
        lines.append(f"\n[bold]{backend_name}[/bold]")
        for env_var, label in vars_list:
            try_fred = backend_name.startswith("FRED") and "OP_FRED" in env_var
            try_alpha_vantage = backend_name == "Alpha Vantage" and "OP_ALPHA_VANTAGE" in env_var
            try_finnhub = backend_name == "Finnhub" and "OP_FINNHUB" in env_var
            status, is_placeholder = _secret_status(
                env_var, try_fred_discovery=try_fred, try_alpha_vantage_discovery=try_alpha_vantage, try_finnhub_discovery=try_finnhub
            )
            if status == "Set (op://)":
                hint = " — replace in 1Password app" if is_placeholder else ""
                lines.append(f"  {label}: [green]{status}{hint}[/green]")
            elif status == "Set (vault)":
                lines.append(f"  {label}: [green]{status}[/green]")
            elif status == "Not set":
                lines.append(f"  {label}: [dim]{status}[/dim]")
            else:
                lines.append(f"  {label}: {status}")
    table_text = "\n".join(lines) if lines else "No backends configured."
    suggested_alpaca = _suggested_alpaca_exports()
    suggested_fred = _suggested_fred_export()
    suggested_parts = []
    if suggested_alpaca:
        suggested_parts.append("[bold]Alpaca[/bold]\n[dim]" + suggested_alpaca + "[/dim]")
    if suggested_fred:
        suggested_parts.append("[bold]FRED[/bold]\n[dim]" + suggested_fred + "[/dim]")
    suggested = "\n\n".join(suggested_parts) if suggested_parts else None
    return sdk_auth, table_text, suggested


def _build_table_rows() -> Tuple[List[Tuple[str, str, str, str, str]], List[Optional[str]]]:
    """Build (Backend, Secret, Status, Env var, Open) rows and a list of op ref per row (for open link)."""
    rows: List[Tuple[str, str, str, str, str]] = []
    op_refs: List[Optional[str]] = []
    for backend_name, vars_list in BACKEND_SECRET_VARS:
        for env_var, label in vars_list:
            try_fred = backend_name.startswith("FRED") and "OP_FRED" in env_var
            try_alpha_vantage = backend_name == "Alpha Vantage" and "OP_ALPHA_VANTAGE" in env_var
            try_finnhub = backend_name == "Finnhub" and "OP_FINNHUB" in env_var
            status, is_placeholder = _secret_status(
                env_var,
                try_fred_discovery=try_fred,
                try_alpha_vantage_discovery=try_alpha_vantage,
                try_finnhub_discovery=try_finnhub,
            )
            if status == "Set (op://)" and is_placeholder:
                status = "Set (op://) — replace in 1Password"
            val = os.getenv(env_var, "").strip()
            op_ref = val if _is_op_ref(val) else None
            open_label = "↗ Open" if op_ref else "—"
            rows.append((backend_name, label, status, env_var, open_label))
            op_refs.append(op_ref if op_ref else None)
    return rows, op_refs


class OnePasswordScreen(Screen[None]):
    """Modal screen showing 1Password OP_*_SECRET status and SDK/auth availability."""

    BINDINGS = [
        Binding("escape", "close", "Close"),
        Binding("q", "close", "Close"),
        Binding("enter", "open_in_1password", "Open in 1Password"),
    ]

    CSS = """
    #op-body {
        padding: 1 2;
        width: 100%;
        height: 1fr;
    }
    #op-title {
        margin-bottom: 1;
    }
    #op-table-section {
        height: 1fr;
        min-height: 8;
        max-height: 14;
    }
    #op-secrets-table {
        height: 1fr;
        scrollbar-size: 1 1;
    }
    .op-heading {
        color: $accent;
        margin-top: 1;
    }
    .op-hint {
        color: $text-muted;
        margin: 1 0;
    }
    .op-line {
        margin: 0 0;
        overflow: hidden;
    }
    """

    def compose(self):
        yield Header(show_clock=False)
        with Container(id="op-body"):
            yield Static("[bold]1Password / Secrets[/bold] — OP_*_SECRET status", id="op-title")
            yield Static(
                "Export op:// refs in the shell where you start the TUI. Use [dim]./scripts/setup_op_service_account.sh[/] or [dim]docs/ONEPASSWORD_INTEGRATION.md[/].",
                classes="op-hint op-line",
            )
            yield Static("", id="op-sdk-auth", classes="op-line")
            with Container(id="op-table-section"):
                yield DataTable(id="op-secrets-table")
            yield Static("", id="op-suggested", classes="op-hint op-line")
            with Container():
                yield Button("Refresh", id="refresh", variant="primary")
                yield Button("Open in 1Password", id="open-in-1password")
                yield Button("Close", id="close")
            yield Static(
                "Sign in: 1Password desktop app, or set OP_SERVICE_ACCOUNT_TOKEN. See docs/ONEPASSWORD_INTEGRATION.md.",
                classes="op-hint op-line",
            )
        yield Footer()

    def on_mount(self) -> None:
        table = self.query_one("#op-secrets-table", DataTable)
        table.add_columns("Backend", "Secret", "Status", "Env var", "Open")
        table.cursor_type = "row"
        self._op_refs: List[Optional[str]] = []
        self._refresh_content()

    def _refresh_content(self) -> None:
        sdk_auth, _table_text, suggested = _build_status_text()
        self.query_one("#op-sdk-auth", Static).update(sdk_auth)
        table = self.query_one("#op-secrets-table", DataTable)
        table.clear()
        table.add_columns("Backend", "Secret", "Status", "Env var", "Open")
        rows, op_refs = _build_table_rows()
        self._op_refs = op_refs
        for row in rows:
            table.add_row(*row)
        if suggested:
            self.query_one("#op-suggested", Static).update(
                "[bold]Suggested exports[/] (copy into the shell where you start the TUI):\n\n" + suggested
            )
        else:
            self.query_one("#op-suggested", Static).update("")

    def _open_selected_ref(self) -> bool:
        """Open the op:// ref for the currently selected table row. Returns True if opened."""
        table = self.query_one("#op-secrets-table", DataTable)
        row_index = table.cursor_row
        if row_index is None or row_index < 0 or row_index >= len(self._op_refs):
            self.notify("Select a row with an op:// ref", title="1Password", severity="warning")
            return False
        ref = self._op_refs[row_index]
        if not ref:
            self.notify("No op:// ref for this row", title="1Password", severity="warning")
            return False
        if _open_url(ref):
            self.notify("Opened in default handler (e.g. 1Password)", title="1Password")
            return True
        self.notify("Could not open link", title="1Password", severity="error")
        return False

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "close":
            self.dismiss(None)
        elif event.button.id == "refresh":
            self._refresh_content()
            self.notify("Refreshed", title="1Password")
        elif event.button.id == "open-in-1password":
            self._open_selected_ref()

    def action_open_in_1password(self) -> None:
        """Open the selected row's op:// ref in the system default handler (e.g. 1Password app)."""
        self._open_selected_ref()

    def action_close(self) -> None:
        self.dismiss(None)
