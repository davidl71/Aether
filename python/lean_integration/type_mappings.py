"""
type_mappings.py - Type mapping utilities for LEAN and C++ conversions

This module provides mapping functions between LEAN and C++ enum types.
"""

from AlgorithmImports import *

# Import C++ bindings
try:
    from ..bindings.box_spread_bindings import OptionType
except ImportError:
    OptionType = None


class TypeMappings:
    """Type mapping utilities."""

    @staticmethod
    def lean_option_right_to_cpp(lean_right) -> int:
        """
        Convert LEAN OptionRight to C++ OptionType.

        Args:
            lean_right: LEAN OptionRight enum (Call or Put)

        Returns:
            C++ OptionType (0=Call, 1=Put)
        """
        if lean_right == OptionRight.Call:
            return 0  # Call
        elif lean_right == OptionRight.Put:
            return 1  # Put
        else:
            raise ValueError(f"Unknown OptionRight: {lean_right}")

    @staticmethod
    def cpp_option_type_to_lean(cpp_type: int) -> OptionRight:
        """
        Convert C++ OptionType to LEAN OptionRight.

        Args:
            cpp_type: C++ OptionType (0=Call, 1=Put)

        Returns:
            LEAN OptionRight enum
        """
        if cpp_type == 0:
            return OptionRight.Call
        elif cpp_type == 1:
            return OptionRight.Put
        else:
            raise ValueError(f"Unknown C++ OptionType: {cpp_type}")

    @staticmethod
    def lean_order_status_to_cpp(lean_status) -> int:
        """
        Convert LEAN OrderStatus to C++ OrderStatus.

        Args:
            lean_status: LEAN OrderStatus enum

        Returns:
            C++ OrderStatus (0=Pending, 1=Submitted, 2=Filled, etc.)
        """
        status_map = {
            OrderStatus.New: 0,  # Pending
            OrderStatus.Submitted: 1,  # Submitted
            OrderStatus.Filled: 2,  # Filled
            OrderStatus.PartiallyFilled: 3,  # PartiallyFilled
            OrderStatus.Canceled: 4,  # Cancelled
            OrderStatus.Invalid: 5,  # Rejected
            OrderStatus.CancelPending: 0,  # Pending
        }

        return status_map.get(lean_status, 6)  # Default to Error

    @staticmethod
    def cpp_order_status_to_lean(cpp_status: int) -> OrderStatus:
        """
        Convert C++ OrderStatus to LEAN OrderStatus.

        Args:
            cpp_status: C++ OrderStatus (0=Pending, 1=Submitted, etc.)

        Returns:
            LEAN OrderStatus enum
        """
        status_map = {
            0: OrderStatus.New,  # Pending
            1: OrderStatus.Submitted,  # Submitted
            2: OrderStatus.Filled,  # Filled
            3: OrderStatus.PartiallyFilled,  # PartiallyFilled
            4: OrderStatus.Canceled,  # Cancelled
            5: OrderStatus.Invalid,  # Rejected
            6: OrderStatus.Invalid,  # Error
        }

        return status_map.get(cpp_status, OrderStatus.Invalid)
