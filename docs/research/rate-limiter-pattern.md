# Rate Limiter Pattern Research

**Source:** longbridge-terminal  
**Date:** 2026-03-22  
**Status:** Todo

## Overview

longbridge-terminal implements a rate limiter wrapper pattern around their SDK contexts to handle API rate limits (10 req/s).

## Implementation Pattern

### longbridge-terminal: `src/openapi/rate_limiter.rs`

```rust
// Global rate-limited contexts
pub static RATE_LIMITED_QUOTE_CTX: OnceLock<RateLimitedQuoteContext> = OnceLock::new();
pub static RATE_LIMITED_TRADE_CTX: OnceLock<RateLimitedTradeContext> = OnceLock::new();

// Initialization
RATE_LIMITED_QUOTE_CTX
    .set(RateLimitedQuoteContext::new(quote_ref))
    .map_err(...);
```

### Key Characteristics

1. **Wrapper Struct Pattern** - `RateLimitedQuoteContext` wraps the raw SDK context
2. **OnceLock Initialization** - Contexts initialized once at startup
3. **Token Bucket / Semaphore** - Rate limiting via async semaphore
4. **Burst Capacity** - 10 req/s with burst of 20

## Aether Opportunity

Currently Aether's `ib_adapter` makes direct IBKR API calls without explicit rate limiting. Potential improvements:

1. Add `RateLimitedBrokerEngine` wrapper
2. Implement token bucket for IBKR API calls
3. Consider `LazyLock` for globals vs manual initialization

## Evaluation Criteria

- [ ] Thread-safety of rate limiter
- [ ] Integration with existing `IbAdapter`
- [ ] Backpressure handling
- [ ] Testing approach

## Related Tasks

- T-1774192005612584000: Research rate limiter pattern
