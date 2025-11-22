# TWS API Connection Test Guide

**Date**: 2025-11-13
**Purpose**: Test and verify TWS API connection with detailed diagnostics

## Overview

This guide helps you test the TWS API connection using the test programs we've created. The test tools provide detailed diagnostic information to help identify and resolve connection issues.

## Test Tools

### 1. C++ Test Program

**File**: `scripts/test_tws_connection.cpp`

A standalone C++ program that tests the TWS API connection with comprehensive diagnostics.

### 2. Shell Script

**File**: `scripts/test_tws_connection.sh`

A convenience script that:

- Checks if TWS/Gateway is running
- Verifies the port is listening
- Builds and runs the test program

## Quick Start

### Option 1: Using the Shell Script (Recommended)

```bash
# Basic usage (uses defaults: 127.0.0.1, port 4002, client_id 999)
./scripts/test_tws_connection.sh

# Custom host, port, and client ID
./scripts/test_tws_connection.sh 127.0.0.1 4002 999
```

### Option 2: Using the Main Application

```bash
# Run with debug logging
./build/ib_box_spread --config config/config.json --log-level debug
```

The enhanced diagnostic logging will show:

- Connection attempt details
- Socket state at each step
- Callback progression (connectAck → managedAccounts → nextValidId)
- EReader thread statistics
- Error details with troubleshooting guidance

## Enhanced Diagnostic Logging

The updated `tws_client.cpp` now includes:

### 1. Connection Attempt Logging

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Calling eConnect() for 127.0.0.1:7497 (client_id=1)...
  → Socket state before eConnect: disconnected
  → Async mode enabled: true
  → Signal timeout: 2000ms
eConnect() returned: true (took 5ms)
  → Socket state after eConnect: connected
```

### 2. EReader Thread Logging

```
Starting EReader thread...
EReader::start() completed in 2ms
EReader thread started (thread_id: 12345678)
EReader thread created (waiting for signals from TWS...)
```

### 3. Connection Progress Logging

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Connection Progress: [████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 66%
  Callbacks received: connectAck ✓, managedAccounts ✓, nextValidId ✗ (waiting)
  ✓ Step 1/3: connectAck received
  ✓ Step 2/3: managedAccounts received (150ms ago)
  ⏳ Step 3/3: Waiting for nextValidId...
```

### 4. Error Diagnostics

Enhanced error messages with actionable guidance:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Connection rejected by TWS/Gateway (error 502) for 127.0.0.1:7497
Error message: Couldn't connect to TWS.  Please confirm that "Enable ActiveX and Socket Clients" is enabled and connection port is the same as "Socket port" on the TWS "Edit->Global Configuration->API->Settings" menu.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Possible causes:
  1. API not enabled in TWS/Gateway
     → Go to: File → Global Configuration → API → Settings
     → Enable: 'Enable ActiveX and Socket Clients'
  2. IP address not trusted (127.0.0.1 should be trusted by default)
     → Check 'Trusted IPs' in API settings
  3. TWS/Gateway not fully started
     → Wait for TWS/Gateway to fully load before connecting
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Troubleshooting

### Issue 1: Port Not Listening

**Symptoms**:

```
✗ Port 7497 is not listening
```

**Solution**:

1. Start TWS or IB Gateway
2. Verify API is enabled in settings
3. Check the port number matches your configuration

### Issue 2: Connection Rejected (Error 502)

**Symptoms**:

```
Connection rejected by TWS/Gateway (error 502)
```

**Solution**:

1. Go to `File → Global Configuration → API → Settings`
2. Enable "Enable ActiveX and Socket Clients"
3. Verify port number matches
4. Check "Trusted IPs" includes `127.0.0.1`
5. Restart TWS/Gateway after changing settings

### Issue 3: Authentication Required (Error 162/200)

**Symptoms**:

```
Authentication required (error 162)
TWS/Gateway is waiting for you to accept the connection
```

**Solution**:

1. Check TWS/Gateway window for connection prompt
2. Click "Accept" or "OK" to allow the connection
3. If using IB Gateway, ensure it's not in read-only mode

### Issue 4: Connection Timeout

**Symptoms**:

```
Connection timeout after 60000ms (waiting for nextValidId from TWS)
```

**Possible Causes**:

1. TWS/Gateway waiting for connection approval
2. API not fully enabled
3. Client ID conflict (another application using same ID)
4. TWS/Gateway requires authentication

**Solution**:

- Check TWS/Gateway window for prompts
- Verify API settings
- Try a different client ID
- Review error messages for specific guidance

### Issue 5: Callbacks Not Received

**Symptoms**:

- `connectAck` received but `managedAccounts` never arrives
- `managedAccounts` received but `nextValidId` never arrives

**Possible Causes**:

1. EReader thread not running (should be logged)
2. TWS/Gateway waiting for approval
3. Connection stalled

**Solution**:

- Check logs for "EReader thread started"
- Verify TWS/Gateway is responsive
- Check for error messages

## What the Enhanced Logging Shows

### Connection Flow

The enhanced logging shows the complete connection flow:

1. **Port Detection**
   - Which ports are checked
   - Which ports are open
   - Port priority selection

2. **Connection Attempt**
   - `eConnect()` call details
   - Socket state before/after
   - Connection duration

3. **Reader Thread Start**
   - Thread creation timing
   - Thread ID
   - Initialization duration

4. **Callback Progression**
   - `connectAck` received
   - `managedAccounts` received
   - `nextValidId` received
   - Timing between callbacks

5. **Message Processing**
   - EReader thread activity
   - Message count statistics
   - Error count statistics

## Running the Test

### Step 1: Verify TWS/Gateway is Running

```bash
# Check if port is listening
lsof -i :7497

# Or for IB Gateway
lsof -i :4002
```

### Step 2: Run the Test

```bash
# Using shell script
./scripts/test_tws_connection.sh

# Or using main application with debug logging
./build/ib_box_spread --config config/config.json --log-level debug
```

### Step 3: Review Logs

Look for:

- ✅ Connection sequence completion
- ✅ All three callbacks received
- ✅ Next valid order ID received
- ✅ Connection stability (5-second test)

### Step 4: Verify Connection

The test will:

- Connect to TWS/Gateway
- Wait for all callbacks
- Verify connection stability
- Display connection statistics
- Disconnect cleanly

## Expected Output

### Successful Connection

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  TWS API Connection Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configuration:
  Host: 127.0.0.1
  Port: 7497
  Client ID: 999

Port Reference:
  7497 = TWS Paper Trading
  7496 = TWS Live Trading
  4002 = IB Gateway Paper Trading
  4001 = IB Gateway Live Trading

Creating TWS client...
✓ SUCCESS: Connected to TWS in 1250ms

Connection State: Connected
Next Valid Order ID: 1001

Connection is stable. Keeping connection alive for 5 seconds...
  5 seconds remaining...
  4 seconds remaining...
  3 seconds remaining...
  2 seconds remaining...
  1 seconds remaining...
✓ Connection remained stable for 5 seconds

Disconnecting...
✓ Disconnected successfully
```

## Next Steps

After successful connection test:

1. **Test Market Data**: Request market data for a symbol
2. **Test Orders**: Place a test order (paper trading)
3. **Test Positions**: Request current positions
4. **Monitor Stability**: Keep connection open and monitor logs

## References

- **TWS API Best Practices**: `docs/TWS_API_BEST_PRACTICES.md`
- **Implementation Comparison**: `docs/TWS_API_IMPLEMENTATION_COMPARISON.md`
- **Connection Test Script**: `scripts/test_tws_connection.cpp`
- **Connection Test Shell Script**: `scripts/test_tws_connection.sh`

---

**Last Updated**: 2025-11-13
**Status**: ✅ Enhanced logging and test scripts ready for use
