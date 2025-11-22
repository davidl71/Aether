# Quick Setup: Adding Global Docs to Cursor

This is a step-by-step guide to add the recommended documentation files as global Docs in Cursor.

## Step 1: Open Cursor Settings

1. Open Cursor
2. Press `Cmd+,` (macOS) or `Ctrl+,` (Windows/Linux) to open Settings
3. Or: Click **Cursor** → **Settings** (macOS) or **File** → **Preferences** → **Settings** (Windows/Linux)

## Step 2: Navigate to Docs Feature

1. In Settings, search for "Docs" or "Documentation"
2. Click on **Features** → **Docs** (or similar)
3. You should see an option to **Add Doc** or **Add Folder**

## Step 3: Add High-Priority Files

Add these 8 files first (copy-paste the paths below):

### Must-Have Files (Add These First)

```
docs/API_DOCUMENTATION_INDEX.md
docs/CODEBASE_ARCHITECTURE.md
docs/COMMON_PATTERNS.md
docs/AI_FRIENDLY_CODE.md
docs/TWS_INTEGRATION_STATUS.md
docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md
docs/STATIC_ANALYSIS_ANNOTATIONS.md
docs/IMPLEMENTATION_GUIDE.md
```

**Quick Method**: If Cursor supports adding folders, you can add the entire `docs/` folder, but be aware this includes many files.

## Step 4: Add External Documentation (Optional but Recommended)

Add these external reference files:

```
docs/external/TWS_API_QUICK_REFERENCE.md
docs/external/ECLIENT_EWRAPPER_PATTERNS.md
docs/external/CMake_PRESETS_GUIDE.md
docs/external/CPP20_FEATURES.md
```

## Step 5: Verify Setup

Test that the global Docs are working:

1. Open a new chat in Cursor
2. Try this prompt:

   ```
   @docs API_DOCUMENTATION_INDEX.md
   What APIs are available in this project?
   ```

3. The AI should reference the API documentation index

## Alternative: Using File Paths

If Cursor requires absolute paths, use these (adjust for your system):

### macOS/Linux

```
/Users/davidlowes/ib_box_spread_full_universal/docs/API_DOCUMENTATION_INDEX.md
/Users/davidlowes/ib_box_spread_full_universal/docs/CODEBASE_ARCHITECTURE.md
/Users/davidlowes/ib_box_spread_full_universal/docs/COMMON_PATTERNS.md
/Users/davidlowes/ib_box_spread_full_universal/docs/AI_FRIENDLY_CODE.md
/Users/davidlowes/ib_box_spread_full_universal/docs/TWS_INTEGRATION_STATUS.md
/Users/davidlowes/ib_box_spread_full_universal/docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md
/Users/davidlowes/ib_box_spread_full_universal/docs/STATIC_ANALYSIS_ANNOTATIONS.md
/Users/davidlowes/ib_box_spread_full_universal/docs/IMPLEMENTATION_GUIDE.md
```

### External Docs (Absolute Paths)

```
/Users/davidlowes/ib_box_spread_full_universal/docs/external/TWS_API_QUICK_REFERENCE.md
/Users/davidlowes/ib_box_spread_full_universal/docs/external/ECLIENT_EWRAPPER_PATTERNS.md
/Users/davidlowes/ib_box_spread_full_universal/docs/external/CMake_PRESETS_GUIDE.md
/Users/davidlowes/ib_box_spread_full_universal/docs/external/CPP20_FEATURES.md
```

## Quick Test Prompts

After adding the docs, test with these prompts:

### Test 1: API Documentation

```
@docs API_DOCUMENTATION_INDEX.md
How do I connect to TWS API?
```

### Test 2: Architecture

```
@docs CODEBASE_ARCHITECTURE.md
How does the order manager work?
```

### Test 3: TWS API Quick Reference

```
@docs external/TWS_API_QUICK_REFERENCE.md
What's the EClient class structure?
```

### Test 4: Code Patterns

```
@docs COMMON_PATTERNS.md
How should I structure a new class?
```

## Troubleshooting

### Docs Not Found

- **Check paths**: Ensure you're using the correct relative or absolute paths
- **Check file exists**: Verify files exist at the specified paths
- **Restart Cursor**: Sometimes Cursor needs a restart to index new docs

### AI Not Using Docs

- **Be explicit**: Use `@docs` syntax explicitly in your prompts
- **Check syntax**: Ensure `@docs filename.md` format is correct
- **Multiple docs**: You can reference multiple: `@docs FILE1.md @docs FILE2.md`

### Can't Find Docs Setting

- **Cursor version**: Ensure you're using a recent version of Cursor
- **Search settings**: Try searching for "documentation" or "@docs" in settings
- **Check docs**: See `docs/CURSOR_DOCS_USAGE.md` for more details

## Checklist

Use this checklist to track your setup:

- [ ] Opened Cursor Settings
- [ ] Found Docs/Documentation feature
- [ ] Added `docs/API_DOCUMENTATION_INDEX.md`
- [ ] Added `docs/CODEBASE_ARCHITECTURE.md`
- [ ] Added `docs/COMMON_PATTERNS.md`
- [ ] Added `docs/AI_FRIENDLY_CODE.md`
- [ ] Added `docs/TWS_INTEGRATION_STATUS.md`
- [ ] Added `docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md`
- [ ] Added `docs/STATIC_ANALYSIS_ANNOTATIONS.md`
- [ ] Added `docs/IMPLEMENTATION_GUIDE.md`
- [ ] Added `docs/external/TWS_API_QUICK_REFERENCE.md` (optional)
- [ ] Added `docs/external/ECLIENT_EWRAPPER_PATTERNS.md` (optional)
- [ ] Added `docs/external/CMake_PRESETS_GUIDE.md` (optional)
- [ ] Added `docs/external/CPP20_FEATURES.md` (optional)
- [ ] Tested with `@docs` prompt
- [ ] Verified AI can reference the documentation

## Next Steps

After setting up global Docs:

1. **Start using `@docs`**: Reference docs in your prompts for better AI assistance
2. **Update as needed**: Add more docs as the project grows
3. **Share with team**: If working with others, share this setup guide

## Related Documentation

- **`docs/CURSOR_GLOBAL_DOCS.md`** - Complete guide on global Docs strategy
- **`docs/CURSOR_DOCS_USAGE.md`** - How to use `@docs` in prompts
- **`.cursorrules`** - Project rules that reference global Docs
