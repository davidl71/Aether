# LEAN REST API Wrapper Architecture Design

**Date**: 2025-11-18
**Status**: Design Complete
**Purpose**: Architecture design for REST API wrapper around LEAN that exposes LEAN's internal state to PWA/TUI clients

---

## Overview

This document designs a FastAPI-based REST API wrapper that exposes LEAN's internal state (portfolio, positions, orders, metrics) via REST endpoints matching the existing API contract. The wrapper bridges LEAN's headless execution engine with PWA/TUI frontends.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    PWA/TUI Clients                         │
│  - React PWA (web/)                                         │
│  - C++ TUI (native/src/tui_app.cpp)                        │
│  - iPad App (future)                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ HTTP REST API
                       │ (matches API_CONTRACT.md)
                       │
┌──────────────────────▼──────────────────────────────────────┐
│         FastAPI REST Wrapper (Python)                       │
│  - FastAPI application                                      │
│  - REST endpoints (/api/v1/snapshot, /strategy/start, etc.)│
│  - Data conversion (LEAN format → API contract)            │
│  - Error handling and validation                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Python API
                       │
┌──────────────────────▼──────────────────────────────────────┐
│         LEAN Client Wrapper (Python)                       │
│  - Manages LEAN algorithm instance                         │
│  - Queries Portfolio, Positions, Orders                    │
│  - Subscribes to LEAN events                               │
│  - Handles connection state                                │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ LEAN Python API
                       │
┌──────────────────────▼──────────────────────────────────────┐
│              LEAN Algorithm (Running)                       │
│  class BoxSpreadAlgorithm(QCAlgorithm):                     │
│    - Portfolio[symbol] → Position data                     │
│    - Securities[symbol] → Security data                     │
│    - active_positions → Tracked positions                   │
│    - pending_orders → Tracked orders                        │
│    - OnOrderEvent() → Order events                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Broker APIs
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│   IBKR TWS   │ │  Alpaca   │ │   ORATS   │
│     API      │ │    API    │ │    API    │
└──────────────┘ └───────────┘ └───────────┘
```

---

## Component Design

### 1. LEAN Client Wrapper

**File**: `python/lean_integration/lean_client.py`

**Purpose**: Manages LEAN algorithm instance and provides methods to query internal state.

```python
from typing import Optional, Dict, List
from AlgorithmImports import *
from lean_integration.box_spread_algorithm import BoxSpreadAlgorithm

class LeanClient:
    """Wrapper for accessing LEAN algorithm internal state."""

    def __init__(self):
        self.algorithm: Optional[BoxSpreadAlgorithm] = None
        self.is_running = False

    def set_algorithm(self, algorithm: BoxSpreadAlgorithm):
        """Set the running LEAN algorithm instance."""
        self.algorithm = algorithm
        self.is_running = algorithm is not None

    def get_portfolio(self) -> Dict:
        """Get portfolio summary from LEAN."""
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        portfolio = self.algorithm.Portfolio
        return {
            "total_portfolio_value": float(portfolio.TotalPortfolioValue),
            "cash": float(portfolio.Cash),
            "unrealized_profit": float(portfolio.TotalUnrealizedProfit),
            "realized_profit": float(portfolio.TotalFeesPaid)  # Approximation
        }

    def get_positions(self) -> List[Dict]:
        """Get all positions from LEAN."""
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        positions = []
        for symbol, holding in self.algorithm.Portfolio.items():
            if holding.Quantity != 0:
                positions.append({
                    "symbol": symbol.Value,
                    "quantity": int(holding.Quantity),
                    "average_price": float(holding.AveragePrice),
                    "unrealized_profit": float(holding.UnrealizedProfit),
                    "holdings_value": float(holding.HoldingsValue)
                })
        return positions

    def get_orders(self) -> List[Dict]:
        """Get order history from LEAN."""
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        orders = []
        # Access LEAN's Transactions or track via OnOrderEvent
        for order_id, order_info in self.algorithm.pending_orders.items():
            orders.append({
                "id": str(order_id),
                "status": "PENDING",
                "symbol": order_info.get("symbol", "UNKNOWN"),
                "timestamp": order_info.get("timestamp", None)
            })

        # Add filled orders from active_positions
        for order_id, position_info in self.algorithm.active_positions.items():
            orders.append({
                "id": str(order_id),
                "status": "FILLED",
                "symbol": position_info.get("spread", {}).get("symbol", "UNKNOWN"),
                "timestamp": position_info.get("entry_time", None)
            })

        return orders

    def get_metrics(self) -> Dict:
        """Calculate metrics from LEAN portfolio."""
        if not self.algorithm:
            raise RuntimeError("LEAN algorithm not running")

        portfolio = self.algorithm.Portfolio
        return {
            "net_liq": float(portfolio.TotalPortfolioValue),
            "buying_power": float(portfolio.MarginRemaining),
            "excess_liquidity": float(portfolio.MarginRemaining),
            "margin_requirement": float(portfolio.TotalMarginUsed),
            "commissions": float(portfolio.TotalFeesPaid)
        }

    def start_algorithm(self):
        """Start LEAN algorithm (if not already running)."""
        if self.is_running:
            raise RuntimeError("Algorithm already running")
        # Implementation depends on how LEAN is launched
        # This would typically be handled by LEAN CLI or launcher

    def stop_algorithm(self):
        """Stop LEAN algorithm gracefully."""
        if not self.is_running:
            raise RuntimeError("Algorithm not running")
        # Implementation depends on how LEAN is launched
```

---

### 2. FastAPI Application

**File**: `python/lean_integration/api_wrapper.py`

**Purpose**: FastAPI application with REST endpoints matching API contract.

```python
from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional
from datetime import datetime
from lean_integration.lean_client import LeanClient
from lean_integration.api_converter import ApiConverter
from lean_integration.api_models import (
    SnapshotResponse,
    StrategyStartRequest,
    StrategyStopRequest
)

app = FastAPI(
    title="LEAN REST API Wrapper",
    description="REST API wrapper for QuantConnect LEAN algorithmic trading engine",
    version="1.0.0"
)

# CORS middleware for PWA access
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global LEAN client instance
lean_client = LeanClient()
api_converter = ApiConverter()

def get_lean_client() -> LeanClient:
    """Dependency to get LEAN client."""
    if not lean_client.is_running:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="LEAN algorithm is not running"
        )
    return lean_client

@app.get("/health")
async def health():
    """Health check endpoint."""
    return {
        "status": "ok",
        "lean_running": lean_client.is_running,
        "timestamp": datetime.utcnow().isoformat()
    }

@app.get("/api/v1/snapshot", response_model=SnapshotResponse)
async def get_snapshot(client: LeanClient = Depends(get_lean_client)):
    """Get system snapshot matching API contract."""
    try:
        # Query LEAN internal state
        portfolio = client.get_portfolio()
        positions = client.get_positions()
        orders = client.get_orders()
        metrics = client.get_metrics()

        # Convert to API contract format
        snapshot = api_converter.build_snapshot(
            portfolio=portfolio,
            positions=positions,
            orders=orders,
            metrics=metrics,
            algorithm=client.algorithm
        )

        return snapshot

    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=str(e)
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to get snapshot: {str(e)}"
        )

@app.post("/api/v1/strategy/start")
async def strategy_start(
    request: StrategyStartRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """Start LEAN strategy."""
    try:
        client.start_algorithm()
        return {"status": "started", "timestamp": datetime.utcnow().isoformat()}
    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )

@app.post("/api/v1/strategy/stop")
async def strategy_stop(
    request: StrategyStopRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """Stop LEAN strategy."""
    try:
        client.stop_algorithm()
        return {"status": "stopped", "timestamp": datetime.utcnow().isoformat()}
    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )

@app.post("/api/v1/orders/cancel")
async def cancel_order(
    order_id: str,
    client: LeanClient = Depends(get_lean_client)
):
    """Cancel a specific order."""
    # Implementation: Cancel order in LEAN
    # LEAN API: algorithm.Transactions.CancelOrder(order_id)
    pass

@app.post("/api/v1/combos/buy")
async def buy_combo(
    combo_request: dict,
    client: LeanClient = Depends(get_lean_client)
):
    """Place buy combo order."""
    # Implementation: Place combo order via LEAN
    pass

@app.post("/api/v1/combos/sell")
async def sell_combo(
    combo_request: dict,
    client: LeanClient = Depends(get_lean_client)
):
    """Place sell combo order."""
    # Implementation: Place combo order via LEAN
    pass

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

---

### 3. Data Converter

**File**: `python/lean_integration/api_converter.py`

**Purpose**: Convert LEAN data format to API contract format.

```python
from typing import Dict, List, Optional
from datetime import datetime, timezone
from lean_integration.api_models import (
    SnapshotResponse,
    Metrics,
    SymbolSnapshot,
    PositionSnapshot,
    OrderSnapshot,
    StrategyDecisionSnapshot,
    Alert,
    RiskStatus
)

class ApiConverter:
    """Convert LEAN format to API contract format."""

    def build_snapshot(
        self,
        portfolio: Dict,
        positions: List[Dict],
        orders: List[Dict],
        metrics: Dict,
        algorithm
    ) -> SnapshotResponse:
        """Build snapshot response matching API contract."""

        # Convert positions
        position_snapshots = [
            self._convert_position(pos) for pos in positions
        ]

        # Convert orders
        order_snapshots = [
            self._convert_order(order) for order in orders
        ]

        # Convert symbols (from LEAN Securities)
        symbol_snapshots = self._extract_symbols(algorithm)

        # Build metrics
        metrics_obj = Metrics(
            net_liq=metrics.get("net_liq", 0.0),
            buying_power=metrics.get("buying_power", 0.0),
            excess_liquidity=metrics.get("excess_liquidity", 0.0),
            margin_requirement=metrics.get("margin_requirement", 0.0),
            commissions=metrics.get("commissions", 0.0),
            portal_ok=True,  # From LEAN broker connection status
            tws_ok=True,    # From LEAN broker connection status
            orats_ok=True,  # From configuration
            questdb_ok=True # From configuration
        )

        # Build risk status
        risk_status = RiskStatus(
            allowed=True,  # From LEAN risk checks
            reason=None,
            updated_at=datetime.now(timezone.utc)
        )

        return SnapshotResponse(
            generated_at=datetime.now(timezone.utc),
            mode="DRY-RUN",  # From LEAN configuration
            strategy="RUNNING" if algorithm else "IDLE",
            account_id=self._get_account_id(algorithm),
            metrics=metrics_obj,
            symbols=symbol_snapshots,
            positions=position_snapshots,
            historic=[],  # From QuestDB or LEAN history
            orders=order_snapshots,
            decisions=[],  # From LEAN strategy decisions
            alerts=[],     # From LEAN alerts/logs
            risk=risk_status
        )

    def _convert_position(self, pos: Dict) -> PositionSnapshot:
        """Convert LEAN position to PositionSnapshot."""
        return PositionSnapshot(
            id=f"POS-{pos.get('symbol', 'UNKNOWN')}",
            symbol=pos.get("symbol", ""),
            quantity=pos.get("quantity", 0),
            cost_basis=pos.get("average_price", 0.0),
            mark=pos.get("average_price", 0.0),  # Use current market price
            unrealized_pnl=pos.get("unrealized_profit", 0.0)
        )

    def _convert_order(self, order: Dict) -> OrderSnapshot:
        """Convert LEAN order to OrderSnapshot."""
        return OrderSnapshot(
            id=order.get("id", ""),
            symbol=order.get("symbol", ""),
            side="BUY",  # Determine from order
            quantity=1,  # From order
            status=order.get("status", "UNKNOWN"),
            submitted_at=order.get("timestamp", datetime.now(timezone.utc))
        )

    def _extract_symbols(self, algorithm) -> List[SymbolSnapshot]:
        """Extract symbol data from LEAN Securities."""
        symbols = []
        if not algorithm:
            return symbols

        for symbol, security in algorithm.Securities.items():
            # Get market data from security
            if security.Price > 0:
                symbols.append(SymbolSnapshot(
                    symbol=symbol.Value,
                    last=float(security.Price),
                    bid=float(security.BidPrice) if security.BidPrice > 0 else float(security.Price),
                    ask=float(security.AskPrice) if security.AskPrice > 0 else float(security.Price),
                    spread=float(security.AskPrice - security.BidPrice) if security.AskPrice > 0 and security.BidPrice > 0 else 0.0,
                    roi=0.0,  # Calculate from box spread opportunities
                    maker_count=0,
                    taker_count=0,
                    volume=int(security.Volume) if security.Volume > 0 else 0,
                    candle=None  # Build from historical data
                ))

        return symbols

    def _get_account_id(self, algorithm) -> str:
        """Get account ID from LEAN."""
        if algorithm and hasattr(algorithm, 'AccountId'):
            return algorithm.AccountId
        return "DU123456"  # Default
```

---

### 4. API Models (Pydantic)

**File**: `python/lean_integration/api_models.py`

**Purpose**: Pydantic models matching API contract schema.

```python
from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime

class Metrics(BaseModel):
    net_liq: float
    buying_power: float
    excess_liquidity: float
    margin_requirement: float
    commissions: float
    portal_ok: bool
    tws_ok: bool
    orats_ok: bool
    questdb_ok: bool

class CandleData(BaseModel):
    open: float
    high: float
    low: float
    close: float
    volume: int
    entry: Optional[float] = None
    updated: datetime

class SymbolSnapshot(BaseModel):
    symbol: str
    last: float
    bid: float
    ask: float
    spread: float
    roi: float
    maker_count: int
    taker_count: int
    volume: int
    candle: Optional[CandleData] = None

class PositionSnapshot(BaseModel):
    id: str
    symbol: str
    quantity: int
    cost_basis: float
    mark: float
    unrealized_pnl: float

class HistoricPosition(BaseModel):
    id: str
    symbol: str
    quantity: int
    realized_pnl: float
    closed_at: datetime

class OrderSnapshot(BaseModel):
    id: str
    symbol: str
    side: str
    quantity: int
    status: str
    submitted_at: datetime

class StrategyDecisionSnapshot(BaseModel):
    symbol: str
    quantity: int
    side: str
    mark: float
    created_at: datetime

class Alert(BaseModel):
    level: str  # "info", "warning", "error"
    message: str
    timestamp: datetime

class RiskStatus(BaseModel):
    allowed: bool
    reason: Optional[str] = None
    updated_at: datetime

class SnapshotResponse(BaseModel):
    generated_at: datetime
    mode: str
    strategy: str
    account_id: str
    metrics: Metrics
    symbols: List[SymbolSnapshot]
    positions: List[PositionSnapshot]
    historic: List[HistoricPosition]
    orders: List[OrderSnapshot]
    decisions: List[StrategyDecisionSnapshot]
    alerts: List[Alert]
    risk: RiskStatus

class StrategyStartRequest(BaseModel):
    confirm: bool = Field(default=False, description="Confirmation required")

class StrategyStopRequest(BaseModel):
    confirm: bool = Field(default=False, description="Confirmation required")
```

---

## Data Flow

### 1. Snapshot Request Flow

```
PWA/TUI Client
    ↓ HTTP GET /api/v1/snapshot
FastAPI Endpoint
    ↓ Depends(get_lean_client)
LEAN Client Wrapper
    ↓ Query algorithm.Portfolio, algorithm.Securities, algorithm.active_positions
LEAN Algorithm Instance
    ↓ Return data
LEAN Client Wrapper
    ↓ Format data
Data Converter
    ↓ Convert to API contract format
FastAPI Endpoint
    ↓ Return JSON
PWA/TUI Client
```

### 2. Strategy Control Flow

```
PWA/TUI Client
    ↓ HTTP POST /api/v1/strategy/start
FastAPI Endpoint
    ↓ Depends(get_lean_client)
LEAN Client Wrapper
    ↓ client.start_algorithm()
LEAN Algorithm Instance
    ↓ Start execution
LEAN Client Wrapper
    ↓ Update state
FastAPI Endpoint
    ↓ Return status
PWA/TUI Client
```

### 3. Real-Time Updates Flow (Future - T-51)

```
LEAN Algorithm
    ↓ OnOrderEvent(), OnSecuritiesChanged()
LEAN Client Wrapper
    ↓ Event subscription
WebSocket Bridge (T-51)
    ↓ Push to clients
PWA/TUI Clients (WebSocket)
```

---

## Integration Points

### LEAN Python API Used

1. **Portfolio Access:**
   - `algorithm.Portfolio[symbol]` → `Holding` object
   - `algorithm.Portfolio.TotalPortfolioValue` → Total value
   - `algorithm.Portfolio.Cash` → Cash balance
   - `algorithm.Portfolio.MarginRemaining` → Buying power

2. **Securities Access:**
   - `algorithm.Securities[symbol]` → `Security` object
   - `algorithm.Securities[symbol].Price` → Last price
   - `algorithm.Securities[symbol].BidPrice` → Bid price
   - `algorithm.Securities[symbol].AskPrice` → Ask price
   - `algorithm.Securities[symbol].Volume` → Volume

3. **Order Tracking:**
   - `algorithm.Transactions` → Order history
   - `algorithm.pending_orders` → Pending orders (custom tracking)
   - `algorithm.active_positions` → Active positions (custom tracking)

4. **Event Callbacks:**
   - `OnOrderEvent(orderEvent)` → Order fills, cancellations
   - `OnSecuritiesChanged(changes)` → Security additions/removals

### API Contract Compliance

All endpoints must match `agents/shared/API_CONTRACT.md`:

- ✅ `GET /api/v1/snapshot` → Returns `SnapshotResponse`
- ✅ `POST /api/v1/strategy/start` → Starts strategy
- ✅ `POST /api/v1/strategy/stop` → Stops strategy
- ✅ `POST /api/v1/orders/cancel` → Cancels order
- ✅ `POST /api/v1/combos/buy` → Places buy combo
- ✅ `POST /api/v1/combos/sell` → Places sell combo

---

## Error Handling

### LEAN Not Running

```python
if not lean_client.is_running:
    raise HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail="LEAN algorithm is not running"
    )
```

### LEAN Connection Lost

```python
try:
    portfolio = client.get_portfolio()
except RuntimeError as e:
    raise HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=f"LEAN connection lost: {str(e)}"
    )
```

### Invalid Data Format

```python
try:
    snapshot = api_converter.build_snapshot(...)
except ValueError as e:
    raise HTTPException(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        detail=f"Data conversion failed: {str(e)}"
    )
```

---

## Performance Considerations

### Caching Strategy

- **Snapshot Data**: Cache for 100-500ms to reduce LEAN queries
- **Portfolio Data**: Cache for 1-2 seconds (less frequently changing)
- **Order History**: Cache for 5-10 seconds (historical data)

### Async Operations

- Use `async/await` for all endpoints
- Non-blocking LEAN queries where possible
- Background tasks for data aggregation

### Connection Pooling

- Single LEAN algorithm instance (shared across requests)
- Thread-safe access to LEAN client
- Lock mechanism for concurrent requests

---

## Security Considerations

### Authentication (Future)

- API key authentication
- Token-based authentication
- OAuth2 for production

### CORS Configuration

- Configure allowed origins for PWA
- Restrict methods and headers
- Enable credentials for authenticated requests

### Input Validation

- Pydantic models validate all inputs
- Sanitize order IDs, symbols
- Validate confirmation flags

---

## Configuration

### Environment Variables

```bash
LEAN_API_HOST=0.0.0.0
LEAN_API_PORT=8000
LEAN_ALGORITHM_PATH=Main/box_spread_algorithm.py
LEAN_CONFIG_PATH=config/lean_config.json
CORS_ORIGINS=http://localhost:3000,http://localhost:5173
```

### Configuration File

```json
{
  "lean": {
    "algorithm_path": "Main/box_spread_algorithm.py",
    "config_path": "config/lean_config.json",
    "broker": "InteractiveBrokers",
    "account": "DU123456"
  },
  "api": {
    "host": "0.0.0.0",
    "port": 8000,
    "cors_origins": ["http://localhost:3000"]
  },
  "cache": {
    "snapshot_ttl_ms": 200,
    "portfolio_ttl_ms": 1000
  }
}
```

---

## Testing Strategy

### Unit Tests

1. **LEAN Client:**
   - Test portfolio query
   - Test positions query
   - Test orders query
   - Test error handling

2. **Data Converter:**
   - Test LEAN → API contract conversion
   - Test missing data handling
   - Test edge cases

3. **API Endpoints:**
   - Test snapshot endpoint
   - Test strategy control endpoints
   - Test error responses

### Integration Tests

1. **With Mock LEAN:**
   - Mock LEAN algorithm instance
   - Test complete flow
   - Test error scenarios

2. **With Real LEAN (Paper Trading):**
   - Test with paper trading account
   - Validate data accuracy
   - Test strategy controls

---

## Deployment

### Development

```bash
cd python/lean_integration
uvicorn api_wrapper:app --reload --host 0.0.0.0 --port 8000
```

### Production

```bash
# Use gunicorn with uvicorn workers
gunicorn api_wrapper:app \
  --workers 4 \
  --worker-class uvicorn.workers.UvicornWorker \
  --bind 0.0.0.0:8000
```

### Docker (Future)

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
CMD ["uvicorn", "api_wrapper:app", "--host", "0.0.0.0", "--port", "8000"]
```

---

## Next Steps

1. ✅ **Architecture Design Complete** (this document)
2. ⏳ **Implement LEAN Client Wrapper** (T-50)
3. ⏳ **Implement FastAPI Application** (T-50)
4. ⏳ **Implement Data Converter** (T-50)
5. ⏳ **Implement WebSocket Bridge** (T-51)
6. ⏳ **Integration Testing** (T-52)

---

## References

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [LEAN Algorithm Structure](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/algorithm-structure)
- [LEAN Portfolio Management](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/portfolio-management)
- [API Contract](./agents/shared/API_CONTRACT.md)
- [LEAN Strategy Architecture](../../LEAN_STRATEGY_ARCHITECTURE.md)
- [LEAN PWA/TUI Integration Analysis](./LEAN_PWA_TUI_INTEGRATION_ANALYSIS.md)

---

## Status

- ✅ Architecture designed
- ✅ Component structure defined
- ✅ Data flow documented
- ✅ Integration points identified
- ✅ Error handling planned
- ✅ Performance considerations addressed
- ✅ Security considerations documented
- ✅ Testing strategy defined
