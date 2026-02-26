"""
questdb_client.py - QuestDB ILP writer and REST query client for market/trade data.
"""
from __future__ import annotations

import logging
import socket
from datetime import datetime
from typing import Any, Dict, List, Optional

import requests


logger = logging.getLogger(__name__)


class QuestDBClient:
    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 9009,
        http_port: int = 9000,
        quote_table: str = "quotes",
        trade_table: str = "trades",
        max_retries: int = 3,
    ) -> None:
        self.host = host
        self.port = port
        self.http_port = http_port
        self.quote_table = quote_table
        self.trade_table = trade_table
        self.max_retries = max_retries
        self._socket: Optional[socket.socket] = None
        self._http_base = f"http://{host}:{http_port}"

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

    # ------------------------------------------------------------------
    # Query layer (QuestDB REST/HTTP)
    # ------------------------------------------------------------------

    def query(self, sql: str) -> List[Dict[str, Any]]:
        """Execute a SQL query against QuestDB and return rows as dicts."""
        try:
            resp = requests.get(
                f"{self._http_base}/exec",
                params={"query": sql},
                timeout=5,
            )
            resp.raise_for_status()
            payload = resp.json()
            columns = [c["name"] for c in payload.get("columns", [])]
            return [dict(zip(columns, row)) for row in payload.get("dataset", [])]
        except Exception as exc:
            logger.warning("QuestDB query failed: %s", exc)
            return []

    def get_ohlcv(
        self,
        symbol: str,
        interval: str = "1h",
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        """Return OHLCV bars for a symbol using SAMPLE BY."""
        sql = (
            f"SELECT timestamp, first(last) AS open, max(last) AS high, "
            f"min(last) AS low, last(last) AS close, sum(volume) AS volume "
            f"FROM {self.quote_table} "
            f"WHERE symbol = '{symbol}' "
            f"SAMPLE BY {interval} "
            f"ORDER BY timestamp DESC LIMIT {limit}"
        )
        return self.query(sql)

    def get_latest_quotes(self, symbols: Optional[List[str]] = None) -> List[Dict[str, Any]]:
        """Return the most recent quote for each symbol."""
        where = ""
        if symbols:
            syms = ",".join(f"'{s}'" for s in symbols)
            where = f"WHERE symbol IN ({syms}) "
        sql = (
            f"SELECT symbol, last(bid) AS bid, last(ask) AS ask, "
            f"last(last) AS last, last(spread) AS spread, max(timestamp) AS ts "
            f"FROM {self.quote_table} {where}"
            f"LATEST ON timestamp PARTITION BY symbol"
        )
        return self.query(sql)

