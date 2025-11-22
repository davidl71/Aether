# LiveVol Quoted Spreads - Quick Start

**TL;DR**: ⚠️ **POSSIBLY** - LiveVol may support quoted spreads, but needs verification during your trial.

---

## Quick Answer

**Can LiveVol get quoted spreads?**: ⚠️ **UNKNOWN - Needs Testing**

### Why It Might Work

1. ✅ **LiveVol is CBOE-owned** - Direct relationship with QSB provider
2. ✅ **API exists** - `https://api.livevol.com/v1`
3. ✅ **Strategy quotes mentioned** - API contract references "Cboe strategy quotes"
4. ✅ **Codebase has stub** - `LiveVolProvider` exists but not fully implemented

### What to Test This Week

1. **Sign up for LiveVol Pro trial** (15 days free)
   - URL: <https://datashop.cboe.com/livevol-pro>
   - Get API credentials (client ID, client secret)

2. **Run exploration script**

   ```bash
   python scripts/livevol_api_explorer.py \
     --client-id YOUR_CLIENT_ID \
     --client-secret YOUR_CLIENT_SECRET
   ```

3. **Check API docs for these endpoints**:
   - `/strategy/quotes`
   - `/complex/quotes`
   - `/qsb/quotes`
   - `/qsb/instruments`

4. **Test with SPX box spreads**:
   - Symbol: SPX
   - Strikes: 4000/5000
   - Check if quoted prices are available

---

## Expected Outcomes

### ✅ Best Case: LiveVol Supports Quoted Spreads

**If LiveVol API has quoted spread endpoints**:

- ✅ Direct access to quoted box spread prices
- ✅ Real-time QSB quotes (if subscribed)
- ✅ Integration via existing `LiveVolProvider` code
- ✅ Cost: $380/month subscription

**Next Steps**:

- Complete `LiveVolProvider` implementation
- Add quoted spread endpoints
- Integrate into box spread strategy

### ⚠️ Fallback: Build from Individual Legs

**If LiveVol doesn't support quoted spreads directly**:

- ⚠️ Use LiveVol for individual option quotes
- ⚠️ Build box spreads from 4 legs
- ⚠️ No direct QSB quotes
- ⚠️ Execution risk (partial fills)

**Next Steps**:

- Use LiveVol for options data enrichment
- Continue building box spreads from legs
- Look for other QSB quote sources

---

## This Week's Action Items

### Day 1-2: Setup & Initial Testing

1. ✅ Sign up for LiveVol Pro trial
2. ✅ Get API credentials
3. ✅ Run `livevol_api_explorer.py`
4. ✅ Review API documentation

### Day 3-4: Quoted Spread Testing

1. ✅ Test strategy/spread endpoints
2. ✅ Test QSB instrument queries
3. ✅ Test with SPX box spreads
4. ✅ Document findings

### Day 5-7: Integration Assessment

1. ✅ Evaluate integration complexity
2. ✅ Test data export capabilities
3. ✅ Document API rate limits
4. ✅ Create integration plan

---

## Key Files Created

1. **`docs/LIVEVOL_QUOTED_SPREADS_GUIDE.md`** - Complete guide
2. **`scripts/livevol_api_explorer.py`** - API exploration script
3. **`docs/CBOE_ONE_WEEK_EXPLORATION_PLAN.md`** - Full exploration plan

---

## Quick Test Command

Once you have LiveVol credentials:

```bash
# Install dependencies (if needed)
pip install requests

# Run exploration
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET \
  --output-dir docs/livevol_exploration
```

This will:

- ✅ Authenticate with LiveVol API
- ✅ Discover available endpoints
- ✅ Test quoted spread endpoints
- ✅ Test QSB instrument access
- ✅ Generate exploration report

---

## Questions to Answer

- [ ] Does LiveVol API have quoted spread endpoints?
- [ ] Can we get QSB instrument quotes?
- [ ] Is real-time data available in trial?
- [ ] What's the integration complexity?
- [ ] Is it worth $380/month subscription?

---

**See `docs/LIVEVOL_QUOTED_SPREADS_GUIDE.md` for complete details.**
