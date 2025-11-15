# Cython Bindings Guide for OrderManager

**Purpose**: Guide for creating Cython bindings to enable direct native C++ integration with OrderManager.

**Status**: Documentation and structure ready. Implementation pending.

---

## Overview

Currently, the MCP trading server uses a REST API bridge to communicate with native C++ code. This guide documents how to add direct Cython bindings for better performance and tighter integration.

---

## Architecture

### Current Architecture (REST API)

```
MCP Server (Python)
    ↓
Trading Bridge (bridge.py)
    ↓
REST API → Backend Service → Native C++ OrderManager → TWS API
```

### Future Architecture (Cython Bindings)

```
MCP Server (Python)
    ↓
Trading Bridge (bridge.py)
    ↓
Cython Bindings → Native C++ OrderManager → TWS API
```

---

## Implementation Steps

### Step 1: Add OrderManager Declarations to .pxd

**File**: `python/bindings/box_spread_bindings.pxd`

Add to the end of the file:

```cython
cdef extern from "order_manager.h" namespace "order":
    cdef cppclass ExecutionResult:
        bint success
        vector[int] order_ids
        string error_message
        double total_cost
        double average_fill_price
        int total_quantity_filled

    cdef cppclass OrderManager:
        OrderManager(tws.TWSClient* client, bint dry_run)
        ExecutionResult place_order(
            const types.OptionContract& contract,
            types.OrderAction action,
            int quantity,
            double limit_price,
            types.TimeInForce tif
        )
        bint cancel_order(int order_id)
        void cancel_all_orders()
        optional[types.Order] get_order_status(int order_id)
        ExecutionResult place_box_spread(
            const types.BoxSpreadLeg& spread,
            const string& strategy_id
        )
```

**Note**: This requires TWSClient pointer, which complicates initialization. See "Initialization Strategy" below.

### Step 2: Create Python Wrapper Class

**File**: `python/bindings/box_spread_bindings.pyx`

Add to the end of the file:

```cython
cdef class PyOrderManager:
    """Python wrapper for C++ OrderManager"""
    cdef unique_ptr[c_bindings.OrderManager] _manager
    cdef bint _dry_run

    def __cinit__(self, bint dry_run=False):
        """
        Initialize OrderManager.

        Note: TWSClient must be initialized separately and passed in.
        For now, this is a placeholder that will need TWSClient integration.
        """
        self._dry_run = dry_run
        # TODO: Initialize with TWSClient
        # This requires TWSClient to be initialized first
        # For now, we'll use a factory pattern or dependency injection

    def place_order(
        self,
        PyOptionContract contract,
        int action,  # OrderAction enum
        int quantity,
        double limit_price=0.0,
        int tif=0  # TimeInForce enum, default Day
    ):
        """Place an order."""
        if not self._manager:
            raise RuntimeError("OrderManager not initialized")

        cdef c_bindings.ExecutionResult result = self._manager.get().place_order(
            contract._ptr[0],
            <c_bindings.OrderAction>action,
            quantity,
            limit_price,
            <c_bindings.TimeInForce>tif
        )

        return {
            "success": result.success,
            "order_ids": [oid for oid in result.order_ids],
            "error_message": result.error_message.decode('utf-8') if result.error_message else "",
            "total_cost": result.total_cost,
            "average_fill_price": result.average_fill_price,
            "total_quantity_filled": result.total_quantity_filled
        }

    def place_box_spread(
        self,
        PyBoxSpreadLeg spread,
        str strategy_id=""
    ):
        """Place a box spread order."""
        if not self._manager:
            raise RuntimeError("OrderManager not initialized")

        cdef c_bindings.ExecutionResult result = self._manager.get().place_box_spread(
            spread._ptr[0],
            strategy_id.encode('utf-8')
        )

        return {
            "success": result.success,
            "order_ids": [oid for oid in result.order_ids],
            "error_message": result.error_message.decode('utf-8') if result.error_message else "",
            "total_cost": result.total_cost,
            "average_fill_price": result.average_fill_price,
            "total_quantity_filled": result.total_quantity_filled
        }

    def cancel_order(self, int order_id):
        """Cancel an order."""
        if not self._manager:
            raise RuntimeError("OrderManager not initialized")
        return self._manager.get().cancel_order(order_id)

    def cancel_all_orders(self):
        """Cancel all orders."""
        if not self._manager:
            raise RuntimeError("OrderManager not initialized")
        self._manager.get().cancel_all_orders()
```

### Step 3: Update setup.py

**File**: `python/bindings/setup.py`

Add OrderManager source files:

```python
extensions = [
    Extension(
        "box_spread_bindings",
        sources=[
            "box_spread_bindings.pyx",
            str(src_dir / "box_spread_strategy.cpp"),
            str(src_dir / "risk_calculator.cpp"),
            str(src_dir / "option_chain.cpp"),
            str(src_dir / "config_manager.cpp"),
            str(src_dir / "order_manager.cpp"),  # Add this
            str(src_dir / "tws_client.cpp"),     # Add this if needed
        ],
        # ... rest of config
    )
]
```

**Note**: This will require linking against TWS API library and other dependencies.

### Step 4: Update Trading Bridge

**File**: `mcp/trading_server/bridge.py`

Update to use Cython bindings when available:

```python
class TradingBridge:
    def __init__(self, rest_url: Optional[str] = None):
        self.rest_url = rest_url or BACKEND_REST_URL
        self.session = requests.Session()
        self.session.timeout = 10.0

        # Try to import Cython bindings
        self.use_bindings = False
        self.order_manager = None

        try:
            from native.bindings.box_spread_bindings import (
                PyOrderManager,
                PyOptionContract,
                PyBoxSpreadLeg,
                OrderAction,
                TimeInForce
            )
            # Initialize OrderManager
            # Note: This requires TWSClient to be initialized first
            # For now, we'll use a factory or singleton pattern
            # self.order_manager = PyOrderManager(dry_run=DRY_RUN)
            # self.use_bindings = True
            logger.info("Cython bindings available but not yet initialized")
        except ImportError:
            logger.debug("Cython bindings not available, using REST API")

    def place_order(self, ...):
        if self.use_bindings and self.order_manager:
            # Use direct C++ call
            contract = PyOptionContract(...)
            result = self.order_manager.place_order(
                contract,
                OrderAction.Buy if side == "BUY" else OrderAction.Sell,
                quantity,
                limit_price or 0.0,
                TimeInForce.Day
            )
            return result
        else:
            # Use REST API
            return self._place_order_via_rest(...)
```

---

## Initialization Strategy

### Challenge: TWSClient Dependency

OrderManager requires a TWSClient pointer, which creates a circular dependency:
- OrderManager needs TWSClient
- TWSClient needs to be initialized with config
- TWSClient must be connected before use

### Solution Options

#### Option 1: Factory Pattern

Create a factory that initializes both:

```python
# In bridge.py or separate factory module
def create_order_manager(tws_config: Dict, dry_run: bool = False):
    """
    Factory function to create OrderManager with TWSClient.

    Args:
        tws_config: TWS configuration dictionary
        dry_run: Enable dry-run mode

    Returns:
        PyOrderManager instance
    """
    # Initialize TWSClient first
    # Then create OrderManager with TWSClient pointer
    # This requires Cython bindings for TWSClient as well
    pass
```

#### Option 2: Singleton TWSClient

Use a singleton TWSClient that's initialized once:

```python
# Global TWSClient instance
_tws_client_instance = None

def get_tws_client():
    global _tws_client_instance
    if _tws_client_instance is None:
        # Initialize TWSClient
        _tws_client_instance = PyTWSClient(...)
    return _tws_client_instance

def create_order_manager(dry_run=False):
    tws_client = get_tws_client()
    return PyOrderManager(tws_client, dry_run)
```

#### Option 3: REST API (Current)

Keep using REST API, which is simpler and more decoupled:
- Backend service manages TWSClient lifecycle
- MCP server doesn't need to manage TWS connection
- Easier to scale and maintain
- Better separation of concerns

**Recommendation**: Option 3 (REST API) is the best approach for MCP server. Cython bindings are better suited for:
- High-frequency trading operations
- Direct strategy execution
- Performance-critical calculations

---

## When to Use Cython Bindings

### Use Cython Bindings When:
- ✅ Performance is critical (microsecond latency matters)
- ✅ Direct control over TWS connection is needed
- ✅ Running in same process as strategy execution
- ✅ Need to avoid network overhead

### Use REST API When:
- ✅ Decoupled architecture is preferred
- ✅ Multiple clients need access
- ✅ Easier deployment and scaling
- ✅ Better error isolation
- ✅ MCP server use case (current)

---

## Implementation Checklist

### For Cython Bindings:

- [ ] Add OrderManager declarations to `.pxd` file
- [ ] Create PyOrderManager wrapper class in `.pyx` file
- [ ] Add OrderManager source files to `setup.py`
- [ ] Create TWSClient Cython bindings (if needed)
- [ ] Implement factory pattern for initialization
- [ ] Update bridge.py to use bindings when available
- [ ] Add error handling and type conversions
- [ ] Test with dry-run mode
- [ ] Test with live TWS connection
- [ ] Update documentation

### For REST API (Current):

- [x] Create TradingBridge class
- [x] Implement REST API methods
- [x] Add fallback to mock data
- [x] Add error handling
- [x] Support dry-run mode
- [ ] Add request/response logging
- [ ] Add retry logic
- [ ] Add connection pooling

---

## Testing

### Test Cython Bindings

```python
# test_cython_bindings.py
def test_order_manager_bindings():
    from native.bindings.box_spread_bindings import (
        PyOrderManager,
        PyOptionContract,
        OrderAction
    )

    # Create OrderManager (requires TWSClient)
    manager = create_order_manager(dry_run=True)

    # Create contract
    contract = PyOptionContract(
        symbol="SPX",
        expiry="20250221",
        strike=5000.0,
        option_type=0  # Call
    )

    # Place order
    result = manager.place_order(
        contract=contract,
        action=OrderAction.Buy,
        quantity=1,
        limit_price=10.50
    )

    assert result["success"] == True
    assert len(result["order_ids"]) > 0
```

### Test REST API Bridge

```python
# test_rest_bridge.py
def test_rest_bridge():
    from mcp.trading_server.bridge import TradingBridge

    bridge = TradingBridge(rest_url="http://localhost:8080")

    result = bridge.place_order(
        symbol="SPX",
        side="BUY",
        quantity=1,
        order_type="LIMIT",
        limit_price=10.50
    )

    assert result["success"] == True
```

---

## Performance Comparison

### Expected Latency

- **REST API**: ~10-50ms (network + serialization)
- **Cython Bindings**: ~0.1-1ms (direct function call)

### When It Matters

- **High-frequency trading**: Cython bindings essential
- **MCP server (AI assistant)**: REST API is fine (human response time >> network latency)
- **Strategy execution**: Cython bindings preferred
- **Testing/development**: REST API is easier

---

## Current Recommendation

**For MCP Trading Server**: Continue using REST API approach because:
1. ✅ Simpler architecture
2. ✅ Better separation of concerns
3. ✅ Easier to test and debug
4. ✅ Network latency is negligible for AI assistant use case
5. ✅ Backend service already provides REST API

**For Future**: Consider Cython bindings for:
- Direct strategy execution in Python
- High-performance trading operations
- When running in same process as native code

---

## References

- [Cython Documentation](https://cython.readthedocs.io/)
- [Existing Bindings](./box_spread_bindings.pyx)
- [OrderManager Header](../../native/include/order_manager.h)
- [TWS Client Header](../../native/include/tws_client.h)

---

**Last Updated**: 2025-01-27
