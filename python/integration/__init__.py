"""
Integration package for NautilusTrader and ORATS.
"""

__all__ = []

# Core modules that may require nautilus_trader (import conditionally)
try:
    from .ibkr_portal_client import IBKRPortalClient
    __all__.append("IBKRPortalClient")
except ImportError:
    pass

# NautilusTrader-dependent modules (optional dependency)
try:
    from .market_data_handler import MarketDataHandler
    from .execution_handler import ExecutionHandler
    from .order_factory import OrderFactory
    from .option_chain_manager import OptionChainManager
    from .strategy_runner import BoxSpreadStrategyRunner, StrategyRunner
    from .nautilus_client import NautilusClient

    __all__.extend([
        "MarketDataHandler",
        "ExecutionHandler",
        "OrderFactory",
        "OptionChainManager",
        "BoxSpreadStrategyRunner",
        "StrategyRunner",
        "NautilusClient",
    ])
except ImportError:
    # nautilus_trader not available, skip these imports
    pass

try:
    from .orats_client import ORATSClient, ORATSEnricher

    __all__.extend(["ORATSClient", "ORATSEnricher"])
except ImportError:
    pass

try:
    from .massive_client import MassiveClient
    from .massive_websocket import MassiveWebSocketClient, QuoteCrossValidator

    __all__.extend(["MassiveClient", "MassiveWebSocketClient", "QuoteCrossValidator"])
except ImportError:
    pass
