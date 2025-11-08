"""
strategy_runner.py - Main strategy loop using nautilus_trader events
Implements proper Strategy class pattern with lifecycle methods.
"""
import logging
import asyncio
from typing import Dict, List, Optional, Set
from datetime import datetime

from nautilus_trader.core.nautilus_pyo3 import (
    InstrumentId,
    QuoteTick,
    TradeTick,
)
from nautilus_trader.model.events import OrderFilled, OrderRejected

from .nautilus_client import NautilusClient
from .market_data_handler import MarketDataHandler
from .execution_handler import ExecutionHandler
from .order_factory import OrderFactory
from .option_chain_manager import OptionChainManager
from .notification_center import NotificationCenter
from .data_provider_router import DataProviderRouter
from .questdb_client import QuestDBClient
from .ibkr_portal_client import IBKRPortalClient

# Optional ORATS integration
try:
    from .orats_client import ORATSClient, ORATSEnricher
except ImportError:
    ORATSClient = None
    ORATSEnricher = None

# Import C++ bindings (will be available after compilation)
try:
    from ..bindings.box_spread_bindings import (
        PyBoxSpreadStrategy,
        PyBoxSpreadLeg,
        PyOptionContract,
        PyMarketData,
        OptionType,
        calculate_arbitrage_profit,
        calculate_roi,
    )
except ImportError:
    # Fallback for development
    PyBoxSpreadStrategy = None
    PyBoxSpreadLeg = None
    PyOptionContract = None
    PyMarketData = None
    OptionType = None
    calculate_arbitrage_profit = None
    calculate_roi = None

logger = logging.getLogger(__name__)


class BoxSpreadStrategyRunner:
    """
    Box spread strategy runner following NautilusTrader Strategy pattern.
    Implements lifecycle methods and event-driven architecture.
    """
    
    def __init__(
        self,
        nautilus_client: NautilusClient,
        strategy_config: Dict,
        risk_config: Dict,
        orats_config: Optional[Dict] = None,
        notification_center: Optional[NotificationCenter] = None,
        data_provider_config: Optional[Dict] = None,
        questdb_config: Optional[Dict] = None,
        portal_client: Optional[IBKRPortalClient] = None,
    ):
        """
        Initialize strategy runner.
        
        Args:
            nautilus_client: NautilusTrader client
            strategy_config: Strategy parameters
            risk_config: Risk management configuration
            orats_config: Optional ORATS configuration
        """
        self.client = nautilus_client
        self.strategy_config = strategy_config
        self.risk_config = risk_config
        self.orats_config = orats_config
        self.notifier = notification_center
        self.data_provider_config = data_provider_config or {}
        
        # Components
        self.questdb_config = questdb_config or {}
        self.questdb_client: Optional[QuestDBClient] = None
        self.portal_client = portal_client

        if self.questdb_config.get("enabled", False):
            try:
                self.questdb_client = QuestDBClient(
                    host=self.questdb_config.get("ilp_host", "127.0.0.1"),
                    port=int(self.questdb_config.get("ilp_port", 9009)),
                    quote_table=self.questdb_config.get("quote_table", "quotes"),
                    trade_table=self.questdb_config.get("trade_table", "trades"),
                    max_retries=int(self.questdb_config.get("max_retries", 3)),
                )
                self.questdb_client.connect()
                logger.info("QuestDB ingestion enabled (%s:%s)", self.questdb_config.get("ilp_host"), self.questdb_config.get("ilp_port"))
            except Exception as exc:
                logger.warning("Failed to initialize QuestDB client: %s", exc)
                self.questdb_client = None

        self.market_data_handler = MarketDataHandler(
            questdb_client=self.questdb_client
        )
        self.order_factory = OrderFactory()
        self.execution_handler = ExecutionHandler(
            nautilus_client.exec_client,
            venue="IB",
            order_factory=self.order_factory
        )
        self.option_chain_manager = OptionChainManager()
        
        # Optional ORATS integration
        self.orats_client: Optional[ORATSClient] = None
        self.orats_enricher: Optional[ORATSEnricher] = None
        if orats_config and ORATSClient:
            try:
                self.orats_client = ORATSClient(
                    api_token=orats_config["api_token"],
                    base_url=orats_config.get("base_url", "https://api.orats.io"),
                    cache_duration_seconds=orats_config.get("cache_duration_seconds", 300),
                    rate_limit_per_second=orats_config.get("rate_limit_per_second", 10),
                )
                self.orats_enricher = ORATSEnricher(self.orats_client)
                logger.info("ORATS client initialized")
            except Exception as e:
                logger.warning(f"Failed to initialize ORATS client: {e}")
        
        self.data_router = DataProviderRouter(
            market_data_handler=self.market_data_handler,
            provider_config=self.data_provider_config,
            orats_client=self.orats_client,
            notifier=self.notifier,
        )

        # Initialize C++ strategy (if bindings available)
        self.cpp_strategy: Optional[PyBoxSpreadStrategy] = None
        if PyBoxSpreadStrategy:
            try:
                self.cpp_strategy = PyBoxSpreadStrategy(strategy_config)
            except Exception as e:
                logger.warning(f"Could not initialize C++ strategy: {e}")
        
        # State
        self._running = False
        self._started = False
        self._stopped = False
        self._paused = False
        self._subscribed_instruments: Set[str] = set()
        self._pending_orders: Dict[str, List] = {}  # Track multi-leg orders
        self._active_positions: Dict[str, Dict] = {}
        
        # Configuration
        self.symbols = strategy_config.get("symbols", [])
        self.min_profit = strategy_config.get("min_arbitrage_profit", 0.10)
        self.min_roi = strategy_config.get("min_roi_percent", 0.5)
    
    # ========================================================================
    # Lifecycle Methods (NautilusTrader Pattern)
    # ========================================================================
    
    def on_start(self):
        """
        Called when strategy starts.
        Subscribe to market data and initialize state.
        """
        if self._started:
            logger.warning("Strategy already started")
            return
        
        if not self.client.is_connected():
            logger.error("NautilusTrader client not connected")
            raise RuntimeError("Client not connected")
        
        logger.info("Starting box spread strategy...")

        self._notify(
            event_type="strategy_start",
            title="Strategy started",
            message=f"Monitoring {len(self.symbols)} symbols",
            severity="info",
        )

        if self.portal_client:
            try:
                summary = self.portal_client.get_account_summary()
                nav = summary.get("netLiquidationValue") or summary.get("netLiquidation")
                buying_power = summary.get("buyingPower") or summary.get("availableFunds")
                account_id = summary.get("accountId") or summary.get("account")
                logger.info(
                    "Client Portal summary: account=%s net_liq=%s buying_power=%s",
                    account_id,
                    nav,
                    buying_power,
                )
            except Exception as exc:  # pragma: no cover - network interaction
                logger.warning("Failed to fetch Client Portal account summary: %s", exc)
        
        # Subscribe to market data for all symbols
        for symbol in self.symbols:
            self._subscribe_symbol(symbol)
        
        # Register event callbacks
        self._register_callbacks()
        
        # Initialize state
        self._running = True
        self._started = True
        self._stopped = False
        self._paused = False
        
        logger.info(f"Strategy started - monitoring {len(self.symbols)} symbols")
    
    def on_stop(self):
        """
        Called when strategy stops.
        Clean up subscriptions and cancel pending orders.
        """
        if not self._started or self._stopped:
            return
        
        logger.info("Stopping box spread strategy...")
        
        # Cancel all pending orders
        self._cancel_all_pending_orders()
        
        # Unsubscribe from market data
        for instrument_id in list(self._subscribed_instruments):
            self._unsubscribe_symbol(instrument_id)
        
        # Unregister callbacks
        self._unregister_callbacks()
        
        # Update state
        self._running = False
        self._stopped = True
        self._started = False
        self._paused = False
        
        logger.info("Strategy stopped")

        self._notify(
            event_type="strategy_stop",
            title="Strategy stopped",
            message="Strategy execution halted",
            severity="warning",
        )
    
    def on_reset(self):
        """
        Called when strategy resets.
        Clear state and statistics.
        """
        logger.info("Resetting strategy...")
        
        self._pending_orders.clear()
        self._active_positions.clear()
        self._subscribed_instruments.clear()
        
        # Reset C++ strategy if available
        if self.cpp_strategy and hasattr(self.cpp_strategy, 'reset_statistics'):
            self.cpp_strategy.reset_statistics()
        
        logger.info("Strategy reset complete")
    
    # ========================================================================
    # Event Handlers (Event-Driven Architecture)
    # ========================================================================
    
    def on_quote_tick(self, tick: QuoteTick):
        """
        Handle quote tick event (event-driven).
        Immediately evaluate opportunities on new quotes.
        
        Args:
            tick: QuoteTick from nautilus_trader
        """
        if not self._running:
            return
        
        instrument_id = str(tick.instrument_id)
        
        # Update market data handler
        self.market_data_handler.on_quote_tick(tick)
        
        # Check if this is an option (for box spread evaluation)
        if self._is_option(instrument_id):
            # Update option chain
            self._update_option_chain(instrument_id, tick)
            
            # Evaluate opportunities immediately (event-driven)
            self._evaluate_opportunities(instrument_id)
    
    def on_trade_tick(self, tick: TradeTick):
        """
        Handle trade tick event.
        
        Args:
            tick: TradeTick from nautilus_trader
        """
        if not self._running:
            return
        
        # Update market data handler
        self.market_data_handler.on_trade_tick(tick)
    
    def on_order_filled(self, event: OrderFilled):
        """
        Handle order filled event.
        Update position tracking and check if box spread is complete.
        
        Args:
            event: OrderFilled event
        """
        order_id = str(event.client_order_id)
        instrument_id = str(event.instrument_id)
        
        logger.info(f"Order filled: {order_id} for {instrument_id}")
        
        # Update position tracking
        self._update_position(instrument_id, event)
        
        # Check if box spread is complete
        self._check_box_spread_completion(order_id)
    
    def on_order_rejected(self, event: OrderRejected):
        """
        Handle order rejected event.
        Trigger rollback if part of multi-leg order.
        
        Args:
            event: OrderRejected event
        """
        order_id = str(event.client_order_id)
        reason = getattr(event, 'reason', 'Unknown reason')
        
        logger.warning(f"Order rejected: {order_id} - {reason}")

        self._notify(
            event_type="order_rejected",
            title="Order rejected",
            message=f"Order {order_id} rejected: {reason}",
            severity="warning",
            payload={"order_id": order_id, "reason": reason},
        )
        
        # Check if this is part of a box spread
        self._handle_order_rejection(order_id)
    
    # ========================================================================
    # Private Methods
    # ========================================================================
    
    def _register_callbacks(self):
        """Register all event callbacks."""
        # Register market data callbacks for all symbols
        for symbol in self.symbols:
            self.market_data_handler.register_callback(
                symbol,
                self._on_market_data_update
            )
    
    def _unregister_callbacks(self):
        """Unregister all event callbacks."""
        for symbol in self.symbols:
            self.market_data_handler.unregister_callback(symbol)
    
    def _subscribe_symbol(self, symbol: str):
        """
        Subscribe to market data for a symbol.
        Uses proper InstrumentId format.
        
        Args:
            symbol: Symbol string (e.g., "SPY" or "SPY.US")
        """
        try:
            # Try to parse as InstrumentId, or construct it
            if '.' in symbol:
                instrument_id = InstrumentId.from_str(symbol)
            else:
                # Default to US exchange
                instrument_id = InstrumentId.from_str(f"{symbol}.US")
            
            self.client.subscribe_market_data(instrument_id)
            self._subscribed_instruments.add(str(instrument_id))
            
            logger.info(f"Subscribed to market data: {instrument_id}")
            
        except Exception as e:
            logger.error(f"Failed to subscribe to {symbol}: {e}")
    
    def _unsubscribe_symbol(self, instrument_id: str):
        """Unsubscribe from market data."""
        try:
            if '.' in instrument_id:
                inst_id = InstrumentId.from_str(instrument_id)
            else:
                inst_id = InstrumentId.from_str(f"{instrument_id}.US")
            
            self.client.unsubscribe_market_data(inst_id)
            self._subscribed_instruments.discard(str(inst_id))
            
            logger.info(f"Unsubscribed from market data: {instrument_id}")
            
        except Exception as e:
            logger.error(f"Failed to unsubscribe from {instrument_id}: {e}")
    
    def _on_market_data_update(self, market_data: Dict):
        """
        Handle market data update from handler (callback).
        
        Args:
            market_data: Market data dictionary
        """
        if not self._running:
            return
        
        symbol = market_data.get("symbol")
        logger.debug(f"Market data update for {symbol}")
        
        # Evaluate opportunities using C++ calculations
        if self.cpp_strategy:
            self._evaluate_opportunities(symbol)
    
    def _is_option(self, instrument_id: str) -> bool:
        """
        Check if instrument is an option.
        
        Args:
            instrument_id: Instrument identifier
            
        Returns:
            True if option, False otherwise
        """
        # Simple heuristic: options typically have expiry and strike in symbol
        # This would need proper instrument parsing in production
        return 'C' in instrument_id or 'P' in instrument_id
    
    def _update_option_chain(self, instrument_id: str, tick: QuoteTick):
        """
        Update option chain with new market data.
        
        Args:
            instrument_id: Option instrument ID
            tick: Quote tick
        """
        # Update option chain manager
        try:
            inst_id = InstrumentId.from_str(instrument_id) if isinstance(instrument_id, str) else instrument_id
            self.option_chain_manager.update_option(inst_id, tick)
        except Exception as e:
            logger.debug(f"Failed to update option chain: {e}")
    
    def _evaluate_opportunities(self, symbol: str):
        """
        Evaluate box spread opportunities (event-driven).
        Called immediately on market data updates.
        
        Args:
            symbol: Underlying symbol
        """
        if not self._running:
            return
        
        logger.debug(f"Evaluating opportunities for {symbol}")
        
        # Check ORATS risk events if enabled
        if self.orats_client and self.orats_config:
            should_trade, reason = self.orats_client.should_trade_ticker(
                ticker=symbol,
                earnings_blackout_days=self.orats_config.get("earnings_blackout_days", 7),
                dividend_blackout_days=self.orats_config.get("dividend_blackout_days", 2),
                max_iv_percentile=self.orats_config.get("max_iv_percentile", 80.0),
            )
            
            if not should_trade:
                logger.info(f"Skipping {symbol}: {reason}")
                return
        
        quote, provider = self.data_router.get_quote(symbol)
        if quote is None:
            logger.warning(f"No market data available for {symbol} from configured providers")
            return

        logger.debug(
            "Market data for %s obtained from %s provider", symbol, provider or "unknown"
        )

        # TODO: Implement full opportunity evaluation
        # 1. Get option chain for symbol
        # 2. Find box spread combinations
        # 3. Calculate profitability using C++ functions
        # 4. Execute if profitable
        
        if self.cpp_strategy:
            # Use C++ strategy for evaluation
            pass
    
    def _cancel_all_pending_orders(self):
        """Cancel all pending orders."""
        for order_group in self._pending_orders.values():
            for order in order_group:
                try:
                    order_id = str(order.client_order_id)
                    self.execution_handler.cancel_order(order_id)
                except Exception as e:
                    logger.error(f"Failed to cancel order: {e}")
        
        self._pending_orders.clear()
    
    def _update_position(self, instrument_id: str, event: OrderFilled):
        """
        Update position tracking.
        
        Args:
            instrument_id: Instrument ID
            event: OrderFilled event
        """
        if instrument_id not in self._active_positions:
            self._active_positions[instrument_id] = {
                "quantity": 0,
                "avg_price": 0.0,
                "total_cost": 0.0,
            }
        
        position = self._active_positions[instrument_id]
        fill_qty = int(event.quantity) if hasattr(event, 'quantity') else 0
        fill_price = float(event.price) if hasattr(event, 'price') else 0.0
        
        # Update position
        position["quantity"] += fill_qty
        position["total_cost"] += fill_qty * fill_price
        position["avg_price"] = (
            position["total_cost"] / position["quantity"]
            if position["quantity"] != 0
            else 0.0
        )
    
    def _check_box_spread_completion(self, order_id: str):
        """
        Check if box spread is complete after order fill.
        
        Args:
            order_id: Filled order ID
        """
        # Find which box spread this order belongs to
        for spread_id, orders in self._pending_orders.items():
            order_ids = [str(o.client_order_id) for o in orders]
            if order_id in order_ids:
                # Check if all orders are filled
                # TODO: Implement full completion check
                logger.debug(f"Checking box spread completion: {spread_id}")
                break
    
    def _handle_order_rejection(self, order_id: str):
        """
        Handle order rejection - trigger rollback if needed.
        
        Args:
            order_id: Rejected order ID
        """
        # Find which box spread this order belongs to
        for spread_id, orders in self._pending_orders.items():
            order_ids = [str(o.client_order_id) for o in orders]
            if order_id in order_ids:
                logger.warning(f"Order in box spread {spread_id} rejected - rolling back")
                # Cancel remaining orders
                for order in orders:
                    other_order_id = str(order.client_order_id)
                    if other_order_id != order_id:
                        try:
                            self.execution_handler.cancel_order(other_order_id)
                        except Exception as e:
                            logger.error(f"Failed to cancel order during rollback: {e}")
                # Remove from pending
                self._pending_orders.pop(spread_id, None)
                break
    
    def execute_box_spread(self, spread: PyBoxSpreadLeg) -> bool:
        """
        Execute a box spread trade.
        
        Args:
            spread: Box spread leg to execute
            
        Returns:
            True if order submitted successfully
        """
        if not self._running:
            logger.warning("Strategy not running, cannot execute")
            return False
        
        # Calculate profitability using C++ functions
        if calculate_arbitrage_profit:
            profit = calculate_arbitrage_profit(spread)
            roi = calculate_roi(spread) if calculate_roi else 0.0
            
            if profit < self.min_profit or roi < self.min_roi:
                logger.info(
                    f"Spread does not meet criteria: profit=${profit:.2f}, roi={roi:.2f}%"
                )
                return False
            
            # Create orders using factory
            # TODO: Convert spread to InstrumentIds and prices
            # orders = self.order_factory.create_box_spread_orders(...)
            
            logger.info(f"Executing box spread: profit=${profit:.2f}, roi={roi:.2f}%")
            return True
        
        return False
    
    # ========================================================================
    # Legacy Methods (for backward compatibility)
    # ========================================================================
    
    def start(self):
        """Legacy start method - calls on_start()."""
        self.on_start()
    
    def stop(self):
        """Legacy stop method - calls on_stop()."""
        self.on_stop()

    def pause(self):
        """Pause strategy execution without full teardown."""
        if not self._running:
            return
        logger.info("Pausing strategy execution")
        self.on_stop()
        self._paused = True
        self._notify(
            event_type="strategy_paused",
            title="Strategy paused",
            message="Strategy paused awaiting resume",
            severity="info",
        )

    def resume(self):
        """Resume strategy execution after a pause."""
        if self._running:
            return
        if not self._paused:
            logger.warning("Resume requested but strategy not paused")
            return
        logger.info("Resuming strategy execution")
        self.on_start()
        self._notify(
            event_type="strategy_resumed",
            title="Strategy resumed",
            message="Strategy resumed after pause",
            severity="info",
        )

    @property
    def is_running(self) -> bool:
        return self._running

    @property
    def is_paused(self) -> bool:
        return self._paused

    @property
    def running(self) -> bool:
        """Backwards compatible accessor used by legacy main loop."""
        return self._running

    def _notify(
        self,
        event_type: str,
        title: str,
        message: str,
        severity: str = "info",
        payload: Optional[Dict] = None,
    ) -> None:
        if not self.notifier:
            return
        try:
            self.notifier.notify(
                event_type=event_type,
                title=title,
                message=message,
                severity=severity,
                payload=payload,
            )
        except Exception as exc:  # pragma: no cover - defensive
            logger.error("Failed to dispatch notification: %s", exc)


# Alias for backward compatibility
StrategyRunner = BoxSpreadStrategyRunner



