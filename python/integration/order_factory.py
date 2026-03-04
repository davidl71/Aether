"""
order_factory.py - Order factory for creating nautilus_trader orders
"""
import logging
from typing import List, Optional
from datetime import datetime
from nautilus_trader.core.nautilus_pyo3 import (
    TimeInForce,
    OrderSide,
    InstrumentId,
    Price,
    Quantity,
)
from nautilus_trader.model.orders import LimitOrder, MarketOrder
from nautilus_trader.model.identifiers import ClientOrderId, StrategyId, TraderId

logger = logging.getLogger(__name__)


class OrderFactory:
    """
    Factory for creating nautilus_trader orders.
    Follows NautilusTrader best practices for order creation.
    """
    
    def __init__(
        self,
        trader_id: Optional[TraderId] = None,
        strategy_id: Optional[StrategyId] = None,
    ):
        """
        Initialize order factory.
        
        Args:
            trader_id: Trader identifier
            strategy_id: Strategy identifier
        """
        self._trader_id = trader_id
        self._strategy_id = strategy_id
        self._order_counter = 0
    
    def _generate_client_order_id(self) -> ClientOrderId:
        """Generate unique client order ID."""
        self._order_counter += 1
        timestamp = int(datetime.now().timestamp() * 1e9)  # Nanoseconds
        return ClientOrderId(f"BOX-{timestamp}-{self._order_counter}")
    
    def limit(
        self,
        instrument_id: InstrumentId,
        side: OrderSide,
        quantity: int,
        price: float,
        time_in_force: TimeInForce = TimeInForce.DAY,
        post_only: bool = False,
        reduce_only: bool = False,
    ) -> LimitOrder:
        """
        Create a limit order.
        
        Args:
            instrument_id: Instrument to trade
            side: BUY or SELL
            quantity: Number of contracts
            price: Limit price
            time_in_force: Time in force
            post_only: Post-only order (maker only)
            reduce_only: Reduce-only order
            
        Returns:
            LimitOrder instance
        """
        client_order_id = self._generate_client_order_id()
        
        try:
            order = LimitOrder(
                trader_id=self._trader_id,
                strategy_id=self._strategy_id,
                instrument_id=instrument_id,
                client_order_id=client_order_id,
                order_side=side,
                quantity=Quantity.from_int(quantity),
                price=Price.from_str(str(price)),
                time_in_force=time_in_force,
                post_only=post_only,
                reduce_only=reduce_only,
                init_id=None,  # Will be set by nautilus_trader
                ts_init=datetime.now(),
            )
            
            logger.debug(
                f"Created limit order: {client_order_id} {side} {quantity} "
                f"{instrument_id} @ {price}"
            )
            
            return order
            
        except Exception as e:
            logger.error(f"Failed to create limit order: {e}")
            raise
    
    def market(
        self,
        instrument_id: InstrumentId,
        side: OrderSide,
        quantity: int,
        time_in_force: TimeInForce = TimeInForce.DAY,
        reduce_only: bool = False,
    ) -> MarketOrder:
        """
        Create a market order.
        
        Args:
            instrument_id: Instrument to trade
            side: BUY or SELL
            quantity: Number of contracts
            time_in_force: Time in force
            reduce_only: Reduce-only order
            
        Returns:
            MarketOrder instance
        """
        client_order_id = self._generate_client_order_id()
        
        try:
            order = MarketOrder(
                trader_id=self._trader_id,
                strategy_id=self._strategy_id,
                instrument_id=instrument_id,
                client_order_id=client_order_id,
                order_side=side,
                quantity=Quantity.from_int(quantity),
                time_in_force=time_in_force,
                reduce_only=reduce_only,
                init_id=None,  # Will be set by nautilus_trader
                ts_init=datetime.now(),
            )
            
            logger.debug(
                f"Created market order: {client_order_id} {side} {quantity} {instrument_id}"
            )
            
            return order
            
        except Exception as e:
            logger.error(f"Failed to create market order: {e}")
            raise
    
    def create_box_spread_orders(
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
    ) -> List[LimitOrder]:
        """
        Create all 4 orders for a box spread.
        
        Args:
            long_call_id: Long call instrument ID
            short_call_id: Short call instrument ID
            long_put_id: Long put instrument ID
            short_put_id: Short put instrument ID
            long_call_price: Long call limit price
            short_call_price: Short call limit price
            long_put_price: Long put limit price
            short_put_price: Short put limit price
            quantity: Number of contracts (default: 1)
            time_in_force: Time in force for all orders
            
        Returns:
            List of 4 LimitOrder instances
        """
        orders = []
        
        # Leg 1: Long call (BUY)
        order1 = self.limit(
            instrument_id=long_call_id,
            side=OrderSide.BUY,
            quantity=quantity,
            price=long_call_price,
            time_in_force=time_in_force,
            post_only=False,  # Allow immediate execution for arbitrage
        )
        orders.append(order1)
        
        # Leg 2: Short call (SELL)
        order2 = self.limit(
            instrument_id=short_call_id,
            side=OrderSide.SELL,
            quantity=quantity,
            price=short_call_price,
            time_in_force=time_in_force,
            post_only=False,
        )
        orders.append(order2)
        
        # Leg 3: Long put (BUY)
        order3 = self.limit(
            instrument_id=long_put_id,
            side=OrderSide.BUY,
            quantity=quantity,
            price=long_put_price,
            time_in_force=time_in_force,
            post_only=False,
        )
        orders.append(order3)
        
        # Leg 4: Short put (SELL)
        order4 = self.limit(
            instrument_id=short_put_id,
            side=OrderSide.SELL,
            quantity=quantity,
            price=short_put_price,
            time_in_force=time_in_force,
            post_only=False,
        )
        orders.append(order4)
        
        logger.info(
            f"Created box spread orders: {len(orders)} legs "
            f"(strategy_id={self._strategy_id})"
        )
        
        return orders
    
    def create_combo_order(
        self,
        legs: List[dict],
        time_in_force: TimeInForce = TimeInForce.DAY,
        quantity: int = 1,
        net_price: Optional[float] = None,
        dry_run: bool = False,
    ) -> Optional[dict]:
        """Create an IBKR BAG (combo) order for a multi-leg spread.

        Each *leg* dict should contain:
          - ``symbol``, ``expiry``, ``strike``, ``right`` (C/P)
          - ``action`` (BUY / SELL)
          - ``con_id`` (optional -- IBKR contract ID, resolved upstream)
          - ``ratio`` (optional, default 1)
          - ``exchange`` (optional, default SMART)
          - ``price`` (optional per-leg price for logging)

        When *net_price* is provided the combo is submitted as a LMT
        order at that net debit/credit.  Otherwise the order is sent as
        MKT.

        Args:
            legs: Leg descriptors (at least 2 required).
            time_in_force: DAY, GTC, or IOC.
            quantity: Contract multiplier (number of combos).
            net_price: Net limit price for the combo.
            dry_run: If True, log the order but do not submit.

        Returns:
            A dict describing the combo order (suitable for submission
            via the execution handler), or *None* if validation fails.
        """
        if len(legs) < 2:
            logger.error("Combo order requires at least 2 legs, got %d", len(legs))
            return None

        combo_legs = []
        for i, leg in enumerate(legs):
            action = leg.get("action", "BUY").upper()
            if action not in ("BUY", "SELL"):
                logger.error("Invalid action '%s' in leg %d", action, i)
                return None

            combo_legs.append({
                "con_id": leg.get("con_id", 0),
                "symbol": leg.get("symbol", ""),
                "expiry": leg.get("expiry", ""),
                "strike": float(leg.get("strike", 0)),
                "right": leg.get("right", "C"),
                "action": action,
                "ratio": int(leg.get("ratio", 1)),
                "exchange": leg.get("exchange", "SMART"),
            })

        order_type = "LMT" if net_price is not None else "MKT"
        tif_str = {
            TimeInForce.DAY: "DAY",
            TimeInForce.GTC: "GTC",
            TimeInForce.IOC: "IOC",
        }.get(time_in_force, "DAY")

        client_order_id = self._generate_client_order_id()

        combo_order = {
            "order_id": str(client_order_id),
            "sec_type": "BAG",
            "order_type": order_type,
            "time_in_force": tif_str,
            "quantity": quantity,
            "net_price": net_price,
            "legs": combo_legs,
            "created_at": datetime.now().isoformat(),
            "dry_run": dry_run,
        }

        if dry_run:
            logger.info(
                "DRY RUN combo order %s: %d legs, type=%s, net_price=%s, qty=%d",
                client_order_id, len(combo_legs), order_type,
                net_price, quantity,
            )
            for cl in combo_legs:
                logger.info(
                    "  Leg: %s %s %s %.1f%s con_id=%s",
                    cl["action"], cl["symbol"], cl["expiry"],
                    cl["strike"], cl["right"], cl["con_id"],
                )
        else:
            logger.info(
                "Created combo order %s: %d legs, type=%s, net_price=%s, qty=%d",
                client_order_id, len(combo_legs), order_type,
                net_price, quantity,
            )

        return combo_order

