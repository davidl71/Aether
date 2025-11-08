"""
nautilus_bridge.py - Python bridge connecting nautilus_trader to C++ bindings
"""
import logging
from typing import Optional, Dict, List

logger = logging.getLogger(__name__)

# Import C++ bindings
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
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    logger.warning("C++ bindings not available - install with: cd python/bindings && pip install -e .")


class NautilusBridge:
    """
    Bridge between nautilus_trader and C++ calculation functions.
    """
    
    def __init__(self, strategy_config: Dict):
        """
        Initialize bridge.
        
        Args:
            strategy_config: Strategy configuration dictionary
        """
        self.strategy_config = strategy_config
        self.cpp_strategy: Optional[PyBoxSpreadStrategy] = None
        
        if BINDINGS_AVAILABLE:
            try:
                self.cpp_strategy = PyBoxSpreadStrategy(strategy_config)
                logger.info("C++ strategy initialized via bindings")
            except Exception as e:
                logger.error(f"Failed to initialize C++ strategy: {e}")
        else:
            logger.warning("C++ bindings not available - using Python fallback")
    
    def evaluate_box_spread(self, spread_data: Dict) -> Dict:
        """
        Evaluate a box spread using C++ calculations.
        
        Args:
            spread_data: Dictionary containing box spread data
            
        Returns:
            Dictionary with evaluation results (profit, ROI, etc.)
        """
        if not BINDINGS_AVAILABLE:
            return self._python_fallback_evaluation(spread_data)
        
        try:
            # Convert dict to PyBoxSpreadLeg
            spread = self._dict_to_box_spread_leg(spread_data)
            
            # Calculate using C++ functions
            profit = calculate_arbitrage_profit(spread)
            roi = calculate_roi(spread)
            
            # Check if profitable
            min_profit = self.strategy_config.get("min_arbitrage_profit", 0.10)
            min_roi = self.strategy_config.get("min_roi_percent", 0.5)
            
            is_profitable = profit >= min_profit and roi >= min_roi
            
            return {
                "profit": profit,
                "roi": roi,
                "is_profitable": is_profitable,
                "net_debit": spread.net_debit,
                "theoretical_value": spread.theoretical_value,
            }
        except Exception as e:
            logger.error(f"Error evaluating box spread: {e}")
            return {"error": str(e)}
    
    def _dict_to_box_spread_leg(self, data: Dict) -> PyBoxSpreadLeg:
        """Convert dictionary to PyBoxSpreadLeg."""
        # Extract contract data
        long_call = PyOptionContract(
            symbol=data["long_call"]["symbol"],
            expiry=data["long_call"]["expiry"],
            strike=data["long_call"]["strike"],
            option_type=OptionType.Call,
        )
        
        short_call = PyOptionContract(
            symbol=data["short_call"]["symbol"],
            expiry=data["short_call"]["expiry"],
            strike=data["short_call"]["strike"],
            option_type=OptionType.Call,
        )
        
        long_put = PyOptionContract(
            symbol=data["long_put"]["symbol"],
            expiry=data["long_put"]["expiry"],
            strike=data["long_put"]["strike"],
            option_type=OptionType.Put,
        )
        
        short_put = PyOptionContract(
            symbol=data["short_put"]["symbol"],
            expiry=data["short_put"]["expiry"],
            strike=data["short_put"]["strike"],
            option_type=OptionType.Put,
        )
        
        spread = PyBoxSpreadLeg(long_call, short_call, long_put, short_put)
        spread.net_debit = data.get("net_debit", 0.0)
        spread.theoretical_value = data.get("theoretical_value", 0.0)
        spread.arbitrage_profit = data.get("arbitrage_profit", 0.0)
        
        return spread
    
    def _python_fallback_evaluation(self, spread_data: Dict) -> Dict:
        """Python fallback when C++ bindings are not available."""
        net_debit = spread_data.get("net_debit", 0.0)
        theoretical_value = spread_data.get("theoretical_value", 0.0)
        profit = theoretical_value - net_debit
        roi = (profit / net_debit * 100.0) if net_debit > 0 else 0.0
        
        min_profit = self.strategy_config.get("min_arbitrage_profit", 0.10)
        min_roi = self.strategy_config.get("min_roi_percent", 0.5)
        
        return {
            "profit": profit,
            "roi": roi,
            "is_profitable": profit >= min_profit and roi >= min_roi,
            "net_debit": net_debit,
            "theoretical_value": theoretical_value,
            "note": "Using Python fallback (C++ bindings not available)",
        }



