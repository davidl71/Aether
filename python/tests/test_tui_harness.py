"""Mounted TUI smoke tests using TextualPilot."""

from __future__ import annotations

import pytest

textual = pytest.importorskip("textual")

from textual.widgets import TabbedContent, Footer, Header, Log

from python.tui.app import TUI_TAB_IDS
from python.tui.components.snapshot_display import StatusBar
from python.tests.tui_harness import mounted_tui_app


@pytest.mark.asyncio
async def test_tui_harness_mounts_core_widgets() -> None:
    async with mounted_tui_app() as (app, _pilot):
        assert app.title.startswith("IB Box Spread Terminal")
        assert app.query_one(Header)
        assert app.query_one(Footer)
        assert app.query_one(StatusBar)

        tabs = app.query_one("#tabs", TabbedContent)
        assert tabs.active == "dashboard-tab"
        assert TUI_TAB_IDS


@pytest.mark.asyncio
async def test_tui_harness_can_switch_to_logs_tab() -> None:
    async with mounted_tui_app() as (app, pilot):
        app.switch_to_tab("logs-tab")
        await pilot.pause()

        tabs = app.query_one("#tabs", TabbedContent)
        assert tabs.active == "logs-tab"
        assert app.query_one("#tui-log", Log)
