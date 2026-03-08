"""
dte_utils.py - Days to expiry from YYYYMMDD (trading-day aware when C++ bindings available)

Single entry point for DTE so Python callers (box_spread_models, benchmarks, risk-free rate)
use C++ tws::calculate_dte (trading days via MarketHours) when box_spread_bindings is
available; otherwise fall back to calendar days.

For T-bills/bonds (maturity on a specific calendar day), use days_to_maturity_from_date()
which accepts ISO or YYYYMMDD and returns calendar days.
"""

from __future__ import annotations

from datetime import datetime, date

try:
    from ..bindings.box_spread_bindings import calculate_dte as _cxx_calculate_dte
    _USE_CXX_DTE = True
except ImportError:
    _USE_CXX_DTE = False


def days_to_expiry_from_yyyymmdd(expiry: str) -> int:
    """
    Days to expiry from YYYYMMDD string.
    Uses C++ trading-day logic when bindings are available; otherwise calendar days.
    """
    if not expiry or len(expiry) != 8:
        return 0
    if _USE_CXX_DTE:
        return _cxx_calculate_dte(expiry)
    try:
        expiry_dt = datetime.strptime(expiry, "%Y%m%d")
        return max(0, (expiry_dt - datetime.now()).days)
    except ValueError:
        return 0


def days_to_maturity_from_date(date_str: str | None) -> int | None:
    """
    Days to maturity/expiry from a date string (T-bills, bonds, options).
    Accepts ISO (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS) or YYYYMMDD.
    Returns calendar days (>= 0), or None if date_str is empty/invalid.
    """
    if not date_str or not isinstance(date_str, str):
        return None
    s = date_str.strip()[:10]
    if len(s) == 10 and s[4] == "-":  # YYYY-MM-DD
        try:
            maturity = date.fromisoformat(s)
            return max(0, (maturity - date.today()).days)
        except ValueError:
            return None
    if len(s) == 8 and s.isdigit():  # YYYYMMDD
        try:
            maturity = datetime.strptime(s, "%Y%m%d").date()
            return max(0, (maturity - date.today()).days)
        except ValueError:
            return None
    return None
