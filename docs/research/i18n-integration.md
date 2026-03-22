# i18n Integration Research

**Source:** longbridge-terminal  
**Date:** 2026-03-22  
**Status:** Completed

## Overview

longbridge-terminal uses `rust-i18n` v2.2 for internationalization with YAML locale files.

## Implementation

### Dependencies

```toml
rust-i18n = "2.2"
```

### Locale Files

```
locales/
├── en.yml      (6.9KB)
├── zh-CN.yml   (7.1KB)
└── zh-HK.yml   (6.8KB)
```

### YAML Structure

```yaml
# en.yml
error:
  general: "Something went wrong"
  api:
    heading: "Failed to request"
Candlestick:
  Open: Open
  Close: Close
StockIndex:
  .DJI.US: Dow
  .IXIC.US: NASDAQ
```

### Usage in Code

```rust
use rust_i18n::t;

// Simple string
let msg = t!("error.general");

// Nested
let heading = t!("error.api.heading");

// In locale file
console:
  title: " Console "
```

### Runtime Switching

```rust
rust_i18n::set_locale("zh-CN");
```

## Evaluation for Aether

| Aspect | Assessment |
|--------|------------|
| **Workflow** | Simple - YAML files + `t!()` macro |
| **Coverage** | Need translations for all UI strings |
| **Audience** | Primarily English-speaking traders |
| **Complexity** | Medium - refactor all strings |

### Pros for Aether
- Standard Rust i18n solution
- Runtime locale switching
- Separation of content from code
- Easy to add languages

### Cons for Aether
- Refactoring all hardcoded strings
- Trading terminology needs careful translation
- Primary audience is English-speaking
- Adds build/deployment complexity

## Recommendation

**Not Recommended (Current Phase)** - i18n adds significant refactoring work. Consider after:
1. Feature completion
2. If expanding to non-English markets
3. If Chinese traders become a priority

**When to revisit:** If Aether expands to Asian markets or receives international user requests.

## Related Tasks

- T-1774192024444788000: Research i18n integration (Done)
