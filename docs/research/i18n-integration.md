# i18n Integration Research

**Source:** longbridge-terminal  
**Date:** 2026-03-22  
**Status:** Todo

## Overview

longbridge-terminal uses `rust-i18n` for internationalization with locale files in `locales/*.yml`.

## Implementation Pattern

### Dependencies

```toml
rust-i18n = "2.2"
```

### Locale Files

```
locales/
├── en.yml
├── zh-CN.yml
└── zh-HK.yml
```

### Usage

```rust
use rust_i18n::t;

// In code
let status = t!("TradeStatus.Normal");

// Locale files (en.yml)
TradeStatus:
  Normal: "Normal"
  Trading: "Trading"
  Halted: "Halted"
```

### Runtime Switching

```rust
rust_i18n::set_locale("zh-CN");
```

## Aether Current State

- No i18n support
- All strings hardcoded in TUI code

## Evaluation Criteria

- [ ] Translation workflow
- [ ] Runtime locale switching
- [ ] Impact on existing code
- [ ] Chinese/English coverage for trading terms

## Potential Tradeoff

| Aspect | Aether Consideration |
|--------|---------------------|
| Audience | English-speaking traders primarily |
| Complexity | Adds build/deployment complexity |
| Value | Multi-language support for terminal |

## Related Tasks

- T-1774192024444788000: Research i18n integration for Aether
