"""
alpha_vantage_client.py - Alpha Vantage API client for stock, ETF, and market data

API: https://www.alphavantage.co/documentation/
Credentials: ALPHA_VANTAGE_API_KEY or OP_ALPHA_VANTAGE_API_KEY_SECRET (op:// ref),
or 1Password item titled "Alpha Vantage API" (discovered automatically when SDK/CLI available).
"""

from __future__ import annotations

import logging
import os
from typing import Any, Dict, List, Optional

import requests

try:
    from .onepassword_sdk_helper import (
        getenv_or_resolve,
        get_alpha_vantage_api_key_from_1password,
    )
except ImportError:
    def getenv_or_resolve(env_var: str, _op_ref: str, default: str = "") -> str:
        return os.getenv(env_var, default)

    def get_alpha_vantage_api_key_from_1password() -> Optional[str]:
        return None

logger = logging.getLogger(__name__)

BASE_URL = "https://www.alphavantage.co/query"


class AlphaVantageClient:
    """
    Client for Alpha Vantage REST API (stocks, ETFs, forex, crypto, technical indicators).

    Free tier: 25 requests/day. Use for validation, screening, or non-real-time data.
    """

    def __init__(self, api_key: Optional[str] = None, base_url: str = BASE_URL):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key or getenv_or_resolve(
            "ALPHA_VANTAGE_API_KEY", "OP_ALPHA_VANTAGE_API_KEY_SECRET", ""
        )
        if not self.api_key:
            discovered = get_alpha_vantage_api_key_from_1password()
            if discovered:
                self.api_key = discovered
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "IBBoxSpreadGenerator/1.0",
            "Accept": "application/json",
        })

    def _get(self, params: Dict[str, str], timeout: int = 15) -> Dict[str, Any]:
        """Run GET request with api_key injected."""
        if not self.api_key:
            logger.debug(
                "Alpha Vantage API key not set. Set ALPHA_VANTAGE_API_KEY or "
                "OP_ALPHA_VANTAGE_API_KEY_SECRET='op://vault/Alpha Vantage API/credential'"
            )
            return {}
        p = dict(params)
        p["apikey"] = self.api_key
        try:
            r = self.session.get(self.base_url, params=p, timeout=timeout)
            r.raise_for_status()
            data = r.json()
            if "Error Message" in data:
                logger.warning("Alpha Vantage error: %s", data["Error Message"])
                return {}
            if "Note" in data:
                logger.warning("Alpha Vantage rate limit: %s", data["Note"])
                return {}
            return data
        except requests.RequestException as e:
            logger.warning("Alpha Vantage request failed: %s", e)
            return {}

    def get_quote(self, symbol: str) -> Optional[Dict[str, Any]]:
        """
        Get global quote for a symbol (stock/ETF).

        Returns:
            Dict with price, change, volume, etc., or None if unavailable.
        """
        data = self._get({"function": "GLOBAL_QUOTE", "symbol": symbol.upper()})
        quote = data.get("Global Quote")
        if not quote or not isinstance(quote, dict):
            return None
        return {k.strip(): v for k, v in quote.items()}

    def get_daily(
        self,
        symbol: str,
        outputsize: str = "compact",
    ) -> Optional[Dict[str, List[Dict[str, str]]]]:
        """
        Get daily time series (last 100 points compact, or full for outputsize=full).

        Returns:
            Dict mapping date -> open/high/low/close/volume, or None.
        """
        data = self._get({
            "function": "TIME_SERIES_DAILY",
            "symbol": symbol.upper(),
            "outputsize": outputsize,
        })
        key = "Time Series (Daily)"
        if key not in data or not isinstance(data[key], dict):
            return None
        return data  # caller can use data["Time Series (Daily)"]

    def get_sma(
        self,
        symbol: str,
        interval: str = "daily",
        time_period: int = 20,
        series_type: str = "close",
    ) -> Optional[Dict[str, Dict[str, str]]]:
        """
        Get Simple Moving Average (SMA) for a symbol.

        Returns:
            Dict with "Technical Analysis: SMA" -> date -> SMA value, or None.
        """
        data = self._get({
            "function": "SMA",
            "symbol": symbol.upper(),
            "interval": interval,
            "time_period": str(time_period),
            "series_type": series_type,
        })
        key = "Technical Analysis: SMA"
        if key not in data or not isinstance(data[key], dict):
            return None
        return data

    def search(self, keywords: str) -> List[Dict[str, str]]:
        """
        Search for symbols by keyword (stocks, ETFs).

        Returns:
            List of matches with symbol, name, type, region, etc.
        """
        data = self._get({
            "function": "SYMBOL_SEARCH",
            "keywords": keywords,
        })
        matches = data.get("bestMatches", [])
        if not isinstance(matches, list):
            return []
        return [m for m in matches if isinstance(m, dict)]
