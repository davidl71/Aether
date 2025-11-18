"""
data_converter.py - Convert between LEAN and C++ data formats

This module provides conversion functions to transform LEAN data structures
to C++ formats (for calculations) and vice versa (for order execution).
"""

from typing import List, Optional, Dict
from datetime import datetime
from AlgorithmImports import *

# Import C++ bindings
try:
    from ..bindings.box_spread_bindings import (
        PyOptionContract,
        PyBoxSpreadLeg,
        PyMarketData,
        OptionType
    )
except ImportError:
    # Fallback for development/testing
    PyOptionContract = None
    PyBoxSpreadLeg = None
    PyMarketData = None
    OptionType = None


class DataConverter:
    """Convert between LEAN and C++ data formats."""

    @staticmethod
    def lean_to_cpp_option_chain(lean_chain, symbol: str) -> List[Dict]:
        """
        Convert LEAN OptionChain to list of C++-compatible dictionaries.

        Args:
            lean_chain: LEAN OptionChain object
            symbol: Underlying symbol (e.g., "SPY")

        Returns:
            List of dictionaries with C++-compatible option data
        """
        cpp_options = []

        for contract in lean_chain:
            try:
                cpp_option = {
                    "contract": DataConverter.lean_to_cpp_contract(contract),
                    "market_data": DataConverter.lean_to_cpp_market_data(contract)
                }
                cpp_options.append(cpp_option)
            except Exception as e:
                # Skip invalid contracts
                continue

        return cpp_options

    @staticmethod
    def lean_to_cpp_contract(lean_contract) -> PyOptionContract:
        """
        Convert LEAN OptionContract to C++ PyOptionContract.

        Args:
            lean_contract: LEAN OptionContract object

        Returns:
            PyOptionContract instance
        """
        if PyOptionContract is None:
            raise ImportError("C++ bindings not available")

        # Extract underlying symbol
        underlying_symbol = lean_contract.Symbol.Underlying.Value

        # Format expiry date (YYYYMMDD)
        expiry_str = DataConverter.format_expiry(lean_contract.Expiry)

        # Extract strike
        strike = float(lean_contract.Strike)

        # Map LEAN OptionRight to C++ OptionType
        option_type = 0 if lean_contract.Right == OptionRight.Call else 1  # Call=0, Put=1

        # Extract exchange (default to "SMART" if not available)
        exchange = getattr(lean_contract.Symbol, "ID", {}).get("Market", "SMART")

        # Create C++ contract
        return PyOptionContract(
            symbol=underlying_symbol,
            expiry=expiry_str,
            strike=strike,
            option_type=option_type,
            exchange=exchange,
            local_symbol=""  # LEAN doesn't expose local_symbol directly
        )

    @staticmethod
    def lean_to_cpp_market_data(lean_contract) -> PyMarketData:
        """
        Convert LEAN market data to C++ PyMarketData.

        Args:
            lean_contract: LEAN OptionContract with market data

        Returns:
            PyMarketData instance
        """
        if PyMarketData is None:
            raise ImportError("C++ bindings not available")

        # Extract market data (handle missing values)
        bid = float(lean_contract.BidPrice) if lean_contract.BidPrice > 0 else 0.0
        ask = float(lean_contract.AskPrice) if lean_contract.AskPrice > 0 else 0.0
        last = float(lean_contract.LastPrice) if lean_contract.LastPrice > 0 else (bid + ask) / 2.0 if (bid > 0 and ask > 0) else 0.0
        volume = int(lean_contract.Volume) if hasattr(lean_contract, 'Volume') else 0
        open_interest = int(lean_contract.OpenInterest) if hasattr(lean_contract, 'OpenInterest') else 0

        # Extract Greeks if available
        delta = float(lean_contract.Greeks.Delta) if hasattr(lean_contract, 'Greeks') and lean_contract.Greeks.Delta is not None else 0.0
        gamma = float(lean_contract.Greeks.Gamma) if hasattr(lean_contract, 'Greeks') and lean_contract.Greeks.Gamma is not None else 0.0
        theta = float(lean_contract.Greeks.Theta) if hasattr(lean_contract, 'Greeks') and lean_contract.Greeks.Theta is not None else 0.0
        vega = float(lean_contract.Greeks.Vega) if hasattr(lean_contract, 'Greeks') and lean_contract.Greeks.Vega is not None else 0.0

        # Extract implied volatility if available
        implied_vol = float(lean_contract.ImpliedVolatility) if hasattr(lean_contract, 'ImpliedVolatility') and lean_contract.ImpliedVolatility is not None else 0.0

        # Create C++ market data
        return PyMarketData(
            bid=bid,
            ask=ask,
            last=last,
            volume=volume,
            open_interest=open_interest,
            delta=delta,
            gamma=gamma,
            theta=theta,
            vega=vega,
            implied_volatility=implied_vol
        )

    @staticmethod
    def cpp_to_lean_symbol(cpp_contract: PyOptionContract, option_chain) -> Optional[Symbol]:
        """
        Convert C++ PyOptionContract to LEAN Symbol.

        Args:
            cpp_contract: PyOptionContract instance
            option_chain: LEAN OptionChain to search for matching contract

        Returns:
            LEAN Symbol if found, None otherwise
        """
        # Format expiry for comparison
        expiry_str = cpp_contract.expiry

        # Search for matching contract in option chain
        for contract in option_chain:
            contract_expiry = DataConverter.format_expiry(contract.Expiry)

            # Check if contract matches
            if (contract.Symbol.Underlying.Value == cpp_contract.symbol and
                abs(contract.Strike - cpp_contract.strike) < 0.01 and  # Float comparison tolerance
                contract.Right == (OptionRight.Call if cpp_contract.type == 0 else OptionRight.Put) and
                contract_expiry == expiry_str):
                return contract.Symbol

        return None

    @staticmethod
    def cpp_box_spread_to_lean_combo(cpp_spread: PyBoxSpreadLeg, option_chain) -> Optional[Dict]:
        """
        Convert C++ BoxSpreadLeg to LEAN combo order format.

        Args:
            cpp_spread: PyBoxSpreadLeg instance
            option_chain: LEAN OptionChain to find matching contracts

        Returns:
            Dictionary with 'symbols' and 'quantities' for LEAN ComboMarketOrder,
            or None if contracts not found
        """
        # Convert each leg to LEAN Symbol
        long_call_symbol = DataConverter.cpp_to_lean_symbol(
            cpp_spread.long_call, option_chain
        )
        short_call_symbol = DataConverter.cpp_to_lean_symbol(
            cpp_spread.short_call, option_chain
        )
        long_put_symbol = DataConverter.cpp_to_lean_symbol(
            cpp_spread.long_put, option_chain
        )
        short_put_symbol = DataConverter.cpp_to_lean_symbol(
            cpp_spread.short_put, option_chain
        )

        # Check if all contracts found
        if not all([long_call_symbol, short_call_symbol, long_put_symbol, short_put_symbol]):
            return None

        # Return combo order format
        return {
            "symbols": [long_call_symbol, short_call_symbol, long_put_symbol, short_put_symbol],
            "quantities": [1, -1, 1, -1]  # long, short, long, short
        }

    @staticmethod
    def format_expiry(expiry_date) -> str:
        """
        Format expiry date to YYYYMMDD string.

        Args:
            expiry_date: LEAN DateTime or Python datetime object

        Returns:
            YYYYMMDD formatted string
        """
        if hasattr(expiry_date, 'year'):
            # Python datetime or LEAN DateTime
            return f"{expiry_date.year:04d}{expiry_date.month:02d}{expiry_date.day:02d}"
        elif isinstance(expiry_date, str):
            # Already formatted
            return expiry_date
        else:
            raise ValueError(f"Unsupported expiry date format: {type(expiry_date)}")

    @staticmethod
    def parse_expiry(expiry_str: str) -> datetime:
        """
        Parse YYYYMMDD string to datetime.

        Args:
            expiry_str: YYYYMMDD formatted string

        Returns:
            datetime object
        """
        if len(expiry_str) != 8:
            raise ValueError(f"Invalid expiry format: {expiry_str}")

        year = int(expiry_str[0:4])
        month = int(expiry_str[4:6])
        day = int(expiry_str[6:8])

        return datetime(year, month, day)

    @staticmethod
    def validate_lean_contract(lean_contract) -> bool:
        """
        Validate LEAN contract has required data.

        Args:
            lean_contract: LEAN OptionContract

        Returns:
            True if valid, False otherwise
        """
        try:
            # Check required fields
            if not hasattr(lean_contract, 'Symbol'):
                return False

            if not hasattr(lean_contract, 'Strike'):
                return False

            if not hasattr(lean_contract, 'Expiry'):
                return False

            # Check market data availability
            if lean_contract.BidPrice <= 0 and lean_contract.AskPrice <= 0:
                return False

            return True
        except Exception:
            return False

    @staticmethod
    def validate_cpp_contract(cpp_contract: PyOptionContract) -> bool:
        """
        Validate C++ contract.

        Args:
            cpp_contract: PyOptionContract instance

        Returns:
            True if valid, False otherwise
        """
        if cpp_contract is None:
            return False

        return cpp_contract.is_valid()
