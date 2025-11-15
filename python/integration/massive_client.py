"""
massive_client.py - Client for Massive.com API integration
Provides market data, dividends, fundamentals, and historical data.
"""
import logging
import requests
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime, timedelta
import time

logger = logging.getLogger(__name__)

# Massive.com API endpoints
MASSIVE_BASE_URL = "https://api.massive.com"


class MassiveClient:
    """
    Client for Massive.com REST API.
    Provides market data, dividends, fundamentals, and historical data.

    Reference: https://massive.com/docs/rest/quickstart
    """

    def __init__(
        self,
        api_key: str,
        base_url: str = MASSIVE_BASE_URL,
        cache_duration_seconds: int = 300,
        rate_limit_per_second: int = 10,
    ):
        """
        Initialize Massive.com client.

        Args:
            api_key: Massive.com API key
            base_url: Base URL for Massive.com API
            cache_duration_seconds: Cache duration (default: 5 minutes)
            rate_limit_per_second: Max requests per second
        """
        self.api_key = api_key
        self.base_url = base_url
        self._cache_duration = timedelta(seconds=cache_duration_seconds)
        self._rate_limit = rate_limit_per_second

        # HTTP session
        self._session = requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json"
        })

        # Cache
        self._cache: Dict[str, Dict[str, Any]] = {}

        # Rate limiting
        self._request_times: List[float] = []

    def _check_rate_limit(self):
        """Enforce rate limiting."""
        now = time.time()

        # Remove requests older than 1 second
        self._request_times = [t for t in self._request_times if now - t < 1.0]

        # Check if at limit
        if len(self._request_times) >= self._rate_limit:
            sleep_time = 1.0 - (now - self._request_times[0])
            if sleep_time > 0:
                logger.debug(f"Rate limit reached, sleeping {sleep_time:.2f}s")
                time.sleep(sleep_time)

        # Record this request
        self._request_times.append(time.time())

    def _is_cached(self, key: str) -> bool:
        """Check if data is cached and not stale."""
        if key not in self._cache:
            return False

        age = datetime.now() - self._cache[key]["timestamp"]
        return age < self._cache_duration

    def _get_cached(self, key: str) -> Optional[Any]:
        """Get cached data if available and fresh."""
        if self._is_cached(key):
            return self._cache[key]["data"]
        return None

    def _set_cache(self, key: str, data: Any):
        """Store data in cache."""
        self._cache[key] = {
            "data": data,
            "timestamp": datetime.now()
        }

    def _make_request(
        self,
        endpoint: str,
        params: Optional[Dict] = None,
        timeout: int = 10,
        use_api_key_in_params: bool = True
    ) -> Optional[Dict]:
        """
        Make API request with rate limiting and error handling.

        Args:
            endpoint: API endpoint path
            params: Query parameters
            timeout: Request timeout in seconds
            use_api_key_in_params: Add API key to query params (alternative: header)

        Returns:
            Response data or None on error
        """
        # Rate limiting
        self._check_rate_limit()

        # Build URL
        url = f"{self.base_url}/{endpoint}"

        # Prepare parameters
        request_params = params.copy() if params else {}
        if use_api_key_in_params:
            request_params["apiKey"] = self.api_key

        try:
            logger.debug(f"Massive.com API request: {endpoint}")
            response = self._session.get(url, params=request_params, timeout=timeout)
            response.raise_for_status()

            data = response.json()
            logger.debug(f"Massive.com API response: {len(str(data))} bytes")

            return data

        except requests.exceptions.HTTPError as e:
            logger.error(f"Massive.com API HTTP error: {e}")
            if hasattr(e.response, 'text'):
                logger.debug(f"Response: {e.response.text[:200]}")
            return None

        except requests.exceptions.Timeout:
            logger.error(f"Massive.com API timeout for {endpoint}")
            return None

        except requests.exceptions.RequestException as e:
            logger.error(f"Massive.com API request error: {e}")
            return None

        except ValueError as e:
            logger.error(f"Massive.com API JSON decode error: {e}")
            return None

    def get_dividends(
        self,
        symbol: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        use_cache: bool = True
    ) -> List[Dict]:
        """
        Get dividend records from Massive.com.

        Args:
            symbol: Stock ticker (optional, for filtering)
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            use_cache: Use cached data if available

        Returns:
            List of dividend records
        """
        cache_key = f"dividends_{symbol}_{start_date}_{end_date}"

        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                logger.debug(f"Using cached dividends for {symbol}")
                return cached

        params = {}
        if symbol:
            params["symbol"] = symbol
        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        response = self._make_request("dividends", params)
        if response is None:
            return []

        data = response.get("results", [])

        # Cache result
        self._set_cache(cache_key, data)

        logger.info(f"Retrieved {len(data)} dividend records for {symbol or 'all symbols'}")
        return data

    def get_trades(
        self,
        symbol: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        use_cache: bool = True
    ) -> List[Dict]:
        """
        Get historical trade data from Massive.com.

        Args:
            symbol: Stock ticker
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            use_cache: Use cached data if available

        Returns:
            List of trade records
        """
        cache_key = f"trades_{symbol}_{start_date}_{end_date}"

        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                logger.debug(f"Using cached trades for {symbol}")
                return cached

        params = {"symbol": symbol}
        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        response = self._make_request("trades", params)
        if response is None:
            return []

        data = response.get("results", [])

        # Cache result
        self._set_cache(cache_key, data)

        logger.info(f"Retrieved {len(data)} trade records for {symbol}")
        return data

    def get_quotes(
        self,
        symbol: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        use_cache: bool = True
    ) -> List[Dict]:
        """
        Get historical quote data from Massive.com.

        Args:
            symbol: Stock ticker
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            use_cache: Use cached data if available

        Returns:
            List of quote records
        """
        cache_key = f"quotes_{symbol}_{start_date}_{end_date}"

        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                logger.debug(f"Using cached quotes for {symbol}")
                return cached

        params = {"symbol": symbol}
        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        response = self._make_request("quotes", params)
        if response is None:
            return []

        data = response.get("results", [])

        # Cache result
        self._set_cache(cache_key, data)

        logger.info(f"Retrieved {len(data)} quote records for {symbol}")
        return data

    def get_fundamentals(self, symbol: str, use_cache: bool = True) -> Optional[Dict]:
        """
        Get fundamental data for a symbol.

        Args:
            symbol: Stock ticker
            use_cache: Use cached data if available

        Returns:
            Fundamental data dictionary or None
        """
        cache_key = f"fundamentals_{symbol}"

        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                return cached

        params = {"symbol": symbol}

        response = self._make_request("fundamentals", params)
        if response is None:
            return None

        results = response.get("results", [])
        data = results[0] if results else None

        if data:
            self._set_cache(cache_key, data)

        return data

    def get_next_dividend(self, symbol: str, use_cache: bool = True) -> Optional[Dict]:
        """
        Get next upcoming dividend for a symbol.

        Args:
            symbol: Stock ticker
            use_cache: Use cached data if available

        Returns:
            Next dividend record or None
        """
        dividends = self.get_dividends(symbol, use_cache=use_cache)
        if not dividends:
            return None

        # Filter future dividends and sort by date
        today = datetime.now().date()
        future_dividends = [
            d for d in dividends
            if d.get("exdate") and datetime.strptime(d["exdate"], "%Y-%m-%d").date() >= today
        ]

        if not future_dividends:
            return None

        # Sort by ex-date and return first
        future_dividends.sort(key=lambda x: x.get("exdate", ""))
        return future_dividends[0]

    def is_dividend_blackout(
        self,
        symbol: str,
        blackout_days: int = 2,
        use_cache: bool = True
    ) -> bool:
        """
        Check if symbol is in dividend blackout period.

        Args:
            symbol: Stock ticker
            blackout_days: Days before ex-date to avoid
            use_cache: Use cached data

        Returns:
            True if in blackout period
        """
        next_dividend = self.get_next_dividend(symbol, use_cache=use_cache)
        if not next_dividend:
            return False

        ex_date_str = next_dividend.get("exdate")
        if not ex_date_str:
            return False

        try:
            ex_date = datetime.strptime(ex_date_str, "%Y-%m-%d").date()
            today = datetime.now().date()
            days_to_exdate = (ex_date - today).days

            return 0 <= days_to_exdate <= blackout_days

        except (ValueError, TypeError):
            return False

    def meets_quality_criteria(
        self,
        symbol: str,
        min_market_cap: float = 1e9,
        max_pe_ratio: float = 50.0,
        avoid_penny_stocks: bool = True,
        use_cache: bool = True
    ) -> tuple[bool, str]:
        """
        Check if symbol meets fundamental quality criteria.

        Args:
            symbol: Stock ticker
            min_market_cap: Minimum market cap
            max_pe_ratio: Maximum P/E ratio
            avoid_penny_stocks: Filter out penny stocks (price < $5)
            use_cache: Use cached data

        Returns:
            Tuple of (meets_criteria, reason)
        """
        fundamentals = self.get_fundamentals(symbol, use_cache=use_cache)
        if not fundamentals:
            return (True, "No fundamental data available")  # Allow if no data

        # Check market cap
        market_cap = fundamentals.get("market_cap")
        if market_cap and market_cap < min_market_cap:
            return (False, f"Market cap too low (${market_cap:,.0f} < ${min_market_cap:,.0f})")

        # Check P/E ratio
        pe_ratio = fundamentals.get("pe_ratio")
        if pe_ratio and pe_ratio > max_pe_ratio:
            return (False, f"P/E ratio too high ({pe_ratio:.1f} > {max_pe_ratio:.1f})")

        # Check penny stock
        if avoid_penny_stocks:
            price = fundamentals.get("price")
            if price and price < 5.0:
                return (False, f"Penny stock (price ${price:.2f} < $5.00)")

        return (True, "All quality criteria met")

    def clear_cache(self):
        """Clear all cached data."""
        self._cache.clear()
        logger.info("Massive.com cache cleared")

    def get_cache_stats(self) -> Dict:
        """Get cache statistics."""
        now = datetime.now()
        fresh_count = sum(
            1 for item in self._cache.values()
            if (now - item["timestamp"]) < self._cache_duration
        )

        return {
            "total_entries": len(self._cache),
            "fresh_entries": fresh_count,
            "stale_entries": len(self._cache) - fresh_count,
            "cache_duration_seconds": self._cache_duration.total_seconds(),
        }
