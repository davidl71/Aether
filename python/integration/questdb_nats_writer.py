"""
questdb_nats_writer.py - Subscribe to NATS market-data.tick.* and write ticks to QuestDB via ILP.

DEPRECATED: Prefer the Go nats-questdb-bridge (NATS_USE_CORE=1) run via
scripts/run_questdb_nats_writer.sh when Go is available. This Python module remains
for environments without Go.

Supports:
- Core NATS subject: market-data.tick.> (or market-data.tick.{symbol})
- JSON payloads: envelope {"payload": {symbol, bid, ask, last, volume?, timestamp?}} or flat tick
- Symbol from subject when missing in payload: market-data.tick.SPY -> SPY

Table written: market_data (symbol, bid, ask, last, volume, timestamp).
Compatible with the same schema as agents/go/cmd/nats-questdb-bridge for unified analytics.

Run: python -m python.integration.questdb_nats_writer [--nats-url nats://localhost:4222] [--questdb-host 127.0.0.1] [--questdb-port 9009]
Or: uv run python python/integration/questdb_nats_writer.py
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import os
import sys
from datetime import datetime, timezone
from typing import Any, Optional

# Add project root and python dir for imports when run as script
if __name__ == "__main__" and __package__ is None:
    _script_dir = os.path.dirname(os.path.abspath(__file__))
    _root = os.path.abspath(os.path.join(_script_dir, "..", ".."))
    _python_dir = os.path.join(_root, "python") if os.path.basename(_script_dir) == "integration" else _root
    for _d in (_python_dir, _root):
        if _d not in sys.path:
            sys.path.insert(0, _d)

try:
    import nats
    from nats.aio.client import Client as NATS
    NATS_AVAILABLE = True
except ImportError:
    NATS = None
    NATS_AVAILABLE = False

from integration.questdb_client import QuestDBClient

logger = logging.getLogger(__name__)

# Default NATS subject to subscribe to (Core NATS)
DEFAULT_SUBJECT = "market-data.tick.>"
# QuestDB ILP table name (align with Go bridge)
MARKET_DATA_TABLE = "market_data"


def _parse_tick(data: bytes, subject: str) -> Optional[dict[str, Any]]:
    """Parse a NATS message into a tick dict: symbol, bid, ask, last, volume, timestamp."""
    # Prefer symbol from subject: market-data.tick.SPY -> SPY
    parts = subject.split(".")
    subject_symbol = parts[-1] if len(parts) >= 3 else None

    try:
        raw = json.loads(data.decode("utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError):
        # Future: try protobuf NatsEnvelope + MarketDataEvent here
        logger.debug("Non-JSON payload on %s, skipping", subject)
        return None

    # Envelope format: {"payload": { ... tick ... }}
    if isinstance(raw, dict) and "payload" in raw:
        tick = raw["payload"]
    else:
        tick = raw

    if not isinstance(tick, dict):
        return None

    symbol = tick.get("symbol") or subject_symbol
    if not symbol:
        logger.debug("No symbol in payload or subject %s", subject)
        return None

    bid = tick.get("bid")
    ask = tick.get("ask")
    last = tick.get("last")
    volume = tick.get("volume", 0)
    ts = tick.get("timestamp")

    if bid is None and ask is None and last is None:
        logger.debug("Tick missing bid/ask/last for %s", symbol)
        return None

    def to_ns(t) -> int:
        if t is None:
            return int(datetime.now(timezone.utc).timestamp() * 1e9)
        if isinstance(t, (int, float)):
            if t > 1e15:
                return int(t)
            return int(t * 1e9)
        if isinstance(t, str):
            try:
                dt = datetime.fromisoformat(t.replace("Z", "+00:00"))
                return int(dt.timestamp() * 1e9)
            except ValueError:
                pass
        return int(datetime.now(timezone.utc).timestamp() * 1e9)

    return {
        "symbol": symbol,
        "bid": float(bid) if bid is not None else None,
        "ask": float(ask) if ask is not None else None,
        "last": float(last) if last is not None else None,
        "volume": int(volume) if volume is not None else 0,
        "timestamp_ns": to_ns(ts),
    }


def _ilp_line(tick: dict[str, Any], table: str = MARKET_DATA_TABLE) -> str:
    """Build one ILP line for market_data table."""
    symbol = (tick["symbol"] or "").replace(" ", "\\ ")
    parts = []
    if tick.get("bid") is not None:
        parts.append(f"bid={tick['bid']:.6f}")
    if tick.get("ask") is not None:
        parts.append(f"ask={tick['ask']:.6f}")
    if tick.get("last") is not None:
        parts.append(f"last={tick['last']:.6f}")
    parts.append(f"volume={tick.get('volume', 0)}i")
    ts_ns = tick.get("timestamp_ns") or int(datetime.now(timezone.utc).timestamp() * 1e9)
    return f"{table},symbol={symbol} {','.join(parts)} {ts_ns}\n"


async def run_writer(
    nats_url: str,
    questdb_host: str,
    questdb_port: int,
    subject: str,
    table: str,
) -> None:
    """Connect to NATS, subscribe to subject, write parsed ticks to QuestDB."""
    if not NATS_AVAILABLE:
        logger.error("nats-py not installed - pip install nats-py")
        return

    quest = QuestDBClient(host=questdb_host, port=questdb_port)
    quest.connect()
    if not quest._socket:
        logger.error("QuestDB ILP not reachable at %s:%s", questdb_host, questdb_port)
        return

    nc: Optional[NATS] = None

    async def message_handler(msg):
        tick = _parse_tick(msg.data, msg.subject)
        if not tick:
            return
        line = _ilp_line(tick, table)
        try:
            quest._send_line(line.rstrip())
        except Exception as e:
            logger.warning("QuestDB write failed: %s", e)

    try:
        nc = NATS()
        await nc.connect(
            servers=[nats_url],
            reconnect_time_wait=2,
            max_reconnect_attempts=-1,
            allow_reconnect=True,
        )
        logger.info("Connected to NATS at %s, subscribing to %s", nats_url, subject)
        await nc.subscribe(subject, cb=message_handler)
        logger.info("QuestDB NATS writer running (QuestDB %s:%s, table %s). Ctrl+C to stop.",
                    questdb_host, questdb_port, table)

        # Keep running until cancelled
        while True:
            await asyncio.sleep(3600)
    except asyncio.CancelledError:
        pass
    finally:
        if nc:
            await nc.close()
        quest.close()


def main() -> int:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    parser = argparse.ArgumentParser(description="NATS → QuestDB market data writer")
    parser.add_argument(
        "--nats-url",
        default=os.environ.get("NATS_URL", "nats://localhost:4222"),
        help="NATS server URL",
    )
    parser.add_argument(
        "--questdb-host",
        default=os.environ.get("QUESTDB_ILP_HOST", "127.0.0.1"),
        help="QuestDB ILP host",
    )
    parser.add_argument(
        "--questdb-port",
        type=int,
        default=int(os.environ.get("QUESTDB_ILP_PORT", "9009")),
        help="QuestDB ILP port",
    )
    parser.add_argument(
        "--subject",
        default=os.environ.get("NATS_QUESTDB_SUBJECT", DEFAULT_SUBJECT),
        help="NATS subject to subscribe to",
    )
    parser.add_argument(
        "--table",
        default=os.environ.get("QUESTDB_MARKET_DATA_TABLE", MARKET_DATA_TABLE),
        help="QuestDB table name for market data",
    )
    args = parser.parse_args()

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    try:
        loop.run_until_complete(
            run_writer(
                nats_url=args.nats_url,
                questdb_host=args.questdb_host,
                questdb_port=args.questdb_port,
                subject=args.subject,
                table=args.table,
            )
        )
    except KeyboardInterrupt:
        pass
    finally:
        loop.close()

    return 0


if __name__ == "__main__":
    sys.exit(main())
