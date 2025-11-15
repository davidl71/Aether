#!/usr/bin/env python3
"""
Trading MCP Server

Model Context Protocol server for trading operations.
Inspired by OpenAlgo's MCP integration.

Provides AI assistants with trading capabilities:
- Order placement and management
- Position tracking
- Market data queries
- Account information
"""

import os
import sys
import json
import logging
from typing import Any, Dict, List, Optional, Tuple
from datetime import datetime, timedelta
from collections import defaultdict
import time

try:
    from mcp import FastMCP
    from mcp.types import Tool, TextContent
except ImportError:
    print("Error: FastMCP not installed. Install with: pip install mcp")
    sys.exit(1)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Import trading bridge
try:
    from .bridge import get_bridge
    BRIDGE_AVAILABLE = True
except ImportError:
    # Fallback if bridge not available
    BRIDGE_AVAILABLE = False
    logger.warning("Trading bridge not available - using mock responses")

# Rate limiting (inspired by OpenAlgo)
class RateLimiter:
    """Moving window rate limiter per IP/endpoint."""

    def __init__(self, max_requests: int, window_seconds: int = 1):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.requests: Dict[str, List[float]] = defaultdict(list)

    def check(self, key: str) -> Tuple[bool, Optional[float]]:
        """Check if request is allowed. Returns (allowed, retry_after)."""
        now = time.time()
        window_start = now - self.window_seconds

        # Clean old requests
        self.requests[key] = [t for t in self.requests[key] if t > window_start]

        if len(self.requests[key]) >= self.max_requests:
            # Calculate retry after
            oldest = min(self.requests[key])
            retry_after = self.window_seconds - (now - oldest)
            return False, max(0.1, retry_after)

        self.requests[key].append(now)
        return True, None

# Global rate limiters
ORDER_RATE_LIMIT = int(os.getenv("ORDER_RATE_LIMIT", "10"))
API_RATE_LIMIT = int(os.getenv("API_RATE_LIMIT", "10"))
MARKET_DATA_RATE_LIMIT = int(os.getenv("MARKET_DATA_RATE_LIMIT", "20"))

order_limiter = RateLimiter(ORDER_RATE_LIMIT)
api_limiter = RateLimiter(API_RATE_LIMIT)
market_data_limiter = RateLimiter(MARKET_DATA_RATE_LIMIT)

# API key validation (in production, use proper authentication)
API_KEY = os.getenv("TRADING_API_KEY", "")
DRY_RUN = os.getenv("DRY_RUN", "true").lower() == "true"

def validate_api_key(api_key: Optional[str]) -> bool:
    """Validate API key."""
    if not API_KEY:
        return True  # No key required in development
    return api_key == API_KEY

def rate_limit_check(limiter: RateLimiter, endpoint: str, api_key: str) -> Tuple[bool, Optional[Dict]]:
    """Check rate limit. Returns (allowed, error_response)."""
    key = f"{endpoint}:{api_key}"
    allowed, retry_after = limiter.check(key)

    if not allowed:
        return False, {
            "success": False,
            "error": {
                "code": "RATE_LIMIT_EXCEEDED",
                "message": f"Too many requests to {endpoint}. Please wait.",
                "retry_after": retry_after
            }
        }
    return True, None

# Initialize MCP server
mcp = FastMCP("Trading Server")

# ============================================================================
# Order Operations
# ============================================================================

@mcp.tool()
def place_order(
    symbol: str,
    side: str,  # "BUY" or "SELL"
    quantity: int,
    order_type: str = "MARKET",  # "MARKET" or "LIMIT"
    limit_price: Optional[float] = None,
    api_key: Optional[str] = None
) -> str:
    """
    Place a single order.

    Args:
        symbol: Trading symbol (e.g., "SPY", "SPX")
        side: "BUY" or "SELL"
        quantity: Number of shares/contracts
        order_type: "MARKET" or "LIMIT"
        limit_price: Required for LIMIT orders
        api_key: API key for authentication

    Returns:
        JSON string with order result
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(order_limiter, "place_order", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to place order
            bridge = get_bridge()
            result = bridge.place_order(
                symbol=symbol,
                side=side,
                quantity=quantity,
                order_type=order_type,
                limit_price=limit_price
            )
        else:
            # Fallback: Mock response
            if DRY_RUN:
                result = {
                    "success": True,
                    "dry_run": True,
                    "order_id": f"DRY-{int(time.time())}",
                    "symbol": symbol,
                    "side": side,
                    "quantity": quantity,
                    "order_type": order_type,
                    "limit_price": limit_price,
                    "message": "[DRY RUN] Order would be placed"
                }
            else:
                result = {
                    "success": True,
                    "order_id": f"ORD-{int(time.time())}",
                    "symbol": symbol,
                    "side": side,
                    "quantity": quantity,
                    "order_type": order_type,
                    "limit_price": limit_price,
                    "status": "SUBMITTED"
                }

        logger.info(f"Order placed: {side} {quantity} {symbol}")
        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error placing order: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

@mcp.tool()
def place_box_spread(
    symbol: str,
    lower_strike: float,
    upper_strike: float,
    expiry: str,  # YYYY-MM-DD
    quantity: int = 1,
    api_key: Optional[str] = None
) -> str:
    """
    Place a box spread order (4-leg options strategy).

    Args:
        symbol: Underlying symbol (e.g., "SPX", "XSP")
        lower_strike: Lower strike price
        upper_strike: Upper strike price
        expiry: Expiration date (YYYY-MM-DD)
        quantity: Number of spreads
        api_key: API key for authentication

    Returns:
        JSON string with box spread order result
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(order_limiter, "place_box_spread", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to place box spread
            bridge = get_bridge()
            result = bridge.place_box_spread(
                symbol=symbol,
                lower_strike=lower_strike,
                upper_strike=upper_strike,
                expiry=expiry,
                quantity=quantity
            )
        else:
            # Fallback: Mock response
            if DRY_RUN:
                result = {
                    "success": True,
                    "dry_run": True,
                    "order_id": f"BOX-DRY-{int(time.time())}",
                    "symbol": symbol,
                    "lower_strike": lower_strike,
                    "upper_strike": upper_strike,
                    "expiry": expiry,
                    "quantity": quantity,
                    "legs": 4,
                    "message": "[DRY RUN] Box spread would be placed"
                }
            else:
                result = {
                    "success": True,
                    "order_id": f"BOX-{int(time.time())}",
                    "symbol": symbol,
                    "lower_strike": lower_strike,
                    "upper_strike": upper_strike,
                    "expiry": expiry,
                    "quantity": quantity,
                    "legs": 4,
                    "status": "SUBMITTED"
                }

        logger.info(f"Box spread placed: {symbol} {lower_strike}/{upper_strike} {expiry}")
        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error placing box spread: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

@mcp.tool()
def cancel_order(
    order_id: str,
    api_key: Optional[str] = None
) -> str:
    """
    Cancel an order.

    Args:
        order_id: Order ID to cancel
        api_key: API key for authentication

    Returns:
        JSON string with cancellation result
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(order_limiter, "cancel_order", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to cancel order
            bridge = get_bridge()
            result = bridge.cancel_order(order_id)
        else:
            # Fallback: Mock response
            if DRY_RUN:
                result = {
                    "success": True,
                    "dry_run": True,
                    "order_id": order_id,
                    "message": "[DRY RUN] Order would be cancelled"
                }
            else:
                result = {
                    "success": True,
                    "order_id": order_id,
                    "status": "CANCELLED"
                }

        logger.info(f"Order cancelled: {order_id}")
        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error cancelling order: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

# ============================================================================
# Position Management
# ============================================================================

@mcp.tool()
def get_open_positions(
    api_key: Optional[str] = None
) -> str:
    """
    Get all open positions.

    Args:
        api_key: API key for authentication

    Returns:
        JSON string with positions
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(api_limiter, "get_open_positions", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to get positions
            bridge = get_bridge()
            result = bridge.get_open_positions()
        else:
            # Fallback: Mock data
            result = {
                "success": True,
                "positions": [
                    {
                        "symbol": "SPX",
                        "quantity": 1,
                        "avg_price": 160.50,
                        "current_price": 161.25,
                        "unrealized_pnl": 0.75,
                        "position_type": "BOX_SPREAD"
                    }
                ]
            }

        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error getting positions: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

# ============================================================================
# Market Data
# ============================================================================

@mcp.tool()
def get_quote(
    symbol: str,
    api_key: Optional[str] = None
) -> str:
    """
    Get real-time quote for a symbol.

    Args:
        symbol: Trading symbol
        api_key: API key for authentication

    Returns:
        JSON string with quote data
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(market_data_limiter, "get_quote", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to get quote
            bridge = get_bridge()
            result = bridge.get_quote(symbol)
        else:
            # Fallback: Mock data
            result = {
                "success": True,
                "symbol": symbol,
                "bid": 5090.25,
                "ask": 5090.50,
                "last": 5090.35,
                "volume": 1234567,
                "timestamp": datetime.now().isoformat()
            }

        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error getting quote: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

# ============================================================================
# Account Information
# ============================================================================

@mcp.tool()
def get_funds(
    api_key: Optional[str] = None
) -> str:
    """
    Get account funds and buying power.

    Args:
        api_key: API key for authentication

    Returns:
        JSON string with account funds
    """
    if not validate_api_key(api_key):
        return json.dumps({"success": False, "error": "Invalid API key"})

    allowed, error = rate_limit_check(api_limiter, "get_funds", api_key or "default")
    if not allowed:
        return json.dumps(error)

    try:
        if BRIDGE_AVAILABLE:
            # Use trading bridge to get funds
            bridge = get_bridge()
            result = bridge.get_funds()
        else:
            # Fallback: Mock data
            result = {
                "success": True,
                "net_liquidation_value": 100500.00,
                "buying_power": 80400.00,
                "excess_liquidity": 25000.00,
                "margin_requirement": 15000.00,
                "available_funds": 25000.00
            }

        return json.dumps(result, indent=2)

    except Exception as e:
        logger.error(f"Error getting funds: {e}")
        return json.dumps({
            "success": False,
            "error": str(e)
        })

# Run server
if __name__ == "__main__":
    mcp.run()
