"""
algorithm_hooks.py - Hooks for LEAN Algorithm Event Bridge Integration

This module provides a mechanism to hook LEAN algorithm events into the
WebSocket event bridge. It extends BoxSpreadAlgorithm to call event bridge
methods when events occur.
"""

import logging
from typing import Optional

from .event_bridge import event_bridge

logger = logging.getLogger(__name__)


def hook_algorithm_events(algorithm):
    """
    Hook LEAN algorithm events to event bridge.

    This function wraps the algorithm's event callbacks to also
    call the event bridge, enabling real-time WebSocket updates.

    Args:
        algorithm: BoxSpreadAlgorithm instance
    """
    if event_bridge is None:
        logger.warning("Event bridge not initialized, cannot hook algorithm events")
        return

    # Store original callbacks
    original_on_order_event = algorithm.OnOrderEvent
    original_on_securities_changed = algorithm.OnSecuritiesChanged
    original_on_brokerage_message = algorithm.OnBrokerageMessage

    def wrapped_on_order_event(orderEvent):
        """Wrapped OnOrderEvent that also calls event bridge."""
        # Call original callback
        original_on_order_event(orderEvent)

        # Bridge to WebSocket
        try:
            order = orderEvent.Order
            order_id = order.Id

            # Get order info from algorithm's tracking
            if order_id in algorithm.pending_orders:
                order_info = algorithm.pending_orders[order_id]

                if orderEvent.Status == OrderStatus.Filled:
                    event_bridge.on_order_filled(
                        order_id,
                        order_info,
                        float(orderEvent.FillPrice)
                    )
                elif orderEvent.Status == OrderStatus.Canceled:
                    event_bridge.on_order_cancelled(order_id, order_info)
        except Exception as e:
            logger.error(f"Error bridging order event: {e}")

    def wrapped_on_securities_changed(changes):
        """Wrapped OnSecuritiesChanged that also calls event bridge."""
        # Call original callback
        original_on_securities_changed(changes)

        # Bridge to WebSocket
        try:
            for security in changes.AddedSecurities:
                symbol = security.Symbol.Value if hasattr(security.Symbol, 'Value') else str(security.Symbol)

                # Get market data
                market_data = {
                    "price": float(security.Price) if security.Price > 0 else 0.0,
                    "bid": float(security.BidPrice) if security.BidPrice > 0 else 0.0,
                    "ask": float(security.AskPrice) if security.AskPrice > 0 else 0.0,
                    "volume": int(security.Volume) if security.Volume > 0 else 0
                }

                event_bridge.on_symbol_updated(symbol, market_data)
        except Exception as e:
            logger.error(f"Error bridging securities changed: {e}")

    def wrapped_on_brokerage_message(message):
        """Wrapped OnBrokerageMessage that also calls event bridge."""
        # Call original callback
        original_on_brokerage_message(message)

        # Bridge to WebSocket
        try:
            if message.Type == BrokerageMessageType.Error:
                event_bridge.on_alert("error", message.Message)
            elif message.Type == BrokerageMessageType.Warning:
                event_bridge.on_alert("warning", message.Message)
        except Exception as e:
            logger.error(f"Error bridging brokerage message: {e}")

    # Replace callbacks with wrapped versions
    algorithm.OnOrderEvent = wrapped_on_order_event
    algorithm.OnSecuritiesChanged = wrapped_on_securities_changed
    algorithm.OnBrokerageMessage = wrapped_on_brokerage_message

    logger.info("Algorithm event hooks installed")


# Import LEAN types (will be available when algorithm is running)
try:
    from AlgorithmImports import OrderStatus, BrokerageMessageType
except ImportError:
    # Fallback for development/testing
    OrderStatus = None
    BrokerageMessageType = None
