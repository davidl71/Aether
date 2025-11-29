# Markdownlint Setup

**Date**: 2025-11-30
**Status**: ✅ Configured

---

## Overview

`markdownlint-cli2` is configured to validate markdown formatting across the documentation. This helps identify and fix the 220 format errors detected by the exarp documentation health checker.

---

## Installation

```bash
# Install dependencies
npm install

# Or install globally
npm install -g markdownlint-cli2
```

---

## Usage

### Check Format Errors

```bash
# Check all markdown files in docs/
npm run lint:docs

# Or directly
npx markdownlint-cli2 "docs/**/*.md"
```

### Auto-Fix Format Errors

```bash
# Auto-fix issues where possible
npm run lint:docs:fix

# Or directly
npx markdownlint-cli2 --fix "docs/**/*.md"
```

### CI/CD Usage

```bash
# Use config file explicitly
npm run lint:docs:ci
```

---

## Configuration

### `.markdownlint.json`

Main configuration file with rules tailored for this project:

- **Line Length**: 100 characters (soft limit, relaxed for tables/code blocks)
- **Indentation**: 2 spaces
- **List Style**: Dashes (`-`)
- **Header Style**: ATX (`#`)
- **Blank Lines**: Maximum 1 consecutive blank line
- **Code Blocks**: Fenced style (`` ``` ``)

### `.markdownlintignore`

Files and patterns excluded from linting:

- Generated files (`node_modules/`, `build/`, `dist/`)
- Large generated documentation (`API_DOCUMENTATION_INDEX.md`)
- Summary/status files (different formatting standards)
- Task/analysis files (may have different formatting)

---

## Rules Enabled

Key rules enabled:

- **MD001**: Heading levels should only increment by one
- **MD003**: Heading style (ATX)
- **MD004**: Unordered list style (dash)
- **MD007**: Unordered list indentation (2 spaces)
- **MD009**: Trailing spaces (allows 2 spaces for line breaks)
- **MD012**: Multiple consecutive blank lines (max 1)
- **MD013**: Line length (100 chars, relaxed for tables/code)
- **MD022**: Headings should be surrounded by blank lines
- **MD024**: Multiple headings with same content (siblings only)
- **MD030**: Spaces after list markers
- **MD031**: Fenced code blocks should be surrounded by blank lines
- **MD032**: Lists should be surrounded by blank lines
- **MD046**: Code block style (fenced)
- **MD047**: Files should end with a single newline
- **MD048**: Code fence style (backtick)

---

## Integration with CI/CD

The `.github/workflows/docs-validation.yml` workflow can be updated to use markdownlint:

```yaml
- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version: '18'

- name: Install dependencies
  run: npm install

- name: Lint markdown files
  run: npm run lint:docs
```

---

## Comparison with Exarp Format Errors

The exarp documentation health checker reports **220 format errors**. Running markdownlint will help identify:

1. **Specific rule violations** (which rules are being broken)
2. **File locations** (which files have issues)
3. **Auto-fixable issues** (can be fixed automatically)

---

## Next Steps

1. ✅ **Install**: `npm install`
2. ⏳ **Run**: `npm run lint:docs` to see all format errors
3. ⏳ **Fix**: `npm run lint:docs:fix` to auto-fix issues
4. ⏳ **Verify**: Compare with exarp's 220 format errors
5. ⏳ **Update CI/CD**: Add markdownlint to docs-validation workflow

---

**Last Updated**: 2025-11-30
**Status**: ✅ Configured and ready to use
