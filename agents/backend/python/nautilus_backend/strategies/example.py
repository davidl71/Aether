from __future__ import annotations

import asyncio

from nautilus_backend.strategies.registry import BaseStrategy, StrategyContext


class ExampleStrategy(BaseStrategy):
    def __init__(self, symbol: str = "ESZ4") -> None:
        try:
            super().__init__()
        except TypeError:  # pragma: no cover - fallback when Nautilus expects config
            super().__init__(config=None)
        self.symbol = symbol

    async def run(self, context: StrategyContext) -> None:  # pragma: no cover - scaffold
        while True:
            event = await context.poll()
            if event is None:
                await asyncio.sleep(0.5)
                continue

            decision = {
                "symbol": self.symbol,
                "quantity": 1,
                "side": "BUY",
            }
            await context.submit(decision)
