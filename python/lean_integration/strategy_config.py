"""
strategy_config.py - Strategy configuration manager for LEAN algorithm

This module provides configuration management for the box spread strategy.
"""

import json
import os
from typing import List, Dict, Optional


class StrategyConfig:
    """Strategy configuration manager."""

    def __init__(self, config_path: Optional[str] = None):
        """
        Initialize configuration.

        Args:
            config_path: Path to configuration JSON file
        """
        if config_path and os.path.exists(config_path):
            with open(config_path, 'r') as f:
                self.config = json.load(f)
        else:
            # Use defaults
            self.config = self._default_config()

    @staticmethod
    def default() -> 'StrategyConfig':
        """Create default configuration."""
        return StrategyConfig(None)

    def _default_config(self) -> Dict:
        """Get default configuration."""
        return {
            "strategy": {
                "symbols": ["SPY"],
                "min_roi_percent": 0.5,
                "max_position_size": 5,
                "max_risk": 0.1,
                "min_days_to_expiry": 7,
                "max_days_to_expiry": 60,
                "max_bid_ask_spread": 0.5,
                "min_volume": 10,
                "min_open_interest": 100
            }
        }

    def get_symbols(self) -> List[str]:
        """Get list of symbols to trade."""
        return self.config.get("strategy", {}).get("symbols", ["SPY"])

    def get_min_roi(self) -> float:
        """Get minimum ROI threshold (as decimal, e.g., 0.5 for 0.5%)."""
        return self.config.get("strategy", {}).get("min_roi_percent", 0.5) / 100.0

    def get_max_position_size(self) -> int:
        """Get maximum position size."""
        return self.config.get("strategy", {}).get("max_position_size", 5)

    def get_max_risk(self) -> float:
        """Get maximum risk score."""
        return self.config.get("strategy", {}).get("max_risk", 0.1)

    def get_min_days_to_expiry(self) -> int:
        """Get minimum days to expiry."""
        return self.config.get("strategy", {}).get("min_days_to_expiry", 7)

    def get_max_days_to_expiry(self) -> int:
        """Get maximum days to expiry."""
        return self.config.get("strategy", {}).get("max_days_to_expiry", 60)

    def get_max_bid_ask_spread(self) -> float:
        """Get maximum bid-ask spread."""
        return self.config.get("strategy", {}).get("max_bid_ask_spread", 0.5)

    def get_min_volume(self) -> int:
        """Get minimum volume requirement."""
        return self.config.get("strategy", {}).get("min_volume", 10)

    def get_min_open_interest(self) -> int:
        """Get minimum open interest requirement."""
        return self.config.get("strategy", {}).get("min_open_interest", 100)
