"""
Main TUI application using Textual

This is the Python replacement for the C++ TUI (native/src/tui_app.cpp).
It provides the same functionality with better performance and easier maintenance.

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- UI rendering logic can stay in Python (Textual is Python-only)
- Data processing can be moved to C++ and exposed via pybind11
- Consider keeping Python TUI as reference implementation
- Provider classes can be C++ implementations wrapped in Python
- Keyboard shortcuts and event handling can remain in Python
"""

from __future__ import annotations

import logging
import os
import sys
from pathlib import Path
from typing import Optional, List, Dict, Any

from textual.app import App, ComposeResult
from textual.containers import Container
from textual.widgets import (
    Header,
    Footer,
    TabbedContent,
    TabPane,
    Log,
)
from textual.binding import Binding

from .models import SnapshotPayload, BoxSpreadPayload
from .providers import Provider, MockProvider, RestProvider, FileProvider, NatsProvider, BackendHealthAggregator
from .config import TUIConfig, load_config, PRESET_REST_ENDPOINTS
from .box_spread_loader import get_box_spread_payload
from .log_handler import install_tui_log_handler, remove_tui_log_handler, drain_log_queue, get_buffered_log_lines
from .components.snapshot_display import StatusBar, get_environment
from .components.dashboard import DashboardTab
from .components.positions import PositionsTab
from .components.orders import OrdersTab
from .components.alerts import AlertsTab
from .components.scenarios import ScenariosTab
from .components.historic import HistoricTab
from .components.unified_positions import UnifiedPositionsTab
from .components.cash_flow import CashFlowTab
from .components.opportunity_simulation import OpportunitySimulationTab
from .components.relationship_visualization import RelationshipVisualizationTab
from .components.loan_entry import LoanListTab, LoanManager
from .components.setup_screen import SetupScreen
from .components.onepassword_screen import OnePasswordScreen
from .components.logs_tab import LogsTab
from .components.benchmarks_tab import BenchmarksTab
from .components.brokers_tab import BrokersTab

logger = logging.getLogger(__name__)

# Codified tab IDs for programmatic switching (e.g. QA screenshot per screen).
# Order matches TabbedContent panes; use switch_to_tab(tab_id) to display without user interaction.
TUI_TAB_IDS: List[str] = [
    "dashboard-tab",
    "brokers-tab",
    "unified-positions-tab",
    "cash-flow-tab",
    "simulation-tab",
    "relationships-tab",
    "positions-tab",
    "historic-tab",
    "orders-tab",
    "alerts-tab",
    "scenarios-tab",
    "loans-tab",
    "benchmarks-tab",
    "logs-tab",
]


class TUIApp(App):
    """
    Main TUI application

    MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
    - This class structure can be mirrored in C++ using FTXUI
    - Provider can be C++ class exposed via pybind11
    - Consider keeping Python version as reference during migration
    - Event handling and UI updates can remain in Python
    """

    CSS = """
    Screen {
        background: $surface;
    }

    /* Theme when using mock (synthetic) data source: slate/cyan tint to distinguish from live */
    Screen.theme-mock {
        background: #1a2332;
    }
    Screen.theme-mock #main-container {
        background: #1a2332;
    }
    Screen.theme-mock #status-bar {
        background: #252d3d;
        border-top: tall #3d5a6c;
    }
    Screen.theme-mock .tab-title {
        color: #6b9fb5;
    }
    Screen.theme-mock Header {
        background: #252d3d;
        color: #8fadc0;
    }
    Screen.theme-mock Footer {
        background: #252d3d;
        color: #8fadc0;
    }

    /* Theme for PAPER (dry-run / paper trading): amber/gold tint */
    Screen.theme-paper {
        background: #2a2a1e;
    }
    Screen.theme-paper #main-container {
        background: #2a2a1e;
    }
    Screen.theme-paper #status-bar {
        background: #353520;
        border-top: tall #6c6c3d;
    }
    Screen.theme-paper .tab-title {
        color: #b8a84a;
    }
    Screen.theme-paper Header {
        background: #353520;
        color: #c4b86a;
    }
    Screen.theme-paper Footer {
        background: #353520;
        color: #c4b86a;
    }

    /* Theme for LIVE (real money): red/danger tint */
    Screen.theme-live {
        background: #2a1e1e;
    }
    Screen.theme-live #main-container {
        background: #2a1e1e;
    }
    Screen.theme-live #status-bar {
        background: #352020;
        border-top: tall #8b4040;
    }
    Screen.theme-live .tab-title {
        color: #c48484;
    }
    Screen.theme-live Header {
        background: #352020;
        color: #d4a0a0;
    }
    Screen.theme-live Footer {
        background: #352020;
        color: #d4a0a0;
    }

    /* Use full terminal: main content area fills between header and status/footer */
    #main-container {
        width: 100%;
        height: 1fr;
        overflow: hidden;
    }

    TabbedContent {
        width: 100%;
        height: 1fr;
    }

    /* Compact tab bar: show more tabs when terminal is wide */
    #tabs #tabs-list > * {
        padding: 0 1;
        min-width: 0;
    }

    TabbedContent > Vertical {
        width: 100%;
        height: 1fr;
    }

    /* Tab content panes fill available space */
    .tab-content-fill {
        width: 100%;
        height: 1fr;
        overflow: auto;
    }

    /* Inner scroll/fill container so table/log takes remaining space */
    .tab-content-fill .fill {
        width: 100%;
        height: 1fr;
    }

    #status-bar {
        dock: bottom;
        height: 1;
        padding: 0 1;
        background: $surface-darken-2;
        color: $text;
    }
    #status-bar > Horizontal {
        width: 100%;
        height: auto;
    }
    #status-badge {
        width: auto;
        text-style: bold;
        color: $primary;
    }
    #status-pills {
        width: auto;
        height: auto;
    }
    #status-rest {
        width: 1fr;
        min-width: 20;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .status-pill {
        width: auto;
        text-style: bold;
        padding: 0 0;
        margin: 0 0;
    }
    .status-pill-ok {
        color: $text-success;
    }
    .status-pill-warn {
        color: $text-warning;
    }
    .status-pill-err {
        color: $text-error;
    }
    .status-pill-group {
        color: $text-muted;
        text-style: italic;
    }
    #status-bar.mode-mock {
        background: $primary-darken-3;
        border-top: tall $primary;
    }
    #status-bar.mode-paper {
        background: #3d3d20;
        border-top: tall $warning;
    }
    #status-bar.mode-live {
        background: #3d2020;
        border-top: tall $error;
    }

    .tab-title {
        text-style: bold;
        color: $accent;
        margin: 1;
    }

    /* Cash flow tab: projection period dropdown box size */
    #projection-select {
        width: 8;
        min-width: 8;
    }

    /* Tables and main content expand to use space */
    #symbols-table, #positions-table, #scenarios-table,
    #brokers-table,
    #cash-flow-table, #opportunity-scenarios-table, #relationships-table,
    #loans-table {
        width: 100%;
        height: 1fr;
        min-height: 5;
    }

    #positions-container {
        width: 100%;
        height: 1fr;
        overflow: auto;
        min-height: 5;
    }

    #orders-log, #alerts-log, #tui-log, .logs-container {
        width: 100%;
        height: 1fr;
        min-height: 5;
    }

    #metrics-label {
        margin: 1;
        text-style: dim;
    }

    .position-group-header {
        text-style: bold;
        color: $accent;
        margin: 1 0;
    }

    #action-items-label.warning {
        color: $warning;
        text-style: bold;
        margin: 1;
        padding: 1;
        background: $surface-darken-1;
    }

    .logs-tab-header {
        color: $text-muted;
        margin: 0 0 1 0;
    }
    """

    TITLE = "IB Box Spread Terminal"
    BINDINGS = [
        Binding("q", "quit", "Quit"),
        Binding("f1", "help", "Help", key_display="F1"),
        Binding("f2", "setup", "Setup", key_display="F2"),
        Binding("f3", "op_secrets", "1Password", key_display="F3"),
        Binding("f5", "refresh", "Refresh", key_display="F5"),
        Binding("f10", "quit", "Quit", key_display="F10"),
    ]

    def __init__(
        self,
        provider: Provider,
        config: Optional[TUIConfig] = None,
        tui_log_handler: Optional[logging.Handler] = None,
        tui_log_handler_on_root: bool = False,
    ):
        super().__init__()
        self.provider = provider
        self.config = config or TUIConfig()
        self._tui_log_handler = tui_log_handler
        self._tui_log_handler_on_root = tui_log_handler_on_root
        self.snapshot: Optional[SnapshotPayload] = None
        self.box_spread_data: Optional[BoxSpreadPayload] = None
        self._dashboard_tab: Optional[DashboardTab] = None
        self._unified_positions_tab: Optional[UnifiedPositionsTab] = None
        self._cash_flow_tab: Optional[CashFlowTab] = None
        self._opportunity_simulation_tab: Optional[OpportunitySimulationTab] = None
        self._relationship_visualization_tab: Optional[RelationshipVisualizationTab] = None
        self._positions_tab: Optional[PositionsTab] = None
        self._orders_tab: Optional[OrdersTab] = None
        self._alerts_tab: Optional[AlertsTab] = None
        self._scenarios_tab: Optional[ScenariosTab] = None
        self._historic_tab: Optional[HistoricTab] = None
        self._loan_tab: Optional[LoanListTab] = None
        self._logs_tab: Optional[LogsTab] = None
        self._brokers_tab: Optional[BrokersTab] = None
        self._logs_buffer_loaded = False
        self._box_spread_file_path = Path("web/public/data/box_spread_sample.json")
        self._last_box_spread_mtime: Optional[float] = None
        self._bank_accounts: List[Dict] = []
        self._loan_manager = LoanManager("config/loans.json")
        self._backend_health_aggregator: Optional[BackendHealthAggregator] = None

    def compose(self) -> ComposeResult:
        """Create child widgets for the app"""
        self._apply_theme_for_provider()
        yield Header(show_clock=True)

        with Container(id="main-container"):
            with TabbedContent(id="tabs"):
                with TabPane("Dash", id="dashboard-tab"):
                    self._dashboard_tab = DashboardTab(
                        classes="tab-content-fill",
                        watchlist=getattr(self.config, "watchlist", None),
                    )
                    yield self._dashboard_tab

                with TabPane("Brokers", id="brokers-tab"):
                    self._brokers_tab = BrokersTab(classes="tab-content-fill")
                    yield self._brokers_tab

                with TabPane("Unified", id="unified-positions-tab"):
                    self._unified_positions_tab = UnifiedPositionsTab(classes="tab-content-fill")
                    yield self._unified_positions_tab

                with TabPane("Cash", id="cash-flow-tab"):
                    self._cash_flow_tab = CashFlowTab(classes="tab-content-fill")
                    yield self._cash_flow_tab

                with TabPane("Sim", id="simulation-tab"):
                    self._opportunity_simulation_tab = OpportunitySimulationTab(classes="tab-content-fill")
                    yield self._opportunity_simulation_tab

                with TabPane("Rels", id="relationships-tab"):
                    self._relationship_visualization_tab = RelationshipVisualizationTab(classes="tab-content-fill")
                    yield self._relationship_visualization_tab

                with TabPane("Pos", id="positions-tab"):
                    self._positions_tab = PositionsTab(classes="tab-content-fill")
                    yield self._positions_tab

                with TabPane("Hist", id="historic-tab"):
                    self._historic_tab = HistoricTab(classes="tab-content-fill")
                    yield self._historic_tab

                with TabPane("Orders", id="orders-tab"):
                    self._orders_tab = OrdersTab(classes="tab-content-fill")
                    yield self._orders_tab

                with TabPane("Alerts", id="alerts-tab"):
                    self._alerts_tab = AlertsTab(classes="tab-content-fill")
                    yield self._alerts_tab

                with TabPane("Scen", id="scenarios-tab"):
                    self._scenarios_tab = ScenariosTab(classes="tab-content-fill")
                    yield self._scenarios_tab

                with TabPane("Loans", id="loans-tab"):
                    self._loan_tab = LoanListTab(self._loan_manager, classes="tab-content-fill")
                    yield self._loan_tab

                with TabPane("Rates", id="benchmarks-tab"):
                    self._benchmarks_tab = BenchmarksTab(classes="tab-content-fill")
                    yield self._benchmarks_tab

                with TabPane("Logs", id="logs-tab"):
                    self._logs_tab = LogsTab(max_lines=500, classes="tab-content-fill")
                    yield self._logs_tab

        yield StatusBar(id="status-bar")
        yield Footer()

    def _apply_theme_for_provider(self) -> None:
        """Apply theme class and title badge from current provider and snapshot (mock | paper | live)."""
        env = get_environment(self.provider, self.snapshot)
        self._apply_theme_for_environment(env)

    def _apply_theme_for_environment(self, environment: str) -> None:
        """Apply theme class and title badge for environment (mock | paper | live)."""
        for cls in ("theme-mock", "theme-paper", "theme-live"):
            self.screen.remove_class(cls)
        if environment in ("mock", "paper", "live"):
            self.screen.add_class(f"theme-{environment}")
        if environment:
            self.title = f"{self.TITLE}  [{environment.upper()}]"
        else:
            self.title = self.TITLE

    def on_mount(self) -> None:
        """Called when app starts"""
        if getattr(self, "_config_last_mtimes", None) is None:
            self._config_last_mtimes: Dict[str, float] = {}
        if self._tui_log_handler is None:
            self._tui_log_handler = install_tui_log_handler(level=logging.DEBUG)
            self._tui_log_handler_on_root = False
        # Ensure we always have backend ports so status line shows backend status (merge defaults if empty)
        from .config import DEFAULT_BACKEND_PORTS, DEFAULT_TCP_BACKEND_PORTS
        if not self.config.backend_ports:
            self.config.backend_ports = dict(DEFAULT_BACKEND_PORTS)
        else:
            self.config.backend_ports = {**DEFAULT_BACKEND_PORTS, **self.config.backend_ports}
        if not self.config.tcp_backend_ports:
            self.config.tcp_backend_ports = dict(DEFAULT_TCP_BACKEND_PORTS)
        else:
            self.config.tcp_backend_ports = {**DEFAULT_TCP_BACKEND_PORTS, **self.config.tcp_backend_ports}
        self._apply_theme_for_provider()
        self.provider.start()
        if self.config.backend_ports or self.config.tcp_backend_ports:
            self._backend_health_aggregator = BackendHealthAggregator(
                self.config.backend_ports,
                tcp_backend_ports=self.config.tcp_backend_ports,
                unified_health_url=_effective_health_url(self.config),
            )
            self._backend_health_aggregator.start()
        self.set_interval(0.5, self._update_snapshot)  # Update every 500ms
        self.set_interval(0.25, self._drain_tui_logs)  # Drain log queue for Logs tab
        self.set_interval(2.0, self._update_box_spread_data)  # Update every 2 seconds
        self.set_interval(30.0, self._fetch_bank_accounts)  # Update bank accounts every 30 seconds
        self.set_interval(3.0, self._check_config_reload)  # T-114: config file watch (hot-reload)
        self._fetch_bank_accounts()  # Initial fetch
        # Seed taskbar status bar with placeholder pills so indicators show immediately
        self.call_next(self._seed_status_bar_pills)
        logger.info("TUI application mounted")

    def _seed_status_bar_pills(self) -> None:
        """Set placeholder backend_health on the status bar so pills show in the taskbar before first aggregator result."""
        try:
            status_bar = self.query_one("#status-bar", StatusBar)
            if getattr(status_bar, "backend_health", None) is not None:
                return  # Already set by _update_snapshot
            placeholder: Dict[str, Dict[str, Any]] = {}
            for name in list(self.config.backend_ports.keys()) + list(getattr(self.config, "tcp_backend_ports", {}).keys()):
                placeholder[name] = {"status": "checking", "error": "..."}
            for name, reason in getattr(self.config, "disabled_backends", {}).items():
                placeholder[name] = {"status": "disabled", "error": reason}
            if not placeholder:
                placeholder["connection"] = {"status": "checking", "error": "..."}
            status_bar.backend_health = placeholder
            status_bar._refresh()
        except Exception as e:
            logger.debug("Seed status bar pills: %s", e)

    def on_unmount(self) -> None:
        """Called when app exits"""
        if self._backend_health_aggregator:
            self._backend_health_aggregator.stop()
            self._backend_health_aggregator = None
        if self._tui_log_handler:
            remove_tui_log_handler(self._tui_log_handler, from_root=self._tui_log_handler_on_root)
            self._tui_log_handler = None
        self.provider.stop()
        logger.info("TUI application unmounted")

    def _get_provider_label(self) -> str:
        """Return short label for current data provider and endpoint (e.g. 'rest (8002 HTTP)', 'mock')."""
        from urllib.parse import urlparse
        from .display_utils import format_endpoint_display
        if isinstance(self.provider, RestProvider):
            try:
                endpoint = self.provider.endpoint
                short = format_endpoint_display(endpoint)
                return f"rest ({short})"
            except Exception:
                return "rest"
        if isinstance(self.provider, FileProvider):
            path = getattr(self.provider, "file_path", None)
            name = path.name if path else "file"
            return f"file ({name})"
        if isinstance(self.provider, NatsProvider):
            return f"nats (snapshot.{self.provider.snapshot_backend})"
        return "mock"

    def switch_to_tab(self, tab_id: str) -> None:
        """Switch to a tab by id without user interaction. Use TUI_TAB_IDS for valid ids."""
        if tab_id not in TUI_TAB_IDS:
            raise ValueError(f"Unknown tab_id: {tab_id}. Known: {TUI_TAB_IDS}")
        tabs = self.query_one("#tabs", TabbedContent)
        tabs.active = tab_id

    def get_active_tab_id(self) -> str:
        """Return the id of the currently visible tab."""
        tabs = self.query_one("#tabs", TabbedContent)
        return tabs.active or TUI_TAB_IDS[0]

    def on_tabbed_content_tab_activated(self, event: TabbedContent.TabActivated) -> None:
        """When user switches to Logs tab, refresh with current buffer and queue so logs display."""
        if getattr(event.control, "active", None) == "logs-tab":
            self.call_next(self._refresh_logs_tab)

    def _update_snapshot(self) -> None:
        """Update snapshot from provider. Never raises: on backend loss we show unhealthy and retry."""
        try:
            new_snapshot = self.provider.get_snapshot()
            status_bar = self.query_one("#status-bar", StatusBar)
            status_bar.provider_label = self._get_provider_label()
            status_bar.environment = get_environment(self.provider, new_snapshot)
            self._apply_theme_for_environment(status_bar.environment)
            # All backend health: from aggregator if present, else single backend from current provider
            all_health: Dict[str, Dict[str, Any]] = {}
            if self._backend_health_aggregator:
                all_health = self._backend_health_aggregator.get_all_health()
            # Overlay disabled backends (e.g. missing API keys) so TUI shows "disabled" instead of unreachable
            for name, reason in getattr(self.config, "disabled_backends", {}).items():
                all_health[name] = {"status": "disabled", "error": reason}
            if not all_health and hasattr(self.provider, "get_health"):
                h = self.provider.get_health()
                if h is not None:
                    all_health["current"] = h
            # When aggregator is running but no data yet, show "checking" for each configured backend so taskbar pills appear
            if not all_health and self._backend_health_aggregator:
                for name in list(self.config.backend_ports.keys()) + list(getattr(self.config, "tcp_backend_ports", {}).keys()):
                    all_health[name] = {"status": "checking", "error": "..."}
            status_bar.backend_health = all_health if all_health else None
            status_bar._refresh()
            if new_snapshot != self.snapshot:
                self.snapshot = new_snapshot
                status_bar.snapshot = new_snapshot

                # Update tabs
                if self._dashboard_tab:
                    self._dashboard_tab.update_snapshot(new_snapshot, backend_health=all_health or None)
                if self._brokers_tab:
                    self._brokers_tab.update_snapshot(
                        new_snapshot,
                        backend_health=all_health or None,
                        current_provider_type=getattr(self.config, "provider_type", None),
                    )
                if self._unified_positions_tab:
                    self._unified_positions_tab.update_snapshot(new_snapshot, self._bank_accounts)
                if self._cash_flow_tab:
                    self._cash_flow_tab.update_snapshot(new_snapshot, self._bank_accounts)
                if self._opportunity_simulation_tab:
                    self._opportunity_simulation_tab.update_snapshot(new_snapshot, self._bank_accounts)
                if self._relationship_visualization_tab:
                    self._relationship_visualization_tab.update_snapshot(new_snapshot, self._bank_accounts)
                if self._positions_tab:
                    self._positions_tab.update_snapshot(new_snapshot)
                if self._orders_tab:
                    self._orders_tab.update_snapshot(new_snapshot)
                if self._alerts_tab:
                    self._alerts_tab.update_snapshot(new_snapshot)
                if self._historic_tab:
                    self._historic_tab.update_snapshot(new_snapshot)
        except Exception as e:
            logger.debug("Snapshot update error (backends may be restarting): %s", e)
            try:
                status_bar = self.query_one("#status-bar", StatusBar)
                # Keep last snapshot; mark connection as unhealthy so UI shows retrying
                all_health = {}
                if self._backend_health_aggregator:
                    all_health = self._backend_health_aggregator.get_all_health()
                for name, reason in getattr(self.config, "disabled_backends", {}).items():
                    all_health[name] = {"status": "disabled", "error": reason}
                if not all_health:
                    for name in list(self.config.backend_ports.keys()) + list(getattr(self.config, "tcp_backend_ports", {}).keys()):
                        all_health[name] = {"status": "error", "error": "retrying…", "hint": "Backend may be restarting"}
                status_bar.backend_health = all_health or None
                status_bar._refresh()
                if self._brokers_tab:
                    self._brokers_tab.update_snapshot(
                        self.snapshot,
                        backend_health=all_health or None,
                        current_provider_type=getattr(self.config, "provider_type", None),
                    )
            except Exception:
                pass

    def _drain_tui_logs(self) -> None:
        """Drain TUI log queue and append to Logs tab. Uses app-level query so the Log widget is found when the tab is in the DOM."""
        if not self._logs_tab:
            return
        if not self._logs_buffer_loaded:
            self._logs_buffer_loaded = True
            buf = get_buffered_log_lines()
            if buf:
                try:
                    log = self.query_one("#tui-log", Log)
                    log.clear()
                    for line in buf:
                        log.write(line)
                except Exception:
                    pass
        lines = drain_log_queue()
        if lines:
            try:
                log = self.query_one("#tui-log", Log)
                for line in lines:
                    log.write(line)
            except Exception:
                pass

    def _refresh_logs_tab(self) -> None:
        """Load current buffer and drain queue into Logs tab (call when user switches to Logs tab)."""
        if not self._logs_tab:
            return
        try:
            log = self.query_one("#tui-log", Log)
            log.clear()
            for line in get_buffered_log_lines():
                log.write(line)
            for line in drain_log_queue():
                log.write(line)
        except Exception:
            pass

    def _fetch_bank_accounts(self) -> None:
        """Fetch bank accounts from Discount Bank service or API router."""
        url = "http://localhost:8003/api/bank-accounts"
        if getattr(self.config, "api_base_url", None):
            base = self.config.api_base_url.strip().rstrip("/")
            url = f"{base}/api/bank-accounts"
        try:
            import requests
            response = requests.get(
                url,
                timeout=2.0,
                headers={"cache-control": "no-cache"}
            )
            if response.ok:
                data = response.json()
                self._bank_accounts = data.get("accounts", [])
                # Update unified positions tab if it exists
                if self._unified_positions_tab and self.snapshot:
                    self._unified_positions_tab.update_snapshot(self.snapshot, self._bank_accounts)
        except Exception as e:
            logger.debug(f"Failed to fetch bank accounts: {e}")
            # Silently fail - bank accounts are optional

    def _update_box_spread_data(self) -> None:
        """Update box spread data from REST or file via loader."""
        payload, new_mtime = get_box_spread_payload(
            self.config,
            self._box_spread_file_path,
            self._last_box_spread_mtime,
        )
        if new_mtime is not None:
            self._last_box_spread_mtime = new_mtime
        if payload is None:
            return
        if payload != self.box_spread_data:
            self.box_spread_data = payload
            if self._scenarios_tab:
                self._scenarios_tab.update_data(payload)

    def _get_config_watch_paths(self) -> List[Path]:
        """Paths to watch for config reload (T-114)."""
        root = Path(__file__).resolve().parent.parent.parent
        paths: List[Path] = []
        p = Path(TUIConfig.get_config_path())
        paths.append(p if p.is_absolute() else root / p)
        paths.append(root / "config" / "config.json")
        env_path = os.environ.get("IB_BOX_SPREAD_CONFIG")
        if env_path:
            paths.append(Path(env_path))
        # Watch home config so TWS port / shared config changes (e.g. from Setup) trigger reload
        try:
            from ..integration.shared_config_loader import SharedConfigLoader
            for home_p in SharedConfigLoader._home_config_paths():
                if home_p.exists():
                    paths.append(home_p)
                    break
        except Exception:
            pass
        return [x for x in paths if x.exists()]

    def _check_config_reload(self) -> None:
        """If any watched config file changed, reload config and apply (backend ports, health dashboard URL)."""
        paths = self._get_config_watch_paths()
        if not paths:
            return
        changed = False
        for p in paths:
            try:
                mtime = p.stat().st_mtime
            except OSError:
                continue
            key = str(p)
            if key in self._config_last_mtimes and self._config_last_mtimes[key] != mtime:
                changed = True
            self._config_last_mtimes[key] = mtime
        if not changed:
            return
        try:
            new_config = load_config()
        except Exception as e:
            logger.debug("Config reload failed: %s", e)
            return
        self._apply_config_reload(new_config)
        logger.info("Config reloaded (backend ports / health dashboard)")

    def _apply_config_reload(self, new_config: TUIConfig) -> None:
        """Apply reloaded config: update self.config and restart BackendHealthAggregator."""
        from .config import DEFAULT_BACKEND_PORTS, DEFAULT_TCP_BACKEND_PORTS
        self.config = new_config
        if not self.config.backend_ports:
            self.config.backend_ports = dict(DEFAULT_BACKEND_PORTS)
        else:
            self.config.backend_ports = {**DEFAULT_BACKEND_PORTS, **self.config.backend_ports}
        if not self.config.tcp_backend_ports:
            self.config.tcp_backend_ports = dict(DEFAULT_TCP_BACKEND_PORTS)
        else:
            self.config.tcp_backend_ports = {**DEFAULT_TCP_BACKEND_PORTS, **self.config.tcp_backend_ports}
        if self._backend_health_aggregator:
            self._backend_health_aggregator.stop()
            self._backend_health_aggregator = None
        if self.config.backend_ports or self.config.tcp_backend_ports:
            self._backend_health_aggregator = BackendHealthAggregator(
                self.config.backend_ports,
                tcp_backend_ports=self.config.tcp_backend_ports,
                unified_health_url=_effective_health_url(self.config),
            )
            self._backend_health_aggregator.start()

    def action_quit(self) -> None:
        """Quit the application"""
        self.exit()

    def action_help(self) -> None:
        """Show help screen"""
        self.notify("Press F1 for help, F2 for setup, Q/F10 to quit", title="Help")

    def action_setup(self) -> None:
        """Show setup screen"""
        try:
            status_bar = self.query_one("#status-bar", StatusBar)
            backend_health = getattr(status_bar, "backend_health", None)
        except Exception:
            backend_health = None
        self.push_screen(
            SetupScreen(
                self.config,
                self._get_provider_label(),
                str(self._loan_manager.loans_file_path),
                backend_health=backend_health,
            ),
            self._on_setup_closed,
        )

    def action_op_secrets(self) -> None:
        """Show 1Password / Secrets status screen"""
        self.push_screen(OnePasswordScreen())

    def _on_setup_closed(self, result: Optional[dict]) -> None:
        """Handle setup screen dismiss: apply backend switch if user clicked Apply."""
        if result and isinstance(result, dict) and result.get("provider_type"):
            self._switch_provider(result)

    def _switch_provider(self, params: Dict[str, Optional[str]]) -> None:
        """Replace current provider with a new one from params."""
        ptype = (params.get("provider_type") or "mock").lower()
        rest_endpoint = params.get("rest_endpoint")
        file_path = params.get("file_path")
        nats_url = params.get("nats_url") or self.config.nats_url
        nats_snapshot_backend = params.get("nats_snapshot_backend") or self.config.nats_snapshot_backend
        self.provider.stop()
        temp_config = TUIConfig(
            provider_type=ptype,
            rest_endpoint=rest_endpoint or self.config.rest_endpoint,
            file_path=file_path or self.config.file_path,
            nats_url=nats_url,
            nats_snapshot_backend=nats_snapshot_backend,
            update_interval_ms=self.config.update_interval_ms,
            rest_timeout_ms=self.config.rest_timeout_ms,
        )
        self.provider = create_provider_from_config(temp_config)
        self.config.provider_type = ptype
        if rest_endpoint is not None:
            self.config.rest_endpoint = rest_endpoint
        if file_path is not None:
            self.config.file_path = file_path
        if ptype == "nats":
            self.config.nats_url = nats_url
            self.config.nats_snapshot_backend = nats_snapshot_backend
        self.provider.start()
        label = self._get_provider_label()
        self._apply_theme_for_provider()
        self.notify(f"Backend switched to {label}", title="Switch backend")

    def action_refresh(self) -> None:
        """Force refresh snapshot"""
        self._update_snapshot()
        self.notify("Refreshed", title="Refresh")


def _effective_health_url(config: TUIConfig) -> Optional[str]:
    """Unified health URL: health_dashboard_url, or api_base_url + /api/health when using router."""
    url = getattr(config, "health_dashboard_url", None)
    if url:
        return url
    base = getattr(config, "api_base_url", None)
    if base:
        return base.strip().rstrip("/") + "/api/health"
    return None


def create_provider_from_config(config: TUIConfig) -> Provider:
    """
    Create provider based on configuration

    MIGRATION NOTE: This factory function can call C++ provider constructors
    via pybind11 in the future
    """
    provider_type = (config.provider_type or "mock").lower()

    if provider_type == "mock" or not provider_type:
        return MockProvider(
            update_interval_ms=config.update_interval_ms,
            symbols=getattr(config, "watchlist", None),
        )

    elif provider_type == "rest" or provider_type in PRESET_REST_ENDPOINTS:
        if getattr(config, "api_base_url", None):
            base = config.api_base_url.strip().rstrip("/")
            endpoint = f"{base}/api/v1/snapshot"
        else:
            endpoint = (
                config.rest_endpoint
                or PRESET_REST_ENDPOINTS.get(provider_type)
                or "http://localhost:8002/api/v1/snapshot"
            )
        return RestProvider(
            endpoint=endpoint,
            update_interval_ms=config.update_interval_ms,
            timeout_ms=config.rest_timeout_ms,
            verify_ssl=config.rest_verify_ssl,
        )

    elif provider_type == "file":
        file_path = config.file_path or "web/public/data/snapshot.json"
        return FileProvider(
            file_path=file_path, update_interval_ms=config.update_interval_ms
        )

    elif provider_type == "nats":
        return NatsProvider(
            nats_url=getattr(config, "nats_url", "nats://localhost:4222"),
            snapshot_backend=getattr(config, "nats_snapshot_backend", "ib"),
        )

    else:
        logger.warning(f"Unknown provider type: {provider_type}, using mock")
        return MockProvider(update_interval_ms=config.update_interval_ms)


def main():
    """Main entry point for Python TUI"""

    # Logging: captured to TUI Logs tab only (stderr handler removed so terminal isn't overwritten)
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        stream=sys.stderr,
    )
    tui_handler = install_tui_log_handler(level=logging.INFO, attach_to_root=True)

    # Load configuration
    config = load_config()

    # Create provider
    provider = create_provider_from_config(config)

    # Create and run app (pass handler so it can be removed from root on unmount)
    app = TUIApp(provider, config, tui_log_handler=tui_handler, tui_log_handler_on_root=True)
    app.run()


if __name__ == "__main__":
    main()
