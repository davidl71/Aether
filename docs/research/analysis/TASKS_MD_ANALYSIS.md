# tasks.md Markdown Task Manager Analysis

**Date**: 2025-01-27
**Status**: Research Complete
**Related Task**: T-11

---

## Executive Summary

tasks.md is a privacy-focused task management application that uses plain Markdown files to store tasks. This analysis compares tasks.md with the current Todo2 MCP system and assesses its value for personal task management and development workflow organization.

**Key Finding**: tasks.md offers a simple, privacy-focused approach using Markdown files, while Todo2 provides structured task management with MCP integration. tasks.md is better for personal task management, while Todo2 is better for development workflow with AI integration.

---

## tasks.md Overview

### Project Details

- **Format**: Plain Markdown files
- **Storage**: Local filesystem or Microsoft OneDrive
- **Privacy**: No tracking, full data ownership
- **Platform**: iOS, macOS (native apps), Web (self-hosted)
- **License**: One-time purchase (iOS/macOS), open-source (self-hosted)

### Key Features

1. **Open Data Format**
   - Tasks stored in plain Markdown files
   - Accessible with any text editor
   - Version control friendly (Git)
   - Portable and vendor-independent

2. **Flexible Storage**
   - Local file system
   - Microsoft OneDrive sync
   - iCloud sync (iOS/macOS apps)
   - Self-hosted option (Docker)

3. **Privacy Focused**
   - No tracking
   - No vendor lock-in
   - Full data ownership
   - Local or user-controlled cloud storage

4. **Keyboard Friendly**
   - Keyboard shortcuts for navigation
   - Efficient task management
   - Minimal mouse usage

5. **Task Features**
   - Priority tags (`+urgent`, `+inprogress`)
   - Due dates (`due:2025-06-06`)
   - Tags (`#work`, `#design`)
   - Contexts (`@finance`, `@clientX`)
   - Time tracking (`_spent:2700`)
   - Completion dates (`_done:2025-06-04`)
   - Subtasks (nested lists)

### Markdown Task Format

```markdown
# My Task List

## Project: Website Redesign 🚀 @webdev
- [ ] Draft new homepage mockups #design +inprogress due:2025-06-10
  - [ ] Research competitor websites
  - [ ] Sketch wireframes
- [x] Setup project structure _done:2025-06-01
- [ ] Code HTML and CSS _spent:4500
```

**Syntax Elements**:

- `[ ]` / `[x]` - Task status (incomplete/complete)
- `#tag` - Tags for categorization
- `@context` - Contexts for filtering
- `+priority` - Priority markers
- `due:YYYY-MM-DD` - Due dates
- `_done:YYYY-MM-DD` - Completion dates
- `_spent:seconds` - Time tracking

---

## Current Project Task Management

### Todo2 MCP System

**Location**: `.todo2/state.todo2.json`
**Integration**: MCP (Model Context Protocol) server
**Purpose**: Development workflow task management

**Features**:

- Structured JSON format
- Rich metadata (priority, tags, dependencies)
- Long descriptions with acceptance criteria
- Status tracking (Todo, In Progress, Review, Done)
- AI integration via MCP
- Project-specific storage

**Example Task**:

```json
{
  "id": "T-1",
  "name": "Research pseudo code approaches",
  "status": "In Progress",
  "priority": "high",
  "tags": ["research", "architecture"],
  "long_description": "🎯 Objective: ...",
  "dependencies": []
}
```

### Shared TODO Table

**Location**: `agents/shared/TODO_OVERVIEW.md`
**Format**: Markdown table
**Purpose**: Cross-agent coordination

**Features**:

- Simple markdown table
- Status tracking (pending, in_progress, completed)
- Agent ownership
- Manual updates

### README Task Lists

**Location**: `README.md`
**Format**: Markdown checklists
**Purpose**: High-level feature tracking

**Features**:

- Simple checkboxes
- Status indicators (✅, 🚧, 📋)
- Feature categorization

---

## Comparison Analysis

### Format & Storage

| Aspect | tasks.md | Todo2 MCP | Shared TODO |
|--------|----------|-----------|-------------|
| **Format** | Markdown | JSON | Markdown table |
| **Storage** | Local/OneDrive/iCloud | Project directory | Git repository |
| **Version Control** | Git-friendly | Git-friendly | Git-friendly |
| **Portability** | High (plain text) | Medium (JSON) | High (markdown) |
| **Human Readable** | Yes | Partial | Yes |

**Verdict**: tasks.md is most human-readable, Todo2 is most structured.

### Task Features

| Feature | tasks.md | Todo2 MCP | Shared TODO |
|---------|----------|-----------|-------------|
| **Priority** | `+urgent` tags | Priority field | None |
| **Due Dates** | `due:YYYY-MM-DD` | Not built-in | None |
| **Tags** | `#tag` | Tags array | None |
| **Contexts** | `@context` | Not built-in | None |
| **Time Tracking** | `_spent:seconds` | Not built-in | None |
| **Subtasks** | Nested lists | Not built-in | None |
| **Dependencies** | Not built-in | Dependencies array | None |
| **Rich Descriptions** | Markdown | Long description field | None |
| **Status** | `[ ]` / `[x]` | Status enum | Status text |

**Verdict**: tasks.md has better personal productivity features, Todo2 has better development workflow features.

### Integration & Automation

| Aspect | tasks.md | Todo2 MCP | Shared TODO |
|--------|----------|-----------|-------------|
| **AI Integration** | None | MCP server | None |
| **CLI Tools** | Limited | MCP tools | None |
| **CI/CD** | Manual | MCP automation | Manual |
| **Git Integration** | Native | Native | Native |
| **Cross-Platform** | iOS/macOS/Web | Any (MCP) | Any (markdown) |

**Verdict**: Todo2 has better AI/automation integration, tasks.md has better native apps.

### Privacy & Data Ownership

| Aspect | tasks.md | Todo2 MCP | Shared TODO |
|--------|----------|-----------|-------------|
| **Privacy** | High (local/OneDrive) | High (local) | High (Git) |
| **Tracking** | None | None | None |
| **Vendor Lock-in** | None | None | None |
| **Data Ownership** | Full | Full | Full |

**Verdict**: All three respect privacy and data ownership.

---

## Use Case Analysis

### Personal Task Management

**tasks.md Advantages**:

- Native iOS/macOS apps
- OneDrive/iCloud sync
- Time tracking
- Due dates and reminders
- Better for personal productivity

**Todo2 Advantages**:

- AI integration for task creation
- Rich descriptions for complex tasks
- Dependencies for workflow
- Better for development tasks

**Recommendation**: Use tasks.md for personal tasks, Todo2 for development tasks.

### Development Workflow

**tasks.md Advantages**:

- Simple markdown format
- Easy to edit in any editor
- Git-friendly
- No special tools needed

**Todo2 Advantages**:

- MCP integration with AI
- Structured metadata
- Dependencies tracking
- Automated task management
- Better for complex projects

**Recommendation**: Todo2 is better suited for development workflow.

### Team Collaboration

**tasks.md Advantages**:

- Markdown is universal
- Easy to share files
- OneDrive sharing

**Todo2 Advantages**:

- Structured format
- Better for automation
- MCP server integration

**Recommendation**: Both work, but Todo2 is better for structured team workflows.

---

## Integration Opportunities

### Option 1: Use tasks.md for Personal Tasks (Recommended)

**Action**: Use tasks.md for personal productivity, keep Todo2 for development.

**Benefits**:

- Best tool for each purpose
- Clear separation of concerns
- Native apps for personal tasks
- AI integration for development tasks

**Implementation**:

- Install tasks.md iOS/macOS app for personal tasks
- Continue using Todo2 MCP for development workflow
- No integration needed

**Effort**: Low (just install app)

### Option 2: Migrate to tasks.md Format

**Action**: Convert Todo2 tasks to tasks.md markdown format.

**Benefits**:

- Simpler format
- More human-readable
- Better for personal use

**Drawbacks**:

- Lose AI integration (MCP)
- Lose structured metadata
- Lose dependencies
- Manual conversion needed

**Effort**: High (migration + lose features)

### Option 3: Hybrid Approach

**Action**: Use tasks.md for personal tasks, Todo2 for development, sync manually.

**Benefits**:

- Best of both worlds
- Personal productivity + development workflow

**Drawbacks**:

- Two systems to maintain
- Manual sync if needed

**Effort**: Low (just use both)

---

## Recommendations

### Short-Term (1-3 months)

1. **Try tasks.md for Personal Tasks**
   - Install iOS/macOS app
   - Use for personal productivity
   - Keep Todo2 for development

2. **Enhance Todo2 Usage**
   - Better task descriptions
   - Use dependencies more
   - Leverage MCP automation

### Medium-Term (3-6 months)

1. **Evaluate Workflow**
   - Assess if tasks.md helps personal productivity
   - Consider if Todo2 needs improvements
   - Decide on long-term approach

2. **Document Best Practices**
   - When to use tasks.md vs Todo2
   - How to sync between systems (if needed)
   - Task management workflow guide

### Long-Term (6+ months)

1. **Consider Integration** (if needed)
   - Script to export Todo2 to tasks.md format
   - Or vice versa
   - Only if workflow benefits

---

## Key Takeaways

1. **Different Purposes**: tasks.md for personal productivity, Todo2 for development workflow
2. **Both Privacy-Focused**: Both respect data ownership and privacy
3. **Format Trade-offs**: Markdown (human-readable) vs JSON (structured)
4. **AI Integration**: Todo2 has MCP integration, tasks.md doesn't
5. **Native Apps**: tasks.md has iOS/macOS apps, Todo2 is MCP-based

---

## References

- **tasks.md Website**: <https://tasks.md/>
- **tasks.md GitHub** (self-hosted): <https://github.com/BaldissaraMatheus/Tasks.md>
- **tasks.md App Store**: <https://apps.apple.com/il/app/tasks-md/id6753879372>
- **Todo2 MCP**: See `docs/AGENTIC_TOOLS_USAGE.md`

---

## Related Documentation

- [Agentic Tools Usage](../../AGENTIC_TOOLS_USAGE.md) - Todo2 MCP system guide
- [Coordination Guidelines](agents/shared/COORDINATION.md) - Shared TODO table
- [API Documentation Index](../../API_DOCUMENTATION_INDEX.md) - Complete API reference

---

**Last Updated**: 2025-01-27
**Next Review**: When evaluating personal productivity tools
