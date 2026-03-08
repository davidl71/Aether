"""Tests for snapshot_store - persistent SQLite cache for latest snapshot per backend."""

from pathlib import Path

import pytest

# Ensure integration package is importable (same as test_ib_service.py)
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))


def test_save_and_get_latest(tmp_path):
    from integration.snapshot_store import save_latest, get_latest

    db = tmp_path / "snap.db"
    payload = {
        "generated_at": "2025-03-06T12:00:00Z",
        "mode": "DRY-RUN",
        "positions": [{"name": "SPY", "quantity": 100}],
        "symbols": [],
    }
    save_latest("ib", payload, db_path=db)
    out = get_latest("ib", db_path=db)
    assert out is not None
    assert out["generated_at"] == payload["generated_at"]
    assert len(out["positions"]) == 1
    assert out["positions"][0]["name"] == "SPY"


def test_get_missing_backend_returns_none(tmp_path):
    from integration.snapshot_store import save_latest, get_latest

    db = tmp_path / "snap.db"
    save_latest("ib", {"generated_at": "2025-01-01T00:00:00Z"}, db_path=db)
    assert get_latest("alpaca", db_path=db) is None


def test_save_overwrites_previous(tmp_path):
    from integration.snapshot_store import save_latest, get_latest

    db = tmp_path / "snap.db"
    save_latest("ib", {"generated_at": "2025-01-01T10:00:00Z"}, db_path=db)
    save_latest("ib", {"generated_at": "2025-01-01T12:00:00Z"}, db_path=db)
    out = get_latest("ib", db_path=db)
    assert out["generated_at"] == "2025-01-01T12:00:00Z"


def test_save_empty_backend_id_no_op(tmp_path):
    from integration.snapshot_store import save_latest, get_latest

    db = tmp_path / "snap.db"
    save_latest("", {"x": 1}, db_path=db)
    save_latest("ib", {"x": 1}, db_path=db)
    assert get_latest("", db_path=db) is None
    assert get_latest("ib", db_path=db) is not None
