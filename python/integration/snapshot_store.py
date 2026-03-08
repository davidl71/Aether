"""
Persistent store for latest snapshot per backend (SQLite).

Used by the TUI RestProvider to:
- Persist the latest snapshot after each successful fetch so we have something to display
  when the backend is down or on next launch.
- Load the last saved snapshot on startup so the UI shows data immediately while the
  first REST poll is in flight (or when the backend is unreachable).

Environment:
- SNAPSHOT_CACHE_DB: Path to SQLite database (default: ~/.config/ib_box_spread/snapshot_cache.db).
  If unset or empty, the store is disabled (save/get are no-ops or return None).

Usage:
- save_latest(backend_id, payload_dict) — store snapshot JSON for this backend.
- get_latest(backend_id) — return last stored snapshot dict or None.
"""

from __future__ import annotations

import json
import logging
import os
import sqlite3
from pathlib import Path
from typing import Any, Dict, Optional

logger = logging.getLogger(__name__)

_DEFAULT_DB_DIR = Path.home() / ".config" / "ib_box_spread"
_DEFAULT_DB_PATH = _DEFAULT_DB_DIR / "snapshot_cache.db"


def _get_db_path() -> Optional[Path]:
    """Return configured DB path or default; None if persistence is disabled."""
    env = os.getenv("SNAPSHOT_CACHE_DB", "").strip()
    if env.lower() in ("0", "false", "no", "off", "disable", "disabled"):
        return None
    if env:
        return Path(env).expanduser().resolve()
    return _DEFAULT_DB_PATH


def _ensure_db(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(path))
    try:
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS snapshot_cache (
                backend_id TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            """
        )
        conn.commit()
    finally:
        conn.close()


def save_latest(backend_id: str, payload: Dict[str, Any], db_path: Optional[Path] = None) -> None:
    """Store the latest snapshot for the given backend. Overwrites any previous snapshot for that backend."""
    path = db_path or _get_db_path()
    if not path:
        return
    if not backend_id or not payload:
        return
    try:
        _ensure_db(path)
        conn = sqlite3.connect(str(path))
        try:
            from datetime import datetime, timezone
            now = datetime.now(timezone.utc).isoformat()
            conn.execute(
                """
                INSERT INTO snapshot_cache (backend_id, payload_json, created_at)
                VALUES (?, ?, ?)
                ON CONFLICT(backend_id) DO UPDATE SET
                    payload_json = excluded.payload_json,
                    created_at = excluded.created_at
                """,
                (backend_id, json.dumps(payload), now),
            )
            conn.commit()
        finally:
            conn.close()
    except Exception as e:
        logger.debug("Snapshot store save failed for %s: %s", backend_id, e)


def get_latest(backend_id: str, db_path: Optional[Path] = None) -> Optional[Dict[str, Any]]:
    """Return the most recent snapshot dict for the backend, or None if missing or store disabled."""
    path = db_path or _get_db_path()
    if not path or not path.exists() or not backend_id:
        return None
    try:
        conn = sqlite3.connect(str(path))
        try:
            row = conn.execute(
                "SELECT payload_json FROM snapshot_cache WHERE backend_id = ?",
                (backend_id,),
            ).fetchone()
            if row:
                return json.loads(row[0])
            return None
        finally:
            conn.close()
    except Exception as e:
        logger.debug("Snapshot store get failed for %s: %s", backend_id, e)
        return None
