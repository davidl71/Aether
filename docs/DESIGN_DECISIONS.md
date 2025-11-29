# Design Decisions

This document documents why custom implementations were chosen over standard libraries or frameworks for key components of this project.

**Last Updated**: 2025-11-29

## Overview

This project includes several custom implementations instead of using off-the-shelf libraries. This document explains the rationale for each decision, helping future maintainers understand why these choices were made and when they might need to be reconsidered.

## Custom Implementations

### 1. Custom HTTP Client (`native/src/brokers/http_client.cpp`)

**Decision**: Implement custom HTTP client wrapper around libcurl instead of using higher-level HTTP libraries.

**Rationale**:
- **Minimal Dependencies**: Uses libcurl which is already required for broker API integrations (Alpaca, IBKR Client Portal)
- **Lightweight**: Thin wrapper provides only what's needed - no unnecessary abstractions
- **Control**: Direct control over timeouts, redirects, and error handling
- **C++20 Compatibility**: Works seamlessly with C++20 codebase without additional dependencies
- **Trading-Specific**: Simple interface matches trading API patterns (GET/POST/PUT/DELETE)

**Alternatives Considered**:
- **cpp-httplib**: Too heavyweight, includes server functionality not needed
- **libcurlpp**: C++ wrapper, but adds complexity without significant benefit
- **Boost.Beast**: Part of Boost (large dependency), more complex than needed

**When to Reconsider**:
- If we need advanced features (HTTP/2, WebSockets, connection pooling)
- If maintenance burden becomes too high
- If security requirements demand more robust HTTP handling

**Status**: ✅ Stable - Works well for current use cases

---

### 2. Custom Rate Limiter (`native/src/rate_limiter.cpp`)

**Decision**: Implement custom rate limiter instead of using a library like `ratelimit` or `token-bucket`.

**Rationale**:
- **TWS API Compliance**: IBKR TWS API has specific rate limiting requirements (messages/sec, concurrent requests)
- **Trading-Specific**: Tracks both message rate and concurrent market data subscriptions/historical requests
- **Lightweight**: Simple in-memory implementation without external dependencies
- **Thread-Safe**: Uses mutexes for thread-safe operation in multi-threaded TWS client
- **Observable**: Provides status and metrics for monitoring

**Key Features**:
- Per-second message rate limiting
- Concurrent request tracking (historical data, market data subscriptions)
- Stale request cleanup
- Configurable limits per request type

**Alternatives Considered**:
- **Token Bucket Libraries**: Too generic, don't handle concurrent request limits
- **Redis-based Rate Limiting**: Overkill for single-process application
- **Guava RateLimiter (Java)**: Not applicable to C++ codebase

**When to Reconsider**:
- If we need distributed rate limiting across multiple processes
- If we need more sophisticated algorithms (sliding window, leaky bucket)
- If rate limiting becomes a bottleneck

**Status**: ✅ Stable - Meets TWS API compliance requirements

---

### 3. Custom Box Spread Validator (`native/src/strategies/box_spread/box_spread_strategy.cpp`)

**Decision**: Implement custom box spread validation logic instead of using a general options validation library.

**Rationale**:
- **Domain-Specific**: Box spreads have specific validation rules (4 legs, matching strikes, same expiry)
- **Trading Logic**: Validates pricing relationships, bid/ask spreads, and arbitrage opportunities
- **Performance**: Fast validation without external dependencies
- **Integration**: Tightly integrated with box spread calculation logic

**Validation Rules**:
- Structure validation (4 legs: long call, short call, long put, short put)
- Strike validation (lower strike < higher strike)
- Expiry validation (all legs same expiry)
- Symbol validation (all legs same underlying)
- Pricing validation (positive prices, reasonable spreads)

**Alternatives Considered**:
- **QuantLib**: Too heavyweight for simple validation, focuses on pricing not validation
- **General Options Libraries**: Don't understand box spread structure
- **External Validation Services**: Adds latency and dependency

**When to Reconsider**:
- If validation rules become more complex
- If we need to validate other option strategies
- If validation becomes a performance bottleneck

**Status**: ✅ Stable - Handles all current box spread validation needs

---

### 4. Custom TWS Client Wrapper (`native/src/tws_client.cpp`)

**Decision**: Implement custom wrapper around IBKR TWS API instead of using existing TWS wrappers.

**Rationale**:
- **Full Control**: Complete control over API interaction patterns
- **Modern C++20**: Uses modern C++ features (smart pointers, optional, variant)
- **Integration**: Tightly integrated with project's type system and error handling
- **Rate Limiting**: Built-in rate limiting and request tracking
- **Error Handling**: Comprehensive error catalog with actionable guidance
- **Testing**: Easier to test and mock for unit tests

**Key Features**:
- Full EWrapper implementation with default handlers
- Rate limiting integration
- Request ID management
- Error code mapping to actionable messages
- Synchronous and asynchronous API patterns

**Alternatives Considered**:
- **IB API Wrappers**: Existing wrappers are older C++98/03 style, don't use modern C++
- **Python IB-Insync**: Not applicable to C++ codebase
- **Third-Party Wrappers**: Don't match project's architecture and error handling patterns

**When to Reconsider**:
- If IBKR releases official modern C++ wrapper
- If maintenance burden becomes too high
- If we need features that are easier to implement with existing wrappers

**Status**: ✅ Stable - Core trading functionality working

---

### 5. Custom Configuration Manager (`native/src/config_manager.cpp`)

**Decision**: Implement custom JSON configuration manager instead of using libraries like `nlohmann/json` directly or `boost::property_tree`.

**Rationale**:
- **Validation**: Built-in validation of configuration values (ports, rates, limits)
- **Type Safety**: Strongly typed configuration structures
- **Error Messages**: Clear error messages for invalid configuration
- **Defaults**: Sensible defaults for all configuration options
- **Trading-Specific**: Validates trading-specific constraints (rate limits, risk parameters)

**Key Features**:
- JSON schema validation
- Type-safe configuration access
- Validation of trading parameters
- Environment variable overrides
- Configuration file watching (future)

**Alternatives Considered**:
- **nlohmann/json directly**: Would require validation logic scattered throughout codebase
- **boost::property_tree**: Older API, less type-safe
- **Config Libraries**: Too generic, don't understand trading constraints

**When to Reconsider**:
- If configuration becomes more complex
- If we need hot-reloading of configuration
- If validation rules become too complex to maintain

**Status**: ✅ Stable - Handles all configuration needs

---

### 6. Custom Order Validator (`native/src/order_manager.cpp`)

**Decision**: Implement custom order validation instead of using broker-specific validation libraries.

**Rationale**:
- **Multi-Broker**: Works across multiple brokers (IBKR, Alpaca, etc.)
- **Trading Logic**: Validates trading-specific constraints (quantity, price, action)
- **Error Messages**: Clear, actionable error messages
- **Integration**: Tightly integrated with order manager

**Validation Rules**:
- Contract validation (symbol, expiry, strike, type)
- Quantity validation (positive, within limits)
- Price validation (positive, within reasonable bounds)
- Action validation (BUY/SELL, valid for contract type)

**Alternatives Considered**:
- **Broker SDKs**: Each broker has different validation, would need multiple libraries
- **General Validation Libraries**: Too generic, don't understand trading constraints
- **External Validation Services**: Adds latency and dependency

**When to Reconsider**:
- If validation rules become broker-specific
- If we need more sophisticated validation (margin checks, position limits)
- If validation becomes a performance bottleneck

**Status**: ✅ Stable - Handles current validation needs

---

### 7. Custom Alpaca Adapter Rate Limiter (`native/src/brokers/alpaca_adapter.cpp`)

**Decision**: Implement simple rate limiter in Alpaca adapter instead of reusing TWS rate limiter.

**Rationale**:
- **Different Requirements**: Alpaca has different rate limits (200 requests/minute) than TWS
- **Simplicity**: Simple queue-based implementation sufficient for Alpaca's needs
- **Independence**: Alpaca adapter is independent module, shouldn't depend on TWS rate limiter
- **Lightweight**: Minimal implementation for single-broker use case

**Alternatives Considered**:
- **Reuse TWS Rate Limiter**: Different requirements, would need configuration complexity
- **External Rate Limiting Library**: Overkill for simple per-minute limiting
- **Alpaca SDK Rate Limiting**: Alpaca SDK doesn't provide built-in rate limiting

**When to Reconsider**:
- If rate limiting requirements become more complex
- If we need distributed rate limiting
- If we add more brokers with similar rate limiting needs

**Status**: ✅ Stable - Meets Alpaca API requirements

---

## General Principles

### When to Build Custom vs Use Libraries

**Build Custom When**:
1. **Domain-Specific**: Logic is specific to trading/finance domain
2. **Lightweight**: Custom implementation is simpler than library
3. **Control**: Need fine-grained control over behavior
4. **Integration**: Tight integration with existing codebase
5. **Dependencies**: Avoid adding large dependencies

**Use Libraries When**:
1. **Standard Problem**: Well-solved problem with mature libraries
2. **Complexity**: Problem is too complex to solve correctly
3. **Maintenance**: Library is well-maintained and widely used
4. **Features**: Need features that are hard to implement correctly
5. **Security**: Security-critical code benefits from battle-tested libraries

### Maintenance Considerations

All custom implementations should:
- ✅ Have comprehensive tests
- ✅ Be well-documented
- ✅ Follow project coding standards
- ✅ Be reviewed periodically for maintenance burden
- ✅ Have clear upgrade paths if replaced

### Review Schedule

This document should be reviewed:
- **Quarterly**: Check if custom implementations still make sense
- **When Adding Features**: Consider if custom implementation can handle new requirements
- **When Issues Arise**: Re-evaluate if library would solve problems better
- **When Dependencies Change**: Check if new libraries make custom implementation unnecessary

---

## Future Considerations

### Potential Library Replacements

1. **HTTP Client**: Consider `cpp-httplib` if we need HTTP/2 or WebSockets
2. **Rate Limiter**: Consider Redis-based solution if we need distributed rate limiting
3. **Configuration**: Consider `toml11` or `yaml-cpp` if we move away from JSON
4. **Validation**: Consider schema validation libraries if validation becomes more complex

### Areas for Improvement

1. **Path Boundary Enforcement**: Currently implemented in Python services, should add to C++ codebase
2. **Access Control**: Currently basic, may need more sophisticated authentication/authorization
3. **Rate Limiting**: May need distributed rate limiting for multi-process deployments
4. **Error Handling**: Could benefit from structured error types across all components

---

## References

- [C++ Core Guidelines](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines) - General C++ best practices
- [IBKR TWS API Documentation](https://interactivebrokers.github.io/tws-api/) - TWS API requirements
- [Alpaca API Documentation](https://alpaca.markets/docs/) - Alpaca API requirements
- [OWASP Security Guidelines](https://owasp.org/) - Security best practices

---

**Document Maintainer**: Development Team  
**Last Review**: 2025-11-29  
**Next Review**: 2026-02-29
