"""Benchmarks tab: SOFR and Treasury rates via the shared Rust API origin."""

from __future__ import annotations

import logging
from typing import Any, Dict, Optional

import requests

from textual.containers import Container, Vertical
from textual.widgets import Label, DataTable

from textual.app import ComposeResult
from textual.worker import get_current_worker

logger = logging.getLogger(__name__)

SOURCE_LABEL = "Source: FRED (St. Louis Fed) via shared API"
SOURCE_LABEL_DIRECT = "Source: FRED (St. Louis Fed) direct (service unreachable)"


def _get_benchmarks_base_url(app: Any) -> str:
    """Resolve benchmark API base URL from app config."""
    config = getattr(app, "config", None)
    if not config:
        return "http://127.0.0.1:8080"
    api_base_url = getattr(config, "api_base_url", None)
    if api_base_url:
        return api_base_url.strip().rstrip("/")
    return "http://127.0.0.1:8080"


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


class BenchmarksTab(Container):
    """Tab showing SOFR and Treasury benchmark rates from the shared Rust API."""

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
        app = getattr(self, "app", None)
        if app is not None:
            app.call_from_thread(self._apply_benchmarks, sofr, treasury, False)

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
            self._error = "Shared Rust benchmark endpoints unavailable."
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
