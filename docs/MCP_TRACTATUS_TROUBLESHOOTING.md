# Tractatus Thinking MCP Server - Error Messages & Troubleshooting

## Common Error Messages

### 1. **Server Not Found / Connection Failed**

**Error Message:**

```
Failed to connect to tractatus_thinking server
Tool not found: tractatus_thinking
```

**Possible Causes:**

- MCP server not configured in `.cursor/mcp.json`
- Incorrect package name in configuration
- Server process not running

**Solutions:**

1. Verify configuration in `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "tractatus_thinking": {
      "command": "npx",
      "args": ["-y", "tractatus-thinking"],
      "description": "Tractatus Thinking MCP server..."
    }
  }
}
```

2. Restart Cursor to reload MCP configuration
3. Check Cursor's MCP server logs for connection errors
4. Verify npm/npx is available: `which npx`

### 2. **Invalid Operation Parameters**

**Error Message:**

```
Invalid operation: [operation_name]
Missing required parameter: [parameter_name]
```

**Common Issues:**

- Missing `operation` parameter (must be: "start", "add", "navigate", "export", "revise", "undo", "move")
- Missing `concept` parameter for "start" operation
- Missing `session_id` for operations after "start"

**Solutions:**

- Always start with `operation: "start"` and provide a `concept` query
- Save the `session_id` returned from "start" operation
- Use the `session_id` for subsequent operations

**Example Correct Usage:**

```json
{
  "operation": "start",
  "concept": "What is the structure of box spread arbitrage?",
  "depth_limit": 5,
  "style": "analytical"
}
```

### 3. **Session ID Errors**

**Error Message:**

```
Invalid session_id
Session not found
```

**Possible Causes:**

- Using expired or invalid session_id
- Session was cleared/expired
- Typo in session_id

**Solutions:**

- Start a new session with `operation: "start"`
- Sessions may expire after inactivity
- Always use the session_id returned from the "start" operation

### 4. **Missing Required Wrapper Parameters**

**Error Message:**

```
Missing required parameter: __wrapper_contextSummary
Missing required parameter: __wrapper_userPrompt
Missing required parameter: __wrapper_userPromptId
```

**Note:** These are internal wrapper parameters that should be automatically provided by Cursor. If you see these errors:

**Solutions:**

- This indicates an MCP integration issue, not a user error
- Try restarting Cursor
- Check Cursor's MCP server connection status
- Verify MCP server is properly configured

### 5. **Package Installation Errors**

**Error Message:**

```
Cannot find module 'tractatus-thinking'
Package not found
```

**Possible Causes:**

- Package name is incorrect
- Package not published to npm
- Network issues preventing package download

**Solutions:**

1. Verify package name:

   ```bash
   npm search tractatus-thinking
   ```

2. Try alternative package names:
   - `tractatus-thinking`
   - `@modelcontextprotocol/server-tractatus-thinking`
   - `tractatus_thinking`

3. Check npm registry:

   ```bash
   npm view tractatus-thinking
   ```

4. Manual installation test:

   ```bash
   npx -y tractatus-thinking --help
   ```

### 6. **Tool Execution Errors**

**Error Message:**

```
Tool execution failed
Error during tool execution
```

**Possible Causes:**

- Invalid operation parameters
- Server-side error
- Network connectivity issues

**Solutions:**

- Verify all required parameters are provided
- Check operation type is valid
- Review Cursor's MCP server logs
- Try a simpler operation first (e.g., "start" with a simple concept)

## Configuration Verification

### Check Current Configuration

1. **Verify `.cursor/mcp.json` exists and contains:**

```json
{
  "mcpServers": {
    "tractatus_thinking": {
      "command": "npx",
      "args": ["-y", "tractatus-thinking"],
      "description": "Tractatus Thinking MCP server..."
    }
  }
}
```

2. **Test MCP Server Connection:**
   - Open Cursor
   - Check MCP server status in Cursor settings
   - Look for `tractatus_thinking` in available MCP tools

3. **Verify Tool Availability:**
   - The tool should appear as: `mcp_tractatus_thinking_tractatus_thinking`
   - Check Cursor's MCP tools list

## Debugging Steps

1. **Restart Cursor** - Reloads MCP configuration
2. **Check Cursor Logs** - Look for MCP server errors
3. **Verify npm/npx** - Ensure Node.js and npm are installed
4. **Test Package Directly** - Try running the package manually
5. **Check Network** - Ensure npm registry is accessible
6. **Verify JSON Syntax** - Ensure `.cursor/mcp.json` is valid JSON

## Getting Help

If errors persist:

1. **Check Cursor MCP Documentation:**
   - Cursor Settings → MCP Servers
   - Review connection status

2. **Verify Package Availability:**
   - Visit: https://cursor.directory/mcp/tractatus_thinking
   - Check npm registry for latest package name

3. **Review MCP Server Logs:**
   - Check Cursor's developer console
   - Look for MCP-related error messages

4. **Alternative Configuration:**
   If the standard package name doesn't work, the server might need to be:
   - Installed globally: `npm install -g tractatus-thinking`
   - Run as a local server
   - Configured with different parameters

## Example Working Configuration

```json
{
  "mcpServers": {
    "tractatus_thinking": {
      "command": "npx",
      "args": [
        "-y",
        "tractatus-thinking"
      ],
      "description": "Tractatus Thinking MCP server for logical concept analysis and structured thinking - breaks down complex concepts into atomic truths, reveals multiplicative relationships, and finds missing elements"
    }
  }
}
```

## Related Documentation

- `.cursor/rules/tractatus-thinking.mdc` - Usage guidelines
- `docs/research/integration/MCP_SERVERS.md` - MCP server overview
- `docs/MCP_CONFIGURATION_EXAMPLE.json` - Configuration examples
