# WebSocket Implementation Design

**Date**: 2025-11-17
**Status**: Design Document
**Purpose**: Design specification for adding WebSocket support for real-time updates to TUI and Web app

---

## Overview

Replace polling-based data updates with WebSocket connections for real-time updates in both TUI and Web app.

---

## Current State

**Web App:**
- Polls `/api/snapshot` every 2 seconds
- Uses `SnapshotClient` class with `setInterval`
- Location: `web/src/api/snapshot.ts`

**TUI:**
- Polls snapshot JSON file every 2 seconds
- Uses file system watcher or polling
- Location: `native/src/tui_provider.cpp`

**Backend:**
- FastAPI service (`alpaca_service.py`)
- Rust backend service (`agents/backend/`)
- No WebSocket endpoints yet

---

## Design Specification

### 1. Backend WebSocket Endpoint

**FastAPI Implementation:**
```python
from fastapi import WebSocket, WebSocketDisconnect

@app.websocket("/ws/snapshot")
async def websocket_snapshot(websocket: WebSocket):
    await websocket.accept()
    try:
        while True:
            # Get latest snapshot
            snapshot = build_snapshot_payload(...)
            await websocket.send_json(snapshot)
            await asyncio.sleep(2)  # Send updates every 2 seconds
    except WebSocketDisconnect:
        # Client disconnected
        pass
```

**Features:**
- Accept WebSocket connections
- Broadcast snapshot updates
- Handle client disconnections
- Heartbeat/ping to detect dead connections
- Connection management (multiple clients)

---

### 2. Web App WebSocket Client

**Implementation:**
```typescript
class WebSocketSnapshotClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;

  connect(endpoint: string) {
    this.ws = new WebSocket(endpoint);

    this.ws.onopen = () => {
      this.reconnectAttempts = 0;
      // Connection established
    };

    this.ws.onmessage = (event) => {
      const snapshot = JSON.parse(event.data);
      this.listeners.forEach(listener => listener(snapshot));
    };

    this.ws.onerror = (error) => {
      // Handle error, fallback to polling
      this.fallbackToPolling();
    };

    this.ws.onclose = () => {
      // Attempt reconnection
      this.attemptReconnect();
    };
  }

  private fallbackToPolling() {
    // Fall back to existing polling mechanism
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        this.reconnectAttempts++;
        this.connect(this.endpoint);
      }, 1000 * this.reconnectAttempts);
    } else {
      this.fallbackToPolling();
    }
  }
}
```

**Features:**
- Automatic reconnection
- Fallback to polling on failure
- Message buffering during disconnection
- Heartbeat handling

---

### 3. TUI WebSocket Client (C++)

**Implementation:**
```cpp
class WebSocketProvider : public Provider {
public:
  void Start() override {
    // Initialize WebSocket client
    // Connect to ws://localhost:8000/ws/snapshot
    // Start receive loop
  }

  Snapshot GetSnapshot() override {
    std::lock_guard<std::mutex> lock(snapshot_mutex_);
    return latest_snapshot_;
  }

private:
  void ReceiveLoop() {
    while (running_) {
      // Receive WebSocket message
      // Parse JSON
      // Update latest_snapshot_
    }
  }

  void Reconnect() {
    // Attempt reconnection with backoff
    // Fall back to file polling if fails
  }
};
```

**Libraries:**
- `websocketpp` (C++ WebSocket library)
- `nlohmann/json` (already used)
- Async I/O handling

**Features:**
- WebSocket client connection
- JSON message parsing
- Automatic reconnection
- Fallback to file polling

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Backend Service                       │
│  ┌──────────────────────────────────────────────────┐  │
│  │  FastAPI WebSocket Endpoint                       │  │
│  │  /ws/snapshot                                     │  │
│  │  - Broadcasts snapshot updates                    │  │
│  │  - Manages client connections                     │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
          │                    │
          │                    │
┌─────────▼──────────┐  ┌──────▼──────────────┐
│   Web App Client   │  │   TUI Client (C++)  │
│  - WebSocket API   │  │  - websocketpp      │
│  - Auto-reconnect  │  │  - Auto-reconnect   │
│  - Fallback        │  │  - Fallback         │
└────────────────────┘  └─────────────────────┘
```

---

## Implementation Steps

### Phase 1: Backend WebSocket Endpoint

1. Add WebSocket endpoint to FastAPI service
2. Implement connection management
3. Broadcast snapshot updates
4. Add heartbeat/ping
5. Test with WebSocket client

### Phase 2: Web App Integration

1. Create `WebSocketSnapshotClient` class
2. Replace polling with WebSocket
3. Implement reconnection logic
4. Add fallback to polling
5. Test connection/disconnection scenarios

### Phase 3: TUI Integration

1. Add WebSocket client library (websocketpp)
2. Create `WebSocketProvider` class
3. Implement connection and receive loop
4. Add reconnection logic
5. Add fallback to file polling
6. Test with backend

### Phase 4: Testing & Optimization

1. Test with multiple clients
2. Test reconnection scenarios
3. Test fallback mechanisms
4. Performance testing
5. Memory leak testing

---

## Benefits

**Real-Time Updates:**
- Instant data updates (no 2-second delay)
- Lower latency
- Better user experience

**Reduced Server Load:**
- No constant polling requests
- Efficient push-based updates
- Lower bandwidth usage

**Better Scalability:**
- Multiple clients can connect
- Efficient broadcast mechanism
- Connection pooling

---

## Fallback Strategy

**If WebSocket Fails:**
- Web App: Fall back to existing polling mechanism
- TUI: Fall back to file polling
- Automatic detection and switching
- User notification (optional)

**Connection Issues:**
- Automatic reconnection with exponential backoff
- Max reconnection attempts
- Graceful degradation

---

## Security Considerations

**WebSocket Security:**
- Use WSS (WebSocket Secure) in production
- Authentication tokens
- Rate limiting
- Connection limits

**CORS:**
- Configure CORS for WebSocket connections
- Allow specific origins only

---

## Performance Targets

- **Connection Time**: < 100ms
- **Message Latency**: < 50ms
- **Reconnection Time**: < 2s
- **Memory Usage**: Minimal overhead
- **CPU Usage**: < 5% per connection

---

## Files to Modify

**Backend:**
- `python/integration/alpaca_service.py` - Add WebSocket endpoint

**Web App:**
- `web/src/api/snapshot.ts` - Add WebSocket client
- `web/src/hooks/useSnapshot.ts` - Update to use WebSocket

**TUI:**
- `native/src/tui_provider.cpp` - Add WebSocket provider
- `native/CMakeLists.txt` - Add websocketpp dependency

---

## Success Criteria

- [ ] WebSocket endpoint implemented
- [ ] Web app uses WebSocket with fallback
- [ ] TUI uses WebSocket with fallback
- [ ] Automatic reconnection works
- [ ] Fallback to polling works
- [ ] Performance targets met
- [ ] Multiple clients supported
- [ ] Security measures in place

---

**Document Status**: ✅ Complete - Design specification ready for implementation
