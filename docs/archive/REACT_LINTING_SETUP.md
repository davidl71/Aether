# React Linting Setup with ESLint

**Date**: 2025-11-22
**Status**: ✅ Configured and Enhanced

---

## Overview

React linting is fully configured using **ESLint** with React-specific plugins. ESLint is the industry standard for JavaScript/TypeScript/React linting and provides excellent React support.

---

## Current Setup

### ✅ ESLint Configuration

**File**: `web/eslint.config.js` (ESLint 9 flat config)

**Plugins Configured**:

- `eslint-plugin-react` - React-specific linting rules
- `eslint-plugin-react-hooks` - React Hooks linting (Rules of Hooks)
- `eslint-plugin-react-refresh` - React Fast Refresh linting
- `@typescript-eslint/eslint-plugin` - TypeScript linting
- `@typescript-eslint/parser` - TypeScript parser

**Rules Enabled**:

- React recommended rules
- React Hooks recommended rules
- TypeScript recommended rules (type-checked)
- TypeScript stylistic rules
- React Refresh warnings

### ✅ Package.json Scripts

```json
{
  "lint": "eslint . --max-warnings=0",
  "lint:fix": "eslint . --fix"
}
```

**Usage**:

```bash
cd web
npm run lint        # Check for linting errors
npm run lint:fix    # Auto-fix linting errors
```

### ✅ VS Code Integration

**File**: `.vscode/settings.json`

ESLint is configured as the default formatter for:

- TypeScript (`.ts`)
- TypeScript React (`.tsx`)
- JavaScript (`.js`)
- JavaScript React (`.jsx`)

**Extension Required**: `dbaeumer.vscode-eslint`

---

## Universal Linter Integration

**File**: `scripts/run_linters.sh`

ESLint is now integrated into the universal linter script:

```bash
./scripts/run_linters.sh
```

This runs ESLint along with:

- C++ linting (cppcheck, clang-analyze, Infer)
- Python linting (bandit)
- Swift linting (swiftlint)
- **React/TypeScript linting (ESLint)** ← NEW

---

## Configuration Details

### React-Specific Rules

The configuration includes:

1. **React Recommended Rules**:
   - Component prop validation
   - JSX best practices
   - React version detection

2. **React Hooks Rules**:
   - Rules of Hooks enforcement
   - Hook dependency checking
   - Hook naming conventions

3. **React Refresh Rules**:
   - Fast Refresh compatibility
   - Component export validation

4. **TypeScript Rules**:
   - Type safety checks
   - Type-checked rules (requires tsconfig.json)
   - Stylistic type rules

### Ignored Files

The following are automatically ignored:

- `dist/**` - Build output
- `dev-dist/**` - Development build output
- `node_modules/**` - Dependencies
- `*.config.{js,ts}` - Config files
- `vitest.setup.ts` - Test setup

---

## Current Linting Status

### ✅ Working

- ESLint is configured and running
- React plugins are active
- TypeScript rules are enabled
- VS Code integration works
- Universal linter script includes ESLint

### ⚠️ Issues Found

Some linting errors exist in the codebase (these are code quality issues, not configuration problems):

**App.tsx**:

- Floating promises (lines 320, 326, 334)
- Prefer RegExp.exec() (line 386, 387)
- Prefer nullish coalescing (line 386)
- Unsafe any assignments (lines 445, 471)

**To Fix**:

```bash
cd web
npm run lint:fix  # Auto-fix some issues

# Manually fix remaining issues
```

---

## Benefits of ESLint for React

### ✅ Why ESLint is Perfect for React

1. **Industry Standard**: ESLint is the de-facto standard for JavaScript/TypeScript/React
2. **React-Specific Plugins**: Excellent React support via plugins
3. **TypeScript Integration**: Full TypeScript support with type-checked rules
4. **Auto-Fix**: Can automatically fix many issues
5. **VS Code Integration**: Real-time linting in editor
6. **CI/CD Ready**: Easy to integrate into CI/CD pipelines

### ✅ What ESLint Catches

- **React-Specific**:
  - Missing keys in lists
  - Incorrect prop types
  - Rules of Hooks violations
  - Unused React imports
  - JSX best practices

- **TypeScript**:
  - Type safety issues
  - Unsafe any usage
  - Missing type annotations
  - Type-checked rules

- **General**:
  - Unused variables
  - Code style issues
  - Potential bugs
  - Best practices

---

## Usage Examples

### Check for Linting Errors

```bash
cd web
npm run lint
```

### Auto-Fix Issues

```bash
cd web
npm run lint:fix
```

### Run All Linters (Including ESLint)

```bash
./scripts/run_linters.sh
```

### VS Code

ESLint runs automatically in VS Code when you:

- Open a `.tsx` or `.ts` file
- Save a file (if format on save is enabled)
- See red/yellow underlines for linting errors

---

## Configuration Files

### ESLint Config

**Location**: `web/eslint.config.js`

**Key Features**:

- ESLint 9 flat config format
- React plugins configured
- TypeScript type-checked rules
- Separate configs for source, tests, and config files

### VS Code Settings

**Location**: `.vscode/settings.json`

**Key Settings**:

- ESLint as formatter for TS/TSX/JS/JSX
- Format on save (if enabled)
- ESLint extension required

### Package.json

**Location**: `web/package.json`

**Scripts**:

- `lint` - Check for errors
- `lint:fix` - Auto-fix errors

**Dependencies**:

- `eslint` - Core ESLint
- `eslint-plugin-react` - React rules
- `eslint-plugin-react-hooks` - Hooks rules
- `eslint-plugin-react-refresh` - Refresh rules
- `@typescript-eslint/eslint-plugin` - TypeScript rules
- `@typescript-eslint/parser` - TypeScript parser

---

## Next Steps

### Fix Existing Linting Errors

1. Run `npm run lint:fix` to auto-fix some issues
2. Manually fix remaining issues in `App.tsx`:
   - Add `void` to floating promises
   - Use `RegExp.exec()` instead of `String.match()`
   - Use `??` instead of `||`
   - Fix unsafe `any` assignments

### Enhance Configuration (Optional)

You can add more React-specific rules:

```javascript
// In eslint.config.js
rules: {
  // ... existing rules ...
  'react/prop-types': 'error',  // Require prop types
  'react/jsx-key': 'error',      // Require keys in lists
  'react/no-unescaped-entities': 'error',  // Prevent unescaped entities
}
```

---

## Summary

✅ **React linting is fully set up with ESLint**

- ESLint is configured with React plugins
- TypeScript rules are enabled
- VS Code integration works
- Universal linter script includes ESLint
- Auto-fix available via `npm run lint:fix`

**ESLint is the perfect choice for React linting** - it's the industry standard, has excellent React support, and integrates seamlessly with TypeScript and modern tooling.

---

**Last Updated**: 2025-11-22
**Status**: ✅ Configured and Enhanced
