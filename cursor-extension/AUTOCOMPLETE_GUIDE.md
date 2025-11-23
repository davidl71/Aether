# MCP Tools & Prompts Autocomplete Guide

## 🎯 Overview

The Cursor extension provides autocomplete support for MCP tools and prompts in chat, making it easier to discover and use project automation features.

---

## ✨ Features

### 1. **Snippets** (Quick Insert)

Type snippet prefixes to quickly insert tool/prompt patterns:

| Prefix | What It Inserts |
|--------|----------------|
| `mcp-docs-health` | Documentation health check tool |
| `mcp-task-align` | Task alignment analysis tool |
| `mcp-duplicate` | Duplicate detection tool |
| `mcp-security` | Security scan tool |
| `mcp-automation` | Automation discovery tool |
| `mcp-sync` | Task synchronization tool |
| `mcp-pwa` | PWA review tool |
| `mcp-prompt-docs` | Documentation health prompt |
| `mcp-prompt-align` | Task alignment prompt |
| `mcp-prompt-duplicate` | Duplicate cleanup prompt |
| `mcp-prompt-security` | Security scan prompt |
| `mcp-prompt-presprint` | Pre-sprint cleanup prompt |
| `mcp-prompt-weekly` | Weekly maintenance prompt |
| `mcp-prompt-postimpl` | Post-implementation review prompt |

**Usage:**
1. Type the prefix (e.g., `mcp-docs-health`)
2. Press `Tab` or select from autocomplete
3. Fill in parameters as prompted

---

### 2. **IntelliSense Autocomplete** (In Chat)

The extension provides autocomplete suggestions when typing in chat:

#### **Tool Suggestions**

Triggered when you type:
- `Use ` or `use `
- `Run ` or `run `
- `Call ` or `call `
- Tool name prefixes like `check_`, `analyze_`, `detect_`, etc.

**Example:**
```
You type: "Use check_"
↓
Autocomplete suggests:
  ✓ check_documentation_health_tool
  ✓ check_documentation_health_tool(create_tasks=true)
```

#### **Prompt Suggestions**

Triggered when you type:
- `prompt` or `Prompt`
- `Use the` or `use the`
- Prompt name prefixes like `doc_`, `task_`, `duplicate_`, etc.

**Example:**
```
You type: "Use the doc_"
↓
Autocomplete suggests:
  ✓ doc_health_check
  ✓ doc_quick_check
```

---

### 3. **Command Palette Help**

Access a complete reference via Command Palette:

1. `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Project Automation: Show MCP Tools & Prompts Help`
3. View all available tools and prompts in the output channel

---

## 📋 Available Tools

### Documentation
- `check_documentation_health_tool` - Analyze documentation structure and health

### Task Management
- `analyze_todo2_alignment_tool` - Analyze task alignment with project goals
- `detect_duplicate_tasks_tool` - Detect and consolidate duplicate tasks
- `sync_todo_tasks_tool` - Synchronize tasks across systems

### Security
- `scan_dependency_security_tool` - Scan dependencies for security vulnerabilities

### Automation
- `find_automation_opportunities_tool` - Discover automation opportunities

### PWA
- `review_pwa_config_tool` - Review PWA configuration

### System
- `server_status` - Get MCP server status

---

## 📝 Available Prompts

### Documentation
- `doc_health_check` - Full documentation analysis with task creation
- `doc_quick_check` - Quick documentation check (no tasks)

### Task Management
- `task_alignment` - Analyze task alignment with goals
- `duplicate_cleanup` - Find and consolidate duplicate tasks
- `task_sync` - Sync tasks between systems

### Security
- `security_scan_all` - Scan all dependencies
- `security_scan_python` - Scan Python dependencies
- `security_scan_rust` - Scan Rust dependencies

### Automation
- `automation_discovery` - Discover automation opportunities
- `automation_high_value` - Find high-value opportunities

### PWA
- `pwa_review` - Review PWA configuration

### Workflows
- `pre_sprint_cleanup` - Pre-sprint cleanup workflow
- `post_implementation_review` - Post-implementation review workflow
- `weekly_maintenance` - Weekly maintenance workflow

---

## 💡 Usage Examples

### Using Tools in Chat

```
You: "Use check_documentation_health_tool with create_tasks=true"
AI: [Executes tool and shows results]
```

### Using Prompts in Chat

```
You: "Use the doc_health_check prompt"
AI: [Retrieves prompt, understands workflow, executes tools]
```

### Using Snippets

1. Type `mcp-docs-health` in chat
2. Press `Tab`
3. Autocomplete inserts: `Use check_documentation_health_tool with create_tasks=true|false`
4. Select `true` or `false` and continue

---

## 🔧 Configuration

Autocomplete works automatically in:
- **Markdown files** (`.md`)
- **Plain text files** (`.txt`)
- **Chat interface** (when typing)

No additional configuration needed!

---

## 🚀 Tips

1. **Start typing tool/prompt names** - Autocomplete will suggest matches
2. **Use snippets for common patterns** - Type `mcp-` prefix for quick access
3. **Check Command Palette help** - Full reference available anytime
4. **Combine with natural language** - "Use the weekly maintenance prompt" works too

---

## 📚 Related Documentation

- `PROMPTS.md` - Detailed prompt documentation
- `HOW_TO_USE_PROMPTS.md` - Prompt usage guide
- `README.md` - Extension overview

---

**Status:** ✅ Autocomplete enabled for all MCP tools and prompts
