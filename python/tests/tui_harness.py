"""Reusable mounted-app harness for Textual TUI tests."""

from __future__ import annotations

from contextlib import asynccontextmanager
from typing import AsyncIterator
from unittest.mock import patch

from textual.pilot import Pilot

from python.tui.app import TUIApp
from python.tui.config import TUIConfig
from python.tui.providers import MockProvider


class _NoopBackendHealthAggregator:
    """Keep mounted TUI tests deterministic and offline."""

    def __init__(self, *args: object, **kwargs: object) -> None:
        self._health: dict[str, dict[str, str]] = {}

    def start(self) -> None:
        return None

    def stop(self) -> None:
        return None

    def get_all_health(self) -> dict[str, dict[str, str]]:
        return dict(self._health)


def build_test_tui_app(config: TUIConfig | None = None) -> TUIApp:
    """Construct a mock-backed TUI app for mounted tests."""
    app_config = config or TUIConfig(provider_type="mock")
    return TUIApp(MockProvider(), app_config)


@asynccontextmanager
async def mounted_tui_app(
    config: TUIConfig | None = None,
    *,
    size: tuple[int, int] = (140, 40),
) -> AsyncIterator[tuple[TUIApp, Pilot[None]]]:
    """Mount the TUI under TextualPilot with background polling disabled."""
    app = build_test_tui_app(config)
    app._fetch_bank_accounts = lambda: None  # type: ignore[method-assign]
    app._update_box_spread_data = lambda: None  # type: ignore[method-assign]
    app._check_config_reload = lambda: None  # type: ignore[method-assign]

    with patch("python.tui.app.BackendHealthAggregator", _NoopBackendHealthAggregator):
        async with app.run_test(size=size) as pilot:
            await pilot.pause()
            yield app, pilot
