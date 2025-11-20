# TWS API Best Practices & Learnings

## Overview
This document captures best practices and patterns learned from analyzing TWS API implementations, including references to `JanBoonen/TwsApiCpp` and other C++ wrappers.

## Key Learnings

### 1. Async Connection Mode (`asyncEConnect`)

**Current State:** We use synchronous connection mode (default).

**Learning:** The TWS API supports async connection mode via `client_.asyncEConnect(true)`.

**Benefits:**
- Prevents blocking during connection handshake
- More responsive to errors
- Better for UI applications

**Implementation:**
```cpp
// In constructor or before eConnect()
client_.asyncEConnect(true);
```

**Consideration:** With async mode, we need to handle connection state changes via callbacks rather than blocking waits. Our current approach (starting reader thread before waiting) is already compatible with async mode.

### 2. EReader Thread Management

**Current State:** ✅ **FIXED** - We now start the reader thread BEFORE waiting for acknowledgment.

**Learning:** The EReader thread MUST be started before waiting for `nextValidId`, otherwise the connection will hang indefinitely.

**Best Practice:**
1. Call `eConnect()`
2. **Immediately start EReader thread** (if not in async mode, or after async connect)
3. Wait for `nextValidId` callback
4. Process messages via EReader thread

**Our Implementation:**
```cpp
// ✅ CORRECT - Start reader thread before waiting
start_reader_thread();
bool connection_acknowledged = wait_for_connection_with_progress(timeout);
```

### 3. Connection Error Handling

**Current State:** ✅ **IMPROVED** - We now detect error 502 and other connection errors.

**Learning:** Connection errors can arrive asynchronously via the `error()` callback, even after `eConnect()` returns true.

**Best Practice:**
- Check for errors immediately after `eConnect()`
- Monitor error callback during connection wait
- Handle specific error codes (502 = connection rejected, 1100 = connection lost)

**Our Implementation:**
```cpp
// Check for immediate errors
if (last_error_code_ == 502) {
    // Handle connection rejection
}

// Check during wait
if (last_error_code_ == 502) {
    return false; // Abort connection attempt
}
```

### 4. Port Detection & Fallback

**Current State:** ✅ **IMPLEMENTED** - Parallel port checking with priority-based selection.

**Learning:** TWS and IB Gateway use different ports for paper/live trading. Automatic detection improves user experience.

**Best Practice:**
- Check all standard ports in parallel
- Prioritize configured port, then TWS, then IB Gateway
- Detect paper/live mismatches and warn user
- Provide clear error messages

**Our Implementation:**
- ✅ Parallel port checking
- ✅ Priority-based selection
- ✅ Paper/live mismatch detection
- ✅ Clear error messages

### 5. Connection State Management

**Current State:** ✅ **GOOD** - We track connection state with atomic variables.

**Learning:** Connection state should be:
- Thread-safe (use atomics or mutexes)
- Observable (for UI/status reporting)
- Recoverable (auto-reconnect support)

**Best Practice:**
```cpp
enum class ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error
};

std::atomic<ConnectionState> state_;
std::atomic<bool> connected_;
```

### 6. Message Processing Pattern

**Current State:** ✅ **GOOD** - We use EReader thread for message processing.

**Learning:** The standard pattern is:
1. EReader thread reads from socket
2. EReader calls `processMsgs()` which decodes messages
3. Decoded messages trigger EWrapper callbacks
4. Callbacks update application state

**Best Practice:**
- Keep EReader thread alive while connected
- Handle exceptions in message processing
- Use condition variables for signaling

**Our Implementation:**
```cpp
reader_thread_ = std::make_unique<std::thread>([this, r = std::move(reader)]() mutable {
    while (connected_) {
        signal_.waitForSignal();
        if (!connected_) break;
        try {
            r->processMsgs();
        } catch (const std::exception& e) {
            spdlog::error("Error processing messages: {}", e.what());
        }
    }
});
```

### 7. Error Recovery & Reconnection

**Current State:** ✅ **PARTIAL** - We have auto-reconnect support but could improve.

**Learning:** Robust applications should:
- Detect connection loss (error 1100)
- Implement exponential backoff for reconnection
- Preserve pending orders/requests during reconnection
- Handle reconnection race conditions

**Potential Improvement:**
```cpp
void handle_reconnection() {
    if (!config_.auto_reconnect) return;

    int retry_count = 0;
    const int max_retries = 5;

    while (retry_count < max_retries && !connected_) {
        int delay_ms = std::min(1000 * (1 << retry_count), 30000); // Exponential backoff
        std::this_thread::sleep_for(std::chrono::milliseconds(delay_ms));

        if (connect()) {
            spdlog::info("Reconnected after {} attempts", retry_count + 1);
            return;
        }
        retry_count++;
    }
}
```

## Comparison with JanBoonen/TwsApiCpp

**Note:** The `JanBoonen/TwsApiCpp` repository appears to be inactive, but we can learn from common patterns:

1. **Simplified Interface:** Wrappers often provide higher-level abstractions
   - Our `TWSClient` class already does this ✅

2. **Error Handling:** Better error messages and recovery
   - We've improved this with error code tracking ✅

3. **Connection Management:** Automatic port detection and fallback
   - We've implemented this ✅

4. **Thread Safety:** Proper synchronization for multi-threaded access
   - We use mutexes and atomics ✅

## Recommendations

### Immediate Improvements (Already Done)
- ✅ Start EReader thread before waiting for connection
- ✅ Parallel port checking
- ✅ Error detection during connection
- ✅ Progress logging

### Future Enhancements

1. **Async Connection Mode:**
   - Consider enabling `asyncEConnect(true)` for non-blocking connections
   - Requires callback-based state management

2. **Better Reconnection Logic:**
   - Implement exponential backoff
   - Preserve state during reconnection
   - Handle partial reconnection scenarios

3. **Connection Health Monitoring:**
   - Periodic ping/heartbeat
   - Detect stale connections
   - Automatic recovery

4. **Enhanced Error Messages:**
   - More specific guidance for common errors
   - Link to troubleshooting documentation
   - Suggest configuration changes

## References

- **Official TWS API:** `native/third_party/tws-api/IBJts/source/cppclient/`
- **Alternative Wrapper:** `rudimeier/twsapi` (more actively maintained than JanBoonen's)
- **TWS API Documentation:** See `docs/API_DOCUMENTATION_INDEX.md`

## Conclusion

Our current implementation follows all best practices:
- ✅ Proper EReader thread management
- ✅ Parallel port detection
- ✅ Error handling and recovery
- ✅ Thread-safe state management
- ✅ **Async connection mode (asyncEConnect)** - IMPLEMENTED
- ✅ **Exponential backoff reconnection** - IMPLEMENTED
- ✅ **Connection health monitoring** - IMPLEMENTED

### Implementation Status (Updated)

**✅ Async Connection Mode:**
- Enabled `client_.asyncEConnect(true)` in constructor
- Non-blocking connection handshake
- Better responsiveness to errors

**✅ Exponential Backoff Reconnection:**
- Delays: 1s, 2s, 4s, 8s, 16s, max 30s
- Configurable max attempts (default: 10)
- Background reconnection to avoid blocking
- Automatic reset on successful connection

**✅ Connection Health Monitoring:**
- Checks connection every 30 seconds
- Detects stale connections (no messages for 2+ minutes)
- Verifies socket connectivity
- Automatic reconnection on detected failures
- Starts on successful connection, stops on disconnect
