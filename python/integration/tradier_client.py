"""
tradier_client.py - Market data client for Tradier API

Provides quotes, option chains, and option expirations.
Order execution is not included in this version.

Environment variables:
- TRADIER_ACCESS_TOKEN (required)
- TRADIER_BASE_URL (optional, defaults to https://api.tradier.com)
- TRADIER_SANDBOX (optional, "1"/"true" uses sandbox endpoint)
"""
from __future__ import annotations

import os
import logging
from typing import Dict, List, Optional

import requests

logger = logging.getLogger(__name__)


class TradierError(RuntimeError):
    """Generic error raised for Tradier API failures."""


class TradierClient:
    """Market-data client for the Tradier REST API."""

    def __init__(
        self,
        access_token: Optional[str] = None,
        base_url: Optional[str] = None,
        sandbox: Optional[bool] = None,
        session: Optional[requests.Session] = None,
    ) -> None:
        self.access_token = access_token or os.getenv("TRADIER_ACCESS_TOKEN", "")
        if not self.access_token:
            raise RuntimeError(
                "Missing Tradier credentials. Set TRADIER_ACCESS_TOKEN."
            )

        if sandbox is None:
            sandbox = os.getenv("TRADIER_SANDBOX", "").lower() in (
                "1", "true", "yes", "on",
            )
        self.sandbox = sandbox

        if base_url:
            self.base_url = base_url.rstrip("/")
        elif self.sandbox:
            self.base_url = "https://sandbox.tradier.com"
        else:
            self.base_url = os.getenv(
                "TRADIER_BASE_URL", "https://api.tradier.com"
            ).rstrip("/")

        self._session = session or requests.Session()
        self._session.headers.update({
            "Authorization": f"Bearer {self.access_token}",
            "Accept": "application/json",
        })

        mode = "SANDBOX" if self.sandbox else "PRODUCTION"
        logger.info("Tradier client initialized (%s) -> %s", mode, self.base_url)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _get(self, path: str, params: Optional[Dict] = None) -> Dict:
        url = f"{self.base_url}{path}"
        resp = self._session.get(url, params=params or {}, timeout=10)
        resp.raise_for_status()
        result = resp.json()
        return result if isinstance(result, dict) else {}

    # ------------------------------------------------------------------
    # Quotes
    # ------------------------------------------------------------------

    def get_quotes(self, symbols: List[str]) -> List[Dict]:
        """Fetch NBBO quotes for one or more symbols.

        Args:
            symbols: List of ticker strings (e.g. ["SPY", "QQQ"]).

        Returns:
            List of quote dicts with keys: symbol, bid, ask, last, volume,
            open, high, low, close, change, change_percentage.
        """
        if not symbols:
            return []

        data = self._get(
            "/v1/markets/quotes",
            params={"symbols": ",".join(symbols), "greeks": "false"},
        )

        quotes_raw = data.get("quotes", {})
        if not quotes_raw:
            return []

        quote_list = quotes_raw.get("quote", [])
        if isinstance(quote_list, dict):
            quote_list = [quote_list]

        results: List[Dict] = []
        for q in quote_list:
            results.append({
                "symbol": q.get("symbol", ""),
                "bid": float(q.get("bid") or 0),
                "ask": float(q.get("ask") or 0),
                "last": float(q.get("last") or 0),
                "volume": int(q.get("volume") or 0),
                "open": float(q.get("open") or 0),
                "high": float(q.get("high") or 0),
                "low": float(q.get("low") or 0),
                "close": float(q.get("close") or 0),
                "change": float(q.get("change") or 0),
                "change_percentage": float(q.get("change_percentage") or 0),
            })
        return results

    def get_snapshot(self, symbol: str) -> Dict:
        """Convenience wrapper: returns a single-symbol snapshot.

        Returns:
            Dict with keys: symbol, last, bid, ask, spread, volume.
        """
        quotes = self.get_quotes([symbol])
        if not quotes:
            return {
                "symbol": symbol,
                "last": 0.0,
                "bid": 0.0,
                "ask": 0.0,
                "spread": 0.0,
                "volume": 0,
            }
        q = quotes[0]
        bid = q["bid"]
        ask = q["ask"]
        return {
            "symbol": q["symbol"],
            "last": q["last"] or ((bid + ask) / 2.0 if bid and ask else 0.0),
            "bid": bid,
            "ask": ask,
            "spread": (ask - bid) if bid and ask else 0.0,
            "volume": q["volume"],
        }

    # ------------------------------------------------------------------
    # Option Expirations
    # ------------------------------------------------------------------

    def get_option_expirations(self, symbol: str) -> List[str]:
        """Return available expiration dates for an underlying.

        Args:
            symbol: Underlying ticker (e.g. "SPY").

        Returns:
            Sorted list of date strings in YYYY-MM-DD format.
        """
        data = self._get(
            "/v1/markets/options/expirations",
            params={"symbol": symbol},
        )
        expirations_raw = data.get("expirations", {})
        if not expirations_raw:
            return []
        date_list = expirations_raw.get("date", [])
        if isinstance(date_list, str):
            date_list = [date_list]
        return sorted(date_list)

    # ------------------------------------------------------------------
    # Option Chain
    # ------------------------------------------------------------------

    def get_option_chain(
        self,
        symbol: str,
        expiration_date: Optional[str] = None,
    ) -> Dict[str, List[Dict]]:
        """Fetch the option chain for an underlying, grouped by expiration.

        Args:
            symbol: Underlying ticker.
            expiration_date: Optional filter (YYYY-MM-DD).

        Returns:
            Dict mapping expiration date strings to lists of option dicts.
        """
        params: Dict = {"symbol": symbol, "greeks": "true"}
        if expiration_date:
            params["expiration"] = expiration_date

        data = self._get("/v1/markets/options/chains", params=params)
        options_raw = data.get("options", {})
        if not options_raw:
            return {}

        option_list = options_raw.get("option", [])
        if isinstance(option_list, dict):
            option_list = [option_list]

        chain: Dict[str, List[Dict]] = {}
        for opt in option_list:
            exp = opt.get("expiration_date", "unknown")
            entry = {
                "symbol": opt.get("symbol", ""),
                "underlying": opt.get("underlying", symbol),
                "expiration_date": exp,
                "strike": float(opt.get("strike") or 0),
                "option_type": opt.get("option_type", ""),
                "bid": float(opt.get("bid") or 0),
                "ask": float(opt.get("ask") or 0),
                "last": float(opt.get("last") or 0),
                "volume": int(opt.get("volume") or 0),
                "open_interest": int(opt.get("open_interest") or 0),
            }
            greeks = opt.get("greeks") or {}
            if greeks:
                entry["delta"] = float(greeks.get("delta") or 0)
                entry["gamma"] = float(greeks.get("gamma") or 0)
                entry["theta"] = float(greeks.get("theta") or 0)
                entry["vega"] = float(greeks.get("vega") or 0)
                entry["implied_volatility"] = float(
                    greeks.get("mid_iv") or greeks.get("smv_vol") or 0
                )

            chain.setdefault(exp, []).append(entry)

        return chain
