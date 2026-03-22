# Lint log fix suggestions

Concrete code changes to fix the C++ errors reported in `logs/lint_ai_friendly.log`. Apply these so the full linter run passes.

---

## 1. test_box_spread_e2e.cpp – constructor and pointer fixes

**Issue:** `BoxSpreadStrategy` expects `(TWSClient*, OrderManager*, StrategyParams)`; tests pass `(StrategyParams, shared_ptr<TWSClient>)`. `OrderManager` expects `TWSClient*`; tests pass `shared_ptr<TWSClient>`.

**Fix:**

- **OrderManager:** Pass raw pointer: `client.get()`.
- **BoxSpreadStrategy:** Build an `OrderManager` first, then pass `client.get()`, `&order_manager`, and `strategy_params`.

**Edits:**

| Location | Current | Change to |
|----------|---------|-----------|
| Line 109 | `BoxSpreadStrategy strategy(strategy_params, client);` | Create order manager first, then strategy. See block below. |
| Line 156 | `OrderManager order_manager(client, true);` | `OrderManager order_manager(client.get(), true);` |
| Line 238 | `OrderManager order_manager(client, true);` | `OrderManager order_manager(client.get(), true);` |
| Lines 275–277 | `BoxSpreadStrategy strategy(strategy_params, client);`<br>`OrderManager order_manager(client, true);` | Create `OrderManager order_manager(client.get(), true);` then `BoxSpreadStrategy strategy(client.get(), &order_manager, strategy_params);` |
| Line 351 | `OrderManager order_manager(client, true);` | `OrderManager order_manager(client.get(), true);` |

**Block for first test case (opportunity detection) – replace lines 107–109:**

```cpp
  config::StrategyParams strategy_params = create_test_strategy_params();
  OrderManager order_manager(client.get(), false);
  BoxSpreadStrategy strategy(client.get(), &order_manager, strategy_params);
```

**Block for “Complete workflow” test – replace lines 274–277:**

```cpp
  config::StrategyParams strategy_params = create_test_strategy_params();
  OrderManager order_manager(client.get(), true);  // dry_run = true
  BoxSpreadStrategy strategy(client.get(), &order_manager, strategy_params);
```

---

## 2. test_hedge_manager.cpp:298 – Catch2 chained comparison

**Issue:** Catch2 v3 does not support chained comparisons inside `REQUIRE`; the expression is parsed incorrectly.

**Current:**

```cpp
REQUIRE(effectiveness.needs_rebalance == true || effectiveness.needs_rebalance == false);
```

**Fix (always true; use for “is valid bool” check):**

```cpp
REQUIRE((effectiveness.needs_rebalance == true || effectiveness.needs_rebalance == false));
```

Or simplify to:

```cpp
// needs_rebalance is a bool; just ensure we can read it
REQUIRE(effectiveness.needs_rebalance == true || effectiveness.needs_rebalance == false);
```

with the **whole expression wrapped in parentheses** so Catch2 sees one expression:

```cpp
REQUIRE((effectiveness.needs_rebalance == true || effectiveness.needs_rebalance == false));
```

---

## 3. test_box_spread_bag.cpp:227 – same Catch2 pattern

**Current:**

```cpp
REQUIRE(is_neutral == true || is_neutral == false);
```

**Fix:** Wrap in parentheses:

```cpp
REQUIRE((is_neutral == true || is_neutral == false));
```

---

## 4. Broker headers – include path for broker_interface.h

**Issue:** `native/include/brokers/alpaca_adapter.h` (and similar) use `#include <box_spread/brokers/broker_interface.h>`. With include path `native/include`, that path does not exist (file is `native/include/brokers/broker_interface.h`).

**Fix:** Use the path relative to `native/include` in all broker adapter headers that include the interface:

- **Files to change:**  
  `native/include/brokers/alpaca_adapter.h`,  
  `native/include/brokers/tws_adapter.h`,  
  `native/include/brokers/ib_client_portal_stub_adapter.h`,  
  `native/include/brokers/ib_client_portal_adapter.h`,  
  `native/include/brokers/alpaca_stub_adapter.h`

- **Change:**  
  Replace  
  `#include <box_spread/brokers/broker_interface.h>`  
  with  
  `#include "brokers/broker_interface.h"`

Namespace `box_spread::brokers` in the .h files stays as-is; only the include path is fixed.

---

## 5. test_path_validator.cpp – Catch2 v3 include and main

**Issue:** Uses `#include <catch2/catch.hpp>` and `CATCH_CONFIG_MAIN`. Catch2 v3 uses split headers and provides a default main.

**Current (lines 1–4):**

```cpp
#define CATCH_CONFIG_MAIN
#include <catch2/catch.hpp>
#include "path_validator.h"
```

**Fix:**

```cpp
#include <catch2/catch_test_macros.hpp>
#include "path_validator.h"
```

Remove `#define CATCH_CONFIG_MAIN` so the default Catch2 v3 main is used (same as other tests).

---

## Summary checklist

- [ ] **test_box_spread_e2e.cpp:** Use `client.get()` for `OrderManager`; construct `OrderManager` first, then `BoxSpreadStrategy(client.get(), &order_manager, strategy_params)` where needed.
- [ ] **test_hedge_manager.cpp:298:** Wrap the `REQUIRE` expression in parentheses.
- [ ] **test_box_spread_bag.cpp:227:** Wrap the `REQUIRE` expression in parentheses.
- [ ] **Broker headers (5 files):** Replace `<box_spread/brokers/broker_interface.h>` with `"brokers/broker_interface.h"`.
- [ ] **test_path_validator.cpp:** Use `#include <catch2/catch_test_macros.hpp>` and remove `CATCH_CONFIG_MAIN`.

After applying these, re-run the full linter (e.g. `./scripts/run_linters.sh`) and check `logs/lint_ai_friendly.log` for any remaining C++ errors.
