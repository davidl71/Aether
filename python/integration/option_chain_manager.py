"""
option_chain_manager.py - Manages option chains with caching
Implements efficient option chain storage and retrieval.
"""
import logging
from typing import Dict, Optional, List, Set
from datetime import datetime, timezone
from collections import defaultdict

from nautilus_trader.core.nautilus_pyo3 import (
    InstrumentId,
    QuoteTick,
)

logger = logging.getLogger(__name__)


class OptionChainManager:
    """
    Manages option chains with efficient caching and lookup.
    Updates chains in real-time as market data arrives.
    """
    
    def __init__(self, max_chain_age_seconds: float = 60.0):
        """
        Initialize option chain manager.
        
        Args:
            max_chain_age_seconds: Maximum age for cached chains before refresh
        """
        self._max_chain_age = max_chain_age_seconds
        
        # Chain storage: symbol -> expiry -> strike -> option_data
        self._chains: Dict[str, Dict[str, Dict[float, Dict]]] = defaultdict(
            lambda: defaultdict(dict)
        )
        
        # Timestamps for cache invalidation
        self._chain_timestamps: Dict[str, datetime] = {}
        
        # Underlying prices
        self._underlying_prices: Dict[str, float] = {}
    
    def update_option(
        self,
        instrument_id: InstrumentId,
        tick: QuoteTick,
    ):
        """
        Update option chain with new market data (event-driven).
        
        Args:
            instrument_id: Option instrument ID
            tick: Quote tick with market data
        """
        # Parse instrument ID to extract symbol, expiry, strike, type
        symbol, expiry, strike, option_type = self._parse_instrument_id(instrument_id)
        
        if not all([symbol, expiry, strike, option_type]):
            logger.debug(f"Could not parse instrument ID: {instrument_id}")
            return
        
        # Update chain
        option_data = {
            "instrument_id": str(instrument_id),
            "symbol": symbol,
            "expiry": expiry,
            "strike": strike,
            "type": option_type,
            "bid": float(tick.bid_price) if tick.bid_price else 0.0,
            "ask": float(tick.ask_price) if tick.ask_price else 0.0,
            "bid_size": int(tick.bid_size) if tick.bid_size else 0,
            "ask_size": int(tick.ask_size) if tick.ask_size else 0,
            "mid_price": (
                (float(tick.bid_price) + float(tick.ask_price)) / 2.0
                if tick.bid_price and tick.ask_price
                else 0.0
            ),
            "spread": (
                float(tick.ask_price) - float(tick.bid_price)
                if tick.bid_price and tick.ask_price
                else 0.0
            ),
            "timestamp": datetime.now(timezone.utc),
        }
        
        # Store in chain
        self._chains[symbol][expiry][strike] = option_data
        
        # Update timestamp
        self._chain_timestamps[symbol] = datetime.now(timezone.utc)
        
        logger.debug(
            f"Updated option chain: {symbol} {expiry} {strike} {option_type} "
            f"@ {option_data['mid_price']:.2f}"
        )
    
    def update_underlying_price(self, symbol: str, price: float):
        """
        Update underlying price for a symbol.
        
        Args:
            symbol: Underlying symbol
            price: Current underlying price
        """
        self._underlying_prices[symbol] = price
    
    def get_option(
        self,
        symbol: str,
        expiry: str,
        strike: float,
        option_type: str,
    ) -> Optional[Dict]:
        """
        Get option data from chain.
        
        Args:
            symbol: Underlying symbol
            expiry: Expiry date (YYYYMMDD)
            strike: Strike price
            option_type: "C" for call, "P" for put
            
        Returns:
            Option data dict or None if not found
        """
        if symbol not in self._chains:
            return None
        
        if expiry not in self._chains[symbol]:
            return None
        
        if strike not in self._chains[symbol][expiry]:
            return None
        
        option_data = self._chains[symbol][expiry][strike]
        
        # Verify option type matches
        if option_data.get("type") != option_type:
            return None
        
        return option_data
    
    def get_expiries(self, symbol: str) -> List[str]:
        """
        Get all expiries for a symbol.
        
        Args:
            symbol: Underlying symbol
            
        Returns:
            List of expiry dates (YYYYMMDD)
        """
        if symbol not in self._chains:
            return []
        
        return sorted(self._chains[symbol].keys())
    
    def get_strikes(self, symbol: str, expiry: str) -> List[float]:
        """
        Get all strikes for a symbol and expiry.
        
        Args:
            symbol: Underlying symbol
            expiry: Expiry date (YYYYMMDD)
            
        Returns:
            List of strike prices
        """
        if symbol not in self._chains:
            return []
        
        if expiry not in self._chains[symbol]:
            return []
        
        return sorted(self._chains[symbol][expiry].keys())
    
    def find_box_spread_legs(
        self,
        symbol: str,
        expiry: str,
        strike_low: float,
        strike_high: float,
    ) -> Optional[Dict]:
        """
        Find all 4 legs for a box spread.
        
        Args:
            symbol: Underlying symbol
            expiry: Expiry date (YYYYMMDD)
            strike_low: Lower strike price
            strike_high: Higher strike price
            
        Returns:
            Dict with all 4 legs or None if not all found
        """
        # Get all 4 legs
        long_call = self.get_option(symbol, expiry, strike_low, "C")
        short_call = self.get_option(symbol, expiry, strike_high, "C")
        long_put = self.get_option(symbol, expiry, strike_high, "P")
        short_put = self.get_option(symbol, expiry, strike_low, "P")
        
        # Validate all found
        if not all([long_call, short_call, long_put, short_put]):
            return None
        
        # Validate all have valid market data
        for leg_name, leg_data in [
            ("long_call", long_call),
            ("short_call", short_call),
            ("long_put", long_put),
            ("short_put", short_put),
        ]:
            if leg_data["bid"] <= 0 or leg_data["ask"] <= 0:
                logger.debug(f"Invalid market data for {leg_name}")
                return None
        
        return {
            "long_call": long_call,
            "short_call": short_call,
            "long_put": long_put,
            "short_put": short_put,
            "symbol": symbol,
            "expiry": expiry,
            "strike_low": strike_low,
            "strike_high": strike_high,
        }
    
    def is_chain_stale(self, symbol: str) -> bool:
        """
        Check if option chain is stale.
        
        Args:
            symbol: Underlying symbol
            
        Returns:
            True if stale, False otherwise
        """
        if symbol not in self._chain_timestamps:
            return True
        
        age = (datetime.now(timezone.utc) - self._chain_timestamps[symbol]).total_seconds()
        return age > self._max_chain_age
    
    def clear_chain(self, symbol: str):
        """
        Clear option chain for a symbol.
        
        Args:
            symbol: Underlying symbol
        """
        if symbol in self._chains:
            del self._chains[symbol]
        if symbol in self._chain_timestamps:
            del self._chain_timestamps[symbol]
        if symbol in self._underlying_prices:
            del self._underlying_prices[symbol]
        
        logger.info(f"Cleared option chain for {symbol}")
    
    def _parse_instrument_id(self, instrument_id: InstrumentId) -> tuple:
        """
        Parse instrument ID to extract components.
        
        Args:
            instrument_id: Instrument ID
            
        Returns:
            Tuple of (symbol, expiry, strike, option_type) or (None, None, None, None)
        """
        # This is a simplified parser - in production would use proper instrument parsing
        instrument_str = str(instrument_id)
        
        # Try to parse common formats
        # Format: SYMBOL YYMMDD C/P STRIKE
        # Example: SPY 240412 C 500
        
        parts = instrument_str.split()
        if len(parts) >= 4:
            symbol = parts[0]
            expiry = parts[1]
            option_type = parts[2]
            try:
                strike = float(parts[3])
                return (symbol, expiry, strike, option_type)
            except ValueError:
                pass
        
        # Try alternative format: SYMBOLYYMMDDC/PSTRIKE
        # Example: SPY240412C00500000
        if len(instrument_str) > 6:
            # Find option type (C or P)
            if 'C' in instrument_str:
                option_type = 'C'
                split_char = 'C'
            elif 'P' in instrument_str:
                option_type = 'P'
                split_char = 'P'
            else:
                return (None, None, None, None)
            
            # Split on option type
            parts = instrument_str.split(split_char)
            if len(parts) == 2:
                symbol_expiry = parts[0]
                strike_str = parts[1]
                
                # Extract symbol and expiry (assume 6-digit expiry)
                if len(symbol_expiry) >= 6:
                    symbol = symbol_expiry[:-6]
                    expiry = symbol_expiry[-6:]
                    
                    # Parse strike (remove leading zeros, convert to float)
                    try:
                        strike = float(strike_str) / 1000.0  # Adjust for format
                        return (symbol, expiry, strike, option_type)
                    except ValueError:
                        pass
        
        return (None, None, None, None)
    
    def get_chain_stats(self, symbol: str) -> Dict:
        """
        Get statistics for an option chain.
        
        Args:
            symbol: Underlying symbol
            
        Returns:
            Statistics dictionary
        """
        if symbol not in self._chains:
            return {"expiries": 0, "total_options": 0, "stale": True}
        
        expiries = len(self._chains[symbol])
        total_options = sum(
            len(strikes) for strikes in self._chains[symbol].values()
        )
        
        return {
            "expiries": expiries,
            "total_options": total_options,
            "stale": self.is_chain_stale(symbol),
            "last_update": self._chain_timestamps.get(symbol),
            "underlying_price": self._underlying_prices.get(symbol),
        }

