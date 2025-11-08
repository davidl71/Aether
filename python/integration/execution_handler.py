"""
execution_handler.py - Handles order execution via nautilus_trader
"""
import logging
from typing import Dict, Optional, Callable, List
from nautilus_trader.core.nautilus_pyo3 import (
    Order,
    OrderType,
    TimeInForce,
    OrderSide,
    InstrumentId,
    Venue,
    AccountId,
)
from nautilus_trader.execution.messages import SubmitOrder
from .order_factory import OrderFactory

logger = logging.getLogger(__name__)


class ExecutionHandler:
    """
    Converts C++ Order types to nautilus_trader orders and handles execution.
    """
    
    def __init__(self, exec_client, venue: str = "IB", order_factory: Optional[OrderFactory] = None):
        """
        Initialize execution handler.
        
        Args:
            exec_client: NautilusTrader execution client
            venue: Trading venue identifier
            order_factory: Order factory instance (creates new one if None)
        """
        self.exec_client = exec_client
        self.venue = Venue(venue)
        self._order_factory = order_factory or OrderFactory()
        self._order_callbacks: Dict[str, Callable] = {}
        self._pending_orders: Dict[str, Order] = {}
    
    def register_order_callback(self, order_id: str, callback: Callable):
        """Register callback for order status updates."""
        self._order_callbacks[order_id] = callback
    
    def submit_order(
        self,
        instrument_id: InstrumentId,
        side: OrderSide,
        quantity: int,
        price: Optional[float] = None,
        time_in_force: TimeInForce = TimeInForce.DAY,
    ) -> str:
        """
        Submit an order through nautilus_trader.
        
        Args:
            instrument_id: Instrument to trade
            side: BUY or SELL
            quantity: Number of contracts
            price: Limit price (None for market orders)
            time_in_force: Order time in force
            
        Returns:
            Order ID string
        """
        try:
            # Create order using factory
            if price is not None:
                # Limit order
                order = self._order_factory.limit(
                    instrument_id=instrument_id,
                    side=side,
                    quantity=quantity,
                    price=price,
                    time_in_force=time_in_force,
                    post_only=False,  # Allow immediate execution for arbitrage
                )
            else:
                # Market order
                order = self._order_factory.market(
                    instrument_id=instrument_id,
                    side=side,
                    quantity=quantity,
                    time_in_force=time_in_force,
                )
            
            # Submit order
            submit_order = SubmitOrder(
                trader_id=None,  # Will be set by nautilus_trader
                strategy_id=None,  # Will be set by nautilus_trader
                order=order,
            )
            
            self.exec_client.submit_order(submit_order)
            
            order_id = str(order.client_order_id)
            self._pending_orders[order_id] = order
            
            logger.info(
                f"Submitted order: {order_id} {side} {quantity} {instrument_id} @ {price or 'MARKET'}"
            )
            
            return order_id
            
        except Exception as e:
            logger.error(f"Failed to submit order: {e}")
            raise
    
    def submit_box_spread_orders(
        self,
        long_call_id: InstrumentId,
        short_call_id: InstrumentId,
        long_put_id: InstrumentId,
        short_put_id: InstrumentId,
        long_call_price: float,
        short_call_price: float,
        long_put_price: float,
        short_put_price: float,
        quantity: int = 1,
        time_in_force: TimeInForce = TimeInForce.DAY,
    ) -> List[str]:
        """
        Submit all 4 orders for a box spread.
        
        Args:
            long_call_id: Long call instrument ID
            short_call_id: Short call instrument ID
            long_put_id: Long put instrument ID
            short_put_id: Short put instrument ID
            long_call_price: Long call limit price
            short_call_price: Short call limit price
            long_put_price: Long put limit price
            short_put_price: Short put limit price
            quantity: Number of contracts
            time_in_force: Time in force
            
        Returns:
            List of order IDs
        """
        # Create all orders using factory
        orders = self._order_factory.create_box_spread_orders(
            long_call_id=long_call_id,
            short_call_id=short_call_id,
            long_put_id=long_put_id,
            short_put_id=short_put_id,
            long_call_price=long_call_price,
            short_call_price=short_call_price,
            long_put_price=long_put_price,
            short_put_price=short_put_price,
            quantity=quantity,
            time_in_force=time_in_force,
        )
        
        # Submit all orders
        order_ids = []
        for order in orders:
            try:
                submit_order = SubmitOrder(
                    trader_id=None,
                    strategy_id=None,
                    order=order,
                )
                
                self.exec_client.submit_order(submit_order)
                
                order_id = str(order.client_order_id)
                order_ids.append(order_id)
                self._pending_orders[order_id] = order
                
            except Exception as e:
                logger.error(f"Failed to submit box spread leg: {e}")
                # Rollback: cancel already submitted orders
                for submitted_id in order_ids:
                    self.cancel_order(submitted_id)
                raise
        
        logger.info(f"Submitted box spread: {len(order_ids)} orders - {order_ids}")
        return order_ids
    
    def cancel_order(self, order_id: str):
        """Cancel an order."""
        if order_id in self._pending_orders:
            order = self._pending_orders[order_id]
            # Cancel order via nautilus_trader
            # Implementation depends on nautilus_trader API
            logger.info(f"Cancelling order: {order_id}")
        else:
            logger.warning(f"Order not found: {order_id}")
    
    def on_order_update(self, order: Order):
        """Handle order status update from nautilus_trader."""
        order_id = str(order.client_order_id)
        
        if order_id in self._order_callbacks:
            # Convert nautilus_trader order to C++ Order format
            order_dict = self._convert_order(order)
            self._order_callbacks[order_id](order_dict)
    
    def _convert_order(self, order: Order) -> Dict:
        """
        Convert nautilus_trader Order to C++ Order format.
        
        Returns dict compatible with PyOrder.
        """
        return {
            "order_id": str(order.client_order_id),
            "status": self._convert_order_status(order.status),
            "filled_quantity": int(order.filled_qty) if order.filled_qty else 0,
            "quantity": int(order.quantity) if order.quantity else 0,
            # Additional fields would be populated from order
        }
    
    def _convert_order_status(self, status) -> int:
        """Convert nautilus_trader order status to C++ OrderStatus enum."""
        # Map nautilus_trader status to C++ enum values
        status_map = {
            "INITIALIZED": 0,  # Pending
            "SUBMITTED": 1,
            "ACCEPTED": 1,
            "FILLED": 2,
            "PARTIALLY_FILLED": 3,
            "CANCELED": 4,
            "REJECTED": 5,
            "EXPIRED": 4,
        }
        return status_map.get(str(status), 6)  # Default to Error



