# Adding Sources to NotebookLM

## Quick Reference: Sources to Add

### 1. IBKR Troubleshooting Page (with embedded video)
**URL**: `https://www.interactivebrokers.com/campus/trading-lessons/diagnosing-issues-and-troubleshooting-with-the-tws-api/`

### 2. IBKR Market Data Page (with embedded video)
**URL**: `https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/`

### 3. Troubleshooting Learnings Document
**File**: `docs/TWS_API_TROUBLESHOOTING_LEARNINGS.md`

### 4. Market Data Learnings Document
**File**: `docs/TWS_API_MARKET_DATA_LEARNINGS.md`

### 5. Go TWS API Implementation (scmhub/ibapi)
**URL**: `https://github.com/scmhub/ibapi`

### 6. IB Gateway Docker (gnzsnz/ib-gateway-docker)
**URL**: `https://github.com/gnzsnz/ib-gateway-docker`

### 7. IBKR Docker (extrange/ibkr-docker)
**URL**: `https://github.com/extrange/ibkr-docker`

### 8. Docker Learnings Document
**File**: `docs/TWS_API_DOCKER_LEARNINGS.md`

---

## Step-by-Step Instructions

### Step 1: Open Your Notebook
1. Go to: https://notebooklm.google.com/notebook/d08f66c4-e5db-480a-bdc4-50682adc045e
2. Or search for: "TWS Automated Trading - Complete Resources"

### Step 2: Add IBKR Troubleshooting Page
1. Click **"Add source"** or the **"+"** button
2. Select **"Web URL"** or paste URL directly
3. Paste: `https://www.interactivebrokers.com/campus/trading-lessons/diagnosing-issues-and-troubleshooting-with-the-tws-api/`
4. Click **"Add"** or **"Submit"**
5. ✅ The page and embedded video will be indexed automatically

### Step 3: Add IBKR Market Data Page
1. Click **"Add source"** again
2. Select **"Web URL"**
3. Paste: `https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/`
4. Click **"Add"**
5. ✅ The page and embedded video will be indexed automatically

### Step 4: Add Troubleshooting Learnings Document
1. Click **"Add source"**
2. Select **"Upload file"** or **"Paste text"**
3. **Option A - Upload file:**
   - Click **"Choose file"**
   - Navigate to: `docs/TWS_API_TROUBLESHOOTING_LEARNINGS.md`
   - Select and upload
4. **Option B - Paste text:**
   - Open `docs/TWS_API_TROUBLESHOOTING_LEARNINGS.md` in your editor
   - Copy all content (Cmd+A, Cmd+C)
   - Paste into NotebookLM text box
5. Click **"Add"**

### Step 5: Add Market Data Learnings Document
1. Click **"Add source"**
2. Select **"Upload file"** or **"Paste text"**
3. **Option A - Upload file:**
   - Click **"Choose file"**
   - Navigate to: `docs/TWS_API_MARKET_DATA_LEARNINGS.md`
   - Select and upload
4. **Option B - Paste text:**
   - Open `docs/TWS_API_MARKET_DATA_LEARNINGS.md` in your editor
   - Copy all content (Cmd+A, Cmd+C)
   - Paste into NotebookLM text box
5. Click **"Add"**

### Step 6: Add Go TWS API Implementation
1. Click **"Add source"** again
2. Select **"Web URL"**
3. Paste: `https://github.com/scmhub/ibapi`
4. Click **"Add"**
5. ✅ The GitHub repository will be indexed automatically

### Step 7: Add IB Gateway Docker
1. Click **"Add source"** again
2. Select **"Web URL"**
3. Paste: `https://github.com/gnzsnz/ib-gateway-docker`
4. Click **"Add"**
5. ✅ The GitHub repository will be indexed automatically

### Step 8: Add IBKR Docker
1. Click **"Add source"** again
2. Select **"Web URL"**
3. Paste: `https://github.com/extrange/ibkr-docker`
4. Click **"Add"**
5. ✅ The GitHub repository will be indexed automatically

### Step 9: Add Docker Learnings Document
1. Click **"Add source"**
2. Select **"Upload file"** or **"Paste text"**
3. **Option A - Upload file:**
   - Click **"Choose file"**
   - Navigate to: `docs/TWS_API_DOCKER_LEARNINGS.md`
   - Select and upload
4. **Option B - Paste text:**
   - Open `docs/TWS_API_DOCKER_LEARNINGS.md` in your editor
   - Copy all content (Cmd+A, Cmd+C)
   - Paste into NotebookLM text box
5. Click **"Add"**

### Step 10: Wait for Processing
- NotebookLM will process all sources (may take 5-10 minutes for GitHub repos)
- You'll see processing indicators for each source
- Embedded videos will be indexed automatically
- GitHub repositories may take longer to process
- Once complete, all sources will appear in your notebook

---

## Verification

After adding, you can verify by asking NotebookLM:

1. **"What are the main troubleshooting topics covered in the IBKR troubleshooting guide?"**
2. **"What does the market data video demonstrate?"**
3. **"How does our implementation compare to IBKR's market data recommendations?"**
4. **"What are the key best practices for TWS API market data subscriptions?"**
5. **"How can we containerize IB Gateway using Docker?"**
6. **"What patterns can we learn from the Go TWS API implementation?"**
7. **"How does IBC (Interactive Brokers Controller) work for automated login?"**
8. **"What are the benefits of using Protocol Buffers with TWS API?"**

---

## Expected Results

Once indexed, NotebookLM will be able to answer questions about:

✅ **Troubleshooting Topics:**
- Connection issues and solutions
- Authentication problems
- Market data errors
- Order rejections
- Performance issues
- Error code interpretation

✅ **Market Data Topics:**
- Request methods (`reqMktData()`)
- Subscription types (real-time vs delayed)
- Rate limiting and line limits
- Generic tick types
- Snapshot vs streaming data
- Error handling
- Callback patterns

✅ **Implementation Comparison:**
- What we have vs what IBKR recommends
- Potential improvements
- Best practices alignment

✅ **Docker & Containerization:**
- Docker deployment patterns
- IBC (Interactive Brokers Controller) integration
- Headless operation with VNC/RDP
- SSH tunneling for remote access
- Secrets management
- Auto-restart and health monitoring

✅ **Alternative Implementations:**
- Go TWS API implementation patterns
- Protocol Buffers support
- Type safety approaches
- Modern API design patterns

---

## Troubleshooting

**If a URL doesn't index:**
- Try opening the URL in a browser first to ensure it's accessible
- Check if the page requires login (IBKR pages should be public)
- Wait a few minutes and try again

**If a file doesn't upload:**
- Check file size (NotebookLM has limits)
- Try pasting text instead of uploading
- Ensure the file is saved and accessible

**If video doesn't appear:**
- Videos are embedded in the page, so adding the page URL should include the video
- Wait for full processing (videos take longer to index)
- Check the page source to confirm video is embedded

---

## Quick Copy-Paste URLs

```
https://www.interactivebrokers.com/campus/trading-lessons/diagnosing-issues-and-troubleshooting-with-the-tws-api/
https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/
https://github.com/scmhub/ibapi
https://github.com/gnzsnz/ib-gateway-docker
https://github.com/extrange/ibkr-docker
```

---

**Last Updated**: 2025-01-XX
