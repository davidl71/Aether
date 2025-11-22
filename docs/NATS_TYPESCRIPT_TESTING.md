# TypeScript NATS Integration Testing Guide

**Date:** 2025-11-20
**Status:** Ready for Testing

---

## Prerequisites

1. **NATS Server Running**
   ```bash
   ./scripts/start_nats.sh
   ```

2. **NATS WebSocket Enabled**
   - NATS server must have WebSocket support enabled
   - Default WebSocket port: 8080
   - Check: `curl http://localhost:8222/varz | grep -i websocket`

3. **TypeScript Dependencies Installed**
   ```bash
   cd web && npm install
   # Should show: nats.ws@1.30.3
   ```

---

## Testing Steps

### Step 1: Verify NATS Server WebSocket Support

```bash
# Check NATS server config
cat config/nats-server.conf | grep -i websocket

# Check if WebSocket port is listening
lsof -i :8080 | grep LISTEN
```

**Expected:** WebSocket port 8080 should be listening

**If not enabled:** Add to `config/nats-server.conf`:
```conf
websocket {
  port: 8080
  no_tls: true
}
```

### Step 2: Start TypeScript Dev Server

```bash
cd web
npm run dev
```

**Expected:** Dev server starts on `http://localhost:5173` (or similar)

### Step 3: Open Browser and Check Console

1. Open browser to dev server URL
2. Open Developer Tools (F12)
3. Check Console tab for NATS connection messages

**Expected Console Output:**
```
Connected to NATS at ws://localhost:8080
Subscribed to market-data.tick.>
```

### Step 4: Verify NATS Status Badge

1. Look at the header of the application
2. Find the "NATS" status badge
3. Should show:
   - ✅ Green badge if connected
   - ⚠️ Yellow/warn badge if not connected

### Step 5: Test Message Reception

**Terminal 1 - Publish test message:**
```bash
echo '{"id":"test","timestamp":"2025-11-20T00:00:00Z","source":"test","type":"MarketDataTick","payload":{"symbol":"SPY","bid":100.0,"ask":100.1,"timestamp":"2025-11-20T00:00:00Z"}}' | \
  nats pub "market-data.tick.SPY" --stdin
```

**Browser Console:**
- Should see message received
- Check for any errors

---

## Troubleshooting

### NATS Connection Fails

**Error:** `Failed to connect to NATS: ...`

**Solutions:**
1. Check NATS server is running: `curl http://localhost:8222/healthz`
2. Check WebSocket port: `lsof -i :8080`
3. Verify NATS server has WebSocket enabled
4. Check browser console for specific error

### WebSocket Port Not Available

**Error:** Connection refused on port 8080

**Solution:** Enable WebSocket in NATS server config:
```conf
websocket {
  port: 8080
  no_tls: true
}
```

Then restart NATS:
```bash
./scripts/stop_nats.sh
./scripts/start_nats.sh
```

### NATS Status Badge Not Showing

**Check:**
1. Is `useNATS` hook imported in HeaderStatus?
2. Is the badge rendered in the JSX?
3. Check browser console for React errors

### Messages Not Received

**Check:**
1. Is subscription enabled? (Currently set to `false` in HeaderStatus)
2. Check topic name matches publisher
3. Verify message format matches expected structure
4. Check browser console for subscription errors

---

## Expected Behavior

### On Page Load
- NATS hook auto-connects (`autoConnect: true`)
- Connection attempt logged to console
- NATS status badge updates based on connection status

### When Connected
- Badge shows green/ok status
- Console shows: "Connected to NATS at ws://localhost:8080"
- Ready to receive messages (if subscriptions enabled)

### When Disconnected
- Badge shows yellow/warn status
- Console shows error message
- Graceful degradation (app continues to work)

---

## Next Steps After Testing

1. **Enable Subscriptions** - Change `subscribeMarketData: true` in HeaderStatus
2. **Test Real-Time Updates** - Publish messages and verify UI updates
3. **Test Error Handling** - Stop NATS and verify graceful degradation
4. **Performance Testing** - Test with high message volume

---

## Test Checklist

- [ ] NATS server running
- [ ] WebSocket port 8080 available
- [ ] TypeScript dev server starts
- [ ] Browser console shows NATS connection
- [ ] NATS status badge appears in header
- [ ] Badge shows correct status (connected/not connected)
- [ ] No console errors
- [ ] Graceful degradation when NATS unavailable

---

## Notes

- **Current Configuration:** Subscriptions disabled in HeaderStatus (connection status only)
- **WebSocket URL:** `ws://localhost:8080` (default for nats.ws)
- **Auto-Connect:** Enabled by default in `useNATS` hook
- **Error Handling:** Graceful degradation implemented
