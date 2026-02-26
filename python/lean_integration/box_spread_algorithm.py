"""
box_spread_algorithm.py - LEAN Algorithm for Box Spread Trading

This module implements the main LEAN algorithm that integrates C++ box spread
calculations with LEAN's multi-broker execution capabilities.
"""

from AlgorithmImports import *
from typing import List, Dict, Optional
import logging

# Import data conversion utilities
from .data_converter import DataConverter
from .strategy_config import StrategyConfig

# Import C++ bindings
try:
    from ..bindings.box_spread_bindings import (
        PyBoxSpreadStrategy,
        PyBoxSpreadLeg,
        PyOptionContract,
        calculate_arbitrage_profit,
        calculate_roi
    )
except ImportError:
    # Fallback for development/testing
    PyBoxSpreadStrategy = None
    PyBoxSpreadLeg = None
    PyOptionContract = None
    calculate_arbitrage_profit = None
    calculate_roi = None

logger = logging.getLogger(__name__)


class BoxSpreadAlgorithm(QCAlgorithm):
    """
    LEAN algorithm for box spread arbitrage trading.

    Integrates C++ box spread calculations with LEAN's execution engine
    for multi-broker support (IBKR, Alpaca, etc.).
    """

    def Initialize(self):
        """Initialize algorithm state and subscriptions."""
        # Set algorithm parameters
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)
        self.SetBenchmark("SPY")

        # Load strategy configuration
        config_path = self.GetParameter("config_path", "config/lean_strategy_config.json")
        try:
            self.config = StrategyConfig(config_path)
        except Exception as e:
            self.Log(f"Error loading config: {e}. Using defaults.")
            self.config = StrategyConfig.default()

        # Get strategy parameters
        self.symbols = self.config.get_symbols()
        self.min_roi = self.config.get_min_roi()
        self.max_position_size = self.config.get_max_position_size()
        self.max_risk = self.config.get_max_risk()
        self.min_days_to_expiry = self.config.get_min_days_to_expiry()
        self.max_days_to_expiry = self.config.get_max_days_to_expiry()

        # Initialize C++ strategy (if available)
        if PyBoxSpreadStrategy is not None:
            try:
                self.cpp_strategy = PyBoxSpreadStrategy(
                    min_roi=self.min_roi,
                    max_position_size=self.max_position_size
                )
                self.Log("C++ strategy initialized successfully")
            except Exception as e:
                self.Log(f"Error initializing C++ strategy: {e}")
                self.cpp_strategy = None
        else:
            self.Log("C++ bindings not available - using Python-only mode")
            self.cpp_strategy = None

        # Initialize data converter
        self.data_converter = DataConverter()

        # Subscribe to options for each symbol
        for symbol in self.symbols:
            try:
                option = self.AddOption(symbol)
                option.SetFilter(self.OptionFilter)
                self.Log(f"Subscribed to options for {symbol}")
            except Exception as e:
                self.Log(f"Error subscribing to {symbol}: {e}")

        # Track positions and orders
        self.active_positions: Dict[int, Dict] = {}
        self.pending_orders: Dict[int, Dict] = {}

        # Statistics
        self.opportunities_found = 0
        self.trades_executed = 0
        self.total_profit = 0.0
        self.total_opportunities_evaluated = 0

        self.Log("Box Spread Algorithm initialized")

    def OptionFilter(self, universe):
        """
        Filter options by expiration and strike.

        Args:
            universe: LEAN OptionFilterUniverse

        Returns:
            Filtered universe
        """
        # Filter by expiration (days to expiry)
        filtered = universe.Strikes(-20, +20)  # ±20 strikes around ATM

        # Filter by expiration date
        if self.min_days_to_expiry > 0 or self.max_days_to_expiry > 0:
            filtered = filtered.Expiration(self.min_days_to_expiry, self.max_days_to_expiry)
        else:
            filtered = filtered.Expiration(0, 60)  # Default: 0-60 days

        return filtered

    def OnData(self, slice):
        """
        Process market data and evaluate opportunities.

        Args:
            slice: LEAN Slice object with market data
        """
        # Process option chains for each symbol
        for symbol in self.symbols:
            option_chain = slice.OptionChains.get(symbol, None)
            if option_chain is None:
                continue

            # Skip if no contracts
            if len(option_chain) == 0:
                continue

            # Evaluate opportunities for this symbol
            self.evaluate_symbol_opportunities(symbol, option_chain)

    def evaluate_symbol_opportunities(self, symbol: str, option_chain):
        """
        Evaluate box spread opportunities for a symbol.

        Args:
            symbol: Underlying symbol
            option_chain: LEAN OptionChain
        """
        try:
            # Convert LEAN option chain to C++ format
            cpp_options = self.data_converter.lean_to_cpp_option_chain(
                option_chain, symbol
            )

            if len(cpp_options) == 0:
                return

            # Use C++ strategy to find opportunities (if available)
            if self.cpp_strategy is not None:
                opportunities = self.find_box_spreads_cpp(symbol, cpp_options)
            else:
                # Fallback to Python-only mode (simplified)
                opportunities = self.find_box_spreads_python(symbol, option_chain)

            self.total_opportunities_evaluated += len(opportunities)

            # Evaluate and execute best opportunities
            for opp in opportunities:
                if self.should_execute(opp):
                    if self.execute_box_spread(opp, option_chain):
                        break  # One trade per iteration
        except Exception as e:
            self.Log(f"Error evaluating opportunities for {symbol}: {e}")
            self.Error(f"Error in evaluate_symbol_opportunities: {e}")

    def find_box_spreads_cpp(self, symbol: str, cpp_options: List[Dict]) -> List[Dict]:
        """
        Find box spread opportunities using C++ strategy.

        Args:
            symbol: Underlying symbol
            cpp_options: List of C++ option dictionaries with 'contract' and 'market_data' keys

        Returns:
            List of opportunity dictionaries
        """
        if self.cpp_strategy is None:
            return []

        try:
            # Group options by expiry date
            by_expiry: Dict[str, List[Dict]] = {}
            for opt in cpp_options:
                contract = opt.get("contract")
                if contract is None:
                    continue
                expiry = contract.expiry
                by_expiry.setdefault(expiry, []).append(opt)

            opportunities = []

            for expiry, expiry_opts in by_expiry.items():
                # Separate calls and puts, keyed by strike
                calls: Dict[float, Dict] = {}
                puts: Dict[float, Dict] = {}
                for opt in expiry_opts:
                    contract = opt["contract"]
                    md = opt.get("market_data")
                    if md is None:
                        continue
                    strike = contract.strike
                    if contract.type == 0:  # Call
                        calls[strike] = opt
                    else:
                        puts[strike] = opt

                # Get sorted strikes that have both a call and a put
                common_strikes = sorted(set(calls.keys()) & set(puts.keys()))
                if len(common_strikes) < 2:
                    continue

                # Evaluate every pair of strikes as a box spread
                for i, low_strike in enumerate(common_strikes):
                    for high_strike in common_strikes[i + 1:]:
                        long_call = calls[low_strike]
                        short_call = calls[high_strike]
                        long_put = puts[high_strike]
                        short_put = puts[low_strike]

                        lc_md = long_call["market_data"]
                        sc_md = short_call["market_data"]
                        lp_md = long_put["market_data"]
                        sp_md = short_put["market_data"]

                        # Require valid bid/ask on all legs
                        if any(
                            getattr(m, "bid", 0) <= 0 or getattr(m, "ask", 0) <= 0
                            for m in [lc_md, sc_md, lp_md, sp_md]
                        ):
                            continue

                        lc_mid = (lc_md.bid + lc_md.ask) / 2.0
                        sc_mid = (sc_md.bid + sc_md.ask) / 2.0
                        lp_mid = (lp_md.bid + lp_md.ask) / 2.0
                        sp_mid = (sp_md.bid + sp_md.ask) / 2.0

                        strike_width = high_strike - low_strike
                        net_debit = (lc_mid - sc_mid) + (lp_mid - sp_mid)
                        theoretical = strike_width
                        profit = theoretical - net_debit

                        if net_debit <= 0 or profit <= 0:
                            continue

                        roi = (profit / net_debit) * 100.0

                        if roi < self.min_roi:
                            continue

                        opportunities.append({
                            "symbol": symbol,
                            "expiry": expiry,
                            "low_strike": low_strike,
                            "high_strike": high_strike,
                            "net_debit": net_debit,
                            "theoretical_value": theoretical,
                            "profit": profit,
                            "roi_percent": roi,
                            "long_call": long_call["contract"],
                            "short_call": short_call["contract"],
                            "long_put": long_put["contract"],
                            "short_put": short_put["contract"],
                        })

            # Sort by ROI descending
            opportunities.sort(key=lambda x: x.get("roi_percent", 0), reverse=True)
            return opportunities

        except Exception as e:
            self.Log(f"Error in C++ box spread detection: {e}")
            return []

    def find_box_spreads_python(self, symbol: str, option_chain) -> List[Dict]:
        """
        Find box spread opportunities using Python-only logic (fallback).

        Args:
            symbol: Underlying symbol
            option_chain: LEAN OptionChain

        Returns:
            List of opportunity dictionaries
        """
        # Simplified Python-only implementation
        # Full implementation would require duplicating C++ logic
        # This is a placeholder for when C++ bindings are unavailable
        return []

    def should_execute(self, opportunity: Dict) -> bool:
        """
        Determine if opportunity should be executed.

        Args:
            opportunity: Opportunity dictionary

        Returns:
            True if should execute, False otherwise
        """
        # Check profitability
        expected_profit = opportunity.get("expected_profit", 0.0)
        if expected_profit < self.min_roi:
            return False

        # Check position limits
        if len(self.active_positions) >= self.max_position_size:
            return False

        # Check risk limits (if C++ strategy available)
        if self.cpp_strategy is not None:
            try:
                risk_score = self.cpp_strategy.calculate_risk(opportunity.get("spread"))
                if risk_score > self.max_risk:
                    return False
            except Exception:
                pass  # Continue if risk calculation fails

        # Check data quality
        spread = opportunity.get("spread")
        if spread:
            max_spread = spread.get("max_bid_ask_spread", float('inf'))
            if max_spread > self.config.get_max_bid_ask_spread():
                return False

        return True

    def execute_box_spread(self, opportunity: Dict, option_chain) -> bool:
        """
        Execute box spread order via LEAN.

        Args:
            opportunity: Opportunity dictionary
            option_chain: LEAN OptionChain for finding contracts

        Returns:
            True if order placed successfully, False otherwise
        """
        try:
            spread = opportunity.get("spread")
            if not spread:
                return False

            # Convert C++ spread to LEAN combo order format
            if isinstance(spread, PyBoxSpreadLeg):
                combo_data = self.data_converter.cpp_box_spread_to_lean_combo(
                    spread, option_chain
                )
            else:
                # Handle dictionary format
                combo_data = self.convert_dict_spread_to_combo(spread, option_chain)

            if combo_data is None:
                self.Log("Could not convert spread to LEAN combo order")
                return False

            # Place combo order
            symbols = combo_data["symbols"]
            quantities = combo_data["quantities"]

            # Use market order for now (can be made configurable)
            combo_order = self.ComboMarketOrder(symbols, quantities)

            # Track order
            order_id = combo_order.Id
            self.pending_orders[order_id] = {
                "opportunity": opportunity,
                "spread": spread,
                "timestamp": self.Time,
                "symbols": symbols,
                "quantities": quantities
            }

            self.Log(f"Placed box spread combo order: {order_id}")
            self.trades_executed += 1
            self.opportunities_found += 1

            return True
        except Exception as e:
            self.Log(f"Error executing box spread: {e}")
            self.Error(f"Error in execute_box_spread: {e}")
            return False

    def convert_dict_spread_to_combo(self, spread: Dict, option_chain) -> Optional[Dict]:
        """
        Convert dictionary-format spread to LEAN combo order.

        Args:
            spread: Spread dictionary
            option_chain: LEAN OptionChain

        Returns:
            Combo order dictionary or None
        """
        # Extract contracts from spread dictionary
        long_call = spread.get("long_call")
        short_call = spread.get("short_call")
        long_put = spread.get("long_put")
        short_put = spread.get("short_put")

        if not all([long_call, short_call, long_put, short_put]):
            return None

        # Convert to LEAN symbols
        symbols = []
        for contract in [long_call, short_call, long_put, short_put]:
            if isinstance(contract, PyOptionContract):
                symbol = self.data_converter.cpp_to_lean_symbol(contract, option_chain)
            else:
                # Dictionary format - would need to convert
                symbol = None

            if symbol is None:
                return None
            symbols.append(symbol)

        return {
            "symbols": symbols,
            "quantities": [1, -1, 1, -1]  # long, short, long, short
        }

    def OnOrderEvent(self, orderEvent):
        """
        Handle order events (fill, partial fill, cancellation).

        Args:
            orderEvent: LEAN OrderEvent
        """
        order = orderEvent.Order
        order_id = order.Id

        if order_id in self.pending_orders:
            info = self.pending_orders[order_id]

            if orderEvent.Status == OrderStatus.Filled:
                # Order filled - track position
                self.active_positions[order_id] = {
                    "spread": info["spread"],
                    "entry_time": info["timestamp"],
                    "entry_price": orderEvent.FillPrice,
                    "symbols": info["symbols"],
                    "quantities": info["quantities"]
                }
                del self.pending_orders[order_id]

                self.Log(f"Box spread filled: {order_id} at {orderEvent.FillPrice}")

                # Update statistics
                if "expected_profit" in info.get("opportunity", {}):
                    self.total_profit += info["opportunity"]["expected_profit"]

            elif orderEvent.Status == OrderStatus.Canceled:
                # Order cancelled
                del self.pending_orders[order_id]
                self.Log(f"Box spread cancelled: {order_id}")

            elif orderEvent.Status == OrderStatus.Invalid:
                # Order rejected
                del self.pending_orders[order_id]
                self.Log(f"Box spread rejected: {order_id} - {orderEvent.Message}")
                self.Error(f"Order rejected: {orderEvent.Message}")

    def OnSecuritiesChanged(self, changes):
        """
        Handle security additions/removals.

        Args:
            changes: LEAN SecurityChanges
        """
        for security in changes.AddedSecurities:
            self.Log(f"Added security: {security.Symbol}")

        for security in changes.RemovedSecurities:
            self.Log(f"Removed security: {security.Symbol}")

    def OnBrokerageMessage(self, message):
        """
        Handle brokerage messages and errors.

        Args:
            message: LEAN BrokerageMessageEvent
        """
        if message.Type == BrokerageMessageType.Error:
            self.Log(f"Brokerage error: {message.Message}")
            self.Error(f"Brokerage error: {message.Message}")
        elif message.Type == BrokerageMessageType.Warning:
            self.Log(f"Brokerage warning: {message.Message}")

    def OnEndOfAlgorithm(self):
        """Called when algorithm ends - log final statistics."""
        self.Log("=" * 50)
        self.Log("Algorithm Statistics:")
        self.Log(f"  Opportunities Found: {self.opportunities_found}")
        self.Log(f"  Opportunities Evaluated: {self.total_opportunities_evaluated}")
        self.Log(f"  Trades Executed: {self.trades_executed}")
        self.Log(f"  Total Profit: ${self.total_profit:.2f}")
        self.Log(f"  Active Positions: {len(self.active_positions)}")
        self.Log(f"  Pending Orders: {len(self.pending_orders)}")
        self.Log("=" * 50)
