# C++20 Features Reference

**Source**: C++20 Standard and project usage
**Last Updated**: 2025-01-27

This project uses **C++20** (`CMAKE_CXX_STANDARD 20`). This document provides a quick reference for commonly used C++20 features in the codebase.

---

## Core Language Features

### Concepts

**Purpose**: Constrain template parameters

```cpp
template<typename T>
concept Numeric = std::integral<T> || std::floating_point<T>;

template<Numeric T>
T add(T a, T b) {
  return a + b;
}
```

**Project Usage**: Type constraints for trading calculations

---

### Ranges

**Purpose**: Modern iteration and algorithms

```cpp

#include <ranges>

std::vector<int> numbers = {1, 2, 3, 4, 5};

// Filter and transform
auto result = numbers
  | std::views::filter([](int n) { return n % 2 == 0; })
  | std::views::transform([](int n) { return n * 2; });
```

**Project Usage**: Option chain filtering and transformation

---

### Coroutines

**Purpose**: Asynchronous programming

```cpp

#include <coroutine>

generator<int> fibonacci() {
  int a = 0, b = 1;
  while (true) {
    co_yield a;
    auto temp = a;
    a = b;
    b = temp + b;
  }
}
```

**Project Usage**: Async market data streaming (potential future use)

---

### Modules (C++20)

**Purpose**: Faster compilation, better encapsulation

```cpp
// math.cppm
export module math;

export int add(int a, int b) {
  return a + b;
}

// main.cpp
import math;

int main() {
  return add(1, 2);
}
```

**Note**: Limited compiler support, not yet used in project

---

## Library Features

### std::format

**Purpose**: Type-safe string formatting

```cpp

#include <format>

std::string message = std::format("Price: ${:.2f}, Volume: {}", price, volume);
```

**Project Usage**: Logging and error messages (via spdlog which uses fmt)

---

### std::span

**Purpose**: Non-owning view of contiguous sequence

```cpp

#include <span>

void process_prices(std::span<const double> prices) {
  for (auto price : prices) {
    // Process
  }
}

std::vector<double> data = {1.0, 2.0, 3.0};
process_prices(data);  // Works with vector
process_prices({data.data(), 3});  // Works with raw array
```

**Project Usage**: Passing price arrays without ownership transfer

---

### std::jthread

**Purpose**: Joinable thread with automatic join

```cpp

#include <thread>

void worker() {
  // Do work
}

std::jthread t(worker);  // Automatically joins on destruction
// No need to call t.join()
```

**Project Usage**: EReader thread management (potential improvement)

---

### std::atomic_ref

**Purpose**: Atomic operations on non-atomic objects

```cpp

#include <atomic>

int counter = 0;
std::atomic_ref<int> atomic_counter(counter);

atomic_counter.fetch_add(1);  // Thread-safe increment
```

**Project Usage**: Thread-safe counters and flags

---

### std::source_location

**Purpose**: Get current source location

```cpp

#include <source_location>

void log_error(const std::string& message,
               const std::source_location& loc = std::source_location::current()) {
  std::cout << loc.file_name() << ":" << loc.line() << " - " << message << std::endl;
}

log_error("Something went wrong");  // Automatically captures location
```

**Project Usage**: Enhanced logging with source location

---

## Designated Initializers

**Purpose**: Initialize struct members by name

```cpp
struct Config {
  bool enabled = true;
  int max_entries = 1000;
  std::string log_file;
};

Config config{
  .enabled = true,
  .max_entries = 2000,
  .log_file = "app.log"
};
```

**Project Usage**: Configuration struct initialization

---

## Three-Way Comparison (Spaceship Operator)

**Purpose**: Unified comparison operators

```cpp

#include <compare>

struct Price {
  double value;

  auto operator<=>(const Price& other) const = default;
  // Automatically generates: ==, !=, <, <=, >, >=
};

Price p1{100.0}, p2{200.0};
if (p1 < p2) {  // Uses operator<=>
  // ...
}
```

**Project Usage**: Price and order comparisons

---

## constexpr Improvements

**Purpose**: More code can be evaluated at compile-time

```cpp
constexpr int fibonacci(int n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

constexpr int fib_10 = fibonacci(10);  // Computed at compile-time
```

**Project Usage**: Compile-time calculations for trading constants

---

## Lambda Improvements

### Template Lambdas

```cpp
auto lambda = []<typename T>(T value) {
  return value * 2;
};
```

### Capture with Initialization

```cpp
int x = 10;
auto lambda = [y = x + 5]() {
  return y;  // y is 15
};
```

**Project Usage**: Generic algorithms and closures

---

## Commonly Used C++20 Features in This Project

### 1. Designated Initializers

```cpp
// From tui_breadcrumb.h
BreadcrumbLogger(const Config& config = Config{
  .enabled = true,
  .max_entries = 1000,
  .log_to_console = false,
  .log_to_file = true
});
```

### 2. std::format (via spdlog)

```cpp
logger_->info("Price: ${:.2f}, Volume: {}", price, volume);
```

### 3. Concepts (Potential)

```cpp
template<typename T>
concept PriceType = std::floating_point<T> && requires(T t) {
  { t * 100 } -> std::convertible_to<int>;
};
```

### 4. Ranges (Potential)

```cpp
auto profitable_spreads = option_chains
  | std::views::filter([](const auto& spread) { return spread.profit > 0; })
  | std::views::take(10);
```

---

## Migration from C++17

### Before (C++17)

```cpp
// Manual string formatting
std::string message = "Price: " + std::to_string(price) + ", Volume: " + std::to_string(volume);

// Manual struct initialization
Config config;
config.enabled = true;
config.max_entries = 1000;

// Manual comparison operators
bool operator<(const Price& other) const {
  return value < other.value;
}
```

### After (C++20)

```cpp
// std::format
std::string message = std::format("Price: ${:.2f}, Volume: {}", price, volume);

// Designated initializers
Config config{.enabled = true, .max_entries = 1000};

// Three-way comparison
auto operator<=>(const Price& other) const = default;
```

---

## Compiler Support

### Minimum Requirements

- **Clang**: 10.0+ (full C++20 support: 12.0+)
- **GCC**: 10.0+ (full C++20 support: 11.0+)
- **MSVC**: 19.20+ (Visual Studio 2019 16.2+)

### Project Configuration

```cmake
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
```

---

## Best Practices

1. **Use designated initializers** for struct initialization
2. **Prefer std::format** over string concatenation
3. **Use std::span** for non-owning views
4. **Consider concepts** for template constraints
5. **Use three-way comparison** for custom types
6. **Leverage constexpr** for compile-time evaluation

---

## References

- [C++20 Standard](https://en.cppreference.com/w/cpp/20)
- [C++20 Features Overview](https://en.cppreference.com/w/cpp/compiler_support#cpp20)
- [Project CMakeLists.txt](../../native/CMakeLists.txt)
