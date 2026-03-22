# Rate Limiter Pattern Research

**Source:** longbridge-terminal (`src/openapi/rate_limiter.rs`)  
**Date:** 2026-03-22  
**Status:** Completed

## Overview

longbridge-terminal implements a token bucket rate limiter using `tokio::sync::Semaphore`. The pattern wraps SDK contexts to handle API rate limits (10 req/s).

## Implementation

### Core RateLimiter Struct

```rust
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,      // Token bucket
    tokens_per_second: u32,          // Refill rate
    max_tokens: u32,                // Burst capacity
    last_refill: tokio::sync::Mutex<Instant>,
}
```

### Key Features

1. **Token Bucket Algorithm** - Tokens refill based on elapsed time
2. **Semaphore-based** - Non-blocking `try_acquire()` with blocking fallback
3. **Burst Support** - Configurable burst capacity (20 tokens for longbridge)
4. **Retry Logic** - Built-in retry with exponential backoff on 429 errors

### Usage Pattern

```rust
// Global singleton
static RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();

pub async fn execute<F, T, E>(&self, request_name: &str, f: F) -> Result<T, E> {
    self.acquire().await;
    f().await
}
```

## Evaluation for Aether

| Aspect | Assessment |
|--------|------------|
| **Thread-safety** | Excellent - tokio Semaphore is Send+Sync |
| **Integration** | Moderate - needs wrapper per broker method |
| **IBKR limits** | IBKR has ~50 msg/sec; Aether doesn't hit limits currently |
| **Complexity** | Low - ~250 LOC including tests |

### Pros for Aether
- Clean, well-tested implementation
- Aligns with Rust async ecosystem
- Includes retry logic for resilience

### Cons for Aether
- Adds abstraction layer over `ib_adapter`
- IBKR rate limits are higher than Longbridge
- No current evidence of rate limit issues in Aether

## Recommendation

**Low Priority** - Consider as future optimization if IBKR rate limits become an issue. Current Aether usage doesn't warrant the added complexity.

## Related Tasks

- T-1774192005612584000: Research rate limiter pattern (Done)
