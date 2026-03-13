"""Option chain instrument discovery helpers for the NT IB adapter.

Wraps IBInstrumentProvider with box-spread-specific filtering:
- European-style exercise only (SPX, XSP, NDX)
- DTE window filtering
- Contract count guard (mirrors C++ max_contracts_per_symbol guard)
"""

from __future__ import annotations

from datetime import datetime, timezone

import structlog
from nautilus_trader.adapters.interactive_brokers.providers import (
    InteractiveBrokersInstrumentProvider,
)
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.instruments import OptionsContract

log = structlog.get_logger(__name__)

# Index underlyings that always have European-style exercise
EUROPEAN_INDEX_SYMBOLS: frozenset[str] = frozenset({"SPX", "SPXW", "XSP", "NDX", "NDXP", "VIX"})


class BoxSpreadInstrumentHelper:
    """Filter and discover option chain instruments suitable for box spreads."""

    def __init__(
        self,
        provider: InteractiveBrokersInstrumentProvider,
        min_dte: int = 30,
        max_dte: int = 90,
        max_contracts_per_symbol: int = 200,
    ) -> None:
        self._provider = provider
        self._min_dte = min_dte
        self._max_dte = max_dte
        self._max_contracts = max_contracts_per_symbol

    async def load_option_chain(self, underlying: str) -> list[InstrumentId]:
        """Request full option chain from IB adapter and return filtered InstrumentIds.

        Filters to:
        - European-style exercise
        - DTE within [min_dte, max_dte]
        - At most max_contracts_per_symbol instruments (cost guard)
        """
        log.info(
            "loading_option_chain",
            symbol=underlying,
            min_dte=self._min_dte,
            max_dte=self._max_dte,
        )

        try:
            await self._provider.load_all_async()
        except Exception:
            log.exception("instrument_load_failed", symbol=underlying)
            return []

        instruments = self._provider.list_all()
        filtered = self._filter_chain(underlying, instruments)

        if len(filtered) > self._max_contracts:
            log.warning(
                "chain_truncated",
                symbol=underlying,
                total=len(filtered),
                limit=self._max_contracts,
            )
            filtered = filtered[: self._max_contracts]

        log.info("chain_loaded", symbol=underlying, contracts=len(filtered))
        return [inst.id for inst in filtered]

    def _filter_chain(
        self, underlying: str, instruments: list[OptionsContract]
    ) -> list[OptionsContract]:
        """Apply European-style and DTE filters."""
        now = datetime.now(tz=timezone.utc)
        result: list[OptionsContract] = []

        for inst in instruments:
            if not isinstance(inst, OptionsContract):
                continue
            if not str(inst.id.symbol.value).upper().startswith(underlying.upper()):
                continue
            if not self.is_european_style(inst):
                continue
            dte = self._days_to_expiry(inst, now)
            if dte is None or dte < self._min_dte or dte > self._max_dte:
                continue
            result.append(inst)

        return result

    def is_european_style(self, inst: OptionsContract) -> bool:
        """Return True for European-exercise index options."""
        sym = str(inst.id.symbol.value).upper()
        for eu_sym in EUROPEAN_INDEX_SYMBOLS:
            if sym.startswith(eu_sym):
                return True
        # NT OptionsContract may expose exercise_style; check if available
        if hasattr(inst, "exercise_style"):
            style = str(getattr(inst, "exercise_style", "")).upper()
            if style in ("EUROPEAN", "EUR", "E"):
                return True
        return False

    def _days_to_expiry(
        self, inst: OptionsContract, now: datetime
    ) -> int | None:
        """Compute days to expiry from NT instrument expiry field."""
        try:
            expiry = inst.expiry  # UnixNanos or datetime depending on NT version
            if isinstance(expiry, int):
                exp_dt = datetime.fromtimestamp(expiry / 1_000_000_000, tz=timezone.utc)
            else:
                exp_dt = expiry
            return max(0, (exp_dt - now).days)
        except Exception:
            return None
