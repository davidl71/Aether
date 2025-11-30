# ESLint Migration Analysis

**Date:** 2025-01-27
**Purpose:** Analyze which linters can be migrated to ESLint for unified linting

## Current Linting Landscape

### Tools Currently in Use

| Tool | Purpose | Location | Can Migrate? |
|------|---------|----------|--------------|
| `check-json` | JSON validation | pre-commit hooks | ✅ **YES** |
| `check-yaml` | YAML validation | pre-commit hooks | ❌ **NO** |
| `yamllint` | YAML linting | Trunk | ❌ **NO** |
| `markdownlint` | Markdown linting | pre-commit + Trunk | ❌ **NO** |
| `eslint` | TypeScript/React | `web/` | ✅ **Already using** |
| None | HTML linting | - | ✅ **YES** |
| None | CSS linting | - | ⚠️ **Consider stylelint** |

## Migration Recommendations

### ✅ **JSON → ESLint** (Recommended)

**Current:** `check-json` in pre-commit hooks
**Migrate to:** `@eslint/json` plugin (official ESLint plugin)

**Benefits:**

- Unified configuration with TypeScript/React linting
- Better integration with VS Code ESLint extension
- Consistent error reporting format
- Can lint JSON files in `web/` directory (package.json, tsconfig.json, etc.)

**Implementation:**

```bash
npm install --save-dev @eslint/json
```

**Files to lint:**

- `web/package.json`
- `web/tsconfig.json`
- `web/tsconfig.node.json`
- `web/public/manifest.json`
- All JSON config files in `web/`

### ✅ **HTML → ESLint** (Recommended)

**Current:** No HTML linting
**Migrate to:** `eslint-plugin-html`

**Benefits:**

- Lint `web/index.html` for accessibility issues
- Catch common HTML errors (missing alt tags, invalid attributes, etc.)
- Unified linting workflow

**Implementation:**

```bash
npm install --save-dev eslint-plugin-html
```

**Files to lint:**

- `web/index.html`
- Any other HTML files in `web/`

### ❌ **YAML → ESLint** (Not Recommended)

**Current:** `check-yaml` + `yamllint`
**Keep:** Current tools

**Reason:**

- ESLint has no official YAML plugin
- `yamllint` is the industry standard for YAML
- YAML files are primarily config files (not in `web/` directory)
- Trunk already provides `yamllint` integration

**Files:**

- `global_kit/.pre-commit-config.yaml`
- `ib-gateway/root/conf.*.yaml`
- `.trunk/trunk.yaml`

### ❌ **Markdown → ESLint** (Not Recommended)

**Current:** `markdownlint` (pre-commit + Trunk)
**Keep:** Current tools

**Reason:**

- ESLint's `eslint-plugin-markdown` is limited (only lints code blocks)
- `markdownlint` is the industry standard for Markdown
- Better rule coverage for documentation
- Already integrated with pre-commit and Trunk

**Files:**

- All `docs/**/*.md` files
- `README.md`
- Other documentation

### ⚠️ **CSS → ESLint vs stylelint** (Consider stylelint)

**Current:** No CSS linting
**Options:**

1. **ESLint with `eslint-plugin-css`** - Limited CSS support
2. **stylelint** (recommended) - Industry standard, separate tool

**Recommendation:** Use **stylelint** (separate tool)

**Reason:**

- ESLint's CSS support is very limited
- `stylelint` is the industry standard for CSS/SCSS/SASS
- Better rule coverage and auto-fixing
- Can integrate with pre-commit hooks

**Implementation:**

```bash
npm install --save-dev stylelint stylelint-config-standard
```

**Files to lint:**

- `web/src/styles/app.css`
- Any future CSS/SCSS files

## Implementation Plan

### Phase 1: JSON + HTML (ESLint) ✅ **COMPLETED**

1. ✅ Install ESLint plugins:

   ```bash
   cd web
   npm install --save-dev @eslint/json eslint-plugin-html
   ```

2. ✅ Update `web/eslint.config.js`:
   - Added JSON file pattern with `language: 'json/json'`
   - Configured `@eslint/json` recommended rules
   - HTML plugin installed but not configured (flat config support issues)

3. ✅ Update `scripts/run_linters.sh`:
   - ESLint already runs and now includes JSON linting
   - No changes needed

4. ⚠️ Update pre-commit hooks (optional):
   - Can remove `check-json` from pre-commit (ESLint now handles it)
   - ESLint catches JSON errors (duplicate keys, empty keys, etc.)

**Status:** JSON linting is fully functional. HTML linting deferred until plugin supports flat config.

### Phase 2: CSS (stylelint) ⚠️

1. Install stylelint:

   ```bash
   cd web
   npm install --save-dev stylelint stylelint-config-standard
   ```

2. Create `web/.stylelintrc.json`:

   ```json
   {
     "extends": ["stylelint-config-standard"],
     "rules": {
       "color-hex-case": "lower"
     }
   }
   ```

3. Add to `web/package.json`:

   ```json
   {
     "scripts": {
       "lint:css": "stylelint 'src/**/*.css'"
     }
   }
   ```

4. Update `scripts/run_linters.sh`:
   - Add `run_stylelint()` function
   - Integrate into main linting workflow

## Summary

**Migrate to ESLint:**

- ✅ **JSON files** (`@eslint/json`) - **COMPLETED**
- ⚠️ **HTML files** (`eslint-plugin-html`) - Deferred (flat config support issues)

**Keep separate tools:**

- ❌ YAML (`yamllint` via Trunk)
- ❌ Markdown (`markdownlint` via pre-commit + Trunk)
- ⚠️ CSS (`stylelint` - recommended separate tool)

**Benefits of migration:**

- ✅ Unified linting for web frontend (TS/TSX/JSON)
- ✅ Better VS Code integration
- ✅ Consistent error reporting
- ✅ Simplified workflow

**Implementation Status:**

- ✅ JSON linting fully functional
- ✅ Detects duplicate keys, empty keys, unsafe values
- ✅ Lints package.json, tsconfig.json, and other JSON config files
- ✅ Generated files (package-lock.json) properly ignored

**Next Steps:**

- Consider removing `check-json` from pre-commit hooks (ESLint now handles it)
- Monitor `eslint-plugin-html` for flat config support
- Consider adding stylelint for CSS linting
