"""
Integration package for NautilusTrader and ORATS.
"""

from .nautilus_client import NautilusClient
from .market_data_handler import MarketDataHandler
from .execution_handler import ExecutionHandler
from .order_factory import OrderFactory
from .option_chain_manager import OptionChainManager
from .strategy_runner import BoxSpreadStrategyRunner, StrategyRunner
from .ibkr_portal_client import IBKRPortalClient

__all__ = [
    "NautilusClient",
    "MarketDataHandler",
    "ExecutionHandler",
    "OrderFactory",
    "OptionChainManager",
    "BoxSpreadStrategyRunner",
    "StrategyRunner",
    "IBKRPortalClient",
]

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
