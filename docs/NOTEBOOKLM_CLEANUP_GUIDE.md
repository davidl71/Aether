# NotebookLM Cleanup Guide

## Problem

The current notebook has ~50 sources because adding a GitHub repository processes every file individually.

## Solution Options

### Option A: Manual Cleanup in NotebookLM Browser (Recommended)

1. **Open your notebook**:
   - Go to: <https://notebooklm.google.com/notebook/d08f66c4-e5db-480a-bdc4-50682adc045e>

2. **Remove GitHub Repository source**:
   - Click on the **Sources** panel (left side)
   - Find the GitHub repository source
   - Click the **three dots** (...) menu next to it
   - Select **"Remove source"** or **"Delete"**
   - Confirm removal

3. **Add only essential documentation files**:
   Instead of adding the entire GitHub repo, add individual files:
   - `README.md`
   - `docs/ECLIENT_EWRAPPER_ARCHITECTURE.md`
   - `docs/TWS_INTEGRATION_STATUS.md`
   - `docs/API_DOCUMENTATION_INDEX.md`
   - `docs/TWS_API_BEST_PRACTICES.md`
   - And other key documentation files (see list below)

4. **Keep existing videos and article**:
   - The 8 YouTube videos
   - The VitalTrades article

### Option B: Create New Optimized Notebook

1. Create a new notebook in NotebookLM
2. Add only the essential sources (see Essential Sources list below)
3. Share the new notebook
4. Update documentation with new notebook URL

## Essential Documentation Files to Add

### Core Documentation (Priority 1)

- `README.md` - Main project overview
- `docs/ECLIENT_EWRAPPER_ARCHITECTURE.md` - TWS API architecture
- `docs/TWS_INTEGRATION_STATUS.md` - Integration status
- `docs/API_DOCUMENTATION_INDEX.md` - Complete API index
- `docs/TWS_API_BEST_PRACTICES.md` - Best practices

### Implementation Guides (Priority 2)

- `docs/IMPLEMENTATION_GUIDE.md` - Step-by-step implementation
- `docs/CODEBASE_ARCHITECTURE.md` - System architecture
- `docs/COMMON_PATTERNS.md` - Coding patterns

### Configuration & Setup (Priority 3)

- `docs/CURSOR_SETUP.md` - Development environment
- `docs/DISTRIBUTED_COMPILATION.md` - Build system
- `config/config.example.json` - Configuration example

### Learning Resources (Priority 4)

- `docs/IBC_LEARNINGS.md` - IBC automation tool
- `docs/TRADE_FRAME_LEARNINGS.md` - Trade-frame patterns
- Other learning documents as needed

## Optimal Source List (15-20 sources)

### Documentation Files (10-12 files)

- Key documentation from `docs/` directory
- README and main guides

### YouTube Videos (8 videos)

- Keep all existing videos

### Articles (1 article)

- Keep VitalTrades article

**Total**: ~20 sources (much cleaner than 50!)

## Manual Steps for Option A

### Step 1: Remove GitHub Repository

1. Open notebook in browser
2. Go to Sources panel
3. Find GitHub repo source
4. Remove it

### Step 2: Add Essential Files

1. Click "+ Add source"
2. Select "Upload files" or "Google Drive"
3. Add key documentation files one by one
4. Or use "Website" option and link to GitHub raw URLs:

   ```
   https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/README.md
   https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/ECLIENT_EWRAPPER_ARCHITECTURE.md
   ```

### Step 3: Verify

1. Check Sources panel shows ~20 sources
2. Verify all videos are still there
3. Verify article is still there
4. Test queries to ensure knowledge is retained

## GitHub Raw URLs for Key Files

If adding via URL instead of upload:

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
```

## After Cleanup

1. Update `docs/NOTEBOOKLM_RESOURCES_OPTIMIZED.md` with new source count
2. Update `docs/NOTEBOOKLM_STATUS.md` with cleanup status
3. Document which files were added
4. Test notebook queries to ensure functionality

## Notes

- **Processing time**: Adding files individually may take longer than adding the whole repo
- **Knowledge retention**: Removing the repo and re-adding files won't lose knowledge if you add the same files
- **File limits**: NotebookLM has limits (50 sources, 25M words), so we're well within limits
- **Quality over quantity**: Fewer, curated sources often work better than many auto-processed files

## Quick Decision Guide

**Choose Option A (Manual Cleanup) if**:

- You want to keep the same notebook URL
- You have time to manually curate sources
- You want to keep the current notebook history

**Choose Option B (New Notebook) if**:

- You want a fresh start
- You want to test with a smaller set first
- You don't mind having two notebooks temporarily

---

**Recommendation**: Option A (Manual Cleanup) - Remove the GitHub repo source and add only essential documentation files individually.
