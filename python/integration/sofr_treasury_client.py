"""
sofr_treasury_client.py - Fetch SOFR and Treasury rates for comparison with box spread rates

Uses FRED (Federal Reserve Economic Data); API reference: https://fred.stlouisfed.org/docs/api/fred/

This module fetches benchmark risk-free rates from:
- SOFR (Secured Overnight Financing Rate) from Federal Reserve
- Treasury rates from various sources
- SOFR futures from CME Group

Falls back to yfinance when FRED is unavailable.

References:
- FRED API (St. Louis Fed): https://fred.stlouisfed.org/docs/api/fred/
- New York Fed: https://libertystreeteconomics.newyorkfed.org/2023/10/options-for-calculating-risk-free-rates/
- CME Group: https://www.cmegroup.com/articles/2025/price-and-hedging-usd-sofr-interest-swaps-with-sofr-futures.html
- Yahoo Finance Treasury Tickers: ^IRX (13W), ^FVX (5Y), ^TNX (10Y), ^TYX (30Y)
"""

from __future__ import annotations

import logging
import os
import requests
from datetime import datetime
from typing import Dict, List, Optional, TYPE_CHECKING
from dataclasses import dataclass

try:
    import yfinance as yf

    YFINANCE_AVAILABLE = True
except ImportError:
    YFINANCE_AVAILABLE = False

try:
    from .onepassword_sdk_helper import (
        getenv_or_resolve,
        get_fred_api_key_from_1password,
    )
except ImportError:

    def getenv_or_resolve(env_var: str, op_ref: str, default: str = "") -> str:
        return os.getenv(env_var, default)

    def get_fred_api_key_from_1password():
        return None


if TYPE_CHECKING:
    from .risk_free_rate_extractor import RiskFreeRateCurve

logger = logging.getLogger(__name__)


@dataclass
class BenchmarkRate:
    """A benchmark risk-free rate (SOFR, Treasury, etc.)."""

    rate_type: str  # "SOFR", "Treasury", "SOFR_Futures"
    tenor: str  # "Overnight", "1M", "3M", "6M", "1Y", etc.
    days_to_expiry: Optional[int]  # Days to expiration (for term rates)
    rate: float  # Annualized rate in percentage
    timestamp: datetime
    source: str  # Data source/provider
    metadata: Optional[Dict] = None


YFINANCE_RATE_TICKERS = {
    "US": {
        "1M": "^IRX",  # 13-week T-bill as proxy for 1M
        "3M": "^IRX",  # 13-week T-bill
        "6M": "^FVX",  # 5-year Treasury (closest to 6M)
        "1Y": "^FVX",  # 5-year Treasury
        "10Y": "^TNX",  # 10-year Treasury
        "30Y": "^TYX",  # 30-year Treasury
    },
    "UK": {
        "1M": "^IRX",  # Fallback to US T-bill
        "3M": "^IRX",
    },
    "EUR": {
        "1M": "^IRX",
        "3M": "^IRX",
    },
}


class YFinanceRateClient:
    """
    Fallback client for fetching interest rates via yfinance.

    Used when FRED API is unavailable (no API key, rate limits, etc.).
    Yahoo Finance provides Treasury yields via CBOE ticker symbols.

    Note: yfinance does not provide true risk-free rates (SOFR, SONIA, €STR),
    only Treasury yields. These are suitable as proxies but not exact equivalents.
    """

    def __init__(self):
        if not YFINANCE_AVAILABLE:
            raise ImportError(
                "yfinance is required for fallback rate fetching. "
                "Install with: pip install yfinance"
            )

    def get_treasury_rates(self) -> List[BenchmarkRate]:
        """
        Fetch US Treasury rates via yfinance.

        Returns:
            List of BenchmarkRate for available maturities
        """
        rates: List[BenchmarkRate] = []

        tickers = {
            "13W": ("^IRX", 91),
            "5Y": ("^FVX", 1825),
            "10Y": ("^TNX", 3650),
            "30Y": ("^TYX", 10950),
        }

        for tenor, (ticker_str, dte) in tickers.items():
            try:
                ticker = yf.Ticker(ticker_str)
                info = ticker.info

                if info and "currentPrice" in info:
                    rate_value = info.get("currentPrice")
                    if rate_value and rate_value > 0:
                        rates.append(
                            BenchmarkRate(
                                rate_type="Treasury",
                                tenor=tenor,
                                days_to_expiry=dte,
                                rate=float(rate_value),
                                timestamp=datetime.now(),
                                source=f"Yahoo Finance ({ticker_str})",
                                metadata={"ticker": ticker_str, "tenor": tenor},
                            )
                        )
            except Exception as e:
                logger.debug(f"Failed to fetch {tenor} from yfinance: {e}")
                continue

        return rates

    def get_international_rates(self, currency: str = "US") -> List[BenchmarkRate]:
        """
        Fetch international rates via yfinance.

        Yahoo Finance has limited international rates. This provides Treasury
        proxies where direct rates are unavailable.

        Args:
            currency: Currency code (US, UK, EUR, etc.)

        Returns:
            List of BenchmarkRate for available maturities
        """
        rates: List[BenchmarkRate] = []

        currency_upper = currency.upper()

        if currency_upper == "US":
            return self.get_treasury_rates()

        # For other currencies, yfinance has limited data
        # Fall back to US Treasury as proxy with warning
        treasury_rates = self.get_treasury_rates()
        for rate in treasury_rates:
            rate.metadata = rate.metadata or {}
            rate.metadata["proxy_for"] = currency_upper
            rate.metadata["warning"] = (
                f"US Treasury as {currency_upper} proxy - limited yfinance data for {currency_upper}"
            )

        return treasury_rates


class SOFRTreasuryClient:
    """
    Client for fetching SOFR and Treasury benchmark rates.

    SOFR (Secured Overnight Financing Rate) is the primary risk-free rate
    benchmark in the US, replacing LIBOR. Treasury rates represent government
    borrowing costs and are also considered risk-free.
    """

    def __init__(
        self,
        frb_base_url: str = "https://markets.newyorkfed.org/api",
        fred_api_key: Optional[str] = None,
        treasury_base_url: str = "https://www.treasurydirect.gov/GA-FI/FedInvest/selectSecurityPriceDate.htm",
    ):
        """
        Initialize SOFR/Treasury client.

        Args:
            frb_base_url: Base URL for Federal Reserve Bank APIs
            fred_api_key: Optional FRED API key (get from https://fred.stlouisfed.org/docs/api/api_key.html)
            treasury_base_url: Base URL for Treasury data
        """
        self.frb_base_url = frb_base_url.rstrip("/")
        self.treasury_base_url = treasury_base_url
        # Optional 1Password op:// ref via OP_FRED_API_KEY_SECRET, or SDK discovery of FRED API item
        self.fred_api_key = fred_api_key or getenv_or_resolve(
            "FRED_API_KEY", "OP_FRED_API_KEY_SECRET", ""
        )
        if not self.fred_api_key:
            discovered = get_fred_api_key_from_1password()
            if discovered:
                self.fred_api_key = discovered
        if not self.fred_api_key and os.getenv("FRED_DEBUG", "").strip().lower() in (
            "1",
            "true",
            "yes",
        ):
            logger.debug(
                "FRED API key not set. Set FRED_API_KEY, export OP_FRED_API_KEY_SECRET='op://vault/FRED API/credential', "
                "or run this process from a shell where you ran: eval $(op signin)"
            )
        self.fred_base_url = "https://api.stlouisfed.org/fred"

        self.session = requests.Session()
        self.session.headers.update(
            {"User-Agent": "IBBoxSpreadGenerator/1.0", "Accept": "application/json"}
        )

    def get_sofr_overnight(self) -> Optional[BenchmarkRate]:
        """
        Get current SOFR overnight rate.

        Uses FRED API (series SOFR) or New York Fed data.

        Returns:
            BenchmarkRate with overnight SOFR, or None if unavailable
        """
        # Try FRED API first (requires API key)
        if self.fred_api_key:
            try:
                # FRED series ID for SOFR
                endpoint = f"{self.fred_base_url}/series/observations"
                params = {
                    "series_id": "SOFR",
                    "api_key": self.fred_api_key,
                    "file_type": "json",
                    "limit": 1,
                    "sort_order": "desc",
                }
                response = self.session.get(endpoint, params=params, timeout=10)

                if response.status_code == 200:
                    data = response.json()
                    observations = data.get("observations", [])
                    if observations:
                        latest = observations[0]
                        rate_value = float(latest.get("value", 0))
                        if rate_value > 0:
                            return BenchmarkRate(
                                rate_type="SOFR",
                                tenor="Overnight",
                                days_to_expiry=1,
                                rate=rate_value,
                                timestamp=datetime.now(),
                                source="FRED (St. Louis Fed)",
                                metadata={
                                    "date": latest.get("date"),
                                    "series_id": "SOFR",
                                },
                            )
            except Exception as e:
                logger.debug(f"FRED API failed, trying alternative: {e}")

        # Fallback: Try New York Fed API
        try:
            # New York Fed SOFR data endpoint
            # Note: Actual endpoint structure may vary
            endpoint = f"{self.frb_base_url}/rates/all"
            response = self.session.get(endpoint, timeout=10)

            if response.status_code == 200:
                data = response.json()
                # Look for SOFR in response (structure varies)
                sofr_data = data.get("sofr") or data.get("SOFR") or {}
                rate_value = sofr_data.get("rate") or sofr_data.get("value") or 0.0
                if isinstance(rate_value, (int, float)) and rate_value > 0:
                    return BenchmarkRate(
                        rate_type="SOFR",
                        tenor="Overnight",
                        days_to_expiry=1,
                        rate=float(rate_value),
                        timestamp=datetime.now(),
                        source="New York Fed",
                        metadata=data,
                    )
        except Exception as e:
            logger.warning(f"Failed to fetch SOFR overnight rate: {e}")

        return None

    def get_sofr_term_rates(self) -> List[BenchmarkRate]:
        """
        Get SOFR term rates (1M, 3M, 6M, 1Y).

        Uses FRED API for SOFR term rates or derives from SOFR futures.
        See: https://www.cmegroup.com/articles/2025/price-and-hedging-usd-sofr-interest-swaps-with-sofr-futures.html

        Returns:
            List of BenchmarkRate objects for different tenors
        """
        rates: List[BenchmarkRate] = []

        # FRED has SOFR term rates (30/90/180 day averages). Do NOT use SOFRINDEX for 1Y:
        # SOFRINDEX is a cumulative index (starts at 1.0), not a rate—using it produced ~1.23% (the index level).
        if self.fred_api_key:
            try:
                term_series = {
                    "1M": ("SOFR30DAYAVG", 30),
                    "3M": ("SOFR90DAYAVG", 90),
                    "6M": ("SOFR180DAYAVG", 180),
                    "1Y": ("SOFRINDEX", 365),  # Approximate
                }

                for tenor, (series_id, dte) in term_series.items():
                    try:
                        endpoint = f"{self.fred_base_url}/series/observations"
                        params = {
                            "series_id": series_id,
                            "api_key": self.fred_api_key,
                            "file_type": "json",
                            "limit": 1,
                            "sort_order": "desc",
                        }
                        response = self.session.get(endpoint, params=params, timeout=10)

                        if response.status_code == 200:
                            data = response.json()
                            observations = data.get("observations", [])
                            if observations:
                                latest = observations[0]
                                rate_value = float(latest.get("value", 0))
                                if rate_value > 0:
                                    rates.append(
                                        BenchmarkRate(
                                            rate_type="SOFR",
                                            tenor=tenor,
                                            days_to_expiry=dte,
                                            rate=rate_value,
                                            timestamp=datetime.now(),
                                            source="FRED (St. Louis Fed)",
                                            metadata={
                                                "date": latest.get("date"),
                                                "series_id": series_id,
                                            },
                                        )
                                    )
                    except Exception as e:
                        logger.debug(f"Failed to fetch {tenor} SOFR term rate: {e}")
                        continue

                # 1Y: derive from SOFR Index year-over-year (index is cumulative, Apr 2 2018 = 1)
                # Fetch ~400 observations (desc); use [0]=latest and [252]~1 year ago (business days)
                try:
                    endpoint = f"{self.fred_base_url}/series/observations"
                    params = {
                        "series_id": "SOFRINDEX",
                        "api_key": self.fred_api_key,
                        "file_type": "json",
                        "limit": 400,
                        "sort_order": "desc",
                    }
                    response = self.session.get(endpoint, params=params, timeout=10)
                    if response.status_code == 200:
                        data = response.json()
                        observations = data.get("observations", [])
                        # ~252 business days ≈ 1 year
                        idx_1y = (
                            min(252, len(observations) - 1)
                            if len(observations) > 1
                            else 0
                        )
                        if len(observations) > idx_1y and idx_1y > 0:
                            latest = observations[0]
                            year_ago = observations[idx_1y]
                            val_now = float(latest.get("value", 0))
                            val_1y_ago = float(year_ago.get("value", 0))
                            if val_1y_ago > 0 and val_now > 0:
                                rate_1y = (val_now / val_1y_ago - 1.0) * 100.0
                                if (
                                    0.1 < rate_1y < 25.0
                                ):  # sanity: plausible annual rate
                                    rates.append(
                                        BenchmarkRate(
                                            rate_type="SOFR",
                                            tenor="1Y",
                                            days_to_expiry=365,
                                            rate=rate_1y,
                                            timestamp=datetime.now(),
                                            source="FRED (St. Louis Fed)",
                                            metadata={
                                                "date": latest.get("date"),
                                                "series_id": "SOFRINDEX",
                                                "note": "Approx from index YoY",
                                            },
                                        )
                                    )
                except Exception as e:
                    logger.debug("Failed to derive 1Y SOFR from SOFRINDEX: %s", e)

            except Exception as e:
                logger.warning(f"Failed to fetch SOFR term rates from FRED: {e}")

        # Alternative: Could derive from CME SOFR futures prices
        # SOFR futures are financially settled based on compounded SOFR
        # Price = 100 - (implied SOFR rate)
        # This would require CME API access or market data feed

        if not rates:
            logger.info("SOFR term rates not available (FRED API key may be required)")

        return rates

    def get_treasury_rates(self) -> List[BenchmarkRate]:
        """
        Get Treasury rates for various maturities.

        Uses FRED API for Treasury constant maturity rates.

        Returns:
            List of BenchmarkRate objects for Treasury maturities
        """
        rates: List[BenchmarkRate] = []

        if not self.fred_api_key:
            logger.debug("FRED API key not provided - Treasury rates unavailable")
            return rates

        try:
            # FRED series IDs for Treasury constant maturity rates
            treasury_series = {
                "1M": ("DGS1MO", 30),
                "3M": ("DGS3MO", 90),
                "6M": ("DGS6MO", 180),
                "1Y": ("DGS1", 365),
                "2Y": ("DGS2", 730),
                "5Y": ("DGS5", 1825),
                "10Y": ("DGS10", 3650),
                "30Y": ("DGS30", 10950),
            }

            for tenor, (series_id, dte) in treasury_series.items():
                try:
                    endpoint = f"{self.fred_base_url}/series/observations"
                    params = {
                        "series_id": series_id,
                        "api_key": self.fred_api_key,
                        "file_type": "json",
                        "limit": 1,
                        "sort_order": "desc",
                    }
                    response = self.session.get(endpoint, params=params, timeout=10)

                    if response.status_code == 200:
                        data = response.json()
                        observations = data.get("observations", [])
                        if observations:
                            latest = observations[0]
                            rate_value = latest.get("value")
                            # FRED uses "." for missing data
                            if rate_value and rate_value != ".":
                                rate_float = float(rate_value)
                                if rate_float > 0:
                                    rates.append(
                                        BenchmarkRate(
                                            rate_type="Treasury",
                                            tenor=tenor,
                                            days_to_expiry=dte,
                                            rate=rate_float,
                                            timestamp=datetime.now(),
                                            source="FRED (St. Louis Fed)",
                                            metadata={
                                                "date": latest.get("date"),
                                                "series_id": series_id,
                                            },
                                        )
                                    )
                except (ValueError, KeyError, TypeError) as e:
                    logger.debug(f"Failed to fetch {tenor} Treasury rate: {e}")
                    continue
                except Exception as e:
                    logger.warning(f"Error fetching Treasury rate {tenor}: {e}")
                    continue

        except Exception as e:
            logger.warning(f"Failed to fetch Treasury rates: {e}")

        # Fallback to yfinance if FRED failed
        if not rates and YFINANCE_AVAILABLE:
            logger.info("FRED Treasury rates unavailable, falling back to yfinance")
            try:
                yf_client = YFinanceRateClient()
                rates = yf_client.get_treasury_rates()
                for rate in rates:
                    rate.metadata = rate.metadata or {}
                    rate.metadata["fallback"] = "yfinance"
            except Exception as e:
                logger.warning(f"yfinance fallback failed: {e}")

        return rates

    def get_benchmark_at_dte(
        self, days_to_expiry: int, tolerance: int = 5
    ) -> Optional[BenchmarkRate]:
        """
        Get benchmark rate closest to specified days to expiry.

        Args:
            days_to_expiry: Target days to expiration
            tolerance: Acceptable difference in days

        Returns:
            BenchmarkRate closest to target DTE, or None
        """
        # Get all available rates
        all_rates: List[BenchmarkRate] = []

        sofr_overnight = self.get_sofr_overnight()
        if sofr_overnight:
            all_rates.append(sofr_overnight)

        all_rates.extend(self.get_sofr_term_rates())
        all_rates.extend(self.get_treasury_rates())

        # Find closest match
        best_match: Optional[BenchmarkRate] = None
        best_diff = float("inf")

        for rate in all_rates:
            if rate.days_to_expiry is not None:
                diff = abs(rate.days_to_expiry - days_to_expiry)
                if diff <= tolerance and diff < best_diff:
                    best_diff = diff
                    best_match = rate

        return best_match


class RateComparison:
    """Compare box spread rates with benchmark rates."""

    @staticmethod
    def calculate_spread(box_spread_rate: float, benchmark_rate: float) -> float:
        """
        Calculate spread between box spread rate and benchmark.

        Args:
            box_spread_rate: Risk-free rate from box spread (annualized %)
            benchmark_rate: Benchmark rate (annualized %)

        Returns:
            Spread in basis points (bps)
        """
        return (box_spread_rate - benchmark_rate) * 100.0

    @staticmethod
    def compare_curves(
        box_spread_curve: RiskFreeRateCurve, benchmark_rates: List[BenchmarkRate]
    ) -> Dict[int, Dict]:
        """
        Compare box spread curve with benchmark rates.

        Args:
            box_spread_curve: Risk-free rate curve from box spreads
            benchmark_rates: List of benchmark rates

        Returns:
            Dictionary mapping DTE to comparison data
        """
        comparison: Dict[int, Dict] = {}

        for point in box_spread_curve.points:
            if not point.is_valid():
                continue

            # Find closest benchmark
            closest_benchmark: Optional[BenchmarkRate] = None
            min_diff = float("inf")

            for benchmark in benchmark_rates:
                if benchmark.days_to_expiry is not None:
                    diff = abs(benchmark.days_to_expiry - point.days_to_expiry)
                    if diff < min_diff:
                        min_diff = diff
                        closest_benchmark = benchmark

            if closest_benchmark and min_diff <= 10:  # Within 10 days
                spread_bps = RateComparison.calculate_spread(
                    point.mid_rate, closest_benchmark.rate
                )

                comparison[point.days_to_expiry] = {
                    "dte": point.days_to_expiry,
                    "box_spread_rate": point.mid_rate,
                    "benchmark_rate": closest_benchmark.rate,
                    "benchmark_type": closest_benchmark.rate_type,
                    "spread_bps": spread_bps,
                    "liquidity_score": point.liquidity_score,
                    "timestamp": point.timestamp.isoformat(),
                }

        return comparison
