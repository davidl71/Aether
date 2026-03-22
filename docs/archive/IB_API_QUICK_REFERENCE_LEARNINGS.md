# Learnings from IB API Quick Reference

## Reference

[Interactive Brokers C++ API Quick Reference](https://www.interactivebrokers.com/download/C++APIQuickReference.pdf)

## Key Connection Flow (Per Quick Reference)

According to the IB API Quick Reference, the connection flow is:

1. **eConnect()** → Establishes socket connection
2. **connectAck()** → Server version received, connection acknowledged
3. **managedAccounts()** → Account list received (indicates connection progressing)
4. **nextValidId()** → Connection fully established and ready

## Implementation Insights

### Connection Callbacks

**connectAck()** - Called when:

- Socket connection is established
- Server version is received
- TWS has accepted the connection at the socket level
- **Note:** This does NOT mean the connection is fully ready - you still need `nextValidId`

**managedAccounts()** - Called when:

- TWS sends the list of managed accounts
- This happens after `connectAck` but before `nextValidId`
- **Useful:** This is an early indicator that connection is progressing
- If you receive this, it means TWS has accepted the connection and is sending account info

**nextValidId()** - Called when:

- TWS sends the next valid order ID
- **This is the final confirmation** that connection is fully established
- Only after receiving this should you consider the connection "ready"

### TWS Configuration (Per Quick Reference)

The Quick Reference emphasizes:

1. **Enable API in TWS:**
   - Edit → Global Configuration → API → Settings
   - Enable "Enable ActiveX and Socket Clients"
   - Uncheck "Read-Only API" if placing orders

2. **Connection Prompt:**
   - TWS will show a popup: "A new API connection request from 127.0.0.1"
   - User must accept this for connection to proceed
   - This is why `nextValidId` might not arrive - TWS is waiting for user acceptance

3. **Client ID:**
   - Each connection needs a unique client ID
   - Conflicts cause connection failures
   - Default is usually 0 or 1

## Our Implementation

### What We've Implemented

✅ **connectAck()** - Logs connection acknowledgment, requests nextValidId
✅ **managedAccounts()** - Logs account list, indicates connection progress
✅ **nextValidId()** - Sets connected_ = true, starts health monitoring
✅ **Connection timeout handling** - With detailed diagnostics
✅ **Error detection** - Catches errors 502, 162, 200, etc.

### Connection Flow in Our Code

```cpp
1. eConnect() → socket connects
2. connectAck() → logs, requests reqIds(-1)
3. managedAccounts() → logs account list (if received)
4. nextValidId() → sets connected_ = true, starts health monitoring
```

### Why Timeout Might Occur

If `nextValidId` doesn't arrive, it's usually because:

1. **TWS waiting for user acceptance** (most common)
   - Check TWS window for connection prompt
   - Look for "A new API connection request from 127.0.0.1"

2. **API not enabled**
   - Edit → Global Configuration → API → Settings
   - Enable "Enable ActiveX and Socket Clients"

3. **Client ID conflict**
   - Another application using same client_id
   - Try different client_id

4. **Authentication required**
   - TWS might require additional authentication
   - Check TWS for prompts

## Best Practices from Quick Reference

1. **Always wait for nextValidId** before considering connection ready
2. **Handle managedAccounts** to get account information early
3. **Check TWS window** for connection prompts if timeout occurs
4. **Use unique client IDs** to avoid conflicts
5. **Enable API in TWS settings** before connecting

## References

- [IB API Quick Reference PDF](https://www.interactivebrokers.com/download/C++APIQuickReference.pdf)
- [IBKR Campus API Documentation](https://ibkrcampus.com/campus/ibkr-api-page/twsapi-doc/)
