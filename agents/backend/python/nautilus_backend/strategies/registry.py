from __future__ import annotations

from collections.abc import Callable
from typing import Dict

try:
    from nautilus_trader.trading.strategy import Strategy as NautilusStrategy
except ImportError:  # pragma: no cover - fallback for bootstrap environments
    class NautilusStrategy:  # type: ignore[override]
        async def run(self, *_: object, **__: object) -> None:  # pragma: no cover - stub
            raise NotImplementedError


StrategyFactory = Callable[[], "BaseStrategy"]


class StrategyContext:
    """Placeholder context bridged from Rust."""

    async def poll(self) -> dict | None:  # pragma: no cover - scaffold
        return None

    async def submit(self, decision: dict) -> None:  # pragma: no cover - scaffold
        _ = decision


class BaseStrategy(NautilusStrategy):
    async def run(self, context: StrategyContext) -> None:  # pragma: no cover - scaffold
        raise NotImplementedError


class StrategyRegistry:
    def __init__(self) -> None:
        self._strategies: Dict[str, StrategyFactory] = {}

    def register(self, name: str, factory: StrategyFactory) -> None:
        if name in self._strategies:
            msg = f"Strategy {name} already registered"
            raise ValueError(msg)
        self._strategies[name] = factory

    def get(self, name: str) -> StrategyFactory:
        return self._strategies[name]


def default_registry() -> StrategyRegistry:
    registry = StrategyRegistry()
    from .example import ExampleStrategy

    registry.register("example", ExampleStrategy)
    return registry
