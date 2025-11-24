# Automa MCP Server Dependencies

This document describes the complementary MCP servers that work with the automa MCP server.

## Complementary MCP Servers

The automa server is designed to work alongside these complementary MCP servers for enhanced problem-solving workflows:

### 1. tractatus_thinking

**Purpose**: Structural analysis and logical decomposition of complex problems

**When to Use**: BEFORE using automa tools to understand WHAT needs to be analyzed

**Benefits**:
- Breaks down complex concepts into atomic components
- Reveals multiplicative dependencies (A × B × C must ALL be true)
- Separates essential from accidental requirements
- Finds the ONE missing element preventing success

**Configuration**:
```json
{
  "mcpServers": {
    "tractatus_thinking": {
      "command": "npx",
      "args": ["-y", "tractatus_thinking"],
      "description": "Tractatus Thinking MCP server for logical concept analysis and structured thinking"
    }
  }
}
```

**Note**: Package name is `tractatus_thinking` (with underscore), not `tractatus-thinking`.

**Documentation**: See `.cursor/rules/tractatus-thinking.mdc`

---

### 2. sequential_thinking

**Purpose**: Converting structural understanding into actionable implementation workflows

**When to Use**: AFTER tractatus_thinking and automa analysis to plan HOW to proceed

**Benefits**:
- Converts structural analysis into concrete implementation steps
- Creates step-by-step workflows for complex features
- Breaks down "what needs to be done" into "how to do it"
- Documents processes for future reference

**Configuration**:
```json
{
  "mcpServers": {
    "sequential_thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "description": "Sequential Thinking MCP server for structured problem-solving and implementation workflow"
    }
  }
}
```

**Note**: This is an npm package, not a Python module. Installation is automatic via `npx`.

**Documentation**: See `.cursor/rules/sequential-thinking.mdc`

---

## Recommended Workflow

### Complete Problem-Solving Workflow

1. **tractatus_thinking** → Understand WHAT (structure/logic)
   - Analyze the logical structure of the problem
   - Break down concepts into atomic components
   - Identify multiplicative dependencies

2. **automa** → Analyze and automate (project management)
   - Use automa tools to analyze documentation, tasks, security, etc.
   - Generate reports and identify issues
   - Create automation opportunities

3. **sequential_thinking** → Plan HOW (implementation)
   - Convert analysis results into actionable steps
   - Create implementation workflows
   - Document processes

### Example: Documentation Health Analysis

```
1. tractatus_thinking: "What is the structure of documentation health?"
   → Reveals: Health = Valid Links × Current Content × Proper Format × Complete Coverage

2. automa: check_documentation_health_tool()
   → Analyzes actual documentation state
   → Generates report with broken links, stale content, etc.

3. sequential_thinking: "How do we fix documentation health issues?"
   → Creates step-by-step workflow:
      Step 1: Fix broken links
      Step 2: Update stale content
      Step 3: Improve formatting
      Step 4: Add missing documentation
```

### Example: Task Alignment Analysis

```
1. tractatus_thinking: "What makes a task aligned with project goals?"
   → Reveals: Alignment = Goal Match × Priority × Dependencies × Resources

2. automa: analyze_todo2_alignment_tool()
   → Analyzes actual task alignment
   → Identifies misaligned tasks

3. sequential_thinking: "How do we realign misaligned tasks?"
   → Creates workflow for task realignment
```

---

## Integration Notes

- These MCP servers are **complementary**, not required
- Automa can work independently, but works best with tractatus/sequential
- All three servers should be configured in `.cursor/mcp.json`
- The AI assistant will automatically use the appropriate server based on the task

---

## See Also

- `.cursor/rules/tractatus-thinking.mdc` - Detailed Tractatus Thinking usage
- `.cursor/rules/sequential-thinking.mdc` - Detailed Sequential Thinking usage
- `.cursor/rules/project-automation.mdc` - Automa server usage guide
- `README.md` - Automa server overview

