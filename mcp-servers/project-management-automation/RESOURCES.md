# Automa MCP Server Resources

This document describes the resources exposed by the automa MCP server for accessing cached data and agent information.

## Available Resources

### 1. `automation://status`
Server status and health information.

**Returns:**
- Server version and status
- Tools availability
- Error handling status
- Timestamp

**Example:**
```json
{
  "server": "project-management-automation",
  "version": "0.1.0",
  "status": "operational",
  "tools_available": true,
  "timestamp": "2025-11-24T19:00:00"
}
```

---

### 2. `automation://history`
Automation tool execution history.

**Returns:**
- List of recent automation runs
- Execution status
- Health scores
- Issues found
- Timestamps

**Parameters:**
- `limit` (default: 50) - Maximum number of history entries

---

### 3. `automation://tools`
List of available automation tools with descriptions.

**Returns:**
- Tool names and descriptions
- Categories (documentation, task_management, security, etc.)
- Priority levels
- Parameters for each tool

---

### 4. `automation://tasks`
Cached Todo2 task list, optionally filtered.

**Parameters:**
- `agent` (Optional) - Filter by agent name (e.g., 'backend-agent')
- `status` (Optional) - Filter by status ('Todo', 'In Progress', 'Review', 'Done')
- `limit` (Optional, default: 100) - Maximum number of tasks

**Returns:**
- Filtered task list
- Total task counts
- Status breakdown
- Filter information

**Example URIs:**
- `automation://tasks` - All tasks
- `automation://tasks?agent=backend-agent` - Tasks for backend-agent
- `automation://tasks?status=Todo&limit=50` - First 50 Todo tasks

---

### 5. `automation://tasks/agent/{agent_name}`
Tasks for a specific agent.

**Parameters:**
- `agent_name` - Agent name (e.g., 'backend-agent', 'web-agent')
- `status` (Optional) - Additional status filter
- `limit` (Optional, default: 50) - Maximum number of tasks

**Example URIs:**
- `automation://tasks/agent/backend-agent`
- `automation://tasks/agent/web-agent?status=Todo`

**Agent Detection:**
Tasks are matched by:
- Agent name in task name
- Agent name in task description
- Agent name in task tags
- Agent name patterns (e.g., 'backend' matches 'backend-agent')

---

### 6. `automation://tasks/status/{status}`
Tasks filtered by status.

**Parameters:**
- `status` - Status value ('Todo', 'In Progress', 'Review', 'Done', etc.)
- `limit` (Optional, default: 100) - Maximum number of tasks

**Example URIs:**
- `automation://tasks/status/Todo`
- `automation://tasks/status/In Progress`

---

### 7. `automation://agents`
List of available agents with configurations and task counts.

**Returns:**
- Agent names and directories
- Working directories
- Environment variables
- Startup/runtime commands
- Task counts per agent

**Example:**
```json
{
  "agents": [
    {
      "name": "backend-agent",
      "directory": "agents/backend",
      "working_directory": "./agents/backend",
      "env": {"PYTHONPATH": "../../python"},
      "startup_commands": ["bash scripts/setup.sh"],
      "task_count": 15
    }
  ],
  "total_agents": 8
}
```

---

### 8. `automation://cache`
Cache status and metadata.

**Returns:**
- List of cached data sources
- Cache types (task_list, execution_history, agent_metadata)
- File paths and sizes
- Last modified timestamps
- Cache summary statistics

**Cache Types:**
- **task_list**: Todo2 state file (`.todo2/state.todo2.json`)
- **execution_history**: Automation run history files
- **agent_metadata**: Agent configurations from `cursor-agent.json` files

---

## Usage Examples

### Get Tasks for Current Agent
```
"Get tasks for backend-agent using automa"
"Show me tasks assigned to web-agent"
```

### Get Tasks by Status
```
"Get all tasks with status 'Todo'"
"Show tasks in 'Review' status"
```

### Get Agent Information
```
"List all agents and their task counts"
"Show agent configurations"
```

### Check Cache Status
```
"Check what data is cached"
"Show cache status and last modified times"
```

---

## Resource URI Patterns

### Simple Resources
- `automation://status`
- `automation://history`
- `automation://tools`
- `automation://agents`
- `automation://cache`

### Parameterized Resources
- `automation://tasks?agent={name}&status={status}&limit={n}`
- `automation://tasks/agent/{agent_name}?status={status}&limit={n}`
- `automation://tasks/status/{status}?limit={n}`

---

## Integration with Cursor

Resources are automatically available when the automa MCP server is configured. Access them via:

1. **Direct Resource Access**: Cursor can read resources directly
2. **Tool Integration**: Some tools use cached data from resources
3. **Agent Context**: Resources provide agent-specific context

---

## Data Sources

### Todo2 State File
- **Location**: `.todo2/state.todo2.json`
- **Contains**: All tasks, statuses, descriptions, tags, metadata
- **Updated**: When tasks are created/modified via Todo2

### Agent Configurations
- **Location**: `agents/*/cursor-agent.json`
- **Contains**: Agent names, working directories, environment variables
- **Updated**: When agent configurations change

### Execution History
- **Location**: `scripts/.{automation}_history.json`
- **Contains**: Automation run results, timestamps, status
- **Updated**: After each automation tool execution

---

**Last Updated**: 2025-11-24
**Status**: Active
