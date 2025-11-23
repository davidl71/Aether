# How to Use MCP Prompts in Cursor

## Overview

MCP prompts are reusable templates that provide structured guidance for common workflows. They help you use the project management automation tools effectively.

---

## 🚀 Quick Start

### Method 1: Direct AI Chat (Recommended)

Simply ask the AI assistant to use a prompt by name or describe what you want to do:

**Examples:**

```
"Use the pre-sprint cleanup workflow"
```

```
"Run the documentation health check prompt"
```

```
"I want to do weekly maintenance - use the weekly_maintenance prompt"
```

The AI will:
1. Retrieve the prompt template
2. Understand the workflow
3. Execute the recommended tools with correct parameters
4. Guide you through the process

---

### Method 2: Explicit Prompt Reference

You can explicitly reference prompts in your chat:

```
"Use the doc_health_check prompt to analyze documentation"
```

```
"Follow the duplicate_cleanup prompt to find duplicate tasks"
```

```
"Execute the security_scan_all prompt"
```

---

## 📋 Available Prompts

### Individual Tool Prompts

These prompts guide you to use a specific tool:

| Prompt Name | What It Does | Example Usage |
|-------------|--------------|---------------|
| `doc_health_check` | Full documentation analysis with task creation | "Use doc_health_check to check docs before release" |
| `doc_quick_check` | Quick documentation check (no tasks) | "Run doc_quick_check to see doc status" |
| `task_alignment` | Analyze task alignment with goals | "Check task alignment using task_alignment prompt" |
| `duplicate_cleanup` | Find and consolidate duplicate tasks | "Use duplicate_cleanup to clean up tasks" |
| `task_sync` | Sync tasks between systems | "Sync tasks with task_sync prompt" |
| `security_scan_all` | Scan all dependencies for vulnerabilities | "Run security_scan_all before deployment" |
| `security_scan_python` | Scan Python dependencies | "Check Python security with security_scan_python" |
| `security_scan_rust` | Scan Rust dependencies | "Use security_scan_rust to check Rust deps" |
| `automation_discovery` | Find automation opportunities | "Discover automations with automation_discovery" |
| `automation_high_value` | Find high-value automations only | "Find best automations with automation_high_value" |
| `pwa_review` | Review PWA configuration | "Review PWA config with pwa_review prompt" |

### Workflow Prompts

These prompts guide you through multi-step workflows:

| Prompt Name | Workflow Steps | When to Use |
|-------------|----------------|-------------|
| `pre_sprint_cleanup` | 1. Find duplicates<br>2. Check alignment<br>3. Review docs | Before starting new sprint |
| `post_implementation_review` | 1. Update docs<br>2. Check security<br>3. Find automations | After completing a feature |
| `weekly_maintenance` | 1. Check docs<br>2. Clean duplicates<br>3. Scan security<br>4. Sync tasks | Weekly maintenance routine |

---

## 💡 Practical Examples

### Example 1: Before Starting a New Sprint

**You say:**
```
"I'm starting a new sprint. Use the pre_sprint_cleanup workflow."
```

**AI will:**
1. Retrieve the `pre_sprint_cleanup` prompt
2. Understand it's a 3-step workflow
3. Execute:
   - `detect_duplicate_tasks_tool` to find duplicates
   - `analyze_todo2_alignment_tool` to check alignment
   - `check_documentation_health_tool` to review docs
4. Present results and recommendations

---

### Example 2: Quick Documentation Check

**You say:**
```
"Quick check on documentation status using doc_quick_check"
```

**AI will:**
1. Retrieve the `doc_quick_check` prompt
2. Understand it's a report-only check (no task creation)
3. Execute: `check_documentation_health_tool(create_tasks=False)`
4. Show you the health report

---

### Example 3: Security Audit Before Release

**You say:**
```
"Run a full security scan using security_scan_all before we release"
```

**AI will:**
1. Retrieve the `security_scan_all` prompt
2. Understand it scans all languages
3. Execute: `scan_dependency_security_tool(languages=None)`
4. Show vulnerabilities prioritized by severity

---

### Example 4: Weekly Maintenance

**You say:**
```
"It's Monday - let's do weekly maintenance using the weekly_maintenance prompt"
```

**AI will:**
1. Retrieve the `weekly_maintenance` prompt
2. Understand it's a 4-step workflow
3. Execute all steps in sequence:
   - Documentation health check
   - Duplicate task cleanup
   - Security scan
   - Task synchronization
4. Provide a comprehensive maintenance report

---

## 🎯 Best Practices

### 1. Use Workflow Prompts for Multi-Step Tasks

Instead of manually running each tool, use workflow prompts:

✅ **Good:**
```
"Use pre_sprint_cleanup before we start"
```

❌ **Less Efficient:**
```
"Find duplicates, then check alignment, then check docs"
```

### 2. Be Specific About Your Goal

The AI can choose the right prompt if you describe your goal:

✅ **Good:**
```
"I want to check if our tasks are aligned with project goals"
```
→ AI uses `task_alignment` prompt

✅ **Also Good:**
```
"Use task_alignment to check task alignment"
```

### 3. Combine Prompts with Specific Requests

You can combine prompts with specific parameters:

```
"Use doc_health_check but write the report to docs/release_health.md"
```

```
"Run security_scan_all but focus on Python dependencies"
```

---

## 🔍 How Prompts Work

1. **You request a prompt** in Cursor's AI chat
2. **AI retrieves the prompt** from the MCP server
3. **AI reads the prompt description** which includes:
   - What the workflow does
   - Which tools to use
   - Recommended parameters
   - Use case guidance
4. **AI executes the tools** following the prompt's guidance
5. **AI presents results** in a structured format

---

## 🛠️ Troubleshooting

### Prompt Not Found

If you get "prompt not found":
1. **Restart Cursor** - Prompts are loaded when the server starts
2. **Check server status** - Verify the MCP server is running
3. **Use exact prompt name** - Check `PROMPTS.md` for exact names

### Prompt Not Executing Tools

If the prompt is retrieved but tools aren't running:
1. **Be explicit** - Say "execute" or "run" the prompt
2. **Check tool availability** - Verify tools are loaded (see `server_status`)
3. **Review logs** - Check Cursor's MCP server logs for errors

---

## 📚 Related Documentation

- **`PROMPTS.md`** - Complete list of all prompts with descriptions
- **`USAGE.md`** - Detailed tool usage guide
- **`README.md`** - Server overview and setup

---

## 🎓 Learning the Prompts

The best way to learn prompts is to:

1. **Start with workflow prompts** - They show you complete workflows
2. **Try individual tool prompts** - Learn what each tool does
3. **Experiment** - Try different prompts to see what works best for your workflow

**Recommended First Prompts:**
- `pre_sprint_cleanup` - See a complete workflow
- `doc_health_check` - Simple, single-tool prompt
- `weekly_maintenance` - Comprehensive maintenance routine

---

**Last Updated:** 2025-11-23
