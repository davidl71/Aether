# LEAN REST API Wrapper

REST API wrapper for QuantConnect LEAN that exposes LEAN's internal state to PWA/TUI clients.

## Overview

The LEAN REST API wrapper provides a FastAPI-based REST interface that bridges LEAN's headless execution engine with frontend clients (PWA, TUI, iPad app). It exposes portfolio, positions, orders, and metrics via REST endpoints matching the API contract defined in `agents/shared/API_CONTRACT.md`.

## Installation

### 1. Install Dependencies

```bash
# Update requirements
pip-compile requirements.in --allow-unsafe --output-file=requirements.txt

# Install dependencies
pip install -r requirements.txt
```

This will install:
- `fastapi>=0.115.0` - FastAPI framework
- `uvicorn[standard]>=0.30.0` - ASGI server
- `pydantic>=2.0.0` - Data validation

### 2. Verify Installation

```bash
python -c "import fastapi; print(fastapi.__version__)"
python -c "import uvicorn; print(uvicorn.__version__)"
```

## Usage

### Development Mode

```bash
cd python/lean_integration
python api_wrapper.py
```

Or using uvicorn directly:

```bash
uvicorn api_wrapper:app --reload --host 0.0.0.0 --port 8000
```

### Production Mode

```bash
# Using gunicorn with uvicorn workers
gunicorn api_wrapper:app \
  --workers 4 \
  --worker-class uvicorn.workers.UvicornWorker \
  --bind 0.0.0.0:8000
```

### Environment Variables

```bash
export LEAN_API_HOST=0.0.0.0
export LEAN_API_PORT=8000
export CORS_ORIGINS="http://localhost:3000,http://localhost:5173"
```

## API Endpoints

### Health Check

```bash
GET /health
```

Returns API health status and LEAN algorithm running state.

### Get Snapshot

```bash
GET /api/v1/snapshot
```

Returns complete system snapshot including:
- Portfolio metrics (net liq, buying power, etc.)
- Positions
- Orders
- Symbols with market data
- Risk status

### WebSocket Connection

```bash
WS /ws
```

Real-time WebSocket endpoint for receiving LEAN events. Connects to `ws://localhost:8000/ws`.

**Event Types:**
- `connected` - Connection confirmation
- `order_filled` - Order filled event
- `order_cancelled` - Order cancelled event
- `position_updated` - Position changed
- `symbol_updated` - Symbol market data updated
- `alert` - Alert/notification
- `snapshot` - Periodic snapshot update

**Message Format:**
```json
{
  "type": "order_filled",
  "data": {
    "order_id": "12345",
    "status": "FILLED",
    "fill_price": 509.18,
    "symbol": "SPY",
    "timestamp": "2025-11-18T10:00:00Z"
  },
  "timestamp": "2025-11-18T10:00:00Z"
}
```

**Example Client (JavaScript):**
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message.type, message.data);

  if (message.type === 'order_filled') {
    // Handle order filled event
    updateOrderStatus(message.data);
  } else if (message.type === 'position_updated') {
    // Handle position update
    updatePositions(message.data);
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket disconnected');
};
```

### Strategy Control

```bash
POST /api/v1/strategy/start
POST /api/v1/strategy/stop
```

Start or stop the LEAN strategy. Requires `confirm: true` in request body.

### Order Management

```bash
POST /api/v1/orders/cancel
```

Cancel a specific order. Requires `order_id` and `confirm: true` in request body.

### Combo Orders

```bash
POST /api/v1/combos/buy
POST /api/v1/combos/sell
```

Place combo orders (not yet implemented).

## Integration with LEAN

### Setting Algorithm Instance

The API wrapper needs a reference to the running LEAN algorithm instance. This is typically done by the LEAN launcher:

```python
from lean_integration.api_wrapper import set_algorithm
from lean_integration.box_spread_algorithm import BoxSpreadAlgorithm

# After LEAN algorithm is initialized
algorithm = BoxSpreadAlgorithm()
algorithm.Initialize()

# Set algorithm instance in API wrapper
set_algorithm(algorithm)

# Start API server
import uvicorn
uvicorn.run("api_wrapper:app", host="0.0.0.0", port=8000)
```

### LEAN Launcher Integration

The LEAN launcher should:
1. Initialize the LEAN algorithm
2. Call `set_algorithm()` to register the instance
3. Start the API wrapper server
4. Run LEAN algorithm in a separate thread/process

## API Documentation

FastAPI automatically generates OpenAPI documentation:

- **Swagger UI**: http://localhost:8000/docs
- **ReDoc**: http://localhost:8000/redoc

## Testing

### Manual Testing

```bash
# Health check
curl http://localhost:8000/health

# Get snapshot
curl http://localhost:8000/api/v1/snapshot

# Start strategy (requires LEAN running)
curl -X POST http://localhost:8000/api/v1/strategy/start \
  -H "Content-Type: application/json" \
  -d '{"confirm": true}'
```

### Unit Tests

```bash
# Run tests (when implemented)
pytest python/lean_integration/tests/
```

## Architecture

The API wrapper consists of four main components:

1. **`lean_client.py`**: Wraps LEAN algorithm instance, provides thread-safe access to Portfolio, Securities, positions, orders
2. **`api_models.py`**: Pydantic models matching API contract schema
3. **`api_converter.py`**: Converts LEAN data format to API contract format
4. **`api_wrapper.py`**: FastAPI application with REST endpoints

## Error Handling

The API wrapper handles:
- LEAN algorithm not running (503 Service Unavailable)
- Invalid requests (400 Bad Request)
- Order not found (404 Not Found)
- Internal errors (500 Internal Server Error)

## Security

### CORS Configuration

CORS is configured via environment variable `CORS_ORIGINS`. For production, restrict to specific origins:

```bash
export CORS_ORIGINS="https://yourdomain.com,https://app.yourdomain.com"
```

### Authentication (Future)

Authentication is not yet implemented. Future enhancements:
- API key authentication
- Token-based authentication
- OAuth2 for production

## Limitations

1. **Algorithm Instance Management**: Currently requires manual setting of algorithm instance. Future: Automatic detection via LEAN launcher integration.

2. **Strategy Start/Stop**: Placeholder implementations. Actual start/stop must be implemented via LEAN launcher.

3. **Combo Orders**: Not yet implemented. Requires LEAN ComboMarketOrder/ComboLimitOrder integration.

4. **Real-Time Updates**: REST API is polling-based. For real-time updates, use WebSocket bridge (T-51).

## Next Steps

- **T-51**: Implement WebSocket bridge for real-time LEAN events
- **T-52**: Integrate LEAN REST API wrapper with PWA/TUI
- Enhance error handling and logging
- Add authentication
- Implement combo order placement
- Add unit and integration tests

## References

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [LEAN Algorithm Structure](https://www.quantconnect.com/docs/v2/lean-engine/algorithm-framework/algorithm-structure)
- [API Contract](./agents/shared/API_CONTRACT.md)
- [LEAN REST API Wrapper Design](../docs/LEAN_REST_API_WRAPPER_DESIGN.md)
