"""
market_data_handler.py - Handles market data events from nautilus_trader
"""
import logging
from typing import Dict, Optional, Callable, TYPE_CHECKING
from datetime import datetime, timezone
from nautilus_trader.core.nautilus_pyo3 import (
    QuoteTick,
    TradeTick,
)

if TYPE_CHECKING:  # pragma: no cover - typing only
    from .questdb_client import QuestDBClient

logger = logging.getLogger(__name__)

# Data quality thresholds
MAX_DATA_AGE_SECONDS = 5.0  # Maximum age for valid data
MIN_BID_ASK_SPREAD = 0.01  # Minimum spread to consider valid
MAX_SPREAD_PERCENT = 10.0  # Maximum spread percentage


class MarketDataHandler:
    """
    Converts nautilus_trader market data events to C++ MarketData format.
    Implements event-driven architecture with data quality validation.
    """
    
    def __init__(
        self,
        max_data_age: float = MAX_DATA_AGE_SECONDS,
        questdb_client: Optional["QuestDBClient"] = None,
    ):
        """
        Initialize market data handler.
        
        Args:
            max_data_age: Maximum age in seconds for valid data
        """
        self._quote_ticks: Dict[str, QuoteTick] = {}
        self._trade_ticks: Dict[str, TradeTick] = {}
        self._callbacks: Dict[str, Callable] = {}
        self._max_data_age = max_data_age
        self._data_quality_stats: Dict[str, Dict] = {}
        self._questdb = questdb_client
    
    def register_callback(self, instrument_id: str, callback: Callable):
        """
        Register a callback for market data updates.
        
        Args:
            instrument_id: Instrument identifier (string or InstrumentId)
            callback: Callback function to call on updates
        """
        instrument_str = str(instrument_id)
        self._callbacks[instrument_str] = callback
        self._data_quality_stats[instrument_str] = {
            "total_ticks": 0,
            "valid_ticks": 0,
            "stale_ticks": 0,
            "invalid_ticks": 0,
        }
    
    def unregister_callback(self, instrument_id: str):
        """Unregister callback for an instrument."""
        instrument_str = str(instrument_id)
        self._callbacks.pop(instrument_str, None)
        self._data_quality_stats.pop(instrument_str, None)
    
    def on_quote_tick(self, tick: QuoteTick):
        """
        Handle quote tick from nautilus_trader (event-driven).
        
        Args:
            tick: QuoteTick from nautilus_trader
        """
        instrument_id = str(tick.instrument_id)
        self._quote_ticks[instrument_id] = tick
        
        # Update statistics
        if instrument_id in self._data_quality_stats:
            self._data_quality_stats[instrument_id]["total_ticks"] += 1
        
        # Convert to C++ MarketData format with quality checks
        market_data = self._convert_quote_tick(tick)
        
        # Only call callback if data is valid
        if market_data is not None:
            if instrument_id in self._data_quality_stats:
                self._data_quality_stats[instrument_id]["valid_ticks"] += 1
            
            # Call registered callback (event-driven)
            if instrument_id in self._callbacks:
                try:
                    self._callbacks[instrument_id](market_data)
                except Exception as e:
                    logger.error(f"Error in market data callback for {instrument_id}: {e}")
            if self._questdb:
                try:
                    self._questdb.write_quote(instrument_id, market_data)
                except Exception as exc:  # pragma: no cover - defensive
                    logger.warning(f"QuestDB quote ingest failed for {instrument_id}: {exc}")
        else:
            if instrument_id in self._data_quality_stats:
                self._data_quality_stats[instrument_id]["invalid_ticks"] += 1
    
    def on_trade_tick(self, tick: TradeTick):
        """
        Handle trade tick from nautilus_trader (event-driven).
        
        Args:
            tick: TradeTick from nautilus_trader
        """
        instrument_id = str(tick.instrument_id)
        self._trade_ticks[instrument_id] = tick
        
        # Convert to C++ MarketData format
        market_data = self._convert_trade_tick(tick)
        
        # Call registered callback if data is valid
        if market_data is not None and instrument_id in self._callbacks:
            try:
                self._callbacks[instrument_id](market_data)
            except Exception as e:
                logger.error(f"Error in market data callback for {instrument_id}: {e}")
        if market_data is not None and self._questdb:
            try:
                self._questdb.write_trade(instrument_id, market_data)
            except Exception as exc:  # pragma: no cover - defensive
                logger.warning(f"QuestDB trade ingest failed for {instrument_id}: {exc}")
    
    def _convert_quote_tick(self, tick: QuoteTick) -> Optional[Dict]:
        """
        Convert nautilus_trader QuoteTick to C++ MarketData format.
        Includes data quality validation.
        
        Returns:
            Dict compatible with PyMarketData, or None if data is invalid
        """
        # Validate data quality
        if not tick.bid_price or not tick.ask_price:
            logger.debug(f"Missing bid/ask for {tick.instrument_id}")
            return None
        
        bid = float(tick.bid_price)
        ask = float(tick.ask_price)
        
        # Validate prices are positive
        if bid <= 0 or ask <= 0:
            logger.debug(f"Invalid prices for {tick.instrument_id}: bid={bid}, ask={ask}")
            return None
        
        # Validate bid < ask
        if bid >= ask:
            logger.debug(f"Invalid spread for {tick.instrument_id}: bid={bid} >= ask={ask}")
            return None
        
        # Calculate spread
        spread = ask - bid
        mid_price = (bid + ask) / 2.0
        spread_pct = (spread / mid_price * 100.0) if mid_price > 0 else 0.0
        
        # Check spread thresholds
        if spread < MIN_BID_ASK_SPREAD:
            logger.debug(f"Spread too narrow for {tick.instrument_id}: {spread}")
            return None
        
        if spread_pct > MAX_SPREAD_PERCENT:
            logger.debug(f"Spread too wide for {tick.instrument_id}: {spread_pct:.2f}%")
            return None
        
        # Extract timestamp from tick (nanoseconds since epoch)
        try:
            if hasattr(tick, 'ts_event'):
                # Convert nanoseconds to datetime
                timestamp = datetime.fromtimestamp(tick.ts_event / 1e9, tz=timezone.utc)
            elif hasattr(tick, 'timestamp'):
                timestamp = tick.timestamp
            else:
                timestamp = datetime.now(timezone.utc)
        except (AttributeError, ValueError, OSError):
            timestamp = datetime.now(timezone.utc)
        
        # Check for stale data
        age_seconds = (datetime.now(timezone.utc) - timestamp).total_seconds()
        if age_seconds > self._max_data_age:
            instrument_str = str(tick.instrument_id)
            if instrument_str in self._data_quality_stats:
                self._data_quality_stats[instrument_str]["stale_ticks"] += 1
            logger.warning(
                f"Stale data for {tick.instrument_id}: {age_seconds:.2f}s old "
                f"(max: {self._max_data_age}s)"
            )
            return None
        
        # Convert to market data format
        return {
            "symbol": str(tick.instrument_id),
            "bid": bid,
            "ask": ask,
            "bid_size": int(tick.bid_size) if tick.bid_size else 0,
            "ask_size": int(tick.ask_size) if tick.ask_size else 0,
            "last": mid_price,  # Use mid price as last
            "last_size": 0,
            "volume": 0,
            "high": 0.0,
            "low": 0.0,
            "close": 0.0,
            "open": 0.0,
            "timestamp": timestamp,
            "spread": spread,
            "spread_pct": spread_pct,
        }
    
    def _convert_trade_tick(self, tick: TradeTick) -> Optional[Dict]:
        """
        Convert nautilus_trader TradeTick to C++ MarketData format.
        Includes data quality validation.
        
        Returns:
            Dict compatible with PyMarketData, or None if data is invalid
        """
        # Validate data quality
        if not tick.price or tick.price <= 0:
            logger.debug(f"Invalid trade price for {tick.instrument_id}")
            return None
        
        # Extract timestamp
        try:
            if hasattr(tick, 'ts_event'):
                timestamp = datetime.fromtimestamp(tick.ts_event / 1e9, tz=timezone.utc)
            elif hasattr(tick, 'timestamp'):
                timestamp = tick.timestamp
            else:
                timestamp = datetime.now(timezone.utc)
        except (AttributeError, ValueError, OSError):
            timestamp = datetime.now(timezone.utc)
        
        # Check for stale data
        age_seconds = (datetime.now(timezone.utc) - timestamp).total_seconds()
        if age_seconds > self._max_data_age:
            logger.warning(f"Stale trade data for {tick.instrument_id}: {age_seconds:.2f}s old")
            return None
        
        return {
            "symbol": str(tick.instrument_id),
            "last": float(tick.price),
            "last_size": int(tick.size) if tick.size else 0,
            "volume": int(tick.size) if tick.size else 0,
            "bid": 0.0,  # Trade ticks don't have bid/ask
            "ask": 0.0,
            "bid_size": 0,
            "ask_size": 0,
            "high": 0.0,
            "low": 0.0,
            "close": 0.0,
            "open": 0.0,
            "timestamp": timestamp,
        }
    
    def get_latest_quote(self, instrument_id: str) -> Optional[Dict]:
        """
        Get latest quote tick for an instrument.
        
        Args:
            instrument_id: Instrument identifier
            
        Returns:
            Market data dict or None if not available
        """
        instrument_str = str(instrument_id)
        if instrument_str in self._quote_ticks:
            return self._convert_quote_tick(self._quote_ticks[instrument_str])
        return None
    
    def get_latest_trade(self, instrument_id: str) -> Optional[Dict]:
        """
        Get latest trade tick for an instrument.
        
        Args:
            instrument_id: Instrument identifier
            
        Returns:
            Market data dict or None if not available
        """
        instrument_str = str(instrument_id)
        if instrument_str in self._trade_ticks:
            return self._convert_trade_tick(self._trade_ticks[instrument_str])
        return None
    
    def get_data_quality_stats(self, instrument_id: str) -> Optional[Dict]:
        """
        Get data quality statistics for an instrument.
        
        Args:
            instrument_id: Instrument identifier
            
        Returns:
            Statistics dict or None if not available
        """
        instrument_str = str(instrument_id)
        return self._data_quality_stats.get(instrument_str)
    
    def get_all_data_quality_stats(self) -> Dict[str, Dict]:
        """Get data quality statistics for all instruments."""
        return self._data_quality_stats.copy()



