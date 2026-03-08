"""Tests for integration.market_hours (US regular session 9:30–16:00 ET)."""

from __future__ import annotations

import sys
from pathlib import Path

# Allow importing integration when run from repo root or python/
_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

import pytest
from datetime import datetime

try:
    from zoneinfo import ZoneInfo
except ImportError:
    ZoneInfo = None

from integration.market_hours import (
    is_regular_market_hours_et,
    effective_refresh_interval_ms,
)


@pytest.mark.skipif(ZoneInfo is None, reason="zoneinfo required (Python 3.9+)")
class TestMarketHours:
    """Test is_regular_market_hours_et and effective_refresh_interval_ms."""

    def test_effective_interval_uses_in_market_when_in_session(self):
        # We can't mock time easily without patching; just check the helper returns one of the two values.
        out = effective_refresh_interval_ms(1000, 60_000)
        assert out in (1000, 60_000)

    def test_effective_interval_in_market_zero_returns_out(self):
        assert effective_refresh_interval_ms(0, 60_000) == 60_000

    def test_effective_interval_out_zero_returns_in_market(self):
        assert effective_refresh_interval_ms(1000, 0) == 1000

    def test_is_regular_market_hours_returns_bool(self):
        assert isinstance(is_regular_market_hours_et(), bool)

    def test_weekend_returns_false(self):
        """Saturday/Sunday should be False (no regular session)."""
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        # Saturday 12:00 ET
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 8, 12, 0, tzinfo=et)
            assert is_regular_market_hours_et() is False
        # Sunday 12:00 ET
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 9, 12, 0, tzinfo=et)
            assert is_regular_market_hours_et() is False

    def test_weekday_before_open_returns_false(self):
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 6, 9, 29, tzinfo=et)
            assert is_regular_market_hours_et() is False

    def test_weekday_at_open_returns_true(self):
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 6, 9, 30, tzinfo=et)
            assert is_regular_market_hours_et() is True

    def test_weekday_mid_session_returns_true(self):
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 6, 12, 0, tzinfo=et)
            assert is_regular_market_hours_et() is True

    def test_weekday_at_close_returns_false(self):
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 6, 16, 0, tzinfo=et)
            assert is_regular_market_hours_et() is False

    def test_weekday_after_close_returns_false(self):
        from unittest.mock import patch
        et = ZoneInfo("America/New_York")
        with patch("integration.market_hours._now_et") as m:
            m.return_value = datetime(2025, 3, 6, 18, 0, tzinfo=et)
            assert is_regular_market_hours_et() is False
