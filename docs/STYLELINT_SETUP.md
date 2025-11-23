# Stylelint Setup for CSS Linting

**Date:** 2025-01-27
**Status:** ✅ Configured and Integrated

## Overview

Stylelint is the industry-standard CSS linter, providing comprehensive linting for CSS, SCSS, and other CSS-like languages. It's been integrated into our web frontend linting workflow.

## Installation

```bash
cd web
npm install --save-dev stylelint stylelint-config-standard
```

## Configuration

**File:** `web/.stylelintrc.json`

**Key Settings:**
- Extends `stylelint-config-standard` for base rules
- Disabled overly strict rules for our codebase:
  - `rule-empty-line-before` - Allow rules without preceding empty lines
  - `declaration-block-no-redundant-longhand-properties` - Allow longhand properties
  - `shorthand-property-no-redundant-values` - Allow redundant shorthand values
  - `media-feature-range-notation` - Allow legacy media query syntax
  - `alpha-value-notation` - Allow decimal alpha values (0.4 vs 40%)
  - `color-function-notation` - Allow legacy color functions
  - `color-function-alias-notation` - Allow rgba() syntax
  - `font-family-name-quotes` - Allow quotes around font names

**Ignored Files:**
- `dist/**` - Build output
- `dev-dist/**` - Development build output
- `node_modules/**` - Dependencies

## Usage

### Check for Issues

```bash
cd web
npm run lint:css
```

### Auto-fix Issues

```bash
cd web
npm run lint:css:fix
```

### Integrated in Universal Linter

Stylelint runs automatically as part of `./scripts/run_linters.sh`:

```bash
./scripts/run_linters.sh
```

## Rules Enabled

The `stylelint-config-standard` preset includes:
- ✅ Syntax validation
- ✅ Property validation
- ✅ Value validation
- ✅ Best practices
- ✅ Accessibility checks
- ✅ Performance optimizations

## Custom Rules

**Enabled:**
- `color-hex-length: "short"` - Prefer short hex colors (#fff vs #ffffff)

**Disabled (for flexibility):**
- Formatting rules that conflict with our code style
- Modern CSS features that may not be needed
- Strict shorthand/longhand property rules

## Integration with ESLint

Stylelint runs alongside ESLint but is a separate tool:
- **ESLint:** TypeScript/React/JSON linting
- **Stylelint:** CSS linting
- **Both:** Run via `./scripts/run_linters.sh`

## VS Code Integration

Install the Stylelint extension:
```json
{
  "recommendations": [
    "stylelint.vscode-stylelint"
  ]
}
```

The extension will:
- Show linting errors inline
- Auto-fix on save (if configured)
- Format CSS files

## Future Enhancements

- [ ] Add SCSS support if needed
- [ ] Configure stricter rules gradually
- [ ] Add CSS custom properties validation
- [ ] Integrate with pre-commit hooks (optional)

---

**Reference:** [Stylelint Documentation](https://stylelint.io/)
