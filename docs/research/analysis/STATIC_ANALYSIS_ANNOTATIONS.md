# Static Analysis Annotations Guide

This guide explains how to leave "breadcrumbs" for static analysis tools in C++ code. These annotations help compilers and static analyzers understand code intent, reduce false positives, and catch real issues more effectively.

**Bonus Benefit**: Static analysis annotations also improve [Cursor's codebase indexing](https://cursor.com/docs/context/codebase-indexing) by providing semantic hints about code behavior, making AI suggestions more accurate and relevant.

## Table of Contents

1. [Standardized Comments](#standardized-comments)
2. [Function Attributes](#function-attributes)
3. [Pragmas](#pragmas)
4. [Custom Macros](#custom-macros)
5. [Codebase Examples](#codebase-examples)
6. [Tool-Specific Notes](#tool-specific-notes)

## Standardized Comments

Many static analysis tools recognize specific comment formats for hints or warning suppression.

### Warning Suppression

Tools often allow specific keywords within comments to disable warnings on particular lines or blocks.

#### cppcheck Suppression

```cpp
// cppcheck-suppress[warning-id]
void function_with_intentional_issue() {
    // ...
}

// Or inline:
int* ptr = nullptr;  // cppcheck-suppress[nullPointer]
```

#### Clang Static Analyzer Suppression

```cpp
// NOLINT or NOLINTNEXTLINE
void function() {
    int unused;  // NOLINT
}

// Specific check suppression:
void function() {
    int* ptr = nullptr;  // NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init)
}
```

#### Fall-Through in Switch Statements

```cpp
switch (value) {
    case 1:
        do_something();
        // fall-through
    case 2:
        do_something_else();
        break;
    // ...
}
```

**Note**: The exact keyword (`// fall-through`, `// no break`, `// FALLTHROUGH`) is tool-specific. Check your tool's documentation.

### Rationale/Documentation Comments

Use comments to explain non-obvious coding choices or assumptions that might trigger static analysis warnings:

```cpp
// This function intentionally returns nullptr on error to match legacy API
// NOLINTNEXTLINE(cppcoreguidelines-avoid-returning-non-const-pointer)
const char* get_legacy_string() {
    return nullptr;  // Legacy API compatibility
}
```

## Function Attributes

C++ compilers (GCC and Clang) support function attributes that provide explicit information about function behavior, which static analyzers can leverage.

### Memory Management Attributes

#### `__attribute__((malloc))`

Informs the analyzer that a function returns a pointer to a new, non-aliased block of memory, helping with memory leak detection:

```cpp
void* my_malloc(size_t size) __attribute__((malloc));

// Usage
void* ptr = my_malloc(1024);  // Analyzer knows this needs to be freed
```

#### `__attribute__((returns_nonnull))`

Indicates the function never returns a null pointer:

```cpp
void* allocate_guaranteed(size_t size) __attribute__((returns_nonnull));

// Analyzer will flag if you check for null:
void* ptr = allocate_guaranteed(1024);
if (ptr == nullptr) {  // Analyzer warning: condition always false
    // ...
}
```

### Null Pointer Attributes

#### `__attribute__((nonnull(...)))`

Specifies that arguments at given indices must not be null pointers:

```cpp
void copy_data(void* dest, const void* src, size_t size)
    __attribute__((nonnull(1, 2)));

// Analyzer will flag calls that violate this:
copy_data(nullptr, src, size);  // Warning: null passed to nonnull parameter
copy_data(dest, nullptr, size);  // Warning: null passed to nonnull parameter
```

**Note**: Parameter indices start at 1 (not 0).

### Return Value Attributes

#### `__attribute__((warn_unused_result))`

Generates a warning if the caller ignores the function's return value:

```cpp
int check_important_action(void) __attribute__((warn_unused_result));

// Usage
check_important_action();  // Warning: ignoring return value
int result = check_important_action();  // OK
```

This is useful for functions where checking the return code is critical (e.g., error handling).

### Format String Attributes

#### `__attribute__((format(printf, format_index, first_arg_index)))`

Validates format strings for printf-style functions:

```cpp
void log_message(const char* format, ...)
    __attribute__((format(printf, 1, 2)));

// Analyzer will catch format string errors:
log_message("%d %s", "string", 42);  // Warning: format mismatch
log_message("%d", 42);  // OK
```

### Pure/Const Attributes

#### `__attribute__((pure))`

Indicates the function has no side effects and only depends on its arguments:

```cpp
int calculate_value(int x, int y) __attribute__((pure));

// Analyzer can optimize and cache results
```

#### `__attribute__((const))`

Similar to `pure`, but also doesn't depend on global state:

```cpp
int compute_hash(const char* str) __attribute__((const));
```

### Deprecated Attributes

#### `__attribute__((deprecated))` or `__attribute__((deprecated("message")))`

Marks functions as deprecated:

```cpp
void old_function() __attribute__((deprecated("Use new_function() instead")));
```

### Access Control Attributes

#### `__attribute__((access(read_only, param_index)))`

Indicates a parameter is read-only:

```cpp
void process_data(const int* data, size_t len)
    __attribute__((access(read_only, 1)));
```

### Thread Safety Attributes

#### `__attribute__((guarded_by(mutex)))`

Indicates a variable must be accessed while holding a specific mutex:

```cpp

#include <mutex>

std::mutex data_mutex;
int shared_data __attribute__((guarded_by(data_mutex)));
```

## Pragmas

The `#pragma` directive can issue tool-specific instructions or provide additional metadata.

### Clang Static Analyzer Pragmas

```cpp

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wunused-variable"

int unused_var = 42;

#pragma clang diagnostic pop
```

### GCC Pragmas

```cpp

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"

int unused_var = 42;

#pragma GCC diagnostic pop
```

### MSVC Pragmas

```cpp

#pragma warning(push)
#pragma warning(disable: 4100)  // Unused parameter

void function(int unused_param) {
    // ...
}

#pragma warning(pop)
```

### Tool-Specific Pragmas

Some tools support custom pragmas:

```cpp

#pragma ANALYZER_SUPPRESS("warning_id")

void function_with_suppressed_warning() {
    // ...
}
```

## Custom Macros

For proprietary APIs or domain-specific logic, you can define custom macros that expand into standard C++ code while also providing hints to static analysis tools.

### Example: Custom Memory Allocation

```cpp
// In header file

#ifdef __clang_analyzer__
#define CUSTOM_ALLOC(size) \

    (__attribute__((malloc)) void*)custom_alloc_impl(size)

#else
#define CUSTOM_ALLOC(size) custom_alloc_impl(size)
#endif

void* custom_alloc_impl(size_t size);
```

### Example: Null-Safe Access

```cpp
// Macro that provides null-safety hints

#define SAFE_ACCESS(ptr, member) \

    ((ptr) != nullptr ? (ptr)->member : decltype((ptr)->member){})

// Usage
struct Data { int value; };
Data* data = get_data();
int val = SAFE_ACCESS(data, value);  // Analyzer understands null check
```

### Example: Trading-Specific Annotations

```cpp
// For trading software: mark functions that must check return values

#define MUST_CHECK [[nodiscard]] __attribute__((warn_unused_result))

MUST_CHECK bool place_order(const Order& order);
MUST_CHECK ExecutionResult execute_strategy(const Strategy& strat);
```

## Codebase Examples

### Example 1: Order Manager Functions

```cpp
// native/include/order_manager.h

// Mark critical functions that must check return values
class OrderManager {
public:
    // This function's return value should always be checked
    ExecutionResult place_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price = 0.0,
        types::TimeInForce tif = types::TimeInForce::Day
    ) __attribute__((warn_unused_result));

    // Parameters cannot be null
    bool cancel_order(int order_id) __attribute__((nonnull));

    // This function has no side effects (pure query)
    std::optional<types::Order> get_order_status(int order_id) const
        __attribute__((pure));
};
```

### Example 2: TWS Client Functions

```cpp
// native/include/tws_client.h

namespace tws {
    class TWSClient {
    public:
        // Connection functions that must be checked
        bool connect(const std::string& host, int port)
            __attribute__((warn_unused_result));

        // Functions that return newly allocated memory
        std::unique_ptr<MarketData> get_market_data(
            const types::OptionContract& contract
        ) __attribute__((returns_nonnull));

        // Format string validation for logging
        void log(const char* format, ...)
            __attribute__((format(printf, 2, 3)));
    };
}
```

### Example 3: Risk Calculator

```cpp
// native/include/risk_calculator.h

namespace risk {
    class RiskCalculator {
    public:
        // Pure calculation function (no side effects)
        double calculate_position_risk(const types::Position& pos) const
            __attribute__((pure));

        // Parameters must not be null
        bool validate_order(
            const types::OptionContract& contract,
            types::OrderAction action,
            int quantity,
            double limit_price,
            std::string& error_message
        ) const __attribute__((nonnull(1, 5)));
    };
}
```

### Example 4: Suppressing False Positives

```cpp
// native/src/order_manager.cpp

void OrderManager::track_order_fill(int order_id) {
    // Static set to track fills - intentionally persists across calls
    // cppcheck-suppress[staticVariable]
    static std::set<int> tracked_fills;

    if (tracked_fills.count(order_id) > 0) {
        return;  // Already tracked
    }

    // ... rest of implementation
}
```

### Example 5: Switch Statement Fall-Through

```cpp
// native/src/order_manager.cpp

types::OrderStatus parse_status(const std::string& status_str) {
    switch (status_str[0]) {
        case 'P':
            if (status_str == "Pending") {
                return types::OrderStatus::Pending;
            }
            // fall-through
        case 'F':
            return types::OrderStatus::Filled;
        default:
            return types::OrderStatus::Unknown;
    }
}
```

## Tool-Specific Notes

### cppcheck

- Supports `// cppcheck-suppress[warning-id]` comments
- Recognizes `__attribute__` annotations
- Can be configured via `.cppcheckrc` file

### Clang Static Analyzer

- Supports `// NOLINT` and `// NOLINTNEXTLINE` comments
- Fully supports GCC-style `__attribute__` annotations
- Can use `#pragma clang diagnostic` for suppressions

### Infer

- Recognizes many GCC/Clang attributes
- Supports `// NOLINT` comments
- Best used with `compile_commands.json`

### GCC

- Native support for `__attribute__` annotations
- Supports `#pragma GCC diagnostic` for suppressions

### MSVC

- Uses `__declspec` instead of `__attribute__` for some annotations
- Supports `#pragma warning` for suppressions
- Can use `[[nodiscard]]` (C++17) instead of `warn_unused_result`

## Best Practices

1. **Use Standard Attributes**: Prefer standard attributes (`[[nodiscard]]`, `[[maybe_unused]]`) when available (C++17+), fall back to `__attribute__` for compatibility.

2. **Document Intent**: Combine attributes with comments explaining _why_ the annotation is needed.

3. **Suppress Sparingly**: Only suppress warnings when you're certain they're false positives. Document the reason.

4. **Test Annotations**: Verify that your annotations actually help the static analyzer (run the tools before and after adding annotations).

5. **Keep Consistent**: Use consistent annotation patterns across the codebase.

6. **Review Regularly**: Periodically review suppressions to ensure they're still valid.

## Cross-Platform Compatibility

For maximum compatibility, use macros to handle compiler differences:

```cpp
// In common header

#ifdef __GNUC__

    #define ATTR_NONNULL(...) __attribute__((nonnull(__VA_ARGS__)))
    #define ATTR_WARN_UNUSED __attribute__((warn_unused_result))
    #define ATTR_MALLOC __attribute__((malloc))

#elif defined(_MSC_VER)

    #define ATTR_NONNULL(...)  // MSVC doesn't support nonnull
    #define ATTR_WARN_UNUSED [[nodiscard]]
    #define ATTR_MALLOC  // MSVC doesn't support malloc attribute

#else

    #define ATTR_NONNULL(...)
    #define ATTR_WARN_UNUSED
    #define ATTR_MALLOC

#endif

// Usage
ATTR_WARN_UNUSED bool place_order(const Order& order);
void copy_data(void* dest, const void* src, size_t size) ATTR_NONNULL(1, 2);
```

## References

### Academic Research

- **Breadcrumbs: Static Analysis Annotations (PLDI 2010)** - Research paper on using annotations to guide static analysis tools: [https://mdbond.github.io/breadcrumbs-pldi-2010.pdf](https://mdbond.github.io/breadcrumbs-pldi-2010.pdf)
  - This paper introduced the concept of "breadcrumbs" - lightweight annotations that help static analysis tools understand code intent and reduce false positives
  - The techniques documented in this guide are inspired by and build upon the concepts presented in this research

### Compiler & Tool Documentation

- [GCC Function Attributes](https://gcc.gnu.org/onlinedocs/gcc/Function-Attributes.html)
- [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html)
- [cppcheck Manual](http://cppcheck.sourceforge.net/manual.pdf)
- [Clang Static Analyzer](https://clang-analyzer.llvm.org/)
- [Infer Documentation](https://fbinfer.com/docs)
- [C++ Core Guidelines](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines)

## Related Documentation

- [STATIC_ANALYSIS.md](STATIC_ANALYSIS.md) - Overview of static analysis tools used in this project
- [AI_FRIENDLY_CODE.md](../../AI_FRIENDLY_CODE.md) - Best practices for writing AI-friendly code
- [COMMON_PATTERNS.md](../../COMMON_PATTERNS.md) - Common coding patterns used in this codebase
