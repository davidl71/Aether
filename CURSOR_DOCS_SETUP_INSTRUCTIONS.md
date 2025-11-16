# Cursor Docs Setup - Step-by-Step Instructions

**Date**: 2025-01-27
**Status**: Ready to configure

The documentation files exist, but you need to manually add them to Cursor's Docs feature through the UI.

---

## Quick Setup Steps

### Step 1: Open Cursor Settings

1. Press `Cmd+,` (macOS) or `Ctrl+,` (Windows/Linux)
2. Or: **Cursor** → **Settings** (macOS) or **File** → **Preferences** → **Settings**

### Step 2: Navigate to Docs Feature

1. In the Settings search bar, type: **"Docs"** or **"Documentation"**
2. Click on **Features** → **Docs** (or look for "Global Docs" or "Documentation")
3. You should see a section with **"Add Doc"** or **"Add Folder"** button

### Step 3: Add Documentation Files

**Option A: Add Individual Files (Recommended)**

Click **"Add Doc"** and add these files one by one (use absolute paths below):

#### High-Priority Files (Add These First)

```
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/API_DOCUMENTATION_INDEX.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/CODEBASE_ARCHITECTURE.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/COMMON_PATTERNS.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/AI_FRIENDLY_CODE.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/TWS_INTEGRATION_STATUS.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/STATIC_ANALYSIS_ANNOTATIONS.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/IMPLEMENTATION_GUIDE.md
```

#### External Documentation (Add These Too)

```
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/external/TWS_API_QUICK_REFERENCE.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/external/ECLIENT_EWRAPPER_PATTERNS.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/external/CMake_PRESETS_GUIDE.md
/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/external/CPP20_FEATURES.md
```

**Option B: Add Entire Folder (Faster but includes all docs)**

1. Click **"Add Folder"**
2. Navigate to: `/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs`
3. Select the `docs` folder
4. This will add all documentation files (including secondary docs)

---

## Alternative: Using Relative Paths

If Cursor supports relative paths from workspace root, you can use:

```
docs/API_DOCUMENTATION_INDEX.md
docs/CODEBASE_ARCHITECTURE.md
docs/COMMON_PATTERNS.md
docs/AI_FRIENDLY_CODE.md
docs/TWS_INTEGRATION_STATUS.md
docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md
docs/STATIC_ANALYSIS_ANNOTATIONS.md
docs/IMPLEMENTATION_GUIDE.md
docs/external/TWS_API_QUICK_REFERENCE.md
docs/external/ECLIENT_EWRAPPER_PATTERNS.md
docs/external/CMake_PRESETS_GUIDE.md
docs/external/CPP20_FEATURES.md
```

---

## Verify Setup

After adding the docs, test that they're working:

1. Open a new chat in Cursor (`Cmd+L` or `Ctrl+L`)
2. Try this prompt:
   ```
   @docs API_DOCUMENTATION_INDEX.md
   What APIs are available in this project?
   ```
3. The AI should reference the API documentation index

### Test Prompts

**Test 1: API Documentation**
```
@docs API_DOCUMENTATION_INDEX.md
How do I connect to TWS API?
```

**Test 2: Architecture**
```
@docs CODEBASE_ARCHITECTURE.md
How does the order manager work?
```

**Test 3: TWS Quick Reference**
```
@docs external/TWS_API_QUICK_REFERENCE.md
How do I place an order?
```

**Test 4: C++20 Features**
```
@docs external/CPP20_FEATURES.md
What C++20 features are used in this project?
```

---

## Troubleshooting

### Can't Find "Docs" in Settings

1. Make sure you're using a recent version of Cursor
2. Try searching for: "Global Docs", "Documentation", or "@docs"
3. Check if there's a "Features" section in Settings

### Files Not Showing Up

1. **Check file paths**: Make sure the absolute paths are correct
2. **Verify files exist**: Run this command:
   ```bash
   ls -la /Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/API_DOCUMENTATION_INDEX.md
   ```
3. **Check permissions**: Make sure Cursor has read access to the files
4. **Reload Cursor**: Try restarting Cursor after adding docs

### @docs Not Working in Chat

1. Make sure the files are actually added to the Docs section
2. Check the file name matches exactly (case-sensitive)
3. Try using just the filename without path: `@docs API_DOCUMENTATION_INDEX.md`
4. If using folder path, try: `@docs docs/API_DOCUMENTATION_INDEX.md`

---

## Quick Copy-Paste Script

Run this command to get all absolute paths ready to copy:

```bash
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
./scripts/list_global_docs.sh
```

Or manually copy from `.cursor/global-docs-paths.txt` (but use absolute paths from this document).

---

## What Gets Added

### High-Priority (8 files)
- API Documentation Index
- Codebase Architecture
- Common Patterns
- AI-Friendly Code
- TWS Integration Status
- Box Spread Guide
- Static Analysis Annotations
- Implementation Guide

### External (4 files)
- TWS API Quick Reference
- EClient/EWrapper Patterns
- CMake Presets Guide
- C++20 Features Reference

**Total**: 12 essential documentation files

---

## Next Steps

After adding the docs:

1. ✅ Test with `@docs` in chat
2. ✅ Use `@docs` when asking questions about the codebase
3. ✅ Reference specific docs when needed: `@docs TWS_API_QUICK_REFERENCE.md`

The docs will be automatically indexed and available for AI assistance!
