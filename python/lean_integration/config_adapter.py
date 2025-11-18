"""
config_adapter.py - Configuration adapter for LEAN integration

This module provides utilities to convert between the native C++ configuration
format and LEAN configuration format.
"""

import json
import os
from typing import Dict, Optional, List


class LeanConfigAdapter:
    """Adapter for converting between native and LEAN configurations."""

    @staticmethod
    def native_to_lean_brokerage(native_config: Dict) -> Dict:
        """
        Convert native broker configuration to LEAN format.

        Args:
            native_config: Native configuration dictionary

        Returns:
            LEAN brokerage configuration dictionary
        """
        lean_brokerage = {}

        # Check for IBKR TWS configuration
        if "tws" in native_config:
            tws_config = native_config["tws"]
            lean_brokerage = {
                "brokerage-type": "InteractiveBrokers",
                "interactive-brokers": {
                    "host": tws_config.get("host", "127.0.0.1"),
                    "port": tws_config.get("port", 7497),
                    "account": tws_config.get("account", ""),
                    "username": tws_config.get("username", ""),
                    "password": tws_config.get("password", ""),
                    "trading-mode": "paper" if tws_config.get("port", 7497) == 7497 else "live"
                }
            }

        # Check for Alpaca configuration
        elif "alpaca" in native_config:
            alpaca_config = native_config.get("alpaca", {})
            base_url = alpaca_config.get("base_url", "https://paper-api.alpaca.markets")
            lean_brokerage = {
                "brokerage-type": "Alpaca",
                "alpaca": {
                    "key-id": alpaca_config.get("api_key", ""),
                    "secret-key": alpaca_config.get("secret_key", ""),
                    "base-url": base_url,
                    "trading-mode": "paper" if "paper" in base_url else "live"
                }
            }

        return lean_brokerage

    @staticmethod
    def native_to_lean_strategy(native_config: Dict) -> Dict:
        """
        Convert native strategy configuration to LEAN format.

        Args:
            native_config: Native configuration dictionary

        Returns:
            LEAN strategy configuration dictionary
        """
        strategy_config = native_config.get("strategy", {})

        return {
            "symbols": strategy_config.get("symbols", ["SPY"]),
            "min_roi_percent": strategy_config.get("min_roi_percent", 0.5),
            "max_position_size": strategy_config.get("max_position_size", 5),
            "max_risk": strategy_config.get("max_risk", 0.1),
            "min_days_to_expiry": strategy_config.get("min_days_to_expiry", 7),
            "max_days_to_expiry": strategy_config.get("max_days_to_expiry", 60),
            "max_bid_ask_spread": strategy_config.get("max_bid_ask_spread", 0.5),
            "min_volume": strategy_config.get("min_volume", 10),
            "min_open_interest": strategy_config.get("min_open_interest", 100)
        }

    @staticmethod
    def create_lean_config(native_config_path: str, output_path: str) -> bool:
        """
        Create LEAN configuration from native configuration.

        Args:
            native_config_path: Path to native config.json
            output_path: Path to output LEAN config.json

        Returns:
            True if successful, False otherwise
        """
        try:
            # Load native configuration
            with open(native_config_path, 'r') as f:
                native_config = json.load(f)

            # Convert to LEAN format
            lean_config = {
                "algorithm-type-name": "BoxSpreadAlgorithm",
                "algorithm-language": "Python",
                "algorithm-location": "Main/box_spread_algorithm.py",
                "job-queue-handler": "QuantConnect.Queues.JobQueue",
                "api-handler": "QuantConnect.Api.Api",
                "data-folder": "data/",
                "results-folder": "results/",
                "brokerage": LeanConfigAdapter.native_to_lean_brokerage(native_config),
                "data-queue-handler": "QuantConnect.Lean.Engine.DataFeeds.BrokerageDataQueueHandler",
                "data-aggregator": "QuantConnect.Lean.Engine.DataFeeds.AggregationManager"
            }

            # Write LEAN configuration
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            with open(output_path, 'w') as f:
                json.dump(lean_config, f, indent=2)

            return True
        except Exception as e:
            print(f"Error creating LEAN config: {e}")
            return False

    @staticmethod
    def merge_configs(lean_config: Dict, strategy_config: Dict) -> Dict:
        """
        Merge LEAN config with strategy config.

        Args:
            lean_config: LEAN configuration dictionary
            strategy_config: Strategy configuration dictionary

        Returns:
            Merged configuration dictionary
        """
        merged = lean_config.copy()

        # Add strategy config as algorithm parameter
        if "algorithm-parameters" not in merged:
            merged["algorithm-parameters"] = {}

        merged["algorithm-parameters"]["config_path"] = strategy_config.get("config_path", "config/lean_strategy_config.json")

        return merged
