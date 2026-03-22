# HTML Linting Plugin Monitoring

**Date:** 2025-01-27
**Status:** Deferred - Waiting for flat config support

## Current Situation

We have installed `eslint-plugin-html` but are not using it yet because:

1. **Flat Config Support:** ESLint 9 uses the new flat config format (`eslint.config.js`)
2. **Plugin Compatibility:** `eslint-plugin-html` has limited support for flat config
3. **Alternative Available:** ESLint now has official `@html-eslint/eslint-plugin` with better flat config support

## Options

### Option 1: Wait for `eslint-plugin-html` Flat Config Support

**Current Plugin:** `eslint-plugin-html@8.1.3`
**Status:** Installed but not configured
**Monitor:** Check for updates that add full flat config support

**When to Revisit:**

- Plugin version 9.0.0+ (if released)
- Official announcement of flat config support
- GitHub issues/PRs showing flat config work

### Option 2: Migrate to `@html-eslint/eslint-plugin`

**Official ESLint Plugin:** `@html-eslint/eslint-plugin`
**Status:** Better flat config support
**Action Required:** Replace `eslint-plugin-html` with `@html-eslint/eslint-plugin`

**Migration Steps:**

```bash
cd web
npm uninstall eslint-plugin-html
npm install --save-dev @html-eslint/eslint-plugin
```

**Configuration Example:**

```javascript
import html from '@html-eslint/eslint-plugin';

export default [
  {
    files: ['**/*.html'],
    plugins: { html },
    language: 'html/html',
    rules: {
      'html/no-duplicate-class': 'error',
      'html/require-lang': 'error',
      'html/require-img-alt': 'error'
    }
  }
];
```

## Recommendation

**Action:** Monitor `eslint-plugin-html` for 3 months, then evaluate:

- If no flat config support → Migrate to `@html-eslint/eslint-plugin`
- If flat config support added → Configure `eslint-plugin-html`

**Current Workaround:**

- HTML validation handled by browser/IDE
- VS Code HTML extension provides basic linting
- Manual review for accessibility issues

## Monitoring Checklist

- [ ] Check `eslint-plugin-html` releases monthly
- [ ] Monitor GitHub issues for flat config support
- [ ] Evaluate `@html-eslint/eslint-plugin` as alternative
- [ ] Review HTML linting needs (accessibility, semantic HTML, etc.)

## Next Review Date

**2025-04-27** (3 months from setup)

---

**Note:** HTML linting is lower priority than JSON/CSS since:

- HTML files are minimal (mostly `index.html`)
- Browser/IDE validation covers basic issues
- React components handle most HTML structure
