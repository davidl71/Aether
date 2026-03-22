# Troubleshooting Blank Page Issue

**Date:** 2025-11-22
**Issue:** Blank page when opening TypeScript dev server

---

## Root Cause

The PWA service worker was enabled in development mode (`devOptions.enabled: true`), which can cause caching issues and serve stale/broken versions of the app.

---

## Solution

### Option 1: Disable Service Worker in Dev (Recommended)

Service worker is now disabled in dev mode. To apply:

1. **Stop the dev server** (Ctrl+C in terminal)
2. **Clear browser service workers:**
   - Open DevTools (F12)
   - Go to **Application** tab
   - Click **Service Workers** in left sidebar
   - Click **Unregister** for any registered workers
3. **Clear site data:**
   - Still in **Application** tab
   - Click **Storage** in left sidebar
   - Click **Clear site data**
4. **Restart dev server:**

   ```bash
   cd web && npm run dev
   ```

5. **Hard refresh browser:**
   - Mac: `Cmd + Shift + R`
   - Windows/Linux: `Ctrl + Shift + R`

### Option 2: Manual Service Worker Cleanup

If Option 1 doesn't work:

1. **Open DevTools** (F12)
2. **Application** tab → **Service Workers**
3. **Unregister** all service workers
4. **Application** tab → **Storage** → **Clear site data**
5. **Hard refresh** the page

### Option 3: Check Browser Console

The blank page might be caused by JavaScript errors:

1. **Open DevTools** (F12)
2. **Console** tab
3. Look for red error messages
4. Common issues:
   - Import errors
   - API connection failures
   - Missing dependencies

---

## Verification

After applying the fix, you should see:

1. ✅ React app renders (header, tabs, content)
2. ✅ Browser console shows: `"Service Worker ready"` (if enabled)
3. ✅ NATS connection message: `"Connected to NATS at ws://localhost:8080"`
4. ✅ No red errors in console

---

## Prevention

- Service worker is now disabled in dev mode (`devOptions.enabled: false`)
- Service worker will only be active in production builds
- This prevents caching issues during development

---

## Additional Checks

If the page is still blank after clearing service workers:

1. **Check if backend is running:**

   ```bash
   curl http://localhost:8000/api/snapshot
   ```

   - If backend is not running, the app will show a loading state or error message (not blank)

2. **Check for JavaScript errors:**
   - Open DevTools → Console
   - Look for import errors, undefined variables, etc.

3. **Verify React is mounting:**
   - Open DevTools → Elements
   - Look for `<div id="root">` with content inside
   - If root is empty, React failed to mount

4. **Check network requests:**
   - Open DevTools → Network
   - Verify all JS/CSS files load successfully (status 200)
   - Check for failed requests (status 404, 500, etc.)

---

## Related Files

- `web/vite.config.ts` - PWA configuration
- `web/src/main.tsx` - React entry point
- `web/src/App.tsx` - Main app component
