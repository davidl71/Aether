# NotebookLM Automation Guide

## Overview

This guide explains how to automate NotebookLM notebook creation using browser control MCP.

## Prerequisites

1. **Google Authentication**: You must be logged into your Google account
2. **Browser Control MCP**: Available via `cursor-browser-extension` MCP server
3. **NotebookLM Access**: Access to notebooklm.google.com

## Automation Workflow

### Step 1: Manual Authentication

**⚠️ Authentication cannot be automated for security reasons.**

1. Navigate to https://notebooklm.google.com
2. Sign in with your Google account
3. Complete any 2FA if required
4. Ensure you're logged in before proceeding

### Step 2: Automated Notebook Creation

Use browser control MCP to automate:

1. **Navigate to NotebookLM**

   ```
   browser_navigate("https://notebooklm.google.com")
   ```

2. **Create New Notebook**
   - Find and click "Create notebook" button
   - Enter notebook name
   - Confirm creation

3. **Add Sources**
   - Click "Add source"
   - Select "URL" for web sources
   - Paste URL and confirm
   - Wait for processing (5-10 seconds per source)

4. **Share Notebook**
   - Click share button
   - Select "Anyone with link"
   - Copy link
   - Save link for later use

### Step 3: Capture Notebook Links

Save notebook links to `docs/research/external/NOTEBOOKLM_NOTEBOOK_LINKS.json`:

```json
{
  "CME Financing Strategies": "https://notebooklm.google.com/notebook/...",
  "Message Queue Solutions": "https://notebooklm.google.com/notebook/...",
  ...
}
```

## Notebook Definitions

The following notebooks are configured for automation:

### CME Financing Strategies

- **Category**: external
- **Description**: Synthesize CME/Cboe whitepapers on financing strategies
- **Sources**: 4 sources
  - https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html
  - https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/
  - https://www.cmegroup.com/articles/2025/quantifying-and-hedging-equity-financing-risk.html
    - ...

### Message Queue Solutions

- **Category**: architecture
- **Description**: Compare NATS, RabbitMQ, Redis Streams, and ZeroMQ
- **Sources**: 4 sources
  - https://docs.nats.io/
  - https://www.rabbitmq.com/docs/
  - https://redis.io/docs/data-types/streams/
    - ...

### ORATS Options Data

- **Category**: external
- **Description**: ORATS API integration patterns
- **Sources**: 2 sources
  - https://orats.com/docs
  - https://orats.com/data-api

### TWS API Core Patterns

- **Category**: learnings
- **Description**: Core TWS API patterns and best practices
- **Sources**: 3 sources
  - docs/research/learnings/TWS_API_BEST_PRACTICES.md
  - docs/research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md
  - docs/research/learnings/TWS_API_CODE_EXAMPLES_LEARNINGS.md

### TWS API Advanced Topics

- **Category**: learnings
- **Description**: Advanced TWS API topics and troubleshooting
- **Sources**: 3 sources
  - docs/research/learnings/TWS_API_MARKET_DATA_LEARNINGS.md
  - docs/research/learnings/TWS_API_TROUBLESHOOTING_LEARNINGS.md
  - docs/research/learnings/IB_ASYNC_LEARNINGS.md

### TWS API Integration

- **Category**: learnings
- **Description**: TWS API integration and deployment patterns
- **Sources**: 3 sources
  - docs/research/learnings/TWS_API_DOCKER_LEARNINGS.md
  - docs/research/integration/TWS_INTEGRATION_STATUS.md
  - docs/research/learnings/IBC_LEARNINGS.md

## Browser Control MCP Commands

### Navigation

- `browser_navigate(url)` - Navigate to URL
- `browser_wait_for(text="...")` - Wait for text to appear

### Interaction

- `browser_click(element="...", ref="...")` - Click element
- `browser_type(element="...", ref="...", text="...")` - Type text
- `browser_snapshot()` - Get page state

### Example Workflow

```python

# 1. Navigate

browser_navigate("https://notebooklm.google.com")

# 2. Get page state

snapshot = browser_snapshot()

# 3. Find and click "Create notebook"

browser_click(element="Create notebook button", ref="[from snapshot]")

# 4. Enter name

browser_type(element="Notebook name", ref="[from snapshot]", text="CME Financing Strategies")

# 5. Create

browser_click(element="Create", ref="[from snapshot]")
```

## Limitations

1. **Authentication**: Must be done manually
2. **UI Changes**: NotebookLM UI may change, requiring script updates
3. **Rate Limiting**: May need delays between operations
4. **Source Processing**: URLs may take time to process

## Troubleshooting

### "Element not found"

- Use `browser_snapshot()` to get current page state
- Update element references
- Check if page has loaded completely

### "Authentication required"

- Sign in manually first
- Check if session expired
- Re-authenticate if needed

### "Source not processing"

- Wait longer (10-15 seconds)
- Check if URL is accessible
- Verify URL format is correct

## Next Steps

After notebooks are created:

1. **Save Links**: Store notebook links in JSON file
2. **Add to Library**: Use NotebookLM MCP to add notebooks to library
3. **Query Notebooks**: Use NotebookLM MCP to research and synthesize
4. **Update Documents**: Update research documents with findings

## See Also

- [NOTEBOOKLM_DEEP_RESEARCH_WORKFLOW.md](NOTEBOOKLM_DEEP_RESEARCH_WORKFLOW.md) - Complete research workflow
- [NOTEBOOKLM_RESEARCH_SETUP.md](NOTEBOOKLM_RESEARCH_SETUP.md) - Initial setup guide
- [NOTEBOOKLM_OPTIMIZATION_GUIDE.md](NOTEBOOKLM_OPTIMIZATION_GUIDE.md) - Optimization strategies
