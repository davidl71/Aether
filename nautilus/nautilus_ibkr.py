#!/usr/bin/env python3
"""
NautilusTrader IBKR Integration - Simplified Example

This demonstrates how to use NautilusTrader to connect to Interactive Brokers.

Prerequisites:
1. Install: uv venv && uv pip install nautilus_trader ibapi
2. Have IB Gateway or TWS running on localhost:5000 (paper) or 7496 (live)

Note: The ibapi package requires the TWS API to be installed.
      Either use IB Gateway or set up the TWS API locally.
"""

# Check imports
try:
    from nautilus_trader.adapters.interactive_brokers import config

    print("✓ IBKR config imported")
except ImportError as e:
    print(f"✗ IBKR config import failed: {e}")
    print("  Make sure ibapi is installed: uv pip install ibapi")
    exit(1)


# Example configuration for IBKR
def get_ibkr_config():
    """Get IBKR adapter configuration."""
    return {
        "node_id": "IBKR-001",
        "timeout": 30.0,
    }


def example_live_config():
    """Example: Configure for live trading."""
    from nautilus_trader.config import TradingNodeConfig
    from nautilus_trader.adapters.interactive_brokers import (
        InteractiveBrokersInstrumentProviderConfig,
    )

    config = TradingNodeConfig(
        name="IBKR-Live",
        description="Interactive Brokers live trading",
    )
    return config


if __name__ == "__main__":
    print("NautilusTrader IBKR Integration")
    print("=" * 40)
    print()
    print("To use IBKR with NautilusTrader:")
    print("1. Install: uv pip install nautilus_trader ibapi")
    print("2. Start IB Gateway or TWS")
    print("3. Configure the adapter")
    print()
    print("For Docker-based IB Gateway (recommended):")
    print(
        "  See nautilus_trader.adapters.interactive_brokers.config.DockerizedIBGatewayConfig"
    )
