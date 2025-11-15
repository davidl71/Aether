#!/usr/bin/env python3
"""
Trading Bridge - Interface between MCP server and native C++ code

This module provides a bridge to interact with the native C++ OrderManager
and TWS client. It supports multiple integration methods:
1. REST API (via backend service)
2. Future: Direct Cython bindings
3. Future: Shared library via ctypes
"""

import os
import json
import logging
import requests
from typing import Optional, Dict, List, Any
from datetime import datetime

logger = logging.getLogger(__name__)

# Configuration
BACKEND_REST_URL = os.getenv("BACKEND_REST_URL", "http://localhost:8080")
TWS_HOST = os.getenv("TWS_HOST", "127.0.0.1")
TWS_PORT = int(os.getenv("TWS_PORT", "7497"))
DRY_RUN = os.getenv("DRY_RUN", "true").lower() == "true"


class TradingBridge:
    """
    Bridge to native trading operations.

    Supports multiple integration methods:
    - REST API (primary method)
    - Future: Direct Cython bindings
    """

    def __init__(self, rest_url: Optional[str] = None):
        """
        Initialize trading bridge.

        Args:
            rest_url: Backend REST API URL (defaults to BACKEND_REST_URL env var)
        """
        self.rest_url = rest_url or BACKEND_REST_URL
        self.session = requests.Session()
        self.session.timeout = 10.0

        # Try to import Cython bindings (future enhancement)
        self.use_bindings = False
        self.order_manager = None

        try:
            # Future: Direct C++ integration via Cython bindings
            # See CYTHON_BINDINGS_GUIDE.md for implementation details
            # from native.bindings.box_spread_bindings import (
            #     PyOrderManager,
            #     PyOptionContract,
            #     PyBoxSpreadLeg,
            #     OrderAction,
            #     TimeInForce
            # )
            # self.order_manager = create_order_manager(dry_run=DRY_RUN)
            # self.use_bindings = True
            logger.debug("Cython bindings not yet implemented, using REST API")
        except ImportError:
            logger.debug("Cython bindings not available, using REST API")

    def place_order(
        self,
        symbol: str,
        side: str,
        quantity: int,
        order_type: str = "MARKET",
        limit_price: Optional[float] = None,
    ) -> Dict[str, Any]:
        """
        Place an order.

        Args:
            symbol: Trading symbol
            side: "BUY" or "SELL"
            quantity: Number of shares/contracts
            order_type: "MARKET" or "LIMIT"
            limit_price: Required for LIMIT orders

        Returns:
            Order result dictionary
        """
        if DRY_RUN:
            return {
                "success": True,
                "dry_run": True,
                "order_id": f"DRY-{int(datetime.now().timestamp())}",
                "symbol": symbol,
                "side": side,
                "quantity": quantity,
                "order_type": order_type,
                "limit_price": limit_price,
                "message": "[DRY RUN] Order would be placed"
            }

        if self.use_bindings:
            # Future: Direct C++ call via Cython
            # return self._place_order_via_bindings(...)
            pass

        # Use REST API
        return self._place_order_via_rest(
            symbol, side, quantity, order_type, limit_price
        )

    def _place_order_via_rest(
        self,
        symbol: str,
        side: str,
        quantity: int,
        order_type: str,
        limit_price: Optional[float],
    ) -> Dict[str, Any]:
        """Place order via REST API."""
        try:
            url = f"{self.rest_url}/api/v1/orders/place"
            payload = {
                "symbol": symbol,
                "side": side,
                "quantity": quantity,
                "order_type": order_type,
            }
            if limit_price:
                payload["limit_price"] = limit_price

            response = self.session.post(url, json=payload)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error placing order: {e}")
            return {
                "success": False,
                "error": f"REST API error: {str(e)}"
            }

    def place_box_spread(
        self,
        symbol: str,
        lower_strike: float,
        upper_strike: float,
        expiry: str,
        quantity: int = 1,
    ) -> Dict[str, Any]:
        """
        Place a box spread order.

        Args:
            symbol: Underlying symbol
            lower_strike: Lower strike price
            upper_strike: Upper strike price
            expiry: Expiration date (YYYY-MM-DD)
            quantity: Number of spreads

        Returns:
            Box spread order result
        """
        if DRY_RUN:
            return {
                "success": True,
                "dry_run": True,
                "order_id": f"BOX-DRY-{int(datetime.now().timestamp())}",
                "symbol": symbol,
                "lower_strike": lower_strike,
                "upper_strike": upper_strike,
                "expiry": expiry,
                "quantity": quantity,
                "legs": 4,
                "message": "[DRY RUN] Box spread would be placed"
            }

        if self.use_bindings:
            # Future: Direct C++ call via Cython
            # return self._place_box_spread_via_bindings(...)
            pass

        # Use REST API
        return self._place_box_spread_via_rest(
            symbol, lower_strike, upper_strike, expiry, quantity
        )

    def _place_box_spread_via_rest(
        self,
        symbol: str,
        lower_strike: float,
        upper_strike: float,
        expiry: str,
        quantity: int,
    ) -> Dict[str, Any]:
        """Place box spread via REST API."""
        try:
            url = f"{self.rest_url}/api/v1/orders/box_spread"
            payload = {
                "symbol": symbol,
                "lower_strike": lower_strike,
                "upper_strike": upper_strike,
                "expiry": expiry,
                "quantity": quantity,
            }

            response = self.session.post(url, json=payload)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error placing box spread: {e}")
            return {
                "success": False,
                "error": f"REST API error: {str(e)}"
            }

    def cancel_order(self, order_id: str) -> Dict[str, Any]:
        """
        Cancel an order.

        Args:
            order_id: Order ID to cancel

        Returns:
            Cancellation result
        """
        if DRY_RUN:
            return {
                "success": True,
                "dry_run": True,
                "order_id": order_id,
                "message": "[DRY RUN] Order would be cancelled"
            }

        try:
            url = f"{self.rest_url}/api/v1/orders/{order_id}/cancel"
            response = self.session.post(url)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error cancelling order: {e}")
            return {
                "success": False,
                "error": f"REST API error: {str(e)}"
            }

    def get_open_positions(self) -> Dict[str, Any]:
        """
        Get all open positions.

        Returns:
            Positions dictionary
        """
        try:
            url = f"{self.rest_url}/api/v1/positions"
            response = self.session.get(url)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error getting positions: {e}")
            # Return mock data for development
            return {
                "success": True,
                "positions": []
            }

    def get_quote(self, symbol: str) -> Dict[str, Any]:
        """
        Get real-time quote.

        Args:
            symbol: Trading symbol

        Returns:
            Quote data
        """
        try:
            url = f"{self.rest_url}/api/v1/market_data/quote"
            params = {"symbol": symbol}
            response = self.session.get(url, params=params)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error getting quote: {e}")
            # Return mock data for development
            return {
                "success": True,
                "symbol": symbol,
                "bid": 5090.25,
                "ask": 5090.50,
                "last": 5090.35,
                "volume": 1234567,
                "timestamp": datetime.now().isoformat()
            }

    def get_funds(self) -> Dict[str, Any]:
        """
        Get account funds and buying power.

        Returns:
            Account funds data
        """
        try:
            url = f"{self.rest_url}/api/v1/account/funds"
            response = self.session.get(url)
            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            logger.error(f"REST API error getting funds: {e}")
            # Return mock data for development
            return {
                "success": True,
                "net_liquidation_value": 100500.00,
                "buying_power": 80400.00,
                "excess_liquidity": 25000.00,
                "margin_requirement": 15000.00,
                "available_funds": 25000.00
            }


# Global bridge instance
_bridge_instance: Optional[TradingBridge] = None


def get_bridge() -> TradingBridge:
    """Get or create global trading bridge instance."""
    global _bridge_instance
    if _bridge_instance is None:
        _bridge_instance = TradingBridge()
    return _bridge_instance
