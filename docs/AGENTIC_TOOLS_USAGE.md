# Agentic Tools MCP Usage Guide

## Overview

The **Agentic Tools MCP** server provides advanced task management and agent memories capabilities with project-specific storage. This guide shows how to use it effectively in your IBKR Box Spread Generator project.

## What It Provides

### 🎯 Task Management System

- **Unlimited Hierarchy**: Tasks → Subtasks → Sub-subtasks → infinite depth
- **Rich Metadata**: Priority, complexity, dependencies, tags, time tracking
- **Project Organization**: Group tasks into projects
- **Status Workflow**: pending → in-progress → blocked → done
- **Dependency Management**: Link tasks with validation

### 🧠 Agent Memories System

- **Persistent Context**: Store information that persists across AI sessions
- **Category Organization**: Organize memories by type (preferences, technical, context)
- **Text Search**: Intelligent search across all memories
- **Git-Trackable**: Commit memories with your code

## Storage Location

All data is stored in `.agentic-tools-mcp/` in your project root:

```
.agentic-tools-mcp/
├── tasks.json          # Projects, tasks, and subtasks
└── memories/           # JSON file storage
    ├── preferences/   # User preferences
    ├── technical/      # Technical information
    └── context/        # Context information
```

**Note**: This directory can be committed to git to share tasks and memories with your team.

## Common Use Cases for Trading Software

### 1. Track TWS API Integration Tasks

**Example**: Create a project for TWS integration with hierarchical tasks

```
Project: TWS API Integration
├── Task: Implement EWrapper callbacks
│   ├── Subtask: Implement market data callbacks
│   ├── Subtask: Implement order status callbacks
│   └── Subtask: Implement error handling
├── Task: Test paper trading connection
│   ├── Subtask: Configure paper trading port (7497)
│   └── Subtask: Validate connection flow
└── Task: Implement order execution
    ├── Subtask: Multi-leg order support
    └── Subtask: Order validation
```

**Benefits**:
- Track progress on complex integration
- Manage dependencies between tasks
- Share task list with team via git

### 2. Store Agent Memories About Trading Strategies

**Example**: Store information about box spread strategies

```
Memory: Box Spread Risk Management
Category: technical
Content: Box spreads require careful risk management. Always:
- Use dry-run mode for testing
- Validate all calculations before execution
- Monitor position size limits
- Check for arbitrage opportunities
```

**Benefits**:
- AI remembers your trading strategy preferences
- Context persists across sessions
- Team can share knowledge via git

### 3. Track Feature Development

**Example**: Manage box spread bag implementation

```
Project: Box Spread Bag Feature
├── Task: Design data structures
├── Task: Implement core logic
│   ├── Subtask: Add box spread to bag
│   ├── Subtask: Remove box spread from bag
│   └── Subtask: Calculate aggregate metrics
└── Task: Add tests
```

## How to Use with AI Assistant

### Creating Tasks

**Ask the AI**:
- "Create a task for implementing TWS API connection"
- "Add a subtask to test paper trading connection"
- "Create a project for box spread bag feature"

**The AI will**:
- Create tasks with appropriate metadata
- Set up hierarchy automatically
- Add dependencies if mentioned

### Storing Memories

**Ask the AI**:
- "Remember that we always use dry-run mode for testing"
- "Store that box spreads require 4-leg orders"
- "Save my preference for C++20 standard"

**The AI will**:
- Create memory files in appropriate categories
- Store in `.agentic-tools-mcp/memories/`
- Make it searchable for future sessions

### Querying Tasks

**Ask the AI**:
- "What tasks are in progress?"
- "Show me all tasks for TWS integration"
- "What's blocking the box spread bag feature?"

**The AI will**:
- Query the task database
- Show hierarchical structure
- Display dependencies and status

### Searching Memories

**Ask the AI**:
- "What do we know about box spread strategies?"
- "Search memories for TWS API information"
- "What are my preferences for testing?"

**The AI will**:
- Search across all memory files
- Return relevant memories with context
- Use text-based search for intelligent matching

## Example Workflows

### Workflow 1: Starting a New Feature

1. **Create Project**:
   ```
   "Create a project for implementing hedge manager"
   ```

2. **Break Down into Tasks**:
   ```
   "Add tasks for hedge manager: design, implementation, testing"
   ```

3. **Add Subtasks**:
   ```
   "Add subtasks to the implementation task: position calculation, risk limits, execution"
   ```

4. **Set Dependencies**:
   ```
   "Make testing task depend on implementation task"
   ```

### Workflow 2: Storing Knowledge

1. **Store Technical Information**:
   ```
   "Remember that QuestDB uses PostgreSQL wire protocol on port 8812"
   ```

2. **Store Preferences**:
   ```
   "Remember I prefer 2-space indentation and Allman braces"
   ```

3. **Store Context**:
   ```
   "Remember this is trading software - always emphasize safety"
   ```

### Workflow 3: Tracking Progress

1. **Update Task Status**:
   ```
   "Mark TWS API connection task as in-progress"
   ```

2. **Check Dependencies**:
   ```
   "What tasks are blocking the box spread bag feature?"
   ```

3. **Review Progress**:
   ```
   "Show me all tasks in the TWS integration project"
   ```

## Integration with Git

Since task and memory data is stored in `.agentic-tools-mcp/`, you can:

1. **Commit with Code**:
   ```bash
   git add .agentic-tools-mcp/
   git commit -m "Update task list and agent memories"
   ```

2. **Share with Team**:
   - Team members get the same task list
   - Shared agent memories for consistency
   - Track progress together

3. **Version Control**:
   - See task history in git
   - Track memory evolution
   - Rollback if needed

## Best Practices

### Task Management

1. **Use Projects**: Group related tasks into projects
2. **Set Priorities**: Use 1-10 scale for prioritization
3. **Track Dependencies**: Link tasks that depend on each other
4. **Update Status**: Keep task status current
5. **Add Tags**: Use tags for filtering and organization

### Agent Memories

1. **Categorize**: Use appropriate categories (preferences, technical, context)
2. **Be Specific**: Include enough detail for future reference
3. **Update Regularly**: Keep memories current
4. **Search Before Creating**: Check if memory already exists

### Project Organization

1. **One Project Per Feature**: Keep projects focused
2. **Use Subtasks**: Break down complex tasks
3. **Link Related Tasks**: Use dependencies
4. **Track Time**: Estimate and track actual hours

## Troubleshooting

### Tasks Not Appearing

- Check that `.agentic-tools-mcp/tasks.json` exists
- Verify working directory is correct
- Check MCP server is running

### Memories Not Found

- Verify `.agentic-tools-mcp/memories/` directory exists
- Check category names match
- Try different search terms

### Storage Issues

- Ensure project directory is writable
- Check disk space
- Verify git isn't blocking writes

## See Also

- [AGENTIC_TOOLS_WORKFLOW_EXAMPLES.md](AGENTIC_TOOLS_WORKFLOW_EXAMPLES.md) - **Practical workflow examples for this project**
- [MCP_SERVERS.md](MCP_SERVERS.md) - Complete MCP server documentation
- [Agentic Tools MCP Repository](https://github.com/Pimzino/agentic-tools-mcp) - Official documentation
- [.cursorrules](../.cursorrules) - AI assistant guidelines
