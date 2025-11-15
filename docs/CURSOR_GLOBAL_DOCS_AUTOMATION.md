# Automating Global Docs Updates

This guide explains how to automate the management and updating of global documentation files in Cursor IDE.

## Overview

The automation system consists of:
1. **Configuration file** (`.cursor/global-docs.json`) - Defines all global docs
2. **Validation scripts** - Check that all files exist
3. **Path generators** - Create files with paths for easy copy-paste
4. **Sync scripts** - Keep everything in sync

## Quick Start

### Validate All Docs

```bash
# Using Python script (recommended)
python3 scripts/sync_global_docs.py --check

# Using shell script
./scripts/update_global_docs.sh --check-only
```

### Generate Path Lists

```bash
# Generate both absolute and relative path files
python3 scripts/sync_global_docs.py --generate-paths

# Or use shell script
./scripts/update_global_docs.sh
```

### Detect New Documentation Files

```bash
# Find new .md files in docs/ that aren't in config
python3 scripts/sync_global_docs.py --detect-new
```

### Full Sync (Validate + Generate + Update Config)

```bash
python3 scripts/sync_global_docs.py --update-config --generate-paths
```

## Configuration File

The configuration file (`.cursor/global-docs.json`) defines three categories of documentation:

### High-Priority (Must-Have)
These are the essential docs that should always be in global Docs:
- API Documentation Index
- Codebase Architecture
- Common Patterns
- AI-Friendly Code
- TWS Integration Status
- Box Spread Guide
- Static Analysis Annotations
- Implementation Guide

### External Documentation
Third-party reference docs:
- TWS API Quick Reference
- EClient/EWrapper Patterns
- CMake Presets Guide
- C++20 Features

### Secondary Documentation
Additional useful docs:
- EWrapper Status
- Quick Start
- Distributed Compilation
- Cursor Setup

## Scripts

### Python Script (`scripts/sync_global_docs.py`)

**Features:**
- ✅ Validates all files exist
- ✅ Generates path lists (absolute and relative)
- ✅ Detects new documentation files
- ✅ Updates config timestamp
- ✅ Color-coded output
- ✅ Detailed error reporting

**Usage:**
```bash
# Check only
python3 scripts/sync_global_docs.py --check

# Generate paths
python3 scripts/sync_global_docs.py --generate-paths

# Detect new files
python3 scripts/sync_global_docs.py --detect-new

# Full sync
python3 scripts/sync_global_docs.py --update-config --generate-paths

# All options
python3 scripts/sync_global_docs.py --check --generate-paths --detect-new --update-config
```

### Shell Script (`scripts/update_global_docs.sh`)

**Features:**
- ✅ Validates all files exist
- ✅ Generates path lists
- ✅ Works without Python (uses jq if available)
- ✅ Color-coded output

**Usage:**
```bash
# Check only
./scripts/update_global_docs.sh --check-only

# Generate paths
./scripts/update_global_docs.sh

# With sync instructions
./scripts/update_global_docs.sh --sync
```

## Generated Files

After running the scripts, you'll find these files in `.cursor/`:

### `global-docs-paths.txt`
Absolute paths for all documentation files - ready to copy-paste into Cursor Settings.

### `global-docs-paths-relative.txt`
Relative paths (useful if Cursor supports relative paths).

## Automation Workflows

### Pre-Commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Validate global docs before committing
python3 scripts/sync_global_docs.py --check
if [ $? -ne 0 ]; then
  echo "Error: Some global docs are missing!"
  exit 1
fi
```

### CI/CD Integration

Add to your CI pipeline (GitHub Actions example):

```yaml
- name: Validate Global Docs
  run: |
    python3 scripts/sync_global_docs.py --check
    python3 scripts/sync_global_docs.py --generate-paths
```

### Makefile Target

Add to your `Makefile`:

```makefile
.PHONY: validate-docs sync-docs

validate-docs:
	python3 scripts/sync_global_docs.py --check

sync-docs:
	python3 scripts/sync_global_docs.py --update-config --generate-paths

docs: validate-docs sync-docs
	@echo "Global docs validated and synced"
```

### Periodic Updates

Add to your crontab or scheduled tasks:

```bash
# Run daily at 9 AM
0 9 * * * cd /path/to/project && python3 scripts/sync_global_docs.py --update-config --generate-paths
```

## Adding New Documentation

### Manual Process

1. Add the file to `.cursor/global-docs.json`:
   ```json
   {
     "path": "docs/NEW_DOC.md",
     "description": "Description of new doc",
     "category": "category-name"
   }
   ```

2. Run validation:
   ```bash
   python3 scripts/sync_global_docs.py --check
   ```

3. Generate updated paths:
   ```bash
   python3 scripts/sync_global_docs.py --generate-paths
   ```

### Automated Detection

Use the `--detect-new` flag to find new files:

```bash
python3 scripts/sync_global_docs.py --detect-new
```

This will list all `.md` files in `docs/` that aren't in the config yet.

## Updating Cursor Settings

Currently, Cursor doesn't have a CLI for adding global docs, so you need to:

1. Run the sync script to generate paths:
   ```bash
   python3 scripts/sync_global_docs.py --generate-paths
   ```

2. Open `.cursor/global-docs-paths.txt`

3. Copy the paths you want to add

4. In Cursor:
   - Open Settings (Cmd+,)
   - Navigate to Features → Docs
   - Click "Add Doc"
   - Paste the path

## Troubleshooting

### Files Not Found

If validation fails:
1. Check that files exist at the specified paths
2. Verify paths in `.cursor/global-docs.json` are correct
3. Run `python3 scripts/sync_global_docs.py --detect-new` to find actual files

### Config File Errors

If the JSON is invalid:
1. Validate JSON: `python3 -m json.tool .cursor/global-docs.json`
2. Check for missing commas or brackets
3. Restore from git if needed

### Script Errors

If scripts fail:
1. Check Python version: `python3 --version` (needs 3.6+)
2. For shell script, check if `jq` is installed: `brew install jq`
3. Check file permissions: `chmod +x scripts/*.sh`

## Best Practices

1. **Run validation before commits**: Ensure all docs exist
2. **Update config when adding docs**: Keep `.cursor/global-docs.json` current
3. **Generate paths regularly**: Keep path files up-to-date
4. **Use categories**: Organize docs by category in config
5. **Document new additions**: Add descriptions to config entries

## Future Enhancements

Potential future automation:
- Cursor CLI integration (when available)
- Auto-add to Cursor settings via API
- Git hook to validate on commit
- CI/CD integration for validation
- Automatic detection and suggestion of new docs

## Related Documentation

- **`docs/CURSOR_GLOBAL_DOCS.md`** - Complete guide on global Docs
- **`docs/CURSOR_GLOBAL_DOCS_SETUP.md`** - Setup instructions
- **`docs/CURSOR_DOCS_USAGE.md`** - How to use @docs in prompts
