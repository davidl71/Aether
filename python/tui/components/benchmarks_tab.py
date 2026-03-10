"""Benchmarks tab: SOFR and Treasury rates from the risk-free-rate service (FRED)."""

from __future__ import annotations

import logging
from datetime import datetime
from typing import Any, Dict, Optional, Tuple

import requests

from textual.containers import Container, Vertical
from textual.widgets import Label, DataTable

from textual.app import ComposeResult
from textual.worker import get_current_worker

logger = logging.getLogger(__name__)

SOURCE_LABEL = "Source: FRED (St. Louis Fed) via Risk-Free Rate service"
SOURCE_LABEL_DIRECT = "Source: FRED (St. Louis Fed) direct (service unreachable)"


def _get_benchmarks_base_url(app: Any) -> str:
    """Resolve risk-free-rate service base URL from app config."""
    config = getattr(app, "config", None)
    if not config:
        return "http://127.0.0.1:8004"
    ports = getattr(config, "backend_ports", None) or {}
    port = ports.get("risk_free_rate", 8004)
    return f"http://127.0.0.1:{port}"


def _fetch_sofr(base: str) -> Optional[Dict[str, Any]]:
    try:
        r = requests.get(f"{base}/api/benchmarks/sofr", timeout=5)
        r.raise_for_status()
        return r.json()
    except Exception as e:
        logger.debug("Failed to fetch SOFR: %s", e)
        return None


def _fetch_treasury(base: str) -> Optional[Dict[str, Any]]:
    try:
        r = requests.get(f"{base}/api/benchmarks/treasury", timeout=5)
        r.raise_for_status()
        return r.json()
    except Exception as e:
        logger.debug("Failed to fetch Treasury: %s", e)
        return None


def _fetch_benchmarks_direct() -> Tuple[Optional[Dict[str, Any]], Optional[Dict[str, Any]]]:
    """Fetch SOFR and Treasury in-process via FRED when the risk-free-rate service is unreachable."""
    try:
        from ...integration.sofr_treasury_client import SOFRTreasuryClient
    except Exception as e:
        logger.debug("Cannot import SOFRTreasuryClient for direct FRED fallback: %s", e)
        return None, None
    client = SOFRTreasuryClient()
    if not getattr(client, "fred_api_key", None) or not client.fred_api_key.strip():
        logger.debug("FRED API key not available in this process; direct fallback skipped")
        return None, None
    sofr_dict: Optional[Dict[str, Any]] = None
    treasury_dict: Optional[Dict[str, Any]] = None
    try:
        overnight = client.get_sofr_overnight()
        term_rates = client.get_sofr_term_rates()
        sofr_dict = {
            "overnight": {
                "rate": overnight.rate if overnight else None,
                "timestamp": overnight.timestamp.isoformat() if overnight and overnight.timestamp else None,
            },
            "term_rates": [
                {
                    "tenor": r.tenor,
                    "rate": r.rate,
                    "days_to_expiry": r.days_to_expiry,
                    "timestamp": r.timestamp.isoformat(),
                }
                for r in term_rates
            ],
            "timestamp": (overnight.timestamp if overnight else datetime.now()).isoformat(),
        }
        if not overnight and not term_rates:
            sofr_dict = None
    except Exception as e:
        logger.debug("Direct SOFR fetch failed: %s", e)
    try:
        rates = client.get_treasury_rates()
        if rates:
            treasury_dict = {
                "rates": [
                    {
                        "tenor": r.tenor,
                        "rate": r.rate,
                        "days_to_expiry": r.days_to_expiry,
                        "timestamp": r.timestamp.isoformat(),
                    }
                    for r in rates
                ],
                "timestamp": rates[0].timestamp.isoformat() if rates else None,
            }
    except Exception as e:
        logger.debug("Direct Treasury fetch failed: %s", e)
    return sofr_dict, treasury_dict


class BenchmarksTab(Container):
    """Tab showing SOFR and Treasury benchmark rates from the risk-free-rate service."""

    def __init__(
        self,
        *,
        name: Optional[str] = None,
        id: Optional[str] = None,
        classes: Optional[str] = None,
        disabled: bool = False,
    ) -> None:
        super().__init__(name=name, id=id, classes=classes, disabled=disabled)
        self._sofr: Optional[Dict[str, Any]] = None
        self._treasury: Optional[Dict[str, Any]] = None
        self._error: Optional[str] = None
        self._source_direct: bool = False

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Benchmarks (SOFR & Treasury)", classes="tab-title")
            yield DataTable(id="benchmarks-table")
            yield Label(id="benchmarks-source", classes="dim")
            yield Label(id="benchmarks-error")

    def on_mount(self) -> None:
        table = self.query_one("#benchmarks-table", DataTable)
        table.add_columns("Series", "Tenor", "Rate %", "Updated")
        base = _get_benchmarks_base_url(self.app)
        # thread=True: worker runs a no-arg callable; wrap so base is passed in
        self.run_worker(lambda: self._fetch_benchmarks(base), exclusive=False, thread=True)

    def _fetch_benchmarks(self, base: str) -> None:
        worker = get_current_worker()
        if worker and worker.is_cancelled:
            return
        sofr = _fetch_sofr(base)
        if worker and worker.is_cancelled:
            return
        treasury = _fetch_treasury(base)
        source_direct = False
        if not sofr and not treasury:
            if worker and worker.is_cancelled:
                return
            sofr, treasury = _fetch_benchmarks_direct()
            source_direct = sofr is not None or treasury is not None
        app = getattr(self, "app", None)
        if app is not None:
            app.call_from_thread(self._apply_benchmarks, sofr, treasury, source_direct)

    def _apply_benchmarks(
        self,
        sofr: Optional[Dict[str, Any]],
        treasury: Optional[Dict[str, Any]],
        source_direct: bool = False,
    ) -> None:
        self._sofr = sofr
        self._treasury = treasury
        self._source_direct = source_direct
        self._error = None
        if not sofr and not treasury:
            self._error = (
                "Risk-Free Rate service unavailable and no FRED key in this process. "
                "Start the service (port 8004) with FRED_API_KEY set, or set FRED_API_KEY / OP_FRED_API_KEY_SECRET for direct FRED access."
            )
        self._update_data()

    def _update_data(self) -> None:
        table = self.query_one("#benchmarks-table", DataTable)
        source_label = self.query_one("#benchmarks-source", Label)
        error_label = self.query_one("#benchmarks-error", Label)

        table.clear()
        source_label.update(SOURCE_LABEL_DIRECT if getattr(self, "_source_direct", False) else SOURCE_LABEL)

        if self._error:
            error_label.update(self._error)
            error_label.add_class("warning")
            return

        error_label.update("")
        error_label.remove_class("warning")

        # Rows only; columns added in on_mount

        # SOFR overnight
        if self._sofr:
            overnight = self._sofr.get("overnight") or {}
            rate = overnight.get("rate")
            ts = (overnight.get("timestamp") or "").replace("T", " ").split(".")[0]
            table.add_row(
                "SOFR",
                "Overnight",
                f"{rate:.2f}" if rate is not None else "--",
                ts or "--",
            )
            for tr in self._sofr.get("term_rates") or []:
                rate = tr.get("rate")
                ts = (tr.get("timestamp") or "").replace("T", " ").split(".")[0]
                table.add_row(
                    "SOFR",
                    tr.get("tenor", "--"),
                    f"{rate:.2f}" if rate is not None else "--",
                    ts or "--",
                )

        # Treasury
        for r in (self._treasury or {}).get("rates") or []:
            rate = r.get("rate")
            ts = (r.get("timestamp") or "").replace("T", " ").split(".")[0]
            table.add_row(
                "Treasury",
                r.get("tenor", "--"),
                f"{rate:.2f}" if rate is not None else "--",
                ts or "--",
            )
