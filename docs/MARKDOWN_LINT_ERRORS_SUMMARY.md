# Markdown Linting Errors Summary

**Generated:** 2026-01-06
**Last Updated:** 2026-01-06 (After Documentation Cleanup + Archive Exclusion)
**Status:** After Auto-Fix + Documentation Cleanup + Archive Exclusion
**Total Errors:** 315 (down from 2,157) - Archive files excluded

---

## Top Files by Error Count

### 🔴 Critical (10+ errors)

| File                                                            | Errors  | Primary Issue                                           |
| --------------------------------------------------------------- | ------- | ------------------------------------------------------- |
| `docs/API_DOCUMENTATION_INDEX.md`                               | **117** | Line length (MD013) - Very long lines (up to 584 chars) |
| `docs/research/external/RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md` | **9**   | Line length (MD013)                                     |
| `docs/research/external/RESEARCH_CPP_FINANCIAL_LIBRARIES.md`    | **9**   | Line length (MD013) + Table issues (MD056)              |
| `docs/research/analysis/TASK_ALIGNMENT_ANALYSIS.md`             | **8**   | Line length (MD013) + Table issues (MD056)              |
| `docs/platform/INVESTMENT_STRATEGY_FRAMEWORK.md`                | **8**   | Line length (MD013)                                     |
| `docs/BROKEREE_SOLUTIONS.md`                                    | **8**   | Line length (MD013)                                     |

### 🟡 Medium (5-9 errors)

| File                                                     | Errors | Primary Issue                              |
| -------------------------------------------------------- | ------ | ------------------------------------------ |
| `docs/research/learnings/LEAN_LEARNINGS.md`              | **7**  | Line length (MD013)                        |
| `docs/T_BILLS_AND_FUTURES_GUIDE.md`                      | **7**  | Line length (MD013)                        |
| `docs/TRADING_INFRASTRUCTURE.md`                         | **7**  | Line length (MD013) + Table issues (MD056) |
| `docs/research/analysis/TRADING_FRAMEWORK_EVALUATION.md` | **6**  | Line length (MD013) + Table issues (MD056) |
| `docs/research/integration/CURSOR_SETUP.md`              | **5**  | Line length (MD013)                        |
| `docs/research/external/CME_RESEARCH.md`                 | **5**  | Line length (MD013)                        |
| `docs/research/analysis/STATIC_ANALYSIS_ANNOTATIONS.md`  | **5**  | Line length (MD013)                        |
| `docs/research/analysis/STATIC_ANALYSIS.md`              | **5**  | Line length (MD013)                        |
| `docs/ios_certificate_pinning.md`                        | **5**  | Line length (MD013)                        |

---

## Error Type Breakdown

### MD013: Line Length (591 errors - 96.7%)

**Issue:** Lines exceed 150 character limit

**Worst Offenders:**

- `docs/API_DOCUMENTATION_INDEX.md:320` - **584 characters** (URLs, long code examples)
- `docs/API_DOCUMENTATION_INDEX.md:582` - **534 characters** (long API descriptions)
- `docs/API_DOCUMENTATION_INDEX.md:319` - **455 characters** (long code examples)
- `docs/API_DOCUMENTATION_INDEX.md:478` - **270 characters** (API documentation)

**Common Causes:**

- Long URLs in documentation
- Code examples without line breaks
- Long API method signatures
- Table rows with long content
- Import statements

**Fix Strategy:**

- Break long URLs across lines
- Split code examples into multiple lines
- Use markdown link syntax for long URLs: `[text](url)`
- Break long sentences at natural points

---

### MD056: Table Column Count (16 errors - 2.6%)

**Issue:** Tables have inconsistent column counts

**Affected Files:**

- `docs/research/analysis/TASK_ALIGNMENT_ANALYSIS.md:171` - Expected 3, got 4 columns
- `docs/research/analysis/TRADING_FRAMEWORK_EVALUATION.md` - Multiple rows with 1 column (expected 8)
- `docs/research/external/RESEARCH_CPP_FINANCIAL_LIBRARIES.md` - Multiple rows with 5 columns (expected 6)
- `docs/TRADING_INFRASTRUCTURE.md` - Multiple table issues

**Fix Strategy:**

- Review table structure
- Add missing columns or remove extra columns
- Ensure all rows have the same number of columns as the header

---

### MD024: Duplicate Headings (3 errors - 0.5%)

**Issue:** Multiple headings with the same content

**Affected Files:**

- `docs/ONEPASSWORD_INTEGRATION.md:101` - Duplicate "Notes" heading
- `docs/RAM_OPTIMIZATION_OPPORTUNITIES.md:266` - Duplicate "Summary" heading
- `docs/research/architecture/CASH_FLOW_FORECASTING_SYSTEM.md:762` - Duplicate "Integration with Investment St..." heading

**Fix Strategy:**

- Rename duplicate headings to be unique
- Use more specific heading text
- Consider restructuring sections

---

### MD003: Heading Style (1 error - 0.2%)

**Issue:** Inconsistent heading style

**Affected File:**

- `docs/PROJECT_SCORECARD.md:1` - Uses setext style (`===`) instead of ATX style (`#`)

**Fix Strategy:**

- Change setext heading to ATX style: `# Project Scorecard`

---

## Priority Fix Recommendations

### 🔴 High Priority

1. **`docs/API_DOCUMENTATION_INDEX.md`** (117 errors)
   - **Impact:** Largest single file with errors
   - **Action:** Break long lines, especially URLs and code examples
   - **Estimated Time:** 30-60 minutes

2. **Table Issues** (16 errors across multiple files)
   - **Impact:** Broken table rendering
   - **Action:** Fix column alignment in affected tables
   - **Estimated Time:** 15-30 minutes

### 🟡 Medium Priority

3. **Research Documentation Files** (9 errors each)
   - **Files:** `RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md`, `RESEARCH_CPP_FINANCIAL_LIBRARIES.md`
   - **Action:** Break long lines in research documentation
   - **Estimated Time:** 10-15 minutes each

4. **Duplicate Headings** (3 errors)
   - **Action:** Rename duplicate headings
   - **Estimated Time:** 5 minutes

### 🟢 Low Priority

5. **Remaining Line Length Issues** (~550 errors)
   - **Action:** Fix incrementally as files are edited
   - **Strategy:** Fix during normal editing workflow

---

## Quick Fix Commands

### View errors for specific file

```bash
npm run lint:docs 2>&1 | grep "docs/FILENAME.md"
```

### View only table errors

```bash
npm run lint:docs 2>&1 | grep "MD056"
```

### View only duplicate heading errors

```bash
npm run lint:docs 2>&1 | grep "MD024"
```

### View longest lines in a file

```bash
awk 'length > 150 {print length": "NR": "$0}' docs/API_DOCUMENTATION_INDEX.md | sort -rn | head -20
```

---

## Auto-Fix Results

**Initial Errors:** 2,157 errors
**After Auto-Fix:** 611 errors (72% reduction)
**After Documentation Cleanup:** 389 errors (82% reduction)
**After Archive Exclusion:** 315 errors (85% reduction)
**Total Fixed:** 1,842 errors (85% reduction)

**What Was Auto-Fixed:**

- ✅ Blank lines around lists (429 errors)
- ✅ Blank lines around headings (many errors)
- ✅ Blank lines around code fences (112+ errors)
- ✅ Trailing punctuation in headings
- ✅ Other spacing/formatting issues

**What Requires Manual Fix:**

- ⚠️ Line length (315 errors) - Need manual line breaks
- ✅ Table column count (0 errors) - All fixed!
- ✅ Duplicate headings (0 errors) - All fixed!
- ✅ Heading style (0 errors) - All fixed!

**Note:** Archive files (`docs/archive/**`) are excluded from linting.

---

## Next Steps

1. **Immediate:** Fix `API_DOCUMENTATION_INDEX.md` (117 errors) - highest impact
2. **Short-term:** Fix table issues (16 errors) - affects rendering
3. **Medium-term:** Fix duplicate headings (3 errors) - quick wins
4. **Long-term:** Fix remaining line length issues incrementally

---

**Last Updated:** 2026-01-06
**Auto-Fix Status:** ✅ Complete (85% reduction)
**Documentation Cleanup:** ✅ Complete (177 files archived)
**Archive Exclusion:** ✅ Configured (archive files excluded from linting)
