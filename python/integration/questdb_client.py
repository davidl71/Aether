"""
questdb_client.py - Minimal QuestDB ILP writer for market/trade data.
"""
from __future__ import annotations

import logging
import socket
from datetime import datetime
from typing import Dict, Optional


logger = logging.getLogger(__name__)


class QuestDBClient:
    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 9009,
        quote_table: str = "quotes",
        trade_table: str = "trades",
        max_retries: int = 3,
    ) -> None:
        self.host = host
        self.port = port
        self.quote_table = quote_table
        self.trade_table = trade_table
        self.max_retries = max_retries
        self._socket: Optional[socket.socket] = None

    def connect(self) -> None:
        self.close()
        try:
            self._socket = socket.create_connection((self.host, self.port), timeout=1.0)
        except OSError as exc:
            self._socket = None
            logger.error("Failed to connect to QuestDB ILP at %s:%s (%s)", self.host, self.port, exc)

    def close(self) -> None:
        if self._socket:
            try:
                self._socket.close()
            except OSError:
                pass
        self._socket = None

    def _ensure_connection(self) -> bool:
        if self._socket:
            return True
        self.connect()
        return self._socket is not None

    def _send_line(self, line: str) -> None:
        if not self._ensure_connection():
            return

        attempts = 0
        payload = (line + "\n").encode("utf-8")
        while attempts <= self.max_retries:
            try:
                self._socket.sendall(payload)
                return
            except OSError as exc:
                attempts += 1
                logger.warning("QuestDB write failed (%s) retry %d/%d", exc, attempts, self.max_retries)
                self.connect()
        logger.error("Exceeded QuestDB retry budget for line: %s", line[:120])

    @staticmethod
    def _to_timestamp_ns(ts: Optional[datetime]) -> int:
        if ts is None:
            return int(datetime.utcnow().timestamp() * 1e9)
        return int(ts.timestamp() * 1e9)

    def write_quote(self, symbol: str, data: Dict) -> None:
        timestamp = self._to_timestamp_ns(data.get("timestamp"))
        fields = {
            "bid": data.get("bid"),
            "ask": data.get("ask"),
            "spread": data.get("spread"),
            "spread_pct": data.get("spread_pct"),
            "last": data.get("last"),
        }
        field_parts = [f"{name}={value}" for name, value in fields.items() if value is not None]
        line = f"{self.quote_table},symbol={symbol} {','.join(field_parts)} {timestamp}"
        self._send_line(line)

    def write_trade(self, symbol: str, data: Dict) -> None:
        timestamp = self._to_timestamp_ns(data.get("timestamp"))
        fields = {
            "last": data.get("last"),
            "volume": data.get("volume"),
            "last_size": data.get("last_size"),
        }
        field_parts = [f"{name}={value}" for name, value in fields.items() if value is not None]
        line = f"{self.trade_table},symbol={symbol} {','.join(field_parts)} {timestamp}"
        self._send_line(line)

