"""MockProvider: generates synthetic snapshots for testing."""
from __future__ import annotations

import logging
import threading
import time
from datetime import datetime, timezone
from typing import Optional, List

from ..models import SnapshotPayload
from ._base import Provider

logger = logging.getLogger(__name__)


class MockProvider(Provider):
    """
    Mock provider that generates synthetic snapshots for testing.

    When symbols is provided (e.g. from config.watchlist), generates data for those
    symbols so the dashboard watchlist and snapshot stay in sync.
    """

    def __init__(
        self,
        update_interval_ms: int = 1000,
        symbols: Optional[List[str]] = None,
    ):
        super().__init__()
        self.update_interval_ms = update_interval_ms
        self._worker_thread: Optional[threading.Thread] = None
        self._symbols = (
            list(symbols)
            if symbols
            else ["SPX", "XSP", "NDX"]
        )

    def start(self) -> None:
        if self._running:
            return
        self._running = True
        self._worker_thread = threading.Thread(target=self._generate_loop, daemon=True)
        self._worker_thread.start()
        logger.info("MockProvider started")

    def stop(self) -> None:
        self._running = False
        if self._worker_thread:
            self._worker_thread.join(timeout=2.0)
        logger.info("MockProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is None:
                return self._generate_snapshot()
            return self._latest_snapshot

    def is_running(self) -> bool:
        return self._running

    def add_symbol(self, symbol: str) -> None:
        """Add a symbol to the mock provider's rotation"""
        if symbol not in self._symbols:
            self._symbols.append(symbol)

    def _generate_loop(self) -> None:
        while self._running:
            snapshot = self._generate_snapshot()
            with self._lock:
                self._latest_snapshot = snapshot
            time.sleep(self.update_interval_ms / 1000.0)

    def _generate_snapshot(self) -> SnapshotPayload:
        now = datetime.now(timezone.utc).isoformat()

        symbols = []
        for i, symbol in enumerate(self._symbols):
            base_price = 4000.0 + (i * 100.0)
            symbols.append({
                "symbol": symbol,
                "last": base_price + (i * 0.5),
                "bid": base_price + (i * 0.3),
                "ask": base_price + (i * 0.7),
                "spread": 0.4,
                "roi": 2.5 + (i * 0.5),
                "maker_count": i + 1,
                "taker_count": i,
                "volume": 1000 + (i * 100),
                "candle": {
                    "open": base_price,
                    "high": base_price + 1.0,
                    "low": base_price - 1.0,
                    "close": base_price + 0.5,
                    "volume": 1000,
                    "entry": base_price,
                    "updated": now
                },
                "option_chains": []
            })

        return SnapshotPayload.from_dict({
            "generated_at": now,
            "mode": "DRY-RUN",
            "strategy": "RUNNING",
            "account_id": "DU123456",
            "metrics": {
                "net_liq": 100000.0,
                "buying_power": 50000.0,
                "excess_liquidity": 45000.0,
                "margin_requirement": 5000.0,
                "commissions": 0.0,
                "portal_ok": True,
                "tws_ok": True,
                "questdb_ok": True
            },
            "symbols": symbols,
            "positions": [],
            "historic": [],
            "orders": [],
            "alerts": []
        })
