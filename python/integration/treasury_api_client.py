"""
treasury_api_client.py - U.S. Treasury Fiscal Data API client for risk-free rate benchmarks

API documentation: https://fiscaldata.treasury.gov/api-documentation/

This module provides access to U.S. Treasury interest rate data from the Fiscal Data API.
The API provides average interest rates on Treasury securities, which can be used as
risk-free rate benchmarks for comparison with box spread implied rates.
No API key required.
"""

from __future__ import annotations

import logging
import requests
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from functools import lru_cache
import time

logger = logging.getLogger(__name__)

# Treasury API base URL
TREASURY_API_BASE = "https://api.fiscaldata.treasury.gov/services/api/fiscal_service"

# Cache duration: 1 hour (Treasury rates are updated daily)
CACHE_DURATION_SECONDS = 3600


@dataclass
class TreasuryRate:
  """A single Treasury interest rate data point."""

  record_date: str  # YYYY-MM-DD format
  security_type: str  # e.g., "Treasury Bills", "Treasury Notes"
  security_term: str  # e.g., "3-Month", "6-Month", "1-Year"
  avg_interest_rate: float  # Average interest rate (%)
  timestamp: datetime

  def is_valid(self) -> bool:
    """Validate that the rate point has meaningful data."""
    return (
        self.avg_interest_rate > 0.0
        and self.record_date != ""
        and self.security_term != ""
    )


class TreasuryAPIClient:
  """
  Client for U.S. Treasury Fiscal Data API.

  Provides methods to fetch Treasury interest rates for use as risk-free rate
  benchmarks in box spread analysis.
  """

  def __init__(self, base_url: str = TREASURY_API_BASE, cache_duration: int = CACHE_DURATION_SECONDS):
    """
    Initialize the Treasury API client.

    Args:
      base_url: Base URL for Treasury API (default: official API)
      cache_duration: Cache duration in seconds (default: 1 hour)
    """
    self.base_url = base_url
    self.cache_duration = cache_duration
    self._rate_cache: Dict[str, Tuple[List[TreasuryRate], datetime]] = {}
    self._last_request_time: Optional[datetime] = None
    self._min_request_interval = 1.0  # Minimum seconds between requests (rate limiting)

  def _make_request(
      self,
      endpoint: str,
      params: Optional[Dict] = None,
      retries: int = 3
  ) -> Dict:
    """
    Make a request to the Treasury API with retry logic.

    Args:
      endpoint: API endpoint path
      params: Query parameters
      retries: Number of retry attempts

    Returns:
      JSON response as dictionary

    Raises:
      requests.RequestException: If request fails after retries
    """
    url = f"{self.base_url}{endpoint}"

    # Rate limiting: respect minimum interval between requests
    if self._last_request_time:
      elapsed = (datetime.now() - self._last_request_time).total_seconds()
      if elapsed < self._min_request_interval:
        time.sleep(self._min_request_interval - elapsed)

    for attempt in range(retries):
      try:
        response = requests.get(url, params=params, timeout=10)
        response.raise_for_status()
        self._last_request_time = datetime.now()
        return response.json()
      except requests.RequestException as e:
        if attempt == retries - 1:
          logger.error(f"Treasury API request failed after {retries} attempts: {e}")
          raise
        logger.warning(f"Treasury API request failed (attempt {attempt + 1}/{retries}): {e}")
        time.sleep(2 ** attempt)  # Exponential backoff

    raise requests.RequestException("Failed to fetch Treasury data")

  def fetch_average_interest_rates(
      self,
      start_date: Optional[str] = None,
      end_date: Optional[str] = None,
      security_type: Optional[str] = None,
      use_cache: bool = True
  ) -> List[TreasuryRate]:
    """
    Fetch average interest rates from Treasury API.

    Args:
      start_date: Start date in YYYY-MM-DD format (default: 30 days ago)
      end_date: End date in YYYY-MM-DD format (default: today)
      security_type: Filter by security type (optional)
      use_cache: Whether to use cached data if available

    Returns:
      List of TreasuryRate objects
    """
    # Check cache
    cache_key = f"{start_date}_{end_date}_{security_type}"
    if use_cache and cache_key in self._rate_cache:
      cached_data, cached_time = self._rate_cache[cache_key]
      if (datetime.now() - cached_time).total_seconds() < self.cache_duration:
        logger.debug("Returning cached Treasury rate data")
        return cached_data

    # Set default dates
    if end_date is None:
      end_date = datetime.now().strftime("%Y-%m-%d")
    if start_date is None:
      start_date = (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d")

    # Build query parameters
    params = {
      "filter": f"record_date:gte:{start_date},record_date:lte:{end_date}",
      "sort": "-record_date",
      "page[size]": 1000,  # Maximum page size
      "format": "json"
    }

    if security_type:
      params["filter"] += f",security_type_desc:eq:{security_type}"

    # Add fields to fetch
    params["fields"] = "record_date,security_type_desc,security_term_desc,avg_interest_rate_amt"

    try:
      # Fetch data
      data = self._make_request("/v2/accounting/od/avg_interest_rates", params)

      # Parse response
      rates: List[TreasuryRate] = []
      for record in data.get("data", []):
        try:
          rate = TreasuryRate(
            record_date=record.get("record_date", ""),
            security_type=record.get("security_type_desc", ""),
            security_term=record.get("security_term_desc", ""),
            avg_interest_rate=float(record.get("avg_interest_rate_amt", 0.0)),
            timestamp=datetime.now()
          )
          if rate.is_valid():
            rates.append(rate)
        except (ValueError, KeyError) as e:
          logger.warning(f"Failed to parse Treasury rate record: {e}")
          continue

      # Cache results
      self._rate_cache[cache_key] = (rates, datetime.now())

      logger.info(f"Fetched {len(rates)} Treasury rate records")
      return rates

    except requests.RequestException as e:
      logger.error(f"Failed to fetch Treasury rates: {e}")
      # Return cached data if available, even if stale
      if cache_key in self._rate_cache:
        logger.warning("Returning stale cached Treasury data due to API error")
        return self._rate_cache[cache_key][0]
      return []

  def get_latest_rate(
      self,
      security_term: str,
      security_type: Optional[str] = None
  ) -> Optional[TreasuryRate]:
    """
    Get the latest Treasury rate for a specific term.

    Args:
      security_term: Security term (e.g., "3-Month", "6-Month", "1-Year")
      security_type: Security type filter (optional)

    Returns:
      Latest TreasuryRate for the specified term, or None if not found
    """
    rates = self.fetch_average_interest_rates(security_type=security_type)

    # Filter by term and get latest
    matching_rates = [
      r for r in rates
      if security_term.lower() in r.security_term.lower()
    ]

    if not matching_rates:
      return None

    # Sort by date (most recent first) and return latest
    matching_rates.sort(key=lambda r: r.record_date, reverse=True)
    return matching_rates[0]

  def get_rate_for_days(
      self,
      days: int,
      security_type: Optional[str] = None
  ) -> Optional[TreasuryRate]:
    """
    Get Treasury rate closest to a specific number of days.

    Args:
      days: Number of days to maturity
      security_type: Security type filter (optional)

    Returns:
      TreasuryRate closest to the specified days, or None if not found
    """
    # Map days to Treasury security terms
    term_mapping = {
      (0, 60): "1-Month",
      (60, 120): "3-Month",
      (120, 240): "6-Month",
      (240, 400): "1-Year",
      (400, 730): "2-Year",
      (730, 1100): "3-Year",
      (1100, 2000): "5-Year",
      (2000, 10000): "10-Year"
    }

    target_term = None
    for (min_days, max_days), term in term_mapping.items():
      if min_days <= days < max_days:
        target_term = term
        break

    if target_term is None:
      # Use closest available term
      if days < 60:
        target_term = "1-Month"
      elif days < 120:
        target_term = "3-Month"
      elif days < 240:
        target_term = "6-Month"
      else:
        target_term = "1-Year"

    return self.get_latest_rate(target_term, security_type)

  def compare_to_box_spread_rate(
      self,
      box_spread_rate: float,
      days_to_expiry: int,
      security_type: Optional[str] = None
  ) -> Optional[Dict]:
    """
    Compare box spread implied rate to Treasury benchmark.

    Args:
      box_spread_rate: Implied rate from box spread (%)
      days_to_expiry: Days to expiration
      security_type: Security type filter (optional)

    Returns:
      Dictionary with comparison metrics, or None if Treasury rate unavailable
    """
    treasury_rate = self.get_rate_for_days(days_to_expiry, security_type)

    if treasury_rate is None:
      return None

    spread_bps = (box_spread_rate - treasury_rate.avg_interest_rate) * 100.0

    return {
      "box_spread_rate": box_spread_rate,
      "treasury_rate": treasury_rate.avg_interest_rate,
      "spread_bps": spread_bps,
      "treasury_security_term": treasury_rate.security_term,
      "treasury_record_date": treasury_rate.record_date,
      "days_to_expiry": days_to_expiry,
      "beats_treasury": spread_bps > 0
    }


# Convenience function for quick access
@lru_cache(maxsize=128)
def get_treasury_benchmark(days_to_expiry: int) -> Optional[float]:
  """
  Get Treasury benchmark rate for a specific number of days (cached).

  Args:
    days_to_expiry: Days to expiration

  Returns:
    Treasury rate (%), or None if unavailable
  """
  client = TreasuryAPIClient()
  rate = client.get_rate_for_days(days_to_expiry)
  return rate.avg_interest_rate if rate else None
