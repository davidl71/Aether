# Agentic Tools MCP - Practical Workflow Examples

## Real-World Examples for IBKR Box Spread Generator

This document provides concrete examples of how to use Agentic Tools MCP in your daily workflow.

## Example 1: Tracking TWS API Integration

### Scenario

You're implementing the Interactive Brokers TWS API integration. Use Agentic Tools to track progress.

### Step-by-Step

**1. Create the Project**

```
Ask AI: "Create a project called 'TWS API Integration' with description 'Complete integration of Interactive Brokers TWS API for live trading'"
```

**2. Break Down into Main Tasks**

```
Ask AI: "Add these tasks to TWS API Integration project:
- Implement EWrapper interface
- Test paper trading connection
- Implement order execution
- Add error handling and reconnection logic"
```

**3. Add Subtasks with Dependencies**

```
Ask AI: "Add subtasks to 'Implement EWrapper interface':
- Implement market data callbacks (priority 8, complexity 6)
- Implement order status callbacks (priority 9, complexity 5)
- Implement position callbacks (priority 7, complexity 4)
- Implement error handling callbacks (priority 10, complexity 7)

Make order status callbacks depend on market data callbacks"
```

**4. Track Progress**

```
Ask AI: "Mark 'Implement market data callbacks' as in-progress"
Ask AI: "What tasks are blocking the order execution task?"
Ask AI: "Show me the task tree for TWS API Integration project"
```

**5. Store Important Context**

```
Ask AI: "Remember: TWS API requires client ID between 1-32, paper trading uses port 7497, live trading uses port 7496"
Ask AI: "Store: EWrapper callbacks must be thread-safe as TWS API uses multiple threads"
Ask AI: "Remember: Always test with paper trading first, use --dry-run flag in code"
```

## Example 2: Managing Box Spread Feature Development

### Scenario

You're implementing the box spread bag feature with multiple components.

### Workflow

**1. Create Feature Project**

```
Ask AI: "Create a project 'Box Spread Bag Implementation' for managing multiple box spreads simultaneously"
```

**2. Organize by Component**

```
Ask AI: "Add tasks to Box Spread Bag project:
- Design data structures (priority 9, complexity 7)
- Implement core bag operations (priority 10, complexity 8)
- Add risk aggregation (priority 8, complexity 9)
- Write comprehensive tests (priority 7, complexity 6)
- Update documentation (priority 6, complexity 4)"
```

**3. Add Detailed Subtasks**

```
Ask AI: "Break down 'Implement core bag operations' into subtasks:
- Add box spread to bag
- Remove box spread from bag
- Calculate aggregate profit/loss
- Validate bag constraints
- Serialize/deserialize bag state"
```

**4. Set Dependencies**

```
Ask AI: "Make 'Write comprehensive tests' depend on 'Implement core bag operations'"
Ask AI: "Make 'Update documentation' depend on 'Implement core bag operations'"
```

**5. Track Time and Complexity**

```
Ask AI: "Set estimated hours for 'Implement core bag operations' to 16 hours"
Ask AI: "Update actual hours for 'Design data structures' to 4 hours"
```

## Example 3: Storing Trading Strategy Knowledge

### Scenario

You want the AI to remember important trading strategy information across sessions.

### Memory Storage

**Technical Knowledge:**

```
Ask AI: "Store technical memory: Box spreads require exactly 4 legs - long call, short call, long put, short put. All must have same expiration and strike relationships: long call strike < short call strike, long put strike < short put strike"
```

**Risk Management:**

```
Ask AI: "Remember: Always validate box spread arbitrage opportunity before execution. Minimum profit threshold is $50 after commissions. Maximum position size is $250,000 per box spread"
```

**API Patterns:**

```
Ask AI: "Store: TWS API order execution pattern - use placeOrder() for single orders, placeOrderGroup() for multi-leg strategies. Always check order status via orderStatus() callback"
```

**Testing Preferences:**

```
Ask AI: "Remember my preference: Always use --dry-run flag for testing. Paper trading port is 7497. Never execute live trades during development"
```

## Example 4: Daily Workflow

### Morning Routine

**1. Check What's In Progress**

```
Ask AI: "What tasks are currently in-progress?"
Ask AI: "Show me tasks that are blocked"
```

**2. Plan the Day**

```
Ask AI: "What are the highest priority tasks for TWS API Integration?"
Ask AI: "Show me tasks that can be started (dependencies are done)"
```

**3. Update Status**

```
Ask AI: "Mark 'Test paper trading connection' as in-progress"
Ask AI: "Mark 'Implement market data callbacks' as done"
```

### During Development

**1. Create Subtasks as You Work**

```
Ask AI: "Add subtask 'Handle connection timeout' to 'Test paper trading connection' task"
Ask AI: "Add subtask 'Validate client ID range' to 'Test paper trading connection' task"
```

**2. Store Discoveries**

```
Ask AI: "Remember: TWS API requires socket connection to be established before sending requests. Connection timeout is 30 seconds"
Ask AI: "Store: Paper trading account uses different account ID format than live trading"
```

**3. Track Blockers**

```
Ask AI: "Mark 'Implement order execution' as blocked because we need to complete error handling first"
```

### End of Day

**1. Update Progress**

```
Ask AI: "Update actual hours for 'Implement EWrapper interface' to 6 hours"
Ask AI: "Mark 'Test paper trading connection' as done"
```

**2. Plan Tomorrow**

```
Ask AI: "What tasks should I work on tomorrow? Show me pending tasks with highest priority"
```

## Example 5: Team Collaboration

### Sharing via Git

**1. Commit Task Lists**

```bash
git add .agentic-tools-mcp/
git commit -m "Update TWS API integration task list"
git push
```

**2. Team Members Get Updates**

- Pull latest code
- Get updated task lists automatically
- See what others are working on
- Access shared agent memories

**3. Review Task History**

```bash
git log --oneline -- .agentic-tools-mcp/
# See task list evolution over time
```

## Example 6: Complex Feature with Multiple Dependencies

### Scenario: Implementing Hedge Manager

**1. Create Project**

```
Ask AI: "Create project 'Hedge Manager Implementation' for managing position hedging"
```

**2. Create Task Hierarchy**

```
Ask AI: "Add tasks:
- Design hedge strategy (priority 10, complexity 8)
- Implement position calculation (priority 9, complexity 7)
- Implement risk limits (priority 10, complexity 9)
- Add hedge execution (priority 8, complexity 8)
- Integration testing (priority 7, complexity 6)"
```

**3. Add Subtasks with Dependencies**

```
Ask AI: "Add subtasks to 'Implement position calculation':
- Calculate net delta (depends on: Design hedge strategy)
- Calculate net gamma (depends on: Calculate net delta)
- Calculate net theta (depends on: Calculate net delta)
- Aggregate position metrics (depends on: all calculation subtasks)"
```

**4. Track Complex Dependencies**

```
Ask AI: "Show me the dependency graph for Hedge Manager Implementation"
Ask AI: "What's the critical path for completing this project?"
```

## Example 7: Using Agent Memories for Context

### Storing Project Context

**Project-Specific Memories:**

```
Ask AI: "Store context: This project uses C++20, 2-space indentation, Allman braces. Trading logic must be thread-safe"
Ask AI: "Remember: We use QuestDB for time-series data on port 9009 (ILP) and 8812 (PostgreSQL wire protocol)"
Ask AI: "Store: Web UI runs on Vite dev server at localhost:5173, uses React with TypeScript"
```

**Technical Decisions:**

```
Ask AI: "Remember: We chose FTXUI for TUI because it's header-only and supports C++20. Go TUI was removed in favor of C++"
Ask AI: "Store: We use NautilusTrader for advanced market data, but TWS API for execution"
```

**Testing Patterns:**

```
Ask AI: "Remember: All trading-related tests must use --dry-run flag. Integration tests use mock TWS client"
Ask AI: "Store: Test data is generated using mock_data_generator.cpp, located in native/src/"
```

## Example 8: Querying and Searching

### Finding Information

**Search Memories:**

```
Ask AI: "What do we know about TWS API connection requirements?"
Ask AI: "Search memories for box spread strategy information"
Ask AI: "What are my preferences for testing?"
```

**Query Tasks:**

```
Ask AI: "Show me all tasks related to risk management"
Ask AI: "What tasks have complexity greater than 7?"
Ask AI: "Show me tasks tagged with 'critical'"
Ask AI: "What's the progress on TWS API Integration project?"
```

**Get Recommendations:**

```
Ask AI: "What tasks should I work on next based on priority and dependencies?"
Ask AI: "Show me tasks that are ready to start (dependencies complete)"
```

## Best Practices for Trading Software

### 1. Always Store Safety Rules

```
Ask AI: "Remember: NEVER execute live trades via MCP terminal. Always use --dry-run"
Ask AI: "Store: Paper trading port is 7497, live trading port is 7496. Always default to paper trading"
```

### 2. Track Risk-Related Tasks Separately

```
Ask AI: "Create project 'Risk Management' for all risk-related features"
Ask AI: "Tag all risk tasks with 'risk' and 'critical' tags"
```

### 3. Document API Patterns

```
Ask AI: "Store: TWS API connection pattern - connect() → nextValidId callback → ready to send requests"
Ask AI: "Remember: Order IDs must be unique and sequential. Start from nextValidId"
```

### 4. Track Testing Requirements

```
Ask AI: "Add task 'Integration testing' with subtasks:
- Test paper trading connection
- Test order placement (dry-run)
- Test error handling
- Test reconnection logic"
```

## Integration with Existing Workflow

### With Git Workflow

1. **Create tasks for features** before starting work
2. **Update task status** as you progress
3. **Commit task lists** with code changes
4. **Review task history** in git log

### With Documentation

1. **Store important decisions** in agent memories
2. **Link tasks to documentation** via tags
3. **Update memories** when patterns change
4. **Search memories** when writing docs

### With Testing

1. **Create test tasks** for each feature
2. **Track test coverage** via task completion
3. **Store test patterns** in memories
4. **Document test requirements** in task descriptions

## Quick Reference

### Common Commands

**Create:**

- "Create a project called [name]"
- "Add a task [name] to [project]"
- "Add a subtask [name] to [task]"

**Update:**

- "Mark [task] as in-progress"
- "Mark [task] as done"
- "Set priority of [task] to [1-10]"

**Query:**

- "Show me all tasks in [project]"
- "What tasks are in-progress?"
- "Show me the task tree"

**Memory:**

- "Remember: [information]"
- "Store: [technical detail]"
- "What do we know about [topic]?"

## See Also

- [AGENTIC_TOOLS_USAGE.md](AGENTIC_TOOLS_USAGE.md) - Complete usage guide
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - MCP server configuration
- [Agentic Tools MCP Repository](https://github.com/Pimzino/agentic-tools-mcp) - Official docs
