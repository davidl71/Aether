# NotebookLM Troubleshooting Guide

This guide helps troubleshoot common issues with NotebookLM MCP server integration.

## Common Issues

### Query Timeouts

**Symptom**: Queries to NotebookLM timeout with "Timeout waiting for response from NotebookLM"

**Possible Causes**:

1. **Resources Still Processing**: NotebookLM may still be processing resources (GitHub repos, videos, articles)
   - **Solution**: Wait 20-55 minutes for all resources to process
   - **Check**: Go to NotebookLM web interface and verify all resources show as "Ready" or "Processed"

2. **Network Latency**: Slow network connection to NotebookLM
   - **Solution**: Check internet connection, try again later
   - **Check**: Verify you can access notebooklm.google.com in browser

3. **Browser Automation Issues**: Browser automation may be slow or blocked
   - **Solution**: Check browser automation settings, try with `show_browser: true`
   - **Check**: Verify Chrome/Chromium is installed and accessible

4. **NotebookLM Service Issues**: NotebookLM service may be temporarily unavailable
   - **Solution**: Wait a few minutes and try again
   - **Check**: Try accessing NotebookLM directly in browser

**Diagnosis Steps**:

1. Check notebook status: `"Get details about the TWS Automated Trading notebook"`
2. Check active sessions: `"List active NotebookLM sessions"`
3. Check health: `"Check NotebookLM health"`
4. Try simpler query: `"What is TWS API?"`
5. Try with browser visible: Add `show_browser: true` to query

### Resources Not Processing

**Symptom**: Resources in NotebookLM show as "Processing" for a long time

**Possible Causes**:

1. **Large Resources**: GitHub repositories or long videos take longer to process
   - **Solution**: Be patient, large resources can take 30-60 minutes
   - **Check**: Monitor progress in NotebookLM web interface

2. **Invalid URLs**: Some URLs may not be accessible or supported
   - **Solution**: Verify URLs are correct and accessible
   - **Check**: Try accessing URLs directly in browser

3. **Rate Limits**: NotebookLM may have rate limits for processing
   - **Solution**: Wait and try again later
   - **Check**: Check NotebookLM status page

**Diagnosis Steps**:

1. Check notebook in web interface: <https://notebooklm.google.com>
2. Verify all resources are accessible
3. Check if resources are still processing
4. Try re-adding failed resources individually

### Authentication Issues

**Symptom**: "Not authenticated" or authentication errors

**Possible Causes**:

1. **Session Expired**: Google authentication session may have expired
   - **Solution**: Re-authenticate using `"Repair NotebookLM authentication"`
   - **Check**: Use `"Check NotebookLM health"` to verify authentication

2. **Account Issues**: Google account may have issues
   - **Solution**: Try re-authenticating with different account
   - **Check**: Verify you can log into NotebookLM in browser

**Diagnosis Steps**:

1. Check health: `"Check NotebookLM health"`
2. Verify `authenticated: true` in health status
3. If not authenticated: `"Repair NotebookLM authentication"`
4. Try accessing NotebookLM in browser to verify account

### Browser Not Opening

**Symptom**: Browser doesn't open for authentication or queries

**Possible Causes**:

1. **Chrome Not Installed**: Chrome/Chromium may not be installed
   - **Solution**: Install Chrome or Chromium
   - **Check**: Verify Chrome is in PATH

2. **Browser Automation Blocked**: Browser automation may be blocked
   - **Solution**: Check browser automation permissions
   - **Check**: Try with `show_browser: true` in query options

3. **Headless Mode Issues**: Headless mode may not work on your system
   - **Solution**: Use `browser_options: { show: true, headless: false }`
   - **Check**: Try disabling headless mode

**Diagnosis Steps**:

1. Check if Chrome is installed: `which google-chrome` or `which chromium`
2. Try with browser visible: Add `show_browser: true` to query
3. Check browser automation logs
4. Try manual authentication in browser

## Diagnostic Commands

### Check Health

```
"Check NotebookLM health"
```

Returns: Authentication status, active sessions, configuration

### List Notebooks

```
"Show our notebooks" or "List all notebooks"
```

Returns: All notebooks in library with metadata

### Get Notebook Details

```
"Get details about the TWS Automated Trading notebook"
```

Returns: Detailed notebook information including use count, last used

### List Sessions

```
"List active NotebookLM sessions"
```

Returns: Active sessions with age, message count, notebook URL

### Check Library Stats

```
"Get NotebookLM library statistics"
```

Returns: Total notebooks, active notebook, usage statistics

## Solutions by Issue Type

### Timeout Issues

**Option 1: Wait for Processing**

- Resources may still be processing
- Wait 20-55 minutes for all resources to complete
- Check NotebookLM web interface for processing status

**Option 2: Try Simpler Query**

- Start with simple questions: `"What is TWS API?"`
- Build up to more complex queries
- Use shorter, more direct questions

**Option 3: Use Browser Visible Mode**

- Add `show_browser: true` to see what's happening
- May help diagnose browser automation issues
- Example: `"Research TWS API in NotebookLM"` with browser visible

**Option 4: Check Network**

- Verify internet connection
- Check if NotebookLM is accessible: <https://notebooklm.google.com>
- Try again later if service is down

### Processing Issues

**Option 1: Verify Resources**

- Check NotebookLM web interface
- Verify all resources are accessible
- Re-add failed resources individually

**Option 2: Reduce Resource Count**

- Try adding resources in smaller batches
- Process GitHub repo separately from videos
- Add videos one at a time

**Option 3: Check Resource URLs**

- Verify all URLs are correct
- Test URLs in browser
- Remove invalid or inaccessible resources

### Authentication Issues

**Option 1: Re-authenticate**

```
"Repair NotebookLM authentication"
```

This will:

- Clear authentication data
- Open browser for fresh login
- Save new authentication

**Option 2: Switch Accounts**

```
"Re-authenticate with a different Google account"
```

Useful if:

- Current account has rate limits
- Need to use different account
- Authentication is broken

**Option 3: Check Account Status**

- Verify Google account is active
- Check NotebookLM access in browser
- Ensure account has NotebookLM access

## Best Practices

### 1. Wait for Processing

- Always wait for resources to fully process before querying
- Check NotebookLM web interface for processing status
- Large resources (GitHub repos, long videos) take longer

### 2. Start Simple

- Begin with simple queries to test connectivity
- Build up to more complex research questions
- Use shorter, more direct questions initially

### 3. Monitor Sessions

- Check active sessions regularly
- Close unused sessions to free resources
- Reset sessions if they become stuck

### 4. Use Browser Visible Mode for Debugging

- Enable `show_browser: true` when troubleshooting
- Watch browser automation to see what's happening
- Helps diagnose timeout and processing issues

### 5. Verify Resources

- Always verify resources are accessible
- Check URLs in browser before adding
- Monitor processing status in NotebookLM

## Getting Help

### Check Logs

- NotebookLM MCP server logs may contain error details
- Check Cursor/IDE logs for MCP server errors
- Look for timeout or connection errors

### Verify Configuration

- Check `.cursor/mcp.json` for NotebookLM configuration
- Verify MCP server is properly configured
- Ensure NotebookLM MCP server is installed

### Test Connectivity

1. Check health: `"Check NotebookLM health"`
2. List notebooks: `"Show our notebooks"`
3. Try simple query: `"What is TWS API?"`
4. Check sessions: `"List active NotebookLM sessions"`

## See Also

- [NotebookLM Usage Guide](research/integration/NOTEBOOKLM_USAGE.md) - Detailed usage instructions
- [NotebookLM Setup Guide](NOTEBOOKLM_SETUP_GUIDE.md) - Setup instructions
- [NotebookLM Status](NOTEBOOKLM_STATUS.md) - Current notebook status
- [MCP Servers Configuration](research/integration/MCP_SERVERS.md) - MCP server setup

## Quick Reference

| Issue | Command | Expected Result |
|-------|---------|----------------|
| Check health | `"Check NotebookLM health"` | Authentication status, sessions |
| List notebooks | `"Show our notebooks"` | All notebooks in library |
| Get notebook | `"Get details about [notebook]"` | Notebook details |
| List sessions | `"List active NotebookLM sessions"` | Active sessions |
| Re-authenticate | `"Repair NotebookLM authentication"` | Fresh authentication |
| Simple query | `"What is TWS API?"` | Basic answer |
| Complex query | `"Research TWS API architecture"` | Detailed research |
