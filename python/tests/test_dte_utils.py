"""Tests for dte_utils - days to expiry from YYYYMMDD (C++ binding or fallback)."""

from datetime import datetime, timedelta, date

import pytest

from python.integration.dte_utils import days_to_expiry_from_yyyymmdd, days_to_maturity_from_date


def test_empty_returns_zero():
    assert days_to_expiry_from_yyyymmdd("") == 0


def test_invalid_length_returns_zero():
    assert days_to_expiry_from_yyyymmdd("202601") == 0
    assert days_to_expiry_from_yyyymmdd("202601011") == 0


def test_future_expiry_positive():
    future = (datetime.now() + timedelta(days=30)).strftime("%Y%m%d")
    dte = days_to_expiry_from_yyyymmdd(future)
    assert dte >= 28  # calendar or trading days
    assert dte <= 35


def test_past_expiry_zero():
    past = (datetime.now() - timedelta(days=1)).strftime("%Y%m%d")
    assert days_to_expiry_from_yyyymmdd(past) == 0


# --- days_to_maturity_from_date (T-bills, bonds) ---


def test_days_to_maturity_empty_none():
    assert days_to_maturity_from_date("") is None
    assert days_to_maturity_from_date(None) is None


def test_days_to_maturity_iso_future():
    future = (date.today() + timedelta(days=91)).isoformat()
    dte = days_to_maturity_from_date(future)
    assert dte is not None
    assert 88 <= dte <= 95


def test_days_to_maturity_iso_past():
    past = (date.today() - timedelta(days=1)).isoformat()
    assert days_to_maturity_from_date(past) == 0


def test_days_to_maturity_yyyymmdd():
    future = (date.today() + timedelta(days=30)).strftime("%Y%m%d")
    dte = days_to_maturity_from_date(future)
    assert dte is not None
    assert 28 <= dte <= 32


def test_days_to_maturity_iso_with_time():
    future = (date.today() + timedelta(days=7)).isoformat() + "T00:00:00"
    dte = days_to_maturity_from_date(future)
    assert dte is not None
    assert 6 <= dte <= 8


def test_days_to_maturity_invalid_returns_none():
    assert days_to_maturity_from_date("not-a-date") is None
    assert days_to_maturity_from_date("2025-13-01") is None  # bad month
