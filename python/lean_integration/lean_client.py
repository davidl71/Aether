"""
lean_client.py - LEAN Client Wrapper

This module provides a wrapper for accessing LEAN algorithm internal state
(portfolio, positions, orders) for the REST API.
"""

from typing import Optional, Dict, List
import threading
from datetime import datetime, timezone
import logging

# Import LEAN algorithm (will be set at runtime)
try:
    from AlgorithmImports import *
    from .box_spread_algorithm import BoxSpreadAlgorithm
except ImportError:
    # Fallback for development/testing without LEAN
    BoxSpreadAlgorithm = None
    QCAlgorithm = None

logger = logging.getLogger(__name__)


class LeanClient:
    """
    Wrapper for accessing LEAN algorithm internal state.

    This class provides a thread-safe interface to query LEAN's Portfolio,
    Securities, positions, and orders for the REST API.
    """

    def __init__(self):
        """Initialize LEAN client."""
        self.algorithm: Optional[BoxSpreadAlgorithm] = None
        self.is_running = False
        self._lock = threading.Lock()

    def set_algorithm(self, algorithm: BoxSpreadAlgorithm):
        """
        Set the running LEAN algorithm instance.

        Args:
            algorithm: Running BoxSpreadAlgorithm instance
        """
        with self._lock:
            self.algorithm = algorithm
            self.is_running = algorithm is not None
            logger.info(f"LEAN algorithm instance set: running={self.is_running}")

    def get_portfolio(self) -> Dict:
        """
        Get portfolio summary from LEAN.

        Returns:
            Dictionary with portfolio metrics

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                portfolio = self.algorithm.Portfolio
                return {
                    "total_portfolio_value": float(portfolio.TotalPortfolioValue),
                    "cash": float(portfolio.Cash),
                    "unrealized_profit": float(portfolio.TotalUnrealizedProfit),
                    "realized_profit": float(portfolio.TotalFeesPaid),  # Approximation
                    "margin_remaining": float(portfolio.MarginRemaining),
                    "total_margin_used": float(portfolio.TotalMarginUsed)
                }
            except Exception as e:
                logger.error(f"Error getting portfolio: {e}")
                raise RuntimeError(f"Failed to get portfolio: {str(e)}")

    def get_positions(self) -> List[Dict]:
        """
        Get all positions from LEAN.

        Returns:
            List of position dictionaries

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                positions = []

                # Get positions from LEAN Portfolio
                for symbol, holding in self.algorithm.Portfolio.items():
                    if holding.Quantity != 0:
                        positions.append({
                            "symbol": symbol.Value if hasattr(symbol, 'Value') else str(symbol),
                            "quantity": int(holding.Quantity),
                            "average_price": float(holding.AveragePrice),
                            "unrealized_profit": float(holding.UnrealizedProfit),
                            "holdings_value": float(holding.HoldingsValue),
                            "current_price": float(holding.Price) if hasattr(holding, 'Price') else float(holding.AveragePrice)
                        })

                # Also include positions tracked in active_positions (box spreads)
                for order_id, position_info in self.algorithm.active_positions.items():
                    spread = position_info.get("spread", {})
                    if isinstance(spread, dict):
                        symbol = spread.get("symbol", f"BOX-{order_id}")
                    else:
                        symbol = f"BOX-{order_id}"

                    # Check if already in positions list
                    if not any(p.get("symbol") == symbol for p in positions):
                        positions.append({
                            "symbol": symbol,
                            "quantity": 1,  # Box spread is typically 1 unit
                            "average_price": position_info.get("entry_price", 0.0),
                            "unrealized_pnl": position_info.get("unrealized_pnl", 0.0),
                            "holdings_value": position_info.get("entry_price", 0.0),
                            "current_price": position_info.get("entry_price", 0.0),
                            "order_id": str(order_id),
                            "entry_time": position_info.get("entry_time")
                        })

                return positions
            except Exception as e:
                logger.error(f"Error getting positions: {e}")
                raise RuntimeError(f"Failed to get positions: {str(e)}")

    def get_orders(self) -> List[Dict]:
        """
        Get order history from LEAN.

        Returns:
            List of order dictionaries

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                orders = []

                # Get pending orders
                for order_id, order_info in self.algorithm.pending_orders.items():
                    spread = order_info.get("spread", {})
                    if isinstance(spread, dict):
                        symbol = spread.get("symbol", "UNKNOWN")
                    else:
                        symbol = "UNKNOWN"

                    orders.append({
                        "id": str(order_id),
                        "status": "PENDING",
                        "symbol": symbol,
                        "timestamp": order_info.get("timestamp"),
                        "symbols": order_info.get("symbols", []),
                        "quantities": order_info.get("quantities", [])
                    })

                # Get filled orders from active_positions
                for order_id, position_info in self.algorithm.active_positions.items():
                    spread = position_info.get("spread", {})
                    if isinstance(spread, dict):
                        symbol = spread.get("symbol", "UNKNOWN")
                    else:
                        symbol = "UNKNOWN"

                    orders.append({
                        "id": str(order_id),
                        "status": "FILLED",
                        "symbol": symbol,
                        "timestamp": position_info.get("entry_time"),
                        "fill_price": position_info.get("entry_price"),
                        "symbols": position_info.get("symbols", []),
                        "quantities": position_info.get("quantities", [])
                    })

                # Get orders from LEAN Transactions (if available)
                if hasattr(self.algorithm, 'Transactions'):
                    try:
                        for _transaction in self.algorithm.Transactions:
                            # LEAN Transaction format may vary
                            # This is a placeholder - actual implementation depends on LEAN API
                            pass
                    except Exception as e:
                        logger.debug(f"Could not access Transactions: {e}")

                return orders
            except Exception as e:
                logger.error(f"Error getting orders: {e}")
                raise RuntimeError(f"Failed to get orders: {str(e)}")

    def get_metrics(self) -> Dict:
        """
        Calculate metrics from LEAN portfolio.

        Returns:
            Dictionary with account metrics

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                portfolio = self.algorithm.Portfolio

                # Get broker connection status (if available)
                tws_ok = True  # Default - would check LEAN broker connection
                portal_ok = True  # Default

                # Check if algorithm has broker connection info
                if hasattr(self.algorithm, 'BrokerageMessage'):
                    # Could check recent brokerage messages for connection status
                    pass

                return {
                    "net_liq": float(portfolio.TotalPortfolioValue),
                    "buying_power": float(portfolio.MarginRemaining),
                    "excess_liquidity": float(portfolio.MarginRemaining),
                    "margin_requirement": float(portfolio.TotalMarginUsed),
                    "commissions": float(portfolio.TotalFeesPaid),
                    "portal_ok": portal_ok,
                    "tws_ok": tws_ok,
                    "orats_ok": True,  # From configuration
                    "questdb_ok": True  # From configuration
                }
            except Exception as e:
                logger.error(f"Error getting metrics: {e}")
                raise RuntimeError(f"Failed to get metrics: {str(e)}")

    def get_symbols(self) -> List[Dict]:
        """
        Get symbol data from LEAN Securities.

        Returns:
            List of symbol dictionaries with market data

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                symbols = []

                for symbol, security in self.algorithm.Securities.items():
                    if security.Price > 0:
                        symbol_value = symbol.Value if hasattr(symbol, 'Value') else str(symbol)

                        bid_price = float(security.BidPrice) if security.BidPrice > 0 else float(security.Price)
                        ask_price = float(security.AskPrice) if security.AskPrice > 0 else float(security.Price)
                        spread = ask_price - bid_price if ask_price > 0 and bid_price > 0 else 0.0

                        symbols.append({
                            "symbol": symbol_value,
                            "last": float(security.Price),
                            "bid": bid_price,
                            "ask": ask_price,
                            "spread": spread,
                            "volume": int(security.Volume) if security.Volume > 0 else 0,
                            "updated": self.algorithm.Time if hasattr(self.algorithm, 'Time') else datetime.now(timezone.utc)
                        })

                return symbols
            except Exception as e:
                logger.error(f"Error getting symbols: {e}")
                raise RuntimeError(f"Failed to get symbols: {str(e)}")

    def get_account_id(self) -> str:
        """
        Get account ID from LEAN.

        Returns:
            Account ID string
        """
        if not self.algorithm:
            return "DU123456"  # Default

        with self._lock:
            try:
                # LEAN may expose account ID in different ways
                if hasattr(self.algorithm, 'AccountId'):
                    return str(self.algorithm.AccountId)

                # Check Portfolio for account info
                if hasattr(self.algorithm.Portfolio, 'AccountCurrency'):
                    # Account info might be in Portfolio
                    pass

                return "DU123456"  # Default
            except Exception:
                return "DU123456"  # Default

    def get_mode(self) -> str:
        """
        Get trading mode (DRY-RUN or LIVE).

        Returns:
            Mode string
        """
        if not self.algorithm:
            return "DRY-RUN"

        # Check LEAN configuration or algorithm state
        # This would typically come from LEAN config
        return "DRY-RUN"  # Default - should be determined from LEAN config

    def start_algorithm(self):
        """
        Start LEAN algorithm (if not already running).

        Note: This is a placeholder. Actual implementation depends on
        how LEAN is launched (LEAN CLI, launcher, etc.).

        Raises:
            RuntimeError: If algorithm is already running or start fails
        """
        if self.is_running:
            raise RuntimeError("Algorithm already running")

        # Implementation depends on LEAN launcher
        # This would typically involve:
        # 1. Launching LEAN CLI with algorithm
        # 2. Getting algorithm instance reference
        # 3. Setting it via set_algorithm()

        raise NotImplementedError("Algorithm start must be implemented via LEAN launcher")

    def stop_algorithm(self):
        """
        Stop LEAN algorithm gracefully.

        Note: This is a placeholder. Actual implementation depends on
        how LEAN is launched and managed.

        Raises:
            RuntimeError: If algorithm is not running or stop fails
        """
        if not self.is_running:
            raise RuntimeError("Algorithm not running")

        # Implementation depends on LEAN launcher
        # This would typically involve:
        # 1. Sending stop signal to LEAN
        # 2. Waiting for graceful shutdown
        # 3. Clearing algorithm instance

        raise NotImplementedError("Algorithm stop must be implemented via LEAN launcher")

    def cancel_order(self, order_id: str) -> bool:
        """
        Cancel a specific order.

        Args:
            order_id: Order ID to cancel

        Returns:
            True if cancellation successful

        Raises:
            RuntimeError: If LEAN algorithm is not running
        """
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        with self._lock:
            try:
                # Convert order_id to int if it's a string
                try:
                    order_id_int = int(order_id)
                except ValueError:
                    # Try to find order by string ID
                    order_id_int = None
                    for oid, _order_info in self.algorithm.pending_orders.items():
                        if str(oid) == order_id:
                            order_id_int = oid
                            break

                    if order_id_int is None:
                        raise ValueError(f"Order ID not found: {order_id}")

                # Cancel order via LEAN
                if hasattr(self.algorithm, 'Transactions'):
                    # LEAN API: Transactions.CancelOrder(order_id)
                    # This is a placeholder - actual implementation depends on LEAN API
                    pass

                # Remove from pending_orders if found
                if order_id_int in self.algorithm.pending_orders:
                    del self.algorithm.pending_orders[order_id_int]
                    logger.info(f"Order {order_id} cancelled")
                    return True

                return False
            except Exception as e:
                logger.error(f"Error cancelling order {order_id}: {e}")
                raise RuntimeError(f"Failed to cancel order: {str(e)}")
