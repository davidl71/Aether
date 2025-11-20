# LiveVol Pro Trial Setup Guide

**Date**: 2025-01-27
**Status**: ✅ You have LiveVol Pro trial access

---

## Step 1: Get API Credentials

### Where to Find Credentials

1. **Log into LiveVol Pro**
   - URL: <https://datashop.cboe.com/livevol-pro>
   - Sign in with your trial account

2. **Navigate to API Settings**
   - Look for "API" or "Developer" section
   - May be under: Settings → API → Credentials
   - Or: Developer → API Keys

3. **Get These Values**:
   - **Client ID** (OAuth 2.0 client ID)
   - **Client Secret** (OAuth 2.0 client secret)
   - **Base URL** (usually `https://api.livevol.com/v1`)

### Alternative: Check Documentation

- API Docs: <https://api.livevol.com/v1/docs/>
- May have authentication examples
- Look for OAuth 2.0 setup instructions

---

## Step 2: Configure Credentials

### Option A: Environment Variables (Recommended)

```bash
export LIVEVOL_CLIENT_ID="your_client_id_here"
export LIVEVOL_CLIENT_SECRET="your_client_secret_here"
```

### Option B: Config File

Add to `config/config.json`:

```json
{
  "livevol": {
    "base_url": "https://api.livevol.com/v1",
    "client_id": "your_client_id_here",
    "client_secret": "your_client_secret_here",
    "use_real_time": true
  }
}
```

### Option C: Command Line (For Testing)

Use directly in the exploration script (see Step 3).

---

## Step 3: Run Exploration Script

### Quick Test

```bash
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET \
  --output-dir docs/livevol_exploration
```

### With Environment Variables

```bash
python scripts/livevol_api_explorer.py \
  --client-id "$LIVEVOL_CLIENT_ID" \
  --client-secret "$LIVEVOL_CLIENT_SECRET"
```

---

## Step 4: Check Results

The script will generate:

1. **`docs/livevol_exploration/exploration_results.json`** - Raw results
2. **`docs/livevol_exploration/exploration_report_YYYYMMDD.md`** - Summary report
3. **`docs/livevol_exploration/quoted_spread_tests.json`** - Quoted spread test results
4. **`docs/livevol_exploration/qsb_tests.json`** - QSB instrument tests

### What to Look For

✅ **Success Indicators**:
- Authentication successful
- Endpoints discovered (especially `/strategy/*`, `/complex/*`, `/qsb/*`)
- Quoted spread tests return data

❌ **Failure Indicators**:
- Authentication fails (check credentials)
- All endpoints return 404 (may need different base URL)
- No quoted spread endpoints found

---

## Step 5: Manual API Testing

If the script doesn't find everything, test manually:

### Test Authentication

```bash
curl -X POST https://api.livevol.com/v1/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET"
```

### Test Strategy Endpoints

```bash
# Get access token first, then:
curl -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  https://api.livevol.com/v1/strategy/quotes?symbol=SPX
```

---

## Common Issues

### Issue 1: Authentication Fails

**Symptoms**: `401 Unauthorized` or `403 Forbidden`

**Solutions**:
- ✅ Verify client ID and secret are correct
- ✅ Check if credentials are for API (not just web login)
- ✅ Verify OAuth endpoint URL
- ✅ Check if trial includes API access

### Issue 2: No Endpoints Found

**Symptoms**: All endpoints return `404 Not Found`

**Solutions**:
- ✅ Check base URL (may be different)
- ✅ Review API documentation for correct endpoints
- ✅ Verify API access is enabled in your trial

### Issue 3: Rate Limiting

**Symptoms**: `429 Too Many Requests`

**Solutions**:
- ✅ Add delays between requests
- ✅ Check rate limits in API docs
- ✅ Use caching for repeated requests

---

## Next Steps After Exploration

### If Quoted Spreads Are Available

1. ✅ Document endpoint details
2. ✅ Create integration plan
3. ✅ Implement `LiveVolProvider` quoted spread methods
4. ✅ Test with real SPX box spreads

### If Quoted Spreads Are NOT Available

1. ✅ Use LiveVol for individual option quotes
2. ✅ Build box spreads from 4 legs
3. ✅ Continue searching for QSB quote sources
4. ✅ Consider LiveVol for data enrichment only

---

## Questions to Answer

- [ ] Do you have API credentials? (Client ID, Client Secret)
- [ ] Can you authenticate successfully?
- [ ] What endpoints are available?
- [ ] Are there quoted spread endpoints?
- [ ] Can you access QSB instruments?
- [ ] What data is available in the trial?

---

## Support Resources

- **LiveVol API Docs**: <https://api.livevol.com/v1/docs/>
- **LiveVol Support**: Contact through trial account
- **CBOE QSB FAQ**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>

---

**Ready to start? Get your API credentials and run the exploration script!**
