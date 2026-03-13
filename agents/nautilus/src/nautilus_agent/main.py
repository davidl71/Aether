"""NautilusTrader IB agent — entry point.

Usage:
    python -m nautilus_agent.main [config_path]

    config_path defaults to config/default.toml (relative to CWD).

Environment:
    NAUTILUS_CONFIG_PATH  — override config path via env var
"""

from __future__ import annotations

import asyncio
import os
import sys
from pathlib import Path

import structlog
from nautilus_trader.live.node import TradingNode

from nautilus_agent.config import load
from nautilus_agent.nats_bridge import NatsBridge
from nautilus_agent.strategy import BoxSpreadStrategy
from nautilus_agent.generated import messages_pb2 as pb
from google.protobuf import timestamp_pb2
import time

log = structlog.get_logger(__name__)


async def main(config_path: Path) -> None:
    log.info("nautilus_agent.starting", config=str(config_path))

    node_config, strategy_config, raw = load(config_path)
    nats_cfg = raw["nats"]

    bridge = NatsBridge(
        nats_url=nats_cfg["url"],
        source_id=nats_cfg.get("source_id", "nautilus-ib"),
    )
    await bridge.connect()

    strategy = BoxSpreadStrategy(config=strategy_config, nats_bridge=bridge)

    node = TradingNode(config=node_config)
    node.add_strategy(strategy)
    node.build()

    # Publish startup health event
    now = time.time()
    health = pb.BackendHealth(
        backend="nautilus-ib",
        status="ok",
        updated_at=timestamp_pb2.Timestamp(seconds=int(now)),
        hint="NautilusTrader IB agent started (paper mode)" if "7497" in str(raw["ib"].get("port", "")) else "NautilusTrader IB agent started",
    )
    await bridge.publish_health(health)

    try:
        await node.run_async()
    except (KeyboardInterrupt, SystemExit):
        log.info("nautilus_agent.shutdown_requested")
    finally:
        log.info("nautilus_agent.stopping")
        # Publish offline health
        health_down = pb.BackendHealth(
            backend="nautilus-ib",
            status="error",
            updated_at=timestamp_pb2.Timestamp(seconds=int(time.time())),
            error="agent stopped",
        )
        await bridge.publish_health(health_down)
        await bridge.disconnect()
        await node.stop_async()
        log.info("nautilus_agent.stopped")


def _resolve_config(argv: list[str]) -> Path:
    if len(argv) > 1:
        return Path(argv[1])
    env_path = os.environ.get("NAUTILUS_CONFIG_PATH")
    if env_path:
        return Path(env_path)
    default = Path("config/default.toml")
    if not default.exists():
        # Try relative to this file
        here = Path(__file__).parent.parent.parent
        alt = here / "config" / "default.toml"
        if alt.exists():
            return alt
    return default


if __name__ == "__main__":
    config_path = _resolve_config(sys.argv)
    asyncio.run(main(config_path))
