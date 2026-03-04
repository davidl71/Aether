"""Scenarios tab: box spread scenarios table."""

from __future__ import annotations

import logging
from typing import Optional

from textual.containers import Container, Vertical
from textual.widgets import Label, Static, DataTable
from textual.app import ComposeResult

from ..models import BoxSpreadPayload, BoxSpreadSummary

logger = logging.getLogger(__name__)


class ScenariosTab(Container):
    """Scenarios tab showing box spread scenarios."""

    def __init__(self, box_spread_data: Optional[BoxSpreadPayload] = None):
        super().__init__()
        self.box_spread_data = box_spread_data

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Box Spread Scenarios", classes="tab-title")
            yield Static(id="scenario-summary")
            yield DataTable(id="scenarios-table")

    def on_mount(self) -> None:
        table = self.query_one("#scenarios-table", DataTable)
        table.add_columns(
            "Width", "Style", "Buy Profit", "Sell Profit", "APR %", "Fill Prob"
        )
        self._update_data()

    def update_data(self, box_spread_data: BoxSpreadPayload) -> None:
        """Update with new box spread data."""
        self.box_spread_data = box_spread_data
        self._update_data()

    def _update_data(self) -> None:
        if not self.box_spread_data:
            try:
                summary = self.query_one("#scenario-summary", Static)
                summary.update("Loading scenarios...")
            except Exception:
                pass
            return

        try:
            summary_stats = BoxSpreadSummary.calculate(self.box_spread_data)

            summary = self.query_one("#scenario-summary", Static)
            summary_text = (
                f"Total Scenarios: {summary_stats.total_scenarios} | "
                f"Average APR: {summary_stats.avg_apr:.2f}% | "
                f"Probable (fill_prob > 0): {summary_stats.probable_count}"
            )
            if summary_stats.max_apr_scenario:
                summary_text += (
                    f" | Max APR: {summary_stats.max_apr_scenario.annualized_return:.2f}% "
                    f"({summary_stats.max_apr_scenario.width:.2f} pts)"
                )
            summary.update(summary_text)

            table = self.query_one("#scenarios-table", DataTable)
            table.clear()

            european_scenarios = [
                s for s in self.box_spread_data.scenarios
                if s.option_style == "European"
            ]
            scenarios_to_show = european_scenarios if european_scenarios else self.box_spread_data.scenarios

            for scenario in scenarios_to_show:
                buy_profit = scenario.buy_profit if scenario.buy_profit is not None else 0.0
                sell_profit = scenario.sell_profit if scenario.sell_profit is not None else 0.0

                table.add_row(
                    f"{scenario.width:.2f}",
                    scenario.option_style,
                    f"${buy_profit:.2f}" if buy_profit != 0.0 else "—",
                    f"${sell_profit:.2f}" if sell_profit != 0.0 else "—",
                    f"{scenario.annualized_return:.2f}%",
                    f"{scenario.fill_probability:.0f}%"
                )
        except Exception as e:
            logger.error("Error updating scenarios: %s", e)
            try:
                summary = self.query_one("#scenario-summary", Static)
                summary.update(f"Error loading scenarios: {e}")
            except Exception:
                pass
