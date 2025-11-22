#!/usr/bin/env python3
"""
Automate NotebookLM Notebook Creation

This script automates the creation of NotebookLM notebooks using browser control.
Note: Google authentication must be completed manually first.

Usage:
    python3 scripts/automate_notebooklm_creation.py

Prerequisites:
    - User must be logged into Google account in browser
    - Browser control MCP server must be available
    - NotebookLM must be accessible at notebooklm.google.com
"""

import json
import sys
from pathlib import Path
from typing import List, Dict, Optional

# Notebook definitions from research documents
NOTEBOOKS = [
    {
        "name": "CME Financing Strategies",
        "sources": [
            "https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html",
            "https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/",
            "https://www.cmegroup.com/articles/2025/quantifying-and-hedging-equity-financing-risk.html",
            "https://www.cmegroup.com/market-data/license-data/licensed-market-data-distributors.html",
        ],
        "category": "external",
        "description": "Synthesize CME/Cboe whitepapers on financing strategies",
    },
    {
        "name": "Message Queue Solutions",
        "sources": [
            "https://docs.nats.io/",
            "https://www.rabbitmq.com/docs/",
            "https://redis.io/docs/data-types/streams/",
            "https://zeromq.org/",
        ],
        "category": "architecture",
        "description": "Compare NATS, RabbitMQ, Redis Streams, and ZeroMQ",
    },
    {
        "name": "ORATS Options Data",
        "sources": [
            "https://orats.com/docs",
            "https://orats.com/data-api",
        ],
        "category": "external",
        "description": "ORATS API integration patterns",
    },
    {
        "name": "TWS API Core Patterns",
        "sources": [
            "docs/research/learnings/TWS_API_BEST_PRACTICES.md",
            "docs/research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md",
            "docs/research/learnings/TWS_API_CODE_EXAMPLES_LEARNINGS.md",
        ],
        "category": "learnings",
        "description": "Core TWS API patterns and best practices",
    },
    {
        "name": "TWS API Advanced Topics",
        "sources": [
            "docs/research/learnings/TWS_API_MARKET_DATA_LEARNINGS.md",
            "docs/research/learnings/TWS_API_TROUBLESHOOTING_LEARNINGS.md",
            "docs/research/learnings/IB_ASYNC_LEARNINGS.md",
        ],
        "category": "learnings",
        "description": "Advanced TWS API topics and troubleshooting",
    },
    {
        "name": "TWS API Integration",
        "sources": [
            "docs/research/learnings/TWS_API_DOCKER_LEARNINGS.md",
            "docs/research/integration/TWS_INTEGRATION_STATUS.md",
            "docs/research/learnings/IBC_LEARNINGS.md",
        ],
        "category": "learnings",
        "description": "TWS API integration and deployment patterns",
    },
]

# Output file for notebook links
OUTPUT_FILE = Path("docs/research/external/NOTEBOOKLM_NOTEBOOK_LINKS.json")


def generate_browser_control_script(notebook: Dict) -> str:
    """
    Generate browser control commands for creating a notebook.

    Returns a script that can be executed via browser control MCP.
    """
    name = notebook["name"]
    sources = notebook["sources"]

    script = f"""
# Create Notebook: {name}
# Description: {notebook['description']}

# Step 1: Navigate to NotebookLM (if not already there)
# browser_navigate("https://notebooklm.google.com")

# Step 2: Wait for page to load
# browser_wait_for(text="Create notebook")

# Step 3: Click "Create notebook" button
# browser_click(element="Create notebook button", ref="[find create button ref]")

# Step 4: Enter notebook name
# browser_type(element="Notebook name input", ref="[find name input ref]", text="{name}")

# Step 5: Confirm/create notebook
# browser_click(element="Create button", ref="[find create button ref]")

# Step 6: Wait for notebook to load
# browser_wait_for(text="Add source")

# Step 7: Add each source URL
"""

    for i, source in enumerate(sources, 1):
        if source.startswith("http"):
            script += f"""
# Add source {i}: {source}
# browser_click(element="Add source button", ref="[find add source ref]")
# browser_click(element="URL option", ref="[find URL option ref]")
# browser_type(element="URL input", ref="[find URL input ref]", text="{source}")
# browser_click(element="Add button", ref="[find add button ref]")
# browser_wait_for(time=5)  # Wait for source to process
"""
        else:
            # Local file - would need to be uploaded
            script += f"""
# Add source {i}: {source} (local file - requires file upload)
# browser_click(element="Add source button", ref="[find add source ref]")
# browser_click(element="Upload file option", ref="[find upload option ref]")
# [File upload would require file path handling]
"""

    script += """
# Step 8: Share notebook and capture link
# browser_click(element="Share button", ref="[find share button ref]")
# browser_click(element="Anyone with link", ref="[find link option ref]")
# browser_click(element="Copy link", ref="[find copy link ref]")
# [Capture link from clipboard or page]

# Step 9: Save notebook link
# notebook_link = "[captured link]"
"""

    return script


def create_automation_guide():
    """Create a guide for using browser control to automate notebook creation."""

    guide = """# NotebookLM Automation Guide

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

"""

    for notebook in NOTEBOOKS:
        guide += f"""
### {notebook['name']}

- **Category**: {notebook['category']}
- **Description**: {notebook['description']}
- **Sources**: {len(notebook['sources'])} sources
  {chr(10).join(f'  - {s}' for s in notebook['sources'][:3])}
  {'  - ...' if len(notebook['sources']) > 3 else ''}
"""

    guide += """
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
"""

    return guide


def main():
    """Generate automation scripts and guide."""

    print("Generating NotebookLM automation scripts...")

    # Create scripts directory if needed
    scripts_dir = Path("scripts")
    scripts_dir.mkdir(exist_ok=True)

    # Generate individual scripts for each notebook
    for notebook in NOTEBOOKS:
        script = generate_browser_control_script(notebook)
        script_file = scripts_dir / f"notebooklm_create_{notebook['name'].lower().replace(' ', '_')}.txt"
        script_file.write_text(script)
        print(f"  ✓ Generated script: {script_file}")

    # Create automation guide
    guide = create_automation_guide()
    guide_file = Path("docs/research/external/NOTEBOOKLM_AUTOMATION_GUIDE.md")
    guide_file.write_text(guide)
    print(f"  ✓ Generated guide: {guide_file}")

    # Create notebook links template
    links_template = {
        notebook["name"]: "" for notebook in NOTEBOOKS
    }
    OUTPUT_FILE.write_text(json.dumps(links_template, indent=2))
    print(f"  ✓ Created links template: {OUTPUT_FILE}")

    print("\n✅ Automation scripts generated!")
    print("\nNext steps:")
    print("1. Review NOTEBOOKLM_AUTOMATION_GUIDE.md")
    print("2. Sign in to NotebookLM manually")
    print("3. Use browser control MCP to create notebooks")
    print("4. Save notebook links to NOTEBOOKLM_NOTEBOOK_LINKS.json")


if __name__ == "__main__":
    main()
