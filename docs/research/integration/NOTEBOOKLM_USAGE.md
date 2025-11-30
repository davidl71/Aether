# NotebookLM Usage Guide

This guide explains how to use NotebookLM MCP server to summarize YouTube videos, documentation links, and other resources for better AI assistance in your workflow.

## Overview

NotebookLM is Google's zero-hallucination knowledge base powered by Gemini 2.5. It allows you to:

- Upload YouTube videos, PDFs, Google Docs, markdown files, websites, and GitHub repos
- Get intelligent, synthesized answers based on your sources
- Create documentation from video content
- Research topics with citation-backed answers

## Initial Setup

### 1. Authenticate (One-Time)

In Cursor chat, say:

```
"Log me in to NotebookLM" or "Open NotebookLM auth setup"
```

A Chrome window will open. Log in with your Google account.

**Note**: Consider using a dedicated Google account for automation rather than your primary account.

### 2. Create a Notebook

1. Go to [notebooklm.google.com](https://notebooklm.google.com)
2. Click "Create notebook"
3. Upload your sources:
   - **YouTube videos**: Paste YouTube URLs
   - **PDFs**: Upload PDF files
   - **Google Docs**: Share and link Google Docs
   - **Websites**: Paste URLs
   - **GitHub repos**: Link GitHub repositories
   - **Markdown files**: Upload or paste markdown content

4. Share the notebook:
   - Click **⚙️ Share → Anyone with link → Copy**

### 3. Add Notebook to Library

In Cursor chat, say:

```
"Add [notebook-link] to library tagged 'trading, options, ib-api'"
```

The AI will save the notebook with metadata (tags, description) for easy retrieval.

## Common Use Cases

### Summarizing YouTube Videos

**Step 1: Create a Notebook with Video**

1. Go to NotebookLM
2. Create a new notebook
3. Add YouTube video URL (e.g., `https://www.youtube.com/watch?v=...`)
4. Wait for processing (may take a few minutes)
5. Share the notebook and copy the link

**Step 2: Add to Library**

```
"Add [notebook-link] to library tagged 'youtube, tutorial, [topic]'"
```

**Step 3: Summarize Video**

```
"Research this video in NotebookLM and create a markdown summary in docs/video-summaries/[video-name].md"
```

The AI will:

1. Ask NotebookLM questions about the video content
2. Get synthesized answers with citations
3. Create a markdown document with key points, timestamps, and references

### Processing Documentation Links

**Step 1: Create Notebook with Documentation**

1. Create a new notebook in NotebookLM
2. Add documentation URLs (e.g., API docs, GitHub README, blog posts)
3. Share the notebook

**Step 2: Research and Document**

```
"I'm working on [feature]. Research the [topic] documentation in NotebookLM and create a guide in docs/[topic]-guide.md"
```

The AI will:

1. Ask NotebookLM specific questions about the documentation
2. Get accurate answers (no hallucinations)
3. Create comprehensive documentation based on the sources

### Research Before Coding

When building with a new library or API:

```
"I'm building with [library]. Here's my NotebookLM notebook: [link]. Research the API before writing code."
```

The AI will:

1. Ask multiple questions to understand the API
2. Get specific implementation details
3. Write correct code based on accurate information
4. Avoid hallucinated APIs or methods

### Creating Documentation from Multiple Sources

```
"Create documentation for [topic] using these NotebookLM notebooks: [link1] [link2] [link3]"
```

The AI will:

1. Research across multiple notebooks
2. Synthesize information from all sources
3. Create comprehensive documentation with citations
4. Organize content logically

## Library Management

### List All Notebooks

```
"Show our notebooks" or "List all notebooks in the library"
```

### Select Active Notebook

```
"Use the [notebook-name] notebook" or "Select the notebook tagged '[tag]'"
```

### Update Notebook Metadata

```
"Update notebook tags for [notebook-name] to include 'new-tag'"
```

### Remove Notebook

```
"Remove [notebook-name] from library"
```

### Search Notebooks

```
"Find notebooks about [topic]" or "Search notebooks tagged '[tag]'"
```

## Advanced Usage

### Autonomous Research Sessions

The AI can automatically ask follow-up questions to build complete understanding:

```
"Research [topic] in NotebookLM before implementing. Ask multiple questions to understand all details."
```

The AI will:

1. Start with a broad question
2. Ask follow-up questions based on answers
3. Build comprehensive understanding
4. Then write implementation code

### Viewing Browser Activity

To watch the live NotebookLM conversation:

```
"Show me the browser" or "Let me see the NotebookLM chat"
```

This opens the browser window so you can see what the AI is asking and what NotebookLM is responding.

### Handling Rate Limits

NotebookLM free tier has daily query limits (50 queries/day). To switch accounts:

```
"Re-authenticate with a different Google account" or "Switch to a different Google account for NotebookLM"
```

### Cleanup and Reset

To start fresh:

```
"Run NotebookLM cleanup" - Removes all data
"Cleanup but keep my library" - Removes sessions but keeps notebooks
"Delete all NotebookLM data" - Complete removal
```

## Workflow Examples

### Example 1: Summarizing a Trading Tutorial Video

1. **Create Notebook**: Add YouTube video about options trading
2. **Add to Library**: `"Add [link] to library tagged 'trading, options, tutorial'"`
3. **Summarize**: `"Research this video and create a summary in docs/trading-tutorials/options-basics.md"`
4. **Result**: Markdown file with key concepts, timestamps, and citations

### Example 2: Documenting TWS API

1. **Create Notebook**: Add TWS API documentation links
2. **Add to Library**: `"Add [link] to library tagged 'tws, api, documentation'"`
3. **Research**: `"I'm implementing order management. Research TWS API order placement in NotebookLM"`
4. **Code**: AI writes correct code based on accurate API information

### Example 3: Creating Project Documentation

1. **Create Notebook**: Add multiple sources (GitHub repos, blog posts, videos)
2. **Add to Library**: `"Add [link] to library tagged 'project-docs, architecture'"`
3. **Document**: `"Create architecture documentation using the project-docs notebook"`
4. **Result**: Comprehensive documentation synthesized from all sources

## Best Practices

### 1. Tag Notebooks Properly

Use descriptive tags to make notebooks easy to find:

- `trading, options, strategies`
- `tws, api, integration`
- `youtube, tutorial, [topic]`
- `documentation, [library-name]`

### 2. Organize by Topic

Create separate notebooks for different topics:

- One notebook for TWS API documentation
- One notebook for trading strategies
- One notebook for YouTube tutorials
- One notebook for project-specific docs

### 3. Use Descriptive Names

Give notebooks clear names:

- "TWS API Documentation"
- "Options Trading Tutorials"
- "Project Architecture Notes"

### 4. Research Before Coding

Always research in NotebookLM before implementing new features:

```
"Research [topic] in NotebookLM before writing code"
```

### 5. Save Important Summaries

Always save video summaries and research results to `docs/`:

```
"Save this summary to docs/video-summaries/[name].md"
```

## Troubleshooting

### Authentication Issues

If authentication fails:

```
"Repair NotebookLM authentication" or "Fix NotebookLM auth"
```

This will clear auth data and open browser for fresh login.

### Browser Not Opening

Check that Chrome is installed and accessible. The MCP server uses Chrome for automation.

### Rate Limit Reached

Switch to a different Google account:

```
"Re-authenticate with a different Google account"
```

### Notebook Not Found

List all notebooks to see what's available:

```
"Show our notebooks"
```

Then select the correct notebook:

```
"Use the [notebook-name] notebook"
```

### Session Issues

Reset sessions if things aren't working:

```
"Reset NotebookLM session" or "Close all NotebookLM sessions"
```

## Integration with Project Workflow

### Documentation Generation

Use NotebookLM to generate documentation from videos and links:

```bash

# In Cursor chat:

"Summarize [youtube-video-link] and save to docs/video-summaries/"
```

### Research Before Implementation

Before implementing new features:

```
"I'm implementing [feature]. Research [topic] in NotebookLM first."
```

### API Documentation

Keep API documentation up-to-date:

```
"Update API documentation using the TWS API notebook"
```

## Security Considerations

- **Browser Automation**: NotebookLM uses browser automation. Consider using a dedicated Google account.
- **Local Storage**: All data (Chrome profile, sessions) is stored locally on your machine.
- **Credentials**: Your Google credentials never leave your machine.
- **Rate Limits**: Free tier has daily limits. Use multiple accounts if needed.

## See Also

- [NotebookLM Beginner Tips](../../NOTEBOOKLM_BEGINNER_TIPS.md) - Expert tips and best practices from Google's NotebookLM team
- [MCP Servers Configuration](MCP_SERVERS.md) - General MCP server setup
- [NotebookLM Setup Guide](../../NOTEBOOKLM_SETUP_GUIDE.md) - Setup instructions
- [NotebookLM MCP Repository](https://github.com/PleasePrompto/notebooklm-mcp) - Source code and detailed documentation
- [NotebookLM Official Site](https://notebooklm.google.com) - Google's NotebookLM service
- [NotebookLM MCP Documentation](https://github.com/PleasePrompto/notebooklm-mcp/tree/main/docs) - Detailed tool reference

## Quick Reference

| Task | Command |
|------|---------|
| Authenticate | `"Log me in to NotebookLM"` |
| Add notebook | `"Add [link] to library tagged '[tags]'"` |
| List notebooks | `"Show our notebooks"` |
| Research topic | `"Research [topic] in NotebookLM"` |
| Summarize video | `"Summarize [video-link] and save to docs/"` |
| Select notebook | `"Use the [name] notebook"` |
| View browser | `"Show me the browser"` |
| Switch account | `"Re-authenticate with different account"` |
| Cleanup | `"Run NotebookLM cleanup"` |
