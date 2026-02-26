"""
mongo_blotter.py - MongoDB trade blotter for rich trade documents.

Stores denormalized trade records with embedded option chain snapshots,
greeks, and execution metadata that are impractical in a relational schema.
"""
from __future__ import annotations

import logging
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

logger = logging.getLogger(__name__)

try:
    from pymongo import MongoClient, DESCENDING
    MONGO_AVAILABLE = True
except ImportError:
    MONGO_AVAILABLE = False


class MongoTradeBlotter:
    """Append-only trade blotter backed by MongoDB."""

    def __init__(
        self,
        uri: str = "mongodb://localhost:27017",
        database: str = "ib_platform",
        collection: str = "trades",
    ) -> None:
        if not MONGO_AVAILABLE:
            raise RuntimeError("pymongo not installed: pip install pymongo")
        self._client: MongoClient = MongoClient(uri, serverSelectionTimeoutMS=3000)
        self._db = self._client[database]
        self._coll = self._db[collection]

    def record_trade(self, trade: Dict[str, Any]) -> str:
        doc = {
            **trade,
            "recorded_at": datetime.now(timezone.utc),
        }
        result = self._coll.insert_one(doc)
        return str(result.inserted_id)

    def get_trades(
        self,
        symbol: Optional[str] = None,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        query: Dict[str, Any] = {}
        if symbol:
            query["symbol"] = symbol
        cursor = self._coll.find(query).sort("recorded_at", DESCENDING).limit(limit)
        return [{**doc, "_id": str(doc["_id"])} for doc in cursor]

    def is_healthy(self) -> bool:
        try:
            self._client.admin.command("ping")
            return True
        except Exception:
            return False
