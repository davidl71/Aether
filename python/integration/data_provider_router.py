"""
data_provider_router.py - Selects between market data providers with fallback support.
"""
from __future__ import annotations

import logging
from typing import Dict, List, Optional, Tuple


logger = logging.getLogger(__name__)


class DataProviderRouter:
    """Routes market data requests across primary and fallback providers."""

    def __init__(
        self,
        market_data_handler,
        provider_config: Dict,
        orats_client=None,
        notifier=None,
    ) -> None:
        self.market_data_handler = market_data_handler
        self.config = provider_config or {}
        self.orats_client = orats_client
        self.notifier = notifier

        self.primary = (self.config.get("primary") or "ib").lower()
        self.fallbacks: List[str] = [p.lower() for p in self.config.get("fallbacks", [])]

        logger.info(
            "Market data provider order: %s",
            " -> ".join([self.primary] + self.fallbacks) if self.fallbacks else self.primary,
        )

    def get_quote(self, symbol: str) -> Tuple[Optional[Dict], Optional[str]]:
        """Get latest quote using primary with fallback providers."""
        providers = [self.primary] + [p for p in self.fallbacks if p != self.primary]

        for provider in providers:
            if provider == "ib":
                quote = self._get_from_ib(symbol)
                if quote is not None:
                    return quote, "ib"
            elif provider == "orats":
                quote = self._get_from_orats(symbol)
                if quote is not None:
                    return quote, "orats"
            else:
                logger.warning("Unknown data provider '%s' configured", provider)

        return None, None

    def _get_from_ib(self, symbol: str) -> Optional[Dict]:
        try:
            quote = self.market_data_handler.get_latest_quote(symbol)
            if quote is None:
                logger.debug("No IB quote available for %s", symbol)
            return quote
        except Exception as exc:  # pragma: no cover - defensive
            logger.error("Failed to retrieve IB quote for %s: %s", symbol, exc)
            return None

    def _get_from_orats(self, symbol: str) -> Optional[Dict]:
        if not self.orats_client:
            logger.debug("ORATS client not configured; skipping fallback")
            return None

        try:
            core_data = self.orats_client.get_core_data(symbol)
        except Exception as exc:  # pragma: no cover - network failure
            logger.error("ORATS core data request failed for %s: %s", symbol, exc)
            if self.notifier:
                self.notifier.notify(
                    event_type="data_provider_error",
                    title="ORATS data request failed",
                    message=str(exc),
                    severity="warning",
                    payload={"symbol": symbol},
                )
            return None

        if not core_data:
            logger.debug("ORATS returned no data for %s", symbol)
            return None

        price = (
            core_data.get("stockPrice")
            or core_data.get("underlyingPrice")
            or core_data.get("close")
        )

        if price is None:
            logger.debug("ORATS core data missing price for %s", symbol)
            return None

        quote = {
            "symbol": symbol,
            "bid": float(price),
            "ask": float(price),
            "last": float(price),
            "mid": float(price),
            "source": "orats",
            "timestamp": core_data.get("snapshotTime"),
        }

        logger.info("Using ORATS fallback quote for %s at %.2f", symbol, price)

        if self.notifier:
            self.notifier.notify(
                event_type="data_provider_fallback",
                title="Using ORATS market data",
                message=f"Fallback quote used for {symbol}",
                severity="info",
                payload={"symbol": symbol, "price": price},
            )

        return quote


