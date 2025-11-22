# Extension Category Analysis

## Overview

All extension management scripts now use VS Code's built-in `@category` filter to analyze extensions. This provides a more accurate and standardized way to identify redundant extensions and understand extension functionality.

## Category Support

The Cursor/VS Code CLI supports filtering extensions by category using:

```bash
cursor --list-extensions --category "formatters"
```

### Valid Categories

The following categories are supported (use lowercase):

- `formatters` - Code formatting extensions
- `linters` - Code linting extensions
- `debuggers` - Debugging extensions
- `themes` - Theme extensions
- `snippets` - Code snippet extensions
- `other` - Uncategorized extensions
- `extension packs` - Extension bundles
- `programming languages` - Language support
- `scm providers` - Source control providers
- `notebooks` - Jupyter/notebook support
- `data science` - Data science tools
- `machine learning` - ML tools
- `testing` - Testing frameworks
- `visualization` - Visualization tools
- `education` - Educational extensions
- `azure` - Azure-specific extensions
- `ai` - AI-powered extensions
- `chat` - Chat/communication extensions

## Updated Scripts

### 1. `check_extension_redundancy.sh`

Now includes category-based redundancy detection:

```bash
./scripts/check_extension_redundancy.sh
```

**Features:**

- Checks for multiple extensions in the same category (formatters, linters, debuggers, themes)
- Provides category-specific recommendations
- Combines category-based and functional redundancy analysis

**Example Output:**

```
⚠️  formatters Category (6 extensions)
   • anysphere.cpptools
   • davidanson.vscode-markdownlint
   • ms-python.black-formatter
   ...
   Recommendation: Review if multiple formatters conflict
   Note: Language-specific formatters (Black, ESLint) are usually fine
```

### 2. `check_extension_security.sh`

Now highlights security-sensitive categories:

```bash
./scripts/check_extension_security.sh
```

**Features:**

- Identifies extensions in security-sensitive categories (SCM Providers, Debuggers)
- Flags these for manual security review
- Provides context about why these categories need extra scrutiny

### 3. `analyze_by_category.sh` (NEW)

Comprehensive category-based analysis:

```bash
./scripts/analyze_by_category.sh
```

**Features:**

- Lists all extensions organized by category
- Shows count of extensions per category
- Provides recommendations for categories with multiple extensions
- Identifies uncategorized extensions

**Example Output:**

```
Formatters (6 extension(s)):
  • anysphere.cpptools
  • davidanson.vscode-markdownlint
  ...
  ⚠️  Multiple formatters - ensure they don't conflict
  Tip: Language-specific formatters (Black, ESLint) are usually fine
```

## Category-Based Recommendations

### Formatters

- **Multiple formatters are usually fine** if they target different languages
- **Watch for conflicts** between general formatters (e.g., Prettier vs ESLint)
- **Language-specific formatters** (Black for Python, ESLint for JS/TS) are complementary

### Linters

- **Multiple linters are fine** - they typically target different languages
- Examples: ESLint (JS/TS), ShellCheck (shell), markdownlint (markdown)
- Each linter serves a specific purpose

### Debuggers

- **Different debuggers for different targets** are complementary
- Examples: Python debugger, C++ debugger, browser debuggers
- Keep all that you need for your development targets

### Themes

- **Multiple themes are fine** - they don't conflict
- You can switch between themes without issues
- No redundancy concerns

### Extension Packs

- **May include individual extensions** that are also installed separately
- Check if you need both the pack and individual extensions
- Extension packs are convenient but may duplicate functionality

## Usage Examples

### Check for category-based redundancies

```bash
./scripts/check_extension_redundancy.sh
```

### Analyze all extensions by category

```bash
./scripts/analyze_by_category.sh
```

### Check security-sensitive categories

```bash
./scripts/check_extension_security.sh
```

## Benefits of Category-Based Analysis

1. **Standardized Classification**: Uses VS Code's official category system
2. **More Accurate**: Categories are assigned by extension publishers
3. **Better Redundancy Detection**: Identifies conflicts within the same functional category
4. **Security Awareness**: Highlights security-sensitive categories
5. **Comprehensive View**: Shows how extensions are organized by purpose

## Notes

- Categories are case-insensitive in the CLI (use lowercase)
- Some categories may not be valid (e.g., "Programming Languages" vs "programming languages")
- Invalid categories are automatically filtered out
- Extensions can belong to multiple categories
- The "other" category contains many extensions that may need manual review

## Integration with Other Scripts

All scripts now work together:

1. `analyze_by_category.sh` - Overview by category
2. `check_extension_redundancy.sh` - Detailed redundancy analysis (includes categories)
3. `check_extension_security.sh` - Security audit (includes category highlights)
4. `analyze_all_extensions.sh` - Complete extension inventory

Run these scripts in sequence for a comprehensive extension management workflow.
