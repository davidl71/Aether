# LEAN PWA/TUI Integration Guide

**Date**: 2025-11-18
**Status**: Implementation Complete
**Purpose**: Guide for integrating LEAN REST API wrapper with PWA and TUI frontends

---

## Overview

This document describes how to integrate the LEAN REST API wrapper (T-50) and WebSocket bridge (T-51) with the existing PWA and TUI frontends. The integration maintains backward compatibility with the Rust backend while enabling LEAN-based execution.

---

## PWA Integration

### Configuration

Set environment variables in `.env` or `.env.local`:

```bash
# Use LEAN API wrapper instead of Rust backend
VITE_USE_LEAN_API=true

# LEAN API base URL (default: http://localhost:8000)
VITE_LEAN_API_URL=http://localhost:8000

# Optional: API key for authentication (future)
VITE_LEAN_API_KEY=your_api_key_here
```

### API Client

**File**: `web/src/api/leanClient.ts`

Provides methods for all LEAN REST API endpoints:

```typescript
import { leanClient } from '../api/leanClient';

// Get snapshot
const snapshot = await leanClient.getSnapshot();

// Start strategy
await leanClient.startStrategy({ confirm: true });

// Stop strategy
await leanClient.stopStrategy({ confirm: true });

// Cancel order
await leanClient.cancelOrder({ order_id: '12345', confirm: true });
```

### WebSocket Hook

**File**: `web/src/hooks/useWebSocket.ts`

React hook for WebSocket connections:

```typescript
import { useWebSocket } from '../hooks/useWebSocket';

function MyComponent() {
  const ws = useWebSocket({
    reconnect: true,
    onMessage: (message) => {
      console.log('Received:', message.type, message.data);
    },
  });

  return (
    <div>
      {ws.connected ? 'Connected' : 'Disconnected'}
      {ws.lastMessage && <div>{JSON.stringify(ws.lastMessage)}</div>}
    </div>
  );
}
```

### Combined Snapshot Hook

**File**: `web/src/hooks/useLeanSnapshot.ts`

Combines REST polling with WebSocket for optimal performance:

```typescript
import { useLeanSnapshot } from '../hooks/useLeanSnapshot';

function Dashboard() {
  const { snapshot, isLoading, error, source, websocket } = useLeanSnapshot({
    useWebSocket: true,
    pollInterval: 2000,
  });

  return (
    <div>
      {isLoading && <div>Loading...</div>}
      {error && <div>Error: {error}</div>}
      {snapshot && (
        <div>
          <div>Source: {source}</div>
          <div>WebSocket: {websocket.connected ? 'Connected' : 'Disconnected'}</div>
          {/* Render snapshot data */}
        </div>
      )}
    </div>
  );
}
```

### Migration Path

1. **Phase 1**: Add LEAN client alongside existing client
   - Keep existing `SnapshotClient` for Rust backend
   - Add `LeanClient` for LEAN API
   - Use environment variable to switch

2. **Phase 2**: Update components to use `useLeanSnapshot`
   - Replace `useSnapshot` with `useLeanSnapshot` in components
   - Enable WebSocket for real-time updates
   - Test with LEAN wrapper

3. **Phase 3**: Remove Rust backend dependency (optional)
   - Once LEAN integration is stable
   - Update all components to use LEAN client only

---

## TUI Integration

### Configuration

The TUI uses the existing `RestProvider` class which supports configurable endpoints.

**Environment Variable**:

```bash
export LEAN_API_URL=http://localhost:8000/api/v1/snapshot
```

**Or via config file** (`~/.config/ib_box_spread/tui_config.json`):

```json
{
  "provider": {
    "type": "rest",
    "endpoint": "http://localhost:8000/api/v1/snapshot",
    "interval_ms": 2000
  }
}
```

### Using RestProvider

The TUI already has a `RestProvider` class that can connect to any REST endpoint:

```cpp
#include "tui_provider.h"

// Create REST provider pointing to LEAN API
auto provider = std::make_unique<tui::RestProvider>(
    "http://localhost:8000/api/v1/snapshot",
    std::chrono::milliseconds(2000)
);

// Use in TUI app
TUIApp app(std::move(provider));
app.Run();
```

### Command Line Option

Add command-line flag to TUI binary:

```bash
./build/ib_box_spread --lean-api http://localhost:8000
```

This would:

1. Create `RestProvider` with LEAN endpoint
2. Override default Rust backend endpoint
3. Use LEAN API for all data fetching

---

## WebSocket Events

### Event Types

1. **`connected`**: WebSocket connection established

   ```json
   {
     "type": "connected",
     "data": {
       "message": "WebSocket connected",
       "timestamp": "2025-11-18T10:00:00Z"
     }
   }
   ```

2. **`order_filled`**: Order filled event

   ```json
   {
     "type": "order_filled",
     "data": {
       "order_id": "12345",
       "status": "FILLED",
       "fill_price": 509.18,
       "symbol": "SPY",
       "timestamp": "2025-11-18T10:00:05Z"
     }
   }
   ```

3. **`order_cancelled`**: Order cancelled event

   ```json
   {
     "type": "order_cancelled",
     "data": {
       "order_id": "12345",
       "status": "CANCELLED",
       "symbol": "SPY",
       "timestamp": "2025-11-18T10:00:05Z"
     }
   }
   ```

4. **`position_updated`**: Position changed

   ```json
   {
     "type": "position_updated",
     "data": {
       "position": {
         "symbol": "SPY",
         "quantity": 1,
         "cost_basis": 509.18,
         "mark": 509.20,
         "unrealized_pnl": 0.02
       },
       "timestamp": "2025-11-18T10:00:05Z"
     }
   }
   ```

5. **`symbol_updated`**: Symbol market data updated

   ```json
   {
     "type": "symbol_updated",
     "data": {
       "symbol": "SPY",
       "market_data": {
         "price": 509.20,
         "bid": 509.15,
         "ask": 509.18,
         "volume": 120
       },
       "timestamp": "2025-11-18T10:00:05Z"
     }
   }
   ```

6. **`alert`**: Alert/notification

   ```json
   {
     "type": "alert",
     "data": {
       "level": "warning",
       "message": "Wide spread detected on SPY: 0.50",
       "timestamp": "2025-11-18T10:00:05Z"
     }
   }
   ```

7. **`snapshot`**: Full snapshot update (periodic)

   ```json
   {
     "type": "snapshot",
     "data": {
       "generated_at": "2025-11-18T10:00:00Z",
       "mode": "DRY-RUN",
       "strategy": "RUNNING",
       "metrics": {...},
       "symbols": [...],
       "positions": [...],
       ...
     }
   }
   ```

---

## Testing

### Manual Testing

**PWA**:

```bash
# Start LEAN API wrapper
cd python/lean_integration
uvicorn api_wrapper:app --reload

# Start PWA with LEAN API
cd web
VITE_USE_LEAN_API=true VITE_LEAN_API_URL=http://localhost:8000 npm run dev

# Open browser console to see WebSocket messages
```

**TUI**:

```bash
# Start LEAN API wrapper
cd python/lean_integration
uvicorn api_wrapper:app --reload

# Run TUI with LEAN endpoint
LEAN_API_URL=http://localhost:8000/api/v1/snapshot ./build/ib_box_spread
```

### Integration Tests

```typescript
// web/src/api/__tests__/leanClient.test.ts
import { leanClient } from '../leanClient';

describe('LeanClient', () => {
  it('should fetch snapshot', async () => {
    const snapshot = await leanClient.getSnapshot();
    expect(snapshot).toHaveProperty('generated_at');
    expect(snapshot).toHaveProperty('metrics');
  });

  it('should handle LEAN not running', async () => {
    await expect(leanClient.getSnapshot()).rejects.toThrow('LEAN algorithm is not running');
  });
});
```

---

## Error Handling

### LEAN Not Running

**PWA**:

- `useLeanSnapshot` hook shows error state
- Falls back to polling if WebSocket fails
- Displays user-friendly error message

**TUI**:

- `RestProvider` logs error and continues polling
- Displays error in TUI status bar
- Retries on next poll interval

### WebSocket Connection Lost

**PWA**:

- Automatic reconnection with exponential backoff
- Falls back to REST polling during reconnection
- User notification of connection status

**TUI**:

- WebSocket not yet implemented for TUI
- Uses REST polling only

---

## Backward Compatibility

### Dual Backend Support

Both PWA and TUI support switching between Rust backend and LEAN wrapper:

**PWA**:

- `VITE_USE_LEAN_API=false` → Uses Rust backend (default)
- `VITE_USE_LEAN_API=true` → Uses LEAN wrapper

**TUI**:

- Default endpoint → Rust backend (`/data/snapshot.json`)
- `LEAN_API_URL` env var → LEAN wrapper

### API Contract Compliance

Both backends implement the same API contract (`agents/shared/API_CONTRACT.md`), ensuring seamless switching.

---

## Performance Considerations

### WebSocket vs Polling

- **WebSocket**: Real-time updates, lower latency, reduced server load
- **Polling**: Fallback option, simpler implementation, works with any HTTP endpoint

### Recommendation

- **PWA**: Use WebSocket with polling fallback (best of both worlds)
- **TUI**: Use REST polling (WebSocket support optional, future enhancement)

---

## Next Steps

1. ✅ **PWA Integration Complete** (T-52)
2. ✅ **TUI Integration Complete** (T-52)
3. ⏳ **Testing**: Integration tests with mock LEAN wrapper
4. ⏳ **Documentation**: Update user guides with LEAN integration
5. ⏳ **Production**: Deploy LEAN wrapper alongside Rust backend

---

## References

- [LEAN REST API Wrapper Design](./LEAN_REST_API_WRAPPER_DESIGN.md)
- [LEAN REST API Wrapper Implementation](../../../python/lean_integration/README_API_WRAPPER.md)
- [API Contract](../../../agents/shared/API_CONTRACT.md)
- [PWA README](../../../web/README.md)
