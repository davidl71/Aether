# How to Fix IBKR API Connection (No Prompt Issue)

## Problem Summary

Your IB Gateway Live is:
- ✅ Receiving the connection (connectAck)  
- ❌ **Silently rejecting it immediately** (no prompt, no dialog)
- ❌ Never sending `managedAccounts` or `nextValidId`

This means the Gateway has a security setting that **actively rejects** API connections instead of prompting you to accept them.

## Root Cause

The setting **"Accept incoming connection requests automatically"** is either:
1. Not checked (default behavior is REJECT, not prompt)
2. OR there's a whitelist/restriction preventing localhost connections

## Solution Steps

### Step 1: Open IB Gateway Settings

1. Look for IB Gateway in your dock or menu bar
2. Click the Gateway window to bring it to focus
3. In the menu bar, click: **Configure** → **Settings**
   - Or: **File** → **Global Configuration** → **API** → **Settings**
   - Or: Click the gear/wrench icon in the Gateway window

### Step 2: Navigate to API Settings

1. In the settings window, look for **"API"** in the left sidebar
2. Click **API** to expand it
3. Click **"Settings"** under API

### Step 3: Critical Settings to Change

In the API Settings dialog, you should see:

```
☑ Enable ActiveX and Socket Clients
   [This is probably already checked if you see it on port 4001]

☐ Accept incoming connection requests automatically  ← CHECK THIS!
   [This is the key setting that's causing the rejection]

Socket port: 4001  [Should match what you're using]

☑ Allow connections from localhost
   [Make sure this is checked]

Trusted IPs: 127.0.0.1
   [Add this explicitly]

Master API client ID: [Leave empty or set to 0]
   [This restricts which client IDs can connect]

☑ Create API message log file (optional, helpful for debugging)
```

### Step 4: Apply and Restart

1. Click **OK** to save settings
2. **Restart IB Gateway** (important!)
   - Close Gateway completely
   - Reopen it
   - Log in again

### Step 5: Verify

Run the diagnostic:
```bash
./scripts/diagnose_ibkr.sh
```

You should now see:
```
✓ connectAck received
✓ managedAccounts received: U1234567
✓ nextValidId received: 1
SUCCESS: Connection Fully Established!
```

## Alternative: Check Via GUI During Connection Attempt

If you can't find the settings:

1. Run the test:
   ```bash
   DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib \
     ./native/build_native/bin/test_simple_connect
   ```

2. **IMMEDIATELY** look at your IB Gateway window
   - Look for ANY popup, dialog, or notification
   - Check the status bar for messages
   - Look for a system notification

3. If you see NOTHING popup at all, that confirms the setting is blocking connections silently

## What We Know From Your Config

From `/Users/davidl/Jts/jts.ini`:
```ini
TrustedIPs=127.0.0.1  ← This IS set correctly!
ApiOnly=true          ← Gateway is in API mode
```

So the trusted IP is fine. The issue is the **acceptance policy**.

## If Settings Still Don't Work

If you've enabled auto-accept and it still fails:

### Option 1: Check Master Client ID

Some Gateway versions have a "Master Client ID" that ONLY allows that specific ID to connect. Try:
- Leave Master Client ID blank, OR
- Set it to 0 (allows all), OR  
- Set it to the specific ID you're testing (e.g., 777)

### Option 2: Try Read-Only API Mode

Enable:
```
☑ Read-Only API
```

This gives API access without trade permissions. Safer for testing.

### Option 3: Check for IP Restrictions

Look for a setting like:
```
"Allow connections from:" 
  ○ Anywhere
  ● Localhost only  ← Select this
  ○ Specific IPs
```

### Option 4: Check Gateway Permissions

Some corporate or institutional accounts have API disabled at the account level:
- Log into Account Management (https://www.interactivebrokers.com/portal)
- Check: Settings → API → Settings
- Make sure API access is enabled for your account

## Still Not Working?

Try these alternative approaches:

### Test with Paper Trading

The paper trading Gateway (port 4002 or 7497) often has less restrictive security:

1. Log into IB Gateway in **Paper Trading mode**
2. Run: `./native/build_native/bin/test_simple_connect`
3. If paper works but live doesn't, it's an account-level restriction

### Use Docker IB Gateway

The Dockerized Gateway comes pre-configured with API enabled:

```bash
cd ib-gateway
docker-compose up -d
```

Then connect to `localhost:4001` (or whatever port the Docker container exposes).

### Contact IB Support

If nothing works:
1. Call IBKR support: 1-877-442-2757
2. Say: "I need to enable API access for my account. My API connections are being rejected immediately without prompting for approval."
3. They can check if there are account-level restrictions

## Expected Behavior After Fix

Once properly configured, when you run the test:

1. **First connection**: May show a prompt asking to accept (Click "Accept")
2. **Subsequent connections**: Auto-accepted (no prompt)
3. **Test output shows**:
   ```
   ✓ connectAck received
   ✓ managedAccounts received: U1234567
   ✓ nextValidId received: 1
   ```

## Summary

The #1 most common cause of your exact symptoms:
- **"Accept incoming connection requests automatically" is UNCHECKED**

This causes Gateway to silently reject after initial handshake, exactly like you're seeing.

**Go enable that checkbox now!** 🎯
