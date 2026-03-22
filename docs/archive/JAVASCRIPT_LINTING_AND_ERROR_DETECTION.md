# JavaScript Linting and Silent Error Detection

**Date:** 2025-01-27
**Status:** ✅ Configured and Integrated

## Overview

This document describes the tools and strategies for linting JavaScript files and detecting silent errors in the codebase. We use multiple layers of validation to catch issues before they reach production.

## Tools and Strategies

### 1. ESLint - Static Code Analysis ✅

**Purpose:** Lint JavaScript and TypeScript files for code quality issues

**Configuration:** `web/eslint.config.js`

**Features:**

- ✅ Lints both `.js` and `.ts` files
- ✅ TypeScript-aware rules for JS files
- ✅ Catches common errors (undefined variables, unused vars, etc.)
- ✅ Enforces code style and best practices

**Usage:**

```bash
cd web
npm run lint          # Check for issues
npm run lint:fix      # Auto-fix issues
```

**What it catches:**

- Undefined variables (`no-undef`)
- Unused variables (`no-unused-vars`)
- Common mistakes (missing semicolons, etc.)
- Code style violations
- TypeScript errors (in TS files)

### 2. TypeScript Type Checking ✅

**Purpose:** Catch type errors and silent runtime issues

**Configuration:** `web/tsconfig.json`

**Features:**

- ✅ Type checking without emitting files (`--noEmit`)
- ✅ Catches type mismatches before runtime
- ✅ Finds null/undefined access issues
- ✅ Validates function signatures

**Usage:**

```bash
cd web
npm run type-check          # One-time check
npm run type-check:watch    # Watch mode
```

**What it catches:**

- Type mismatches
- Null/undefined access (`strictNullChecks`)
- Missing properties
- Incorrect function calls
- Type inference issues

**Integration:** Runs automatically in `./scripts/run_linters.sh`

### 3. Node.js Syntax Check ✅

**Purpose:** Validate JavaScript syntax before execution

**Tool:** `node --check`

**Features:**

- ✅ Fast syntax validation
- ✅ Catches syntax errors immediately
- ✅ Works on all `.js` files
- ✅ No dependencies required

**Usage:**

```bash

# Check single file

node --check path/to/file.js

# Check all JS files (via script)

./scripts/check_javascript.sh
```

**What it catches:**

- Syntax errors (missing brackets, etc.)
- Invalid JavaScript syntax
- Parse errors

**Integration:** Runs automatically in `./scripts/run_linters.sh`

### 4. Vitest - Runtime Testing ✅

**Purpose:** Catch runtime errors and logic bugs

**Configuration:** `web/vitest.config.ts`

**Features:**

- ✅ Unit and integration tests
- ✅ Test coverage reports
- ✅ Fast execution
- ✅ Watch mode for development

**Usage:**

```bash
cd web
npm test                 # Run tests once
npm run test:watch       # Watch mode
npm run test:coverage    # With coverage
```

**What it catches:**

- Runtime errors
- Logic bugs
- Integration issues
- Edge cases

## Silent Error Detection Strategy

### Layer 1: Static Analysis (ESLint)

- **When:** During development, before commit
- **Catches:** Code quality issues, undefined variables, unused code
- **Speed:** Fast (< 1 second)

### Layer 2: Type Checking (TypeScript)

- **When:** During development, before commit
- **Catches:** Type errors, null/undefined access, incorrect API usage
- **Speed:** Medium (2-5 seconds)

### Layer 3: Syntax Validation (Node.js --check)

- **When:** During development, before commit
- **Catches:** Syntax errors, parse failures
- **Speed:** Fast (< 1 second)

### Layer 4: Runtime Testing (Vitest)

- **When:** During development, CI/CD
- **Catches:** Runtime errors, logic bugs, integration issues
- **Speed:** Depends on test suite size

## Integration with Universal Linter

All checks run automatically via `./scripts/run_linters.sh`:

```bash
./scripts/run_linters.sh
```

**Execution Order:**

1. ESLint (JS/TS linting)
2. Stylelint (CSS linting)
3. TypeScript type check (`tsc --noEmit`)
4. JavaScript syntax check (`node --check`)

## Common Silent Errors Caught

### 1. Undefined Variables

```javascript
// ❌ Error caught by ESLint
function calculate() {
  return result; // 'result' is not defined
}
```

### 2. Type Mismatches

```typescript
// ❌ Error caught by TypeScript
function process(data: string) {
  return data.length;
}
process(123); // Type error: number not assignable to string
```

### 3. Null/Undefined Access

```typescript
// ❌ Error caught by TypeScript (strictNullChecks)
function getName(user: User | null) {
  return user.name; // Error: 'user' is possibly 'null'
}
```

### 4. Syntax Errors

```javascript
// ❌ Error caught by node --check
function test() {
  if (condition { // Missing closing parenthesis
    return true;
}
```

### 5. Runtime Logic Errors

```typescript
// ❌ Error caught by tests
function divide(a: number, b: number) {
  return a / b; // Fails test when b === 0
}
```

## VS Code Integration

### ESLint Extension

- Shows errors inline
- Auto-fixes on save (if configured)
- Real-time feedback

### TypeScript Extension

- Shows type errors inline
- IntelliSense support
- Quick fixes

### Recommended Settings

```json
{
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "typescript.tsdk": "node_modules/typescript/lib"
}
```

## CI/CD Integration

All checks should run in CI:

```yaml

# Example GitHub Actions
- name: Lint and type check
  run: |
    cd web
    npm run lint
    npm run type-check
    npm test
```

## Best Practices

1. **Run checks before committing:**

   ```bash
   ./scripts/run_linters.sh
   ```

2. **Fix type errors immediately:**
   - Type errors often indicate real bugs
   - Don't use `@ts-ignore` unless necessary

3. **Write tests for critical paths:**
   - Business logic
   - API integrations
   - Edge cases

4. **Use strict TypeScript:**
   - `strict: true` in `tsconfig.json`
   - Catches more errors at compile time

5. **Enable ESLint type-checked rules:**
   - Already configured in `eslint.config.js`
   - Catches errors ESLint alone can't find

## Summary

**Current Setup:**

- ✅ ESLint for JS/TS linting
- ✅ TypeScript type checking
- ✅ Node.js syntax validation
- ✅ Vitest for runtime testing
- ✅ All integrated into universal linter

**Coverage:**

- Static analysis: ✅
- Type checking: ✅
- Syntax validation: ✅
- Runtime testing: ✅

**Result:** Comprehensive error detection before code reaches production.

---

**Reference:**

- [ESLint Documentation](https://eslint.org/)
- [TypeScript Documentation](https://www.typescriptlang.org/)
- [Vitest Documentation](https://vitest.dev/)
