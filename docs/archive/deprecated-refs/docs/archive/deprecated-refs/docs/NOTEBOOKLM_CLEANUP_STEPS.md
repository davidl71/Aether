# NotebookLM Cleanup Steps - Action Checklist

**Notebook URL**: <https://notebooklm.google.com/notebook/d08f66c4-e5db-480a-bdc4-50682adc045e>

## Pre-Cleanup: Identify Sources to Keep

### ✅ Keep These Sources (Already in Notebook)

- **8 YouTube Videos** - Keep all of them
- **1 Article** - VitalTrades socket implementation article

### ❌ Remove This Source

- **GitHub Repository** - Remove the entire repository source

### ➕ Add These Individual Documentation Files (After cleanup)

## Step-by-Step Cleanup Process

### Step 1: Remove GitHub Repository Source

1. **Open notebook in browser**:

   ```
   https://notebooklm.google.com/notebook/d08f66c4-e5db-480a-bdc4-50682adc045e
   ```

2. **Navigate to Sources panel**:
   - Click on **"Sources"** tab/panel on the left side
   - You should see the GitHub repository listed

3. **Remove the repository**:
   - Find the GitHub repository source (usually shows as "ib_box_spread_full_universal")
   - Click the **three dots (...)** or **menu icon** next to it
   - Select **"Remove source"** or **"Delete"**
   - Confirm the removal

4. **Verify removal**:
   - The repository should disappear from sources
   - Source count should drop significantly (from ~50 to ~9-10)

---

### Step 2: Add Essential Documentation Files

Click **"+ Add source"** and add each of these files individually using **"Website"** or **"URL"** option:

#### Priority 1: Core Documentation (Add these first)

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/README.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/ECLIENT_EWRAPPER_ARCHITECTURE.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_INTEGRATION_STATUS.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/API_DOCUMENTATION_INDEX.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_API_BEST_PRACTICES.md
```

#### Priority 2: Implementation Guides

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IMPLEMENTATION_GUIDE.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/CODEBASE_ARCHITECTURE.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/COMMON_PATTERNS.md
```

#### Priority 3: Additional Important Docs

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/EWRAPPER_BEST_PRACTICES.md
```

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IBC_LEARNINGS.md
```

#### Priority 4: Configuration Example

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/config/config.example.json
```

---

### Step 3: Verify Final Source List

After cleanup, you should have approximately **19-20 sources**:

**Videos**: 8 sources

- ✅ Essential Components of TWS API Programs
- ✅ Trading & Options Tutorial 1
- ✅ Trading & Options Tutorial 2
- ✅ Trading & Options Tutorial 3
- ✅ Trading & Options Tutorial 4
- ✅ Trading & Options Tutorial 5
- ✅ Trading & Options Tutorial 6
- ✅ Trading & Options Tutorial 7

**Articles**: 1 source

- ✅ VitalTrades C++ Socket Implementation article

**Documentation Files**: ~10-11 sources

- ✅ README.md
- ✅ ECLIENT_EWRAPPER_ARCHITECTURE.md
- ✅ TWS_INTEGRATION_STATUS.md
- ✅ API_DOCUMENTATION_INDEX.md
- ✅ TWS_API_BEST_PRACTICES.md
- ✅ IMPLEMENTATION_GUIDE.md
- ✅ CODEBASE_ARCHITECTURE.md
- ✅ COMMON_PATTERNS.md
- ✅ EWRAPPER_BEST_PRACTICES.md
- ✅ IBC_LEARNINGS.md
- ✅ config.example.json

---

## Quick Copy-Paste: All URLs in One Block

Copy this entire block and add sources one by one:

```
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/README.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/ECLIENT_EWRAPPER_ARCHITECTURE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_INTEGRATION_STATUS.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/API_DOCUMENTATION_INDEX.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_API_BEST_PRACTICES.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IMPLEMENTATION_GUIDE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/CODEBASE_ARCHITECTURE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/COMMON_PATTERNS.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/EWRAPPER_BEST_PRACTICES.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IBC_LEARNINGS.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/config/config.example.json
```

---

## After Cleanup: Update Documentation

Once you've completed the cleanup:

1. **Verify source count** - Should be ~19-20 sources
2. **Test a query** - Ask NotebookLM something to ensure knowledge is retained
3. **Let me know** - I'll update:
   - `docs/NOTEBOOKLM_RESOURCES_OPTIMIZED.md` with new source list
   - `docs/NOTEBOOKLM_STATUS.md` with updated status
   - `docs/notebooklm_resources.json` with new resource count

---

## Tips

- **Processing time**: Each file may take 1-2 minutes to process
- **Add in batches**: Add 3-5 files at a time, wait for processing, then add more
- **Check status**: Watch the processing indicator for each source
- **Test queries**: After adding files, test with: "What is the TWS API architecture?" to verify files are processed

---

## Troubleshooting

**If a URL doesn't work**:

- Make sure you're using the `raw.githubusercontent.com` URL (not regular github.com)
- Check that the file path is correct
- Try copying the URL directly from the browser when viewing the raw file on GitHub

**If files aren't processing**:

- Check that the files exist on GitHub
- Wait a few minutes - processing can take time
- Try refreshing the page

**If sources aren't showing up**:

- Check the Sources panel - they should appear even while processing
- Look for processing indicators (spinning icons)
- Check for any error messages

---

**Ready to start?** Open the notebook and begin with Step 1: Remove the GitHub repository source!
