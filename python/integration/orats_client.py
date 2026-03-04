"""
orats_client.py - Client for ORATS API integration
Provides options data, liquidity scores, and corporate events.
"""
import logging
import requests
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import time

logger = logging.getLogger(__name__)

# ORATS API endpoints
ORATS_BASE_URL = "https://api.orats.io"
ORATS_API_VERSION = "v2"


class ORATSClient:
    """
    Client for ORATS API.
    Provides options data with proprietary indicators, liquidity scores, and events.
    
    Reference: https://orats.com/docs
    """
    
    def __init__(
        self,
        api_token: str,
        base_url: str = ORATS_BASE_URL,
        cache_duration_seconds: int = 300,
        rate_limit_per_second: int = 10,
    ):
        """
        Initialize ORATS client.
        
        Args:
            api_token: ORATS API token
            base_url: Base URL for ORATS API
            cache_duration_seconds: Cache duration (default: 5 minutes)
            rate_limit_per_second: Max requests per second
        """
        self.api_token = api_token
        self.base_url = base_url
        self._cache_duration = timedelta(seconds=cache_duration_seconds)
        self._rate_limit = rate_limit_per_second
        
        # HTTP session
        self._session = requests.Session()
        self._session.headers.update({
            "Authorization": f"Token {api_token}",
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
        timeout: int = 10
    ) -> Optional[Dict]:
        """
        Make API request with rate limiting and error handling.
        
        Args:
            endpoint: API endpoint path
            params: Query parameters
            timeout: Request timeout in seconds
            
        Returns:
            Response data or None on error
        """
        # Rate limiting
        self._check_rate_limit()
        
        # Build URL
        url = f"{self.base_url}/{endpoint}"
        
        try:
            logger.debug(f"ORATS API request: {endpoint}")
            response = self._session.get(url, params=params, timeout=timeout)
            response.raise_for_status()
            
            data = response.json()
            logger.debug(f"ORATS API response: {len(str(data))} bytes")
            
            return data
            
        except requests.exceptions.HTTPError as e:
            logger.error(f"ORATS API HTTP error: {e}")
            if hasattr(e.response, 'text'):
                logger.debug(f"Response: {e.response.text[:200]}")
            return None
            
        except requests.exceptions.Timeout:
            logger.error(f"ORATS API timeout for {endpoint}")
            return None
            
        except requests.exceptions.RequestException as e:
            logger.error(f"ORATS API request error: {e}")
            return None
            
        except ValueError as e:
            logger.error(f"ORATS API JSON decode error: {e}")
            return None
    
    def get_strikes(
        self,
        ticker: str,
        trade_date: Optional[str] = None,
        use_cache: bool = True
    ) -> List[Dict]:
        """
        Get option strikes with ORATS indicators.
        
        Args:
            ticker: Stock ticker (e.g., "SPY")
            trade_date: Date in YYYY-MM-DD format (default: today)
            use_cache: Use cached data if available
            
        Returns:
            List of option data with ORATS indicators
        """
        if not trade_date:
            trade_date = datetime.now().strftime("%Y-%m-%d")
        
        cache_key = f"strikes_{ticker}_{trade_date}"
        
        # Check cache
        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                logger.debug(f"Using cached strikes for {ticker}")
                return cached
        
        # Make request
        endpoint = "datav2/strikes"
        params = {
            "ticker": ticker,
            "tradeDate": trade_date,
        }
        
        response = self._make_request(endpoint, params)
        if response is None:
            return []
        
        data = response.get("data", [])
        
        # Cache result
        self._set_cache(cache_key, data)
        
        logger.info(f"Retrieved {len(data)} option strikes for {ticker}")
        return data
    
    def get_core_data(
        self,
        ticker: str,
        trade_date: Optional[str] = None,
        use_cache: bool = True
    ) -> Optional[Dict]:
        """
        Get core options data including IV, Greeks, and proprietary indicators.
        
        Args:
            ticker: Stock ticker
            trade_date: Date in YYYY-MM-DD format
            use_cache: Use cached data if available
            
        Returns:
            Core data dict or None
        """
        if not trade_date:
            trade_date = datetime.now().strftime("%Y-%m-%d")
        
        cache_key = f"core_{ticker}_{trade_date}"
        
        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                return cached
        
        endpoint = "datav2/cores"
        params = {
            "ticker": ticker,
            "tradeDate": trade_date,
        }
        
        response = self._make_request(endpoint, params)
        if response is None:
            return None
        
        data = response.get("data", [])
        result = data[0] if data else None
        
        if result:
            self._set_cache(cache_key, result)
        
        return result
    
    def get_earnings_calendar(
        self,
        ticker: str,
        use_cache: bool = True
    ) -> Optional[Dict]:
        """
        Get earnings calendar for a ticker.
        
        Args:
            ticker: Stock ticker
            use_cache: Use cached data
            
        Returns:
            Earnings data dict or None
        """
        cache_key = f"earnings_{ticker}"
        
        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                return cached
        
        # Get core data which includes earnings
        core_data = self.get_core_data(ticker, use_cache=False)
        if not core_data:
            return None
        
        # Extract earnings information
        earnings_data = {
            "ticker": ticker,
            "next_earnings_date": core_data.get("earningsDate"),
            "next_earnings_time": core_data.get("earningsTime"),
            "days_to_earnings": core_data.get("daysToEarnings"),
        }
        
        self._set_cache(cache_key, earnings_data)
        return earnings_data
    
    def get_dividend_schedule(
        self,
        ticker: str,
        use_cache: bool = True
    ) -> Optional[Dict]:
        """
        Get dividend schedule for a ticker.
        
        Args:
            ticker: Stock ticker
            use_cache: Use cached data
            
        Returns:
            Dividend data dict or None
        """
        cache_key = f"dividend_{ticker}"
        
        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                return cached
        
        # Get core data which includes dividends
        core_data = self.get_core_data(ticker, use_cache=False)
        if not core_data:
            return None
        
        # Extract dividend information
        dividend_data = {
            "ticker": ticker,
            "next_div_ex_date": core_data.get("divExDate"),
            "next_div_amount": core_data.get("divAmount"),
            "div_frequency": core_data.get("divFreq"),
            "div_yield": core_data.get("divYield"),
        }
        
        self._set_cache(cache_key, dividend_data)
        return dividend_data
    
    def is_earnings_blackout(
        self,
        ticker: str,
        blackout_days: int = 7
    ) -> bool:
        """
        Check if ticker is in earnings blackout period.
        
        Args:
            ticker: Stock ticker
            blackout_days: Days before/after earnings to avoid
            
        Returns:
            True if in blackout period
        """
        earnings = self.get_earnings_calendar(ticker)
        if not earnings:
            return False
        
        days_to_earnings = earnings.get("days_to_earnings")
        if days_to_earnings is None:
            return False
        
        return abs(days_to_earnings) <= blackout_days
    
    def is_dividend_blackout(
        self,
        ticker: str,
        blackout_days: int = 2
    ) -> bool:
        """
        Check if ticker is in dividend blackout period.
        
        Args:
            ticker: Stock ticker
            blackout_days: Days before ex-date to avoid
            
        Returns:
            True if in blackout period
        """
        dividend = self.get_dividend_schedule(ticker)
        if not dividend:
            return False
        
        ex_date_str = dividend.get("next_div_ex_date")
        if not ex_date_str:
            return False
        
        try:
            ex_date = datetime.strptime(ex_date_str, "%Y-%m-%d")
            days_to_exdate = (ex_date - datetime.now()).days
            
            return 0 <= days_to_exdate <= blackout_days
            
        except (ValueError, TypeError):
            return False
    
    def get_liquidity_score(
        self,
        ticker: str,
        expiry: str,
        strike: float,
        option_type: str,
        use_cache: bool = True
    ) -> Optional[float]:
        """
        Get ORATS liquidity score for a specific option.
        
        Args:
            ticker: Stock ticker
            expiry: Expiry date (YYYYMMDD or YYYY-MM-DD)
            strike: Strike price
            option_type: "C" for call, "P" for put
            use_cache: Use cached data
            
        Returns:
            Liquidity score (0-100) or None
        """
        strikes = self.get_strikes(ticker, use_cache=use_cache)
        
        # Convert expiry format if needed
        if len(expiry) == 8 and "-" not in expiry:
            # YYYYMMDD → YYYY-MM-DD
            expiry_formatted = f"{expiry[:4]}-{expiry[4:6]}-{expiry[6:8]}"
        else:
            expiry_formatted = expiry
        
        # Find matching option
        for option in strikes:
            if (option.get("expirDate") == expiry_formatted and
                abs(float(option.get("strike", 0)) - strike) < 0.01 and
                option.get("type") == option_type):
                
                # ORATS provides various liquidity metrics
                # Use a composite score if available, or calculate from components
                liquidity = option.get("liquidity", 50.0)
                return float(liquidity) if liquidity else 50.0
        
        return None
    
    def get_historical_data(
        self,
        ticker: str,
        start_date: str,
        end_date: str,
        use_cache: bool = True
    ) -> List[Dict]:
        """
        Get historical options data for backtesting.
        
        Args:
            ticker: Stock ticker
            start_date: Start date (YYYY-MM-DD)
            end_date: End date (YYYY-MM-DD)
            use_cache: Use cached data
            
        Returns:
            List of historical data
        """
        cache_key = f"historical_{ticker}_{start_date}_{end_date}"
        
        if use_cache:
            cached = self._get_cached(cache_key)
            if cached is not None:
                return cached
        
        endpoint = "datav2/hist/strikes"
        params = {
            "ticker": ticker,
            "startDate": start_date,
            "endDate": end_date,
        }
        
        response = self._make_request(endpoint, params)
        if response is None:
            return []
        
        data = response.get("data", [])
        self._set_cache(cache_key, data)
        
        logger.info(f"Retrieved {len(data)} historical records for {ticker}")
        return data
    
    def enrich_option_data(
        self,
        ticker: str,
        expiry: str,
        strike: float,
        option_type: str,
        market_data: Dict,
        use_cache: bool = True
    ) -> Dict:
        """
        Enrich market data with ORATS indicators.
        
        Args:
            ticker: Stock ticker
            expiry: Expiry date
            strike: Strike price
            option_type: "C" or "P"
            market_data: Existing market data dict
            use_cache: Use cached ORATS data
            
        Returns:
            Enhanced market data dict with ORATS fields
        """
        # Get ORATS strikes data
        strikes = self.get_strikes(ticker, use_cache=use_cache)
        
        # Convert expiry format
        if len(expiry) == 8 and "-" not in expiry:
            expiry_formatted = f"{expiry[:4]}-{expiry[4:6]}-{expiry[6:8]}"
        else:
            expiry_formatted = expiry
        
        # Find matching option
        orats_data = None
        for option in strikes:
            if (option.get("expirDate") == expiry_formatted and
                abs(float(option.get("strike", 0)) - strike) < 0.01 and
                option.get("type") == option_type):
                orats_data = option
                break
        
        # Enhance market data with ORATS fields
        enhanced = market_data.copy()
        
        if orats_data:
            # Liquidity metrics
            enhanced["orats_liquidity_score"] = orats_data.get("liquidity", 50.0)
            enhanced["orats_volume"] = orats_data.get("volume", 0)
            enhanced["orats_open_interest"] = orats_data.get("openInterest", 0)
            
            # IV metrics
            enhanced["orats_smoothed_iv"] = orats_data.get("smoothIv", 0.0)
            enhanced["orats_iv_rank"] = orats_data.get("ivRank", 50.0)
            enhanced["orats_iv_percentile"] = orats_data.get("ivPercentile", 50.0)
            
            # Greeks (if available)
            enhanced["orats_delta"] = orats_data.get("delta", 0.0)
            enhanced["orats_gamma"] = orats_data.get("gamma", 0.0)
            enhanced["orats_theta"] = orats_data.get("theta", 0.0)
            enhanced["orats_vega"] = orats_data.get("vega", 0.0)
            
            # Pricing
            enhanced["orats_theo_price"] = orats_data.get("theoPrice", 0.0)
            enhanced["orats_bid_ask_spread_pct"] = orats_data.get("spreadPct", 0.0)
            
            # Execution metrics
            enhanced["orats_slippage_estimate"] = orats_data.get("slippage", 0.0)
            enhanced["orats_execution_probability"] = self._calculate_execution_prob(orats_data)
            
            logger.debug(
                f"Enriched {ticker} {expiry} {strike} {option_type} "
                f"with ORATS data (liquidity={enhanced['orats_liquidity_score']:.1f})"
            )
        else:
            logger.debug(f"No ORATS data found for {ticker} {expiry} {strike} {option_type}")
            # Set default values
            enhanced["orats_liquidity_score"] = 50.0
            enhanced["orats_execution_probability"] = 0.5
        
        return enhanced
    
    def _calculate_execution_prob(self, orats_data: Dict) -> float:
        """
        Calculate execution probability from ORATS metrics.
        
        Args:
            orats_data: ORATS option data
            
        Returns:
            Execution probability (0-1.0)
        """
        liquidity = float(orats_data.get("liquidity", 50.0))
        volume = int(orats_data.get("volume", 0))
        oi = int(orats_data.get("openInterest", 0))
        spread_pct = float(orats_data.get("spreadPct", 5.0))
        
        # Factors:
        # - High liquidity score → high probability
        # - High volume/OI → high probability
        # - Low spread → high probability
        
        liquidity_factor = liquidity / 100.0
        volume_factor = min(volume / 1000.0, 1.0)  # Cap at 1000 volume
        oi_factor = min(oi / 5000.0, 1.0)  # Cap at 5000 OI
        spread_factor = max(0.0, 1.0 - (spread_pct / 10.0))  # 10% spread = 0 prob
        
        # Weighted average
        exec_prob = (
            liquidity_factor * 0.4 +
            volume_factor * 0.2 +
            oi_factor * 0.2 +
            spread_factor * 0.2
        )
        
        return min(1.0, max(0.0, exec_prob))
    
    def get_iv_rank(self, ticker: str, use_cache: bool = True) -> Optional[float]:
        """
        Get IV rank for a ticker (0-100).
        
        Args:
            ticker: Stock ticker
            use_cache: Use cached data
            
        Returns:
            IV rank or None
        """
        core_data = self.get_core_data(ticker, use_cache=use_cache)
        if core_data:
            return core_data.get("ivRank")
        return None
    
    def get_iv_percentile(self, ticker: str, use_cache: bool = True) -> Optional[float]:
        """
        Get IV percentile for a ticker (0-100).
        
        Args:
            ticker: Stock ticker
            use_cache: Use cached data
            
        Returns:
            IV percentile or None
        """
        core_data = self.get_core_data(ticker, use_cache=use_cache)
        if core_data:
            return core_data.get("ivPercentile")
        return None
    
    def should_trade_ticker(
        self,
        ticker: str,
        earnings_blackout_days: int = 7,
        dividend_blackout_days: int = 2,
        max_iv_percentile: float = 80.0,
    ) -> tuple[bool, str]:
        """
        Check if ticker should be traded based on ORATS data.
        
        Args:
            ticker: Stock ticker
            earnings_blackout_days: Avoid N days before/after earnings
            dividend_blackout_days: Avoid N days before ex-date
            max_iv_percentile: Maximum IV percentile to trade
            
        Returns:
            Tuple of (should_trade, reason)
        """
        # Check earnings blackout
        if self.is_earnings_blackout(ticker, earnings_blackout_days):
            return (False, f"In earnings blackout period ({earnings_blackout_days} days)")
        
        # Check dividend blackout
        if self.is_dividend_blackout(ticker, dividend_blackout_days):
            return (False, f"In dividend blackout period ({dividend_blackout_days} days)")
        
        # Check IV percentile
        iv_percentile = self.get_iv_percentile(ticker)
        if iv_percentile and iv_percentile > max_iv_percentile:
            return (False, f"IV percentile too high ({iv_percentile:.1f}% > {max_iv_percentile}%)")
        
        return (True, "All checks passed")
    
    def clear_cache(self):
        """Clear all cached data."""
        self._cache.clear()
        logger.info("ORATS cache cleared")
    
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


class ORATSEnricher:
    """
    Helper class to enrich option chains with ORATS data.
    """
    
    def __init__(self, orats_client: ORATSClient):
        """
        Initialize enricher.
        
        Args:
            orats_client: ORATS client instance
        """
        self.client = orats_client
    
    def enrich_option_chain(
        self,
        ticker: str,
        options: List[Dict],
        trade_date: Optional[str] = None
    ) -> List[Dict]:
        """
        Enrich list of options with ORATS data.
        
        Args:
            ticker: Stock ticker
            options: List of option dicts with market data
            trade_date: Trade date
            
        Returns:
            List of enriched option dicts
        """
        enriched = []
        
        for option in options:
            enhanced = self.client.enrich_option_data(
                ticker=ticker,
                expiry=option.get("expiry", ""),
                strike=option.get("strike", 0.0),
                option_type=option.get("type", "C"),
                market_data=option,
                use_cache=True
            )
            enriched.append(enhanced)
        
        return enriched
    
    def filter_by_liquidity(
        self,
        options: List[Dict],
        min_liquidity_score: float = 70.0
    ) -> List[Dict]:
        """
        Filter options by ORATS liquidity score.
        
        Args:
            options: List of option dicts (with ORATS data)
            min_liquidity_score: Minimum liquidity score
            
        Returns:
            Filtered list
        """
        return [
            opt for opt in options
            if opt.get("orats_liquidity_score", 0.0) >= min_liquidity_score
        ]
    
    def filter_by_iv_percentile(
        self,
        ticker: str,
        max_iv_percentile: float = 80.0
    ) -> bool:
        """
        Check if ticker passes IV percentile filter.
        
        Args:
            ticker: Stock ticker
            max_iv_percentile: Maximum IV percentile
            
        Returns:
            True if passes filter
        """
        iv_percentile = self.client.get_iv_percentile(ticker)
        if iv_percentile is None:
            return True  # No data, allow
        
        return iv_percentile <= max_iv_percentile

