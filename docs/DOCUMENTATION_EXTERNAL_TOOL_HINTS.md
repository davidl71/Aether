# Documentation External Tool Hints Guide

**Purpose**: Standard pattern for adding hints to external tools (Context7, NotebookLM, etc.) in our documentation to help AI assistants discover and use external resources.

## Overview

Our documentation references many external libraries, frameworks, and APIs. By adding hints to external tools like Context7, we enable AI assistants to:

- Access up-to-date, version-specific documentation
- Get current code examples and best practices
- Avoid hallucinations by using real documentation
- Discover additional resources beyond our local docs

## Standard Hint Format

### Context7 Hints

Use this format when documentation mentions external libraries or frameworks:

```markdown
> 💡 **AI Assistant Hint:** For up-to-date, version-specific documentation on [Library/Framework], use the Context7 MCP server by appending `use context7` to your prompts. For example:
> - "How do I use [Library] [Feature]? use context7"
> - "Show me [Library] [Pattern] examples use context7"
> - "[Library] best practices 2025 use context7"
>
> Context7 provides current documentation (2025), version-specific API references, and real code examples without hallucinations.
```

### Inline Context7 Hints

For shorter, inline references:

```markdown
For more information, see [Library Documentation](link) or query Context7: "[Library] patterns use context7"
```

### NotebookLM Hints

For research tasks or when processing external sources:

```markdown
> 📚 **AI Assistant Hint:** For deep research on [Topic], use NotebookLM MCP server to:
> - Summarize YouTube videos and documentation links
> - Create zero-hallucination knowledge bases
> - Extract key information from multiple sources
>
> Example: "Create a knowledge base from [sources] and summarize key patterns"
```

## Placement Guidelines

### Where to Add Hints

1. **At the top of API/library documentation files**
   - First section after title
   - Helps AI discover tool early

2. **Near library/framework mentions**
   - Inline hints for specific libraries
   - Section-level hints for library groups

3. **In integration guides**
   - When explaining external library usage
   - Before code examples using external libraries

4. **In architecture documentation**
   - When describing external dependencies
   - When explaining technology choices

### Where NOT to Add Hints

- Internal-only documentation (no external references)
- Generated documentation (may be overwritten)
- Very short files (< 50 lines) unless highly relevant

## Examples

### Example 1: API Documentation Index

```markdown
# API Documentation Index

> 💡 **AI Assistant Hint:** For up-to-date, version-specific documentation on any library or framework mentioned in this index, use the Context7 MCP server by appending `use context7` to your prompts. For example:
> - "How do I use FastAPI async endpoints? use context7"
> - "Show me React hooks patterns use context7"
> - "CMake best practices 2025 use context7"
>
> Context7 provides current documentation (2025), version-specific API references, and real code examples without hallucinations.
```

### Example 2: Integration Guide

```markdown
## NATS Integration

We use NATS for message queuing. For current NATS patterns:

> 💡 **AI Assistant Hint:** Query Context7 for up-to-date NATS documentation:
> - "NATS performance optimization patterns use context7"
> - "NATS multi-language client examples use context7"
```

### Example 3: Inline Reference

```markdown
We use FastAPI for the REST API. For current FastAPI patterns, see the [official docs](https://fastapi.tiangolo.com/) or query Context7: "FastAPI async patterns use context7"
```

## Tools Available

### Context7 MCP Server

**Purpose**: Up-to-date, version-specific documentation and code examples

**Best For**:
- Library/framework documentation
- API references
- Code examples
- Best practices (2025)

**Usage**: Append `use context7` to prompts

**Configuration**: `.cursor/mcp.json` - `@upstash/context7-mcp`

### NotebookLM MCP Server

**Purpose**: Research and knowledge base creation

**Best For**:
- Summarizing YouTube videos
- Processing documentation links
- Creating knowledge bases
- Research synthesis

**Usage**: Use NotebookLM MCP tools directly

**Configuration**: `.cursor/mcp.json` - `notebooklm-mcp`

## Maintenance

### Keeping Hints Current

- Review hints when updating documentation
- Update examples if tool usage patterns change
- Remove hints if tools are deprecated
- Add hints when new external tools are added

### Consistency

- Use the standard format across all documentation
- Keep hints concise but actionable
- Include example queries when possible
- Link to tool documentation when available

## Related Documentation

- `.cursor/rules/context7.mdc` - Context7 usage rules
- `.cursor/rules/notebooklm.mdc` - NotebookLM usage rules
- `docs/research/integration/MCP_SERVERS.md` - MCP server documentation
- `docs/API_DOCUMENTATION_INDEX.md` - Example with Context7 hints

---

**Last Updated**: 2025-11-23
**Status**: Active Pattern
