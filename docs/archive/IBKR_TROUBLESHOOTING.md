# IBKR Connection Troubleshooting

## Current Issue

Connection establishes (`connectAck` received) but immediately closes before receiving `managedAccounts`.

## What's Working

✅ IB Gateway is running on port 4001  
✅ Socket connection succeeds  
✅ Initial handshake (`connectAck`) received  
❌ Connection closes before `managedAccounts` callback  

## Root Cause

IB Gateway is actively rejecting the connection after initial handshake. This happens when:

1. **API requires explicit approval but isn't prompting** (most likely)
2. **Trusted IPs not configured properly**
3. **Client ID restrictions**
4. **"Accept incoming requests automatically" is not enabled**

## Solution Steps

### Step 1: Check API Settings in IB Gateway

1. Open IB Gateway
2. Go to: **Configure** → **Settings** → **API** → **Settings**
3. Verify these settings:

```
☑ Enable ActiveX and Socket Clients
☑ Allow connections from localhost
☐ Read-Only API (optional, safer for testing)
☑ Let API applications initiate trades (if you need write access)
```

### Step 2: Enable Auto-Accept (Critical for Live)

In the same API Settings dialog:

```
☑ Create API message log file (helpful for debugging)
☑ Include market data in API log file
☑ Accept incoming connection requests automatically  ← THIS IS KEY
```

**Without "Accept incoming connection requests automatically", IB Gateway will:**

- Accept the socket connection
- Send `connectAck`
- Wait for manual approval (popup dialog)
- Close connection if not approved within ~1 second

### Step 3: Configure Trusted IPs

Still in API Settings:

```
Trusted IPs: 127.0.0.1
```

This explicitly allows localhost connections.

### Step 4: Master Client ID

Some IB Gateway versions have a "Master Client ID" setting:

```
Master Client ID: [leave empty or set to 0]
```

This prevents specific client ID restrictions.

### Step 5: Socket Port

Verify:

```
Socket port: 4001  (Live Trading)
```

## Testing After Configuration

### Test 1: Simple Connection

```bash
DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib \\
  ./native/build_native/bin/test_simple_connect
```

Expected output:

```
✓ Connected successfully with client ID 100
Requesting positions...
Result: Found X positions
```

### Test 2: Full Position Retrieval

```bash
DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib \\
  ./native/build_native/bin/test_positions_live
```

Expected output:

```
✓ Connected successfully
=== Position Summary ===
Total positions: X
```

## Common Errors

### Error: "Connection closed by TWS"

**Symptom**: Gets `connectAck` but closes immediately  
**Solution**: Enable "Accept incoming connection requests automatically"

### Error: "No open ports found"

**Symptom**: Can't connect at all  
**Solution**: Start IB Gateway, check "Enable ActiveX and Socket Clients"

### Error: Code 509 "Exception caught while reading socket"

**Symptom**: Connection times out  
**Solution**: Check Trusted IPs includes 127.0.0.1

### Error: Code 1100 "Connectivity between IB and TWS has been lost"

**Symptom**: Connection drops after working  
**Solution**: Check network stability, firewall settings

## Alternative: Docker IB Gateway

If manual configuration doesn't work, consider using the containerized version:

```bash
cd ib-gateway
docker-compose up -d
```

The Docker version is pre-configured with API enabled and auto-accept turned on.

## Verification Commands

### Check if IB Gateway is running

```bash
ps aux | grep -i ibgateway | grep -v grep
```

### Check if port 4001 is listening

```bash
lsof -i :4001 | grep LISTEN
```

### Check existing connections

```bash
lsof -i :4001 | grep ESTABLISHED
```

### Check IB Gateway logs

```bash
# Location varies by system, typically:
# macOS: ~/Jts/ibgateway/logs/
# Look for files with today's date
ls -lt ~/Jts/ibgateway/*/logs/ | head -20
```

## Success Indicators

When properly configured, you should see:

1. **In test output**:

   ```
   ✓ connectAck received
   ✓ managedAccounts received: U1234567
   ✓ nextValidId received: 1
   ✓ Connected successfully
   ```

2. **In IB Gateway**:
   - No popup dialogs asking to accept connection
   - Connection listed in active API connections
   - No error messages in status bar

3. **In Gateway logs**:

   ```
   Incoming connection from 127.0.0.1 accepted automatically
   Client 100 connected successfully
   ```

## Current Test Status

**Built Test Programs**:

- ✅ `test_tws_connection` - Basic connection test (native/build_native/bin/test_tws_connection:1)
- ✅ `test_positions_live` - Position retrieval test (native/build_native/bin/test_positions_live:1)  
- ✅ `test_simple_connect` - Diagnostic connection test (native/build_native/bin/test_simple_connect:1)

**Next Steps**:

1. Enable "Accept incoming connection requests automatically" in IB Gateway
2. Add 127.0.0.1 to Trusted IPs
3. Re-run `test_simple_connect`
4. If successful, run `test_positions_live`
