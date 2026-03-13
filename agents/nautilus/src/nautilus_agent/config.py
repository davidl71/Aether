"""Parse config/default.toml into NautilusTrader TradingNodeConfig.

NT IB adapter requires two separate clientIds:
  exec client  → ib.client_id        (default 10)
  data client  → ib.client_id + 1    (default 11)
Both must differ from C++ TWSClient (default clientId = 1).
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path
from typing import Any

from nautilus_trader.config import TradingNodeConfig, LoggingConfig
from nautilus_trader.adapters.interactive_brokers.config import (
    InteractiveBrokersGatewayConfig,
    InteractiveBrokersExecClientConfig,
    InteractiveBrokersDataClientConfig,
    InteractiveBrokersInstrumentProviderConfig,
)

from nautilus_agent.strategy import BoxSpreadConfig


def load_toml(path: str | Path) -> dict[str, Any]:
    with open(path, "rb") as f:
        return tomllib.load(f)


def build_node_config(raw: dict[str, Any]) -> TradingNodeConfig:
    """Build TradingNodeConfig from parsed TOML dict."""
    ib = raw["ib"]
    log_cfg = raw.get("logging", {})

    exec_client_id = int(ib["client_id"])
    data_client_id = exec_client_id + 1

    instrument_provider_cfg = InteractiveBrokersInstrumentProviderConfig(
        load_all=False,  # lazy loading; strategy requests specific chains
    )

    exec_client = InteractiveBrokersExecClientConfig(
        ibg_host=ib["host"],
        ibg_port=int(ib["port"]),
        ibg_client_id=exec_client_id,
        account_id=ib.get("account_id"),
        instrument_provider_config=instrument_provider_cfg,
    )

    data_client = InteractiveBrokersDataClientConfig(
        ibg_host=ib["host"],
        ibg_port=int(ib["port"]),
        ibg_client_id=data_client_id,
        instrument_provider_config=instrument_provider_cfg,
    )

    return TradingNodeConfig(
        trader_id="BOX-SPREAD-NT-001",
        logging=LoggingConfig(log_level=log_cfg.get("level", "INFO")),
        exec_clients={"IB": exec_client},
        data_clients={"IB": data_client},
    )


def build_strategy_config(raw: dict[str, Any]) -> BoxSpreadConfig:
    """Build BoxSpreadConfig from parsed TOML dict."""
    s = raw["strategy"]
    return BoxSpreadConfig(
        strategy_id="BOX-SPREAD-STRATEGY-001",
        symbols=tuple(s["symbols"]),
        min_dte=int(s.get("min_days_to_expiry", 30)),
        max_dte=int(s.get("max_days_to_expiry", 90)),
        min_arbitrage_profit=float(s.get("min_arbitrage_profit", 0.10)),
        min_roi_percent=float(s.get("min_roi_percent", 0.5)),
        max_position_size=float(s.get("max_position_size", 10_000.0)),
        max_bid_ask_spread=float(s.get("max_bid_ask_spread", 0.10)),
        min_volume=int(s.get("min_volume", 100)),
        eval_debounce_seconds=float(s.get("eval_debounce_seconds", 1.0)),
        max_contracts_per_symbol=int(s.get("max_contracts_per_symbol", 200)),
    )


def load(config_path: str | Path) -> tuple[TradingNodeConfig, BoxSpreadConfig, dict[str, Any]]:
    """Load config file and return (node_config, strategy_config, raw_dict)."""
    raw = load_toml(config_path)
    return build_node_config(raw), build_strategy_config(raw), raw
