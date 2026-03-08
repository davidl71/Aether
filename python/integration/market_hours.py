"""
US equity market hours (Eastern Time) for refresh-interval logic.

Used by the TUI RestProvider to refresh less often outside regular trading hours
(9:30–16:00 ET, weekdays). Does not include holiday calendar; treats all weekdays
as trading days for simplicity. For full holiday support, use the C++ market_hours
module when exposed via bindings.
"""

from __future__ import annotations

from datetime import datetime
from typing import Optional

try:
    from zoneinfo import ZoneInfo
except ImportError:
    ZoneInfo = None  # type: ignore[misc, assignment]

ET = "America/New_York"


def _now_et() -> Optional[datetime]:
    """Current time in Eastern, or None if zoneinfo unavailable."""
    if ZoneInfo is None:
        return None
    try:
        return datetime.now(ZoneInfo(ET))
    except Exception:
        return None


def is_regular_market_hours_et() -> bool:
    """
    True if current time is within US equity regular session (9:30–16:00 ET) on a weekday.
    Returns False if timezone unavailable (fail closed: assume outside hours).
    """
    now = _now_et()
    if now is None:
        return False
    # Monday = 0, Sunday = 6
    if now.weekday() >= 5:
        return False
    hour, minute = now.hour, now.minute
    if hour == 9:
        return minute >= 30
    return 10 <= hour < 16


def effective_refresh_interval_ms(
    in_market_ms: int,
    out_of_market_ms: int,
) -> int:
    """
    Return in_market_ms when in regular session, else out_of_market_ms.
    Use in poll loops to refresh less often when the market is closed.
    """
    if in_market_ms <= 0:
        return out_of_market_ms
    if out_of_market_ms <= 0:
        return in_market_ms
    return in_market_ms if is_regular_market_hours_et() else out_of_market_ms
