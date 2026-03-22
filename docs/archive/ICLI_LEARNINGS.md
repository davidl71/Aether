# Learnings from icli Project

**Date**: 2025-01-27  
**Source**: <https://github.com/mattsta/icli>  
**Purpose**: Document patterns and approaches from icli that could enhance this C++ box spread project

---

## Overview

`icli` is a Python command-line interface for Interactive Brokers trading, designed for rapid manual trading and scalping. While this project (`ib_box_spread_full_universal`) is focused on automated box spread arbitrage in C++, there are several architectural patterns and features from icli that could be valuable.

---

## Key Architectural Patterns

### 1. Command Dispatch System

**icli Approach:**

- Uses `mutil/dispatch.py` for command routing
- Commands organized in `lang.py` with `OP_MAP` mapping command names to implementation classes
- Each command is a class with custom argument definitions
- Runtime help via `?` command and `cmdname?` for per-command docs

**Potential Application:**

- If adding an interactive CLI mode to this C++ project, consider a similar dispatch pattern
- Could use a map of command strings to function pointers or std::function
- Runtime help system would be valuable for debugging and manual intervention

### 2. Comprehensive API Action Logging

**icli Approach:**

- All IBKR API actions logged to `icli-{timestamp}.log`
- Complete audit trail of every order, modification, cancel
- Critical because IBKR removes intermediate order states after completion
- Logs preserve full history for analysis

**Current State:**

- This project uses spdlog for general logging
- TWS client actions are logged, but may not capture all API interactions

**Recommendation:**

- Ensure all TWS API calls (placeOrder, cancelOrder, reqMktData, etc.) are logged with full parameters
- Consider a dedicated API action log file separate from general application logs
- Include timestamps, order IDs, contract details, and responses

### 3. Session History Persistence

**icli Approach:**

- CLI session history persisted in `~/.tplatcli_ibkr_history.{live,sandbox}`
- Enables search and up/down arrow recall across sessions
- Separate history files for live vs paper trading

**Potential Application:**

- If adding interactive CLI, implement history persistence
- Could use readline or similar library for C++
- Separate history files for dry-run vs live trading

### 4. Multiple Client ID Support

**icli Approach:**

- Supports multiple concurrent sessions via `ICLI_CLIENT_ID` environment variable
- Each session uses unique client ID
- Important: IBKR restricts orders per-client-id (orders placed under one client won't show in others)

**Current State:**

- This project has `client_id` in TWS config
- Currently single-session focused

**Recommendation:**

- Document the client_id behavior clearly
- If running multiple instances, ensure different client IDs
- Consider adding validation/warning if duplicate client IDs detected

### 5. Efficient Quote Management with Pattern Expansion

**icli Approach:**

- Powerful pattern expansion for adding/removing quotes:
  - `add SPY240412{P,C}005{1,2,3}0000` expands to 6 contracts
  - `rm :{31..37}` removes quotes by row ID range
  - Supports ranges: `SPXW24041{5..7}{P05135,C05150}000`

**Potential Application:**

- This project focuses on box spreads (4-leg orders), so pattern expansion less critical
- However, could be useful for:
  - Testing multiple strike widths simultaneously
  - Monitoring option chains efficiently
  - Batch operations on related contracts

### 6. Order Efficiency Ratio Monitoring

**icli Approach:**

- Documents IBKR's order efficiency ratio requirement
- Need at least 1 executed trade per 20 order create/modify/cancel requests
- Tracks daily executed trades to maintain compliance

**Current State:**

- This project tracks order statistics but may not explicitly monitor efficiency ratio

**Recommendation:**

- Add order efficiency ratio tracking to `OrderManager::OrderStats`
- Calculate: `executed_trades / (orders_placed + orders_modified + orders_cancelled)`
- Log warnings if ratio drops below 0.05 (1:20)
- Consider rate limiting or batching to maintain compliance

### 7. Advanced Order Types and Algo Orders

**icli Approach:**

- Supports IBKR algo order types:
  - Primary peg (relative orders)
  - Adaptive limit/market orders
  - Peg to midpoint
  - Snap to market/primary/midpoint
  - Market with protection
- Emphasizes avoiding "marketable orders" (hitting bid/ask exactly) for better commissions

**Current State:**

- This project uses basic limit orders for box spreads
- May benefit from algo orders for better execution

**Recommendation:**

- Consider implementing IBKR algo orders for box spread execution
- For arbitrage, execution speed is critical, but better fills can improve profitability
- Document commission implications of order types
- Test different order types in paper trading

### 8. Real-Time Position and Order Display

**icli Approach:**

- Real-time display of positions, orders, and account info
- Updates in CLI interface as market data arrives
- Shows day trades remaining count (for accounts under $25k)

**Current State:**

- This project logs statistics periodically (every 100 iterations)
- No real-time display interface

**Recommendation:**

- If adding TUI/CLI mode, implement real-time status display
- Show active positions, pending orders, P&L, account balance
- Display day trades remaining if applicable
- Use ncurses or similar for C++ TUI

### 9. Futures Exchange Mapping

**icli Approach:**

- `futsexchanges.py` contains auto-generated mapping of futures symbols to exchanges
- Required because futures don't use SMART router (must specify exchange)
- Includes manual overrides for edge cases (e.g., BRR bitcoin contract multipliers)

**Current State:**

- This project focuses on options (XSP box spreads)
- Futures support not currently implemented

**Potential Application:**

- If expanding to futures box spreads, similar mapping would be needed
- Consider similar auto-generation approach for maintainability

### 10. Error Handling and Connection Management

**icli Approach:**

- Handles connection failures gracefully
- Auto-reconnect logic
- Clear error messages for common issues

**Current State:**

- This project has auto-reconnect in TWS client
- Error handling via callbacks

**Recommendation:**

- Ensure error messages are clear and actionable
- Document common connection issues (wrong port, TWS not running, etc.)
- Consider exponential backoff for reconnection attempts

---

## Features Worth Considering

### 1. Dry Run Mode Enhancement

**icli Approach:**

- Clear distinction between live and paper trading
- Environment-based configuration

**Current State:**

- This project has `dry_run` flag
- Works well but could be more explicit

**Recommendation:**

- Consider separate config files for live vs paper
- Add visual indicators (banners, warnings) for live trading mode
- Log mode clearly in all log entries

### 2. Commission Tracking

**icli Approach:**

- Tracks commissions per trade
- Shows aggregate commission charges in execution details

**Current State:**

- This project calculates costs but may not track actual commissions

**Recommendation:**

- Track actual commissions from IBKR execution reports
- Include in P&L calculations
- Log commission details for analysis

### 3. Rate Limiting

**icli Approach:**

- Documents IBKR rate limits (10,000 simultaneous orders, CBOE 390 rule)
- Implements rate limiting to avoid violations

**Current State:**

- This project has `set_max_orders_per_second` stub

**Recommendation:**

- Implement actual rate limiting
- Track order submission rate
- Add warnings for approaching limits
- Document CBOE 390 rule (avg 1 order/minute limit)

### 4. Day Trading Restrictions Awareness

**icli Approach:**

- Displays day trades remaining for accounts under $25k
- Documents FINRA pattern day trader rules
- Notes that futures/options don't have same-day open/close restrictions

**Current State:**

- This project doesn't track day trading restrictions

**Recommendation:**

- If expanding beyond box spreads, add day trade tracking
- Query account for day trades remaining
- Warn before placing orders that would violate restrictions

---

## Implementation Priorities

### High Priority

1. **Comprehensive API Logging**: Ensure all TWS API calls are logged with full context
2. **Order Efficiency Ratio Tracking**: Monitor and warn about compliance
3. **Clear Error Messages**: Improve user experience for common issues

### Medium Priority

1. **Advanced Order Types**: Consider algo orders for better execution
2. **Commission Tracking**: Track actual commissions from executions
3. **Rate Limiting**: Implement actual rate limiting (not just stub)

### Low Priority (Future Enhancements)

1. **Interactive CLI Mode**: If manual intervention needed, add command dispatch system
2. **Real-Time Display**: TUI for monitoring positions and orders
3. **Pattern Expansion**: For batch operations on option chains

---

## Code Patterns to Study

### Command Structure

- `icli/cli.py`: Main CLI loop (`dorepl()` method)
- `icli/lang.py`: Command implementations and `OP_MAP`
- `mutil/dispatch.py`: Command dispatch system

### Order Management

- `icli/orders.py`: IBKR order type definitions and creation
- How orders are tracked and updated
- Error handling for order rejections

### Market Data

- How quotes are added/removed efficiently
- Pattern expansion implementation
- Real-time updates handling

---

## Notes

- icli is Python-based, so direct code porting isn't applicable
- Focus on architectural patterns and user experience improvements
- This C++ project is more automated/algorithmic, while icli is manual/interactive
- Best practices around IBKR API usage are universally applicable

---

## References

- icli GitHub: <https://github.com/mattsta/icli>
- IBKR API Documentation: <https://interactivebrokers.github.io/tws-api/>
- FINRA Pattern Day Trader Rules: <https://www.finra.org/rules-guidance/rulebooks/finra-rules/4210>
